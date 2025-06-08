use debug_types::{
    events::EventBody,
    requests::{BreakpointLocationsArguments, InitializeRequestArguments, LaunchRequestArguments},
    responses::{BreakpointLocationsResponse, InitializeResponse, Response, ResponseBody},
    types::{BreakpointLocation, Capabilities},
};
use either::Either;

use dawn_infra::debugger::{Client, DebugAdapter, Server, State};
use debug_types::requests::RequestCommand::{
    BreakpointLocations, ConfigurationDone, Disconnect, Initialize, Launch,
};
use nll::nll_todo::nll_todo;
use tokio::process::Command as TokioCommand;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    process::ChildStdin,
};
use tracing::error;
use tvix_debugger::commands::{Command, CommandReply};

impl<R, W> DebugAdapter for NixDebugAdapter<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    async fn handle_request(&mut self, seq: i64, command: debug_types::requests::RequestCommand) {
        match command {
            Initialize(initialize_args) => self.handle_initialize(seq, initialize_args).await,
            ConfigurationDone => self.handle_configuration_done(seq).await,
            Launch(launch_args) => self.handle_launch(seq, launch_args).await,
            Disconnect(disconnect_args) => self.handle_disconnect(seq, disconnect_args).await,
            BreakpointLocations(breakpoint_locations_args) => {
                self.handle_breakpoint_locations(seq, breakpoint_locations_args)
                    .await;
            }
            _ => {
                self.client
                    .send(Either::Right(Response {
                        request_seq: seq,
                        success: false,
                        message: Some("unsupported request".to_string()),
                        body: None,
                    }))
                    .await;
            }
        }
    }
}

impl<R, W> NixDebugAdapter<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    /// handler for receipt of initialize event from client
    async fn handle_initialize(&mut self, seq: i64, args: InitializeRequestArguments) {
        let capabilities = default_capabilities();
        let response = InitializeResponse { capabilities };

        let body = Some(ResponseBody::Initialize(response));
        self.client.set_state(State::Initializing);

        self.client
            .send(Either::Right(Response {
                request_seq: seq,
                success: true,
                message: None,
                body,
            }))
            .await;

        //TODO: pass in the program and stuff
        self.initialie_debugger();

        // println!("HELLO WORLD 1!!");
        self.client.set_state(State::Initialized);
        // println!("HELLO WORLD!!");

        // per spec, send initialized event
        // after responding with capabilities
        self.client
            .send(Either::Left(EventBody::Initialized {}))
            .await;
    }

    /// handler for receipt of configurationDone event from client
    async fn handle_configuration_done(&mut self, seq: i64) {
        let body = Some(ResponseBody::ConfigurationDone);
        self.client
            .send(Either::Right(Response {
                request_seq: seq,
                success: true,
                message: None,
                body,
            }))
            .await;
    }

    /// handler for receipt of launch event from client
    async fn handle_launch(&mut self, seq: i64, args: LaunchRequestArguments) {
        let Some(root_file) = args.manifest.clone() else {
            self.client
                .send(Either::Right(Response {
                    request_seq: seq,
                    success: false,
                    message: Some("Root file must be specified".to_string()),
                    body: None,
                }))
                .await;
            return;
        };
        // TODO open the file.

        // TODO check that this attribute exists
        let Some(flake_attribute) = args.expression.clone() else {
            self.client
                .send(Either::Right(Response {
                    request_seq: seq,
                    success: false,
                    message: Some("Attribute must be specified".to_string()),
                    body: None,
                }))
                .await;
            return;
        };

        // error!("launch args: {args:?}");
        self.server
            .as_mut()
            .unwrap()
            .sender
            .send(tvix_debugger::commands::Command::Launch(args.name))
            .await
            .unwrap();

        let reply = self.server.as_mut().unwrap().receiver.recv().await.unwrap();
        assert!(matches!(reply, CommandReply::LaunchReply));

        // TODO some argument checking I think
        self.client
            .send(Either::Right(Response {
                request_seq: seq,
                success: true,
                message: None,
                body: Some(ResponseBody::Launch),
            }))
            .await;
    }

    /// handle disconnect request
    /// terminates the debugger!
    async fn handle_disconnect(
        &mut self,
        seq: i64,
        _disconnect_args: debug_types::requests::DisconnectArguments,
    ) {
        // blindly disconnect always
        self.client.set_state(State::ShutDown);

        if let Some(server) = self.server.as_mut() {
            match server.debugger.kill().await {
                Ok(_) => tracing::info!("Successfully killed debugger process."),
                Err(e) => tracing::error!("Failed to kill debugger process: {e}"),
            }
        }

        let body = Some(ResponseBody::Disconnect);
        self.client
            .send(Either::Right(Response {
                request_seq: seq,
                success: true,
                message: None,
                body,
            }))
            .await;
    }

    /// handle breapoint locataion request
    async fn handle_breakpoint_locations(
        &mut self,
        _seq: i64,
        // BreakpointLocationsArguments {
        //     source,
        //     line,
        //     column,
        //     end_line,
        //     end_column,
        // }: BreakpointLocationsArguments,
        bruh: BreakpointLocationsArguments,
    ) {
        error!("{:?}", bruh);
        let _body = Some(ResponseBody::BreakpointLocations(
            BreakpointLocationsResponse {
                breakpoints: vec![BreakpointLocation {
                    line: nll_todo(),
                    column: nll_todo(),
                    end_line: nll_todo(),
                    end_column: nll_todo(),
                }],
            },
        ));
        nll_todo()
        // self.client
        // .send(Either::Right(Response {
        //     request_seq: seq,
        //     success: true,
        //     message: None,
        //     body,
        // }))
        // .await;
    }

    fn initialie_debugger(&mut self) {
        use std::process::Stdio;

        //TODO -- where do we assume the binary is
        let mut child = TokioCommand::new("cargo")
            .args([
                "run",
                "--quiet",
                "--release",
                "--manifest-path",
                "../tvix-debugger/Cargo.toml",
                "--",
                "--program",
                "../tvix-debugger/tests/simple.nix",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to launch debugger process");

        let stdin: ChildStdin = child.stdin.take().expect("Missing debugger stdin");
        let stdout = child.stdout.take().expect("Missing debugger stdout");

        let mut stdin = stdin;
        let stdout_reader = BufReader::new(stdout);

        let (cmd_sender, mut cmd_receiver) = tokio::sync::mpsc::channel::<Command>(32);
        let (reply_sender, reply_receiver) = tokio::sync::mpsc::channel::<CommandReply>(32);

        // Writer task: send Command -> stdin
        tokio::spawn(async move {
            while let Some(cmd) = cmd_receiver.recv().await {
                let msg = match serde_json::to_string(&cmd) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Failed to serialize command: {e}");
                        continue;
                    }
                };
                if let Err(e) = stdin.write_all(msg.as_bytes()).await {
                    eprintln!("Failed to write to debugger stdin: {e}");
                    break;
                }
                let _ = stdin.write_all(b"\n").await;
            }
        });

        // Reader task: read CommandReply <- stdout
        tokio::spawn(async move {
            let mut lines = stdout_reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                match serde_json::from_str::<CommandReply>(&line) {
                    Ok(reply) => {
                        let _ = reply_sender.send(reply);
                    }
                    Err(e) => {
                        eprintln!("Failed to deserialize reply: {e} - line: {line}");
                    }
                }
            }
        });

        self.server = Some(Server {
            sender: cmd_sender,
            receiver: reply_receiver,
            debugger: child,
        });
    }
}

/// overarching struct holding dap state and comms
pub struct NixDebugAdapter<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    /// the comms
    pub client: Client<R, W>,
    /// the state
    pub state: NixDebugState,
    /// the server
    pub server: Option<Server>,
}

/// the debug state
#[derive(Default, Debug, Clone)]
pub struct NixDebugState {
    // root_file: std::io
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
