use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

use debug_types::types::Capabilities;
use tvix_eval::{EvalMode, Evaluation, SourceCode};

use crate::commands::{
    default_capabilities, Breakpoint, Command, CommandReply, ObserverCommand, ObserverReply,
};
use crate::config::Args;
use crate::observer::DebugObserver;
use crate::serde_smolstr::SerSmolStr;

struct ObserverClient {
    handle: JoinHandle<()>,
    receiver: Receiver<ObserverReply>,
    sender: Sender<ObserverCommand>,
}

impl ObserverClient {
    pub fn new(prog: SerSmolStr) -> Self {
        let (backend_sender, observer_reciever) = mpsc::channel::<ObserverCommand>();
        let (observer_sender, backend_reciever) = mpsc::channel::<ObserverReply>();
        let handle =
            ObserverClient::initialize_observer(prog.into(), observer_reciever, observer_sender);
        ObserverClient {
            handle,
            receiver: backend_reciever,
            sender: backend_sender,
        }
    }

    fn initialize_observer(
        program: PathBuf,
        observer_reciever: Receiver<ObserverCommand>,
        observer_sender: Sender<ObserverReply>,
    ) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let source_code = SourceCode::default();
            let code_path = program.clone();
            let file_name = code_path.file_name().unwrap().to_os_string();

            source_code.add_file(
                file_name.into_string().unwrap(),
                program.to_str().unwrap().into(),
            );

            let mut observer =
                DebugObserver::new(source_code.clone(), observer_reciever, observer_sender);
            let eval = Evaluation::builder_impure()
                .mode(EvalMode::Strict)
                .with_source_map(source_code)
                .runtime_observer(Some(&mut observer))
                .build();
            let code = std::fs::read_to_string(&code_path).expect(&format!(
                "Error opening file: {}",
                &code_path.to_str().unwrap()
            ));
            let result = eval.evaluate(code, Some(code_path));
            println!("Execution done: {:?}", result);
            observer.set_cmd(ObserverCommand::Done);

            loop {
                match observer.handle_command() {
                    Ok(()) => continue,
                    Err(e) => {
                        println!("observer: ended with - {}", e.to_string());
                        break;
                    }
                }
            }
            println!("observer is done")
        })
    }
}

/// tvix backend struct
// FIXME remove this allow
#[allow(dead_code)]
pub struct TvixBackend {
    state: DebuggerState,
    code_path: Option<PathBuf>,
    code: SourceCode,
    observer_client: Option<ObserverClient>,
}

// FIXME: error handling if the observer_client isn't initialized
impl TvixBackend {
    pub fn new(_args: Args) -> Self {
        let code = SourceCode::default();
        let code_path = None;
        let observer_client = None;

        TvixBackend {
            state: DebuggerState::Uninitialized,
            code_path,
            code,
            observer_client,
        }
    }

    pub fn get_state(&self) -> DebuggerState {
        self.state
    }

    pub fn handle_command(&mut self, command: Command) -> CommandReply {
        match command {
            Command::Initialize => {
                let capabilities = self.handle_initialize();
                CommandReply::InitializeReply(capabilities)
            }
            Command::Launch(prog) => {
                self.handle_launch(prog);
                CommandReply::LaunchReply
            }
            Command::Step => {
                self.handle_step();
                CommandReply::StepReply
            }
            Command::Break(breakpoint) => {
                self.handle_break(breakpoint);
                CommandReply::BreakReply
            }
            Command::Print(var_name) => {
                self.handle_print(var_name);
                CommandReply::PrintReply
            }
            Command::Continue => {
                self.handle_continue();
                CommandReply::LaunchReply
            }
            Command::Exit => {
                println!("backend: got an exit, exting");
                self.handle_exit();
                (*self).exit();
                println!("backend: exited");
                CommandReply::ExitReply
            }
            _ => {
                unreachable!("Unknown command in backend: {}", command)
            }
        }
    }

    fn handle_initialize(&mut self) -> Capabilities {
        Capabilities {
            supports_configuration_done_request: Some(true),
            ..default_capabilities()
        }
    }

    fn handle_exit(&mut self) {
        let _ = self
            .observer_client
            .as_mut()
            .unwrap()
            .sender
            .send(ObserverCommand::Exit);
        let _ = self.observer_client.as_mut().unwrap().receiver.recv();
    }

    fn handle_launch(&mut self, prog: SerSmolStr) {
        self.observer_client = Some(ObserverClient::new(prog.clone()));
        let _ = self
            .observer_client
            .as_mut()
            .unwrap()
            .sender
            .send(ObserverCommand::Launch(prog));
        // let state = self.receiver.recv();
    }

    fn handle_continue(&mut self) {
        let _ = self
            .observer_client
            .as_mut()
            .unwrap()
            .sender
            .send(ObserverCommand::Continue);
    }

    fn handle_step(&mut self) {
        let _ = self
            .observer_client
            .as_mut()
            .unwrap()
            .sender
            .send(ObserverCommand::Step);
    }

    fn handle_break(&mut self, breakpoint: Breakpoint) {
        println!("got breakpoint: {:?}", &breakpoint);
        let _ = self
            .observer_client
            .as_mut()
            .unwrap()
            .sender
            .send(ObserverCommand::Break(breakpoint));
    }

    fn handle_print(&mut self, var_name: SerSmolStr) {
        let _ = self
            .observer_client
            .as_mut()
            .unwrap()
            .sender
            .send(ObserverCommand::Print(var_name));
    }

    pub fn exit(&mut self) {
        // TODO: send exit to the evaluator, join the handle, return
        println!("got exit, joining observer");
        let _ = self.observer_client.take().unwrap().handle.join();
        println!("observer joined");
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum DebuggerState {
    /// Server has not received an `initialize` request.
    Uninitialized = 0,
    /// Server received an `initialize` request, but has not yet responded.
    Initializing = 1,
    /// Server received and responded success to an `initialize` request.
    Initialized = 2,
    /// Server received a `shutdown` request.
    ShutDown = 3,
    /// Server received an `exit` notification.
    Exited = 4,
}

#[cfg(test)]
mod tests {
    #[test]
    fn eval_simple_expr() {
        let code = "1 + 2";
        let eval_builder = tvix_eval::Evaluation::builder_pure();
        let eval = eval_builder.build();

        let tvix_result = eval
            .evaluate(code, None)
            .value
            .expect("tvix evaluation should succeed")
            .to_string()
            .parse::<i32>()
            .unwrap_or_default();
        assert_eq!(tvix_result, 3);
    }
}
