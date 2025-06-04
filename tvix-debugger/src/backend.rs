use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

use debug_types::types::Capabilities;
use smol_str::SmolStr;
use tvix_eval::{EvalMode, Evaluation, SourceCode};

use crate::commands::{Breakpoint, Command, CommandReply, ObserverCommand, ObserverReply};
use crate::config::Args;
use crate::observer::DebugObserver;

/// tvix backend struct
pub struct TvixBackend {
    code_path: PathBuf,
    code: SourceCode,
    observer_handle: JoinHandle<()>,
    receiver: Receiver<ObserverReply>,
    sender: Sender<ObserverCommand>,
}

impl TvixBackend {
    pub fn new(args: Args) -> Self {
        let (backend_sender, observer_reciever) = mpsc::channel::<ObserverCommand>();
        let (observer_sender, backend_reciever) = mpsc::channel::<ObserverReply>();
        let code = SourceCode::default();
        let code_path = args.program.clone();

        let observer_handle = std::thread::spawn(move || {
            let source_code = SourceCode::default();
            let code_path = args.program.clone();
            let file_name = code_path.file_name().unwrap().to_os_string();

            source_code.add_file(
                file_name.into_string().unwrap(),
                args.program.to_str().unwrap().into(),
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
                observer.handle_command();
            }
        });

        TvixBackend {
            code_path,
            code,
            observer_handle,
            receiver: backend_reciever,
            sender: backend_sender,
        }
    }
}

impl TvixBackend {
    pub fn handle_command(&mut self, command: Command) -> CommandReply {
        match command {
            Command::Initialize => {
                let capabilities = self.handle_initialize();
                CommandReply::InitializeReply(capabilities)
            }
            Command::Launch => {
                self.handle_launch();
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

    fn handle_launch(&mut self) {
        let _ = self.sender.send(ObserverCommand::Launch);
        // let state = self.receiver.recv();
    }

    fn handle_continue(&mut self) {
        let _ = self.sender.send(ObserverCommand::Continue);
    }

    fn handle_step(&mut self) {
        let _ = self.sender.send(ObserverCommand::Step);
    }

    fn handle_break(&mut self, breakpoint: Breakpoint) {
        println!("got breakpoint: {:?}", &breakpoint);
        let _ = self.sender.send(ObserverCommand::Break(breakpoint));
    }

    fn handle_print(&mut self, var_name: SmolStr) {
        let _ = self.sender.send(ObserverCommand::Print(var_name));
    }

    pub fn exit(&mut self) {
        // TODO: send exit to the evaluator, join the handle, return
    }
}

// FIXME why does capabilities not implement default?
/// "sane" capabilities: disable everything!
#[must_use]
pub fn default_capabilities() -> Capabilities {
    Capabilities {
        supports_configuration_done_request: None,
        supports_function_breakpoints: None,
        supports_step_in_targets_request: None,
        support_terminate_debuggee: None,
        supports_loaded_sources_request: None,
        supports_data_breakpoints: None,
        supports_breakpoint_locations_request: None,
        supports_conditional_breakpoints: None,
        supports_hit_conditional_breakpoints: None,
        supports_evaluate_for_hovers: None,
        exception_breakpoint_filters: None,
        supports_step_back: None,
        supports_set_variable: None,
        supports_restart_frame: None,
        supports_goto_targets_request: None,
        supports_completions_request: None,
        completion_trigger_characters: None,
        supports_modules_request: None,
        additional_module_columns: None,
        supported_checksum_algorithms: None,
        supports_restart_request: None,
        supports_exception_options: None,
        supports_value_formatting_options: None,
        supports_exception_info_request: None,
        support_suspend_debuggee: None,
        supports_delayed_stack_trace_loading: None,
        supports_log_points: None,
        supports_terminate_threads_request: None,
        supports_set_expression: None,
        supports_terminate_request: None,
        supports_read_memory_request: None,
        supports_write_memory_request: None,
        supports_disassemble_request: None,
        supports_cancel_request: None,
        supports_clipboard_context: None,
        supports_stepping_granularity: None,
        supports_instruction_breakpoints: None,
        supports_exception_filter_options: None,
        supports_single_thread_execution_requests: None,
    }
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
