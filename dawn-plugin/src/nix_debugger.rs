use std::sync::{atomic::AtomicBool, Arc};

use debug_types::{
    events::EventBody,
    requests::{BreakpointLocationsArguments, InitializeRequestArguments},
    responses::{BreakpointLocationsResponse, InitializeResponse, Response, ResponseBody},
    types::BreakpointLocation,
};
use either::Either;

use dawn_infra::dap_requests::{ExtendedLaunchArguments, ExtendedRequestCommand::*};
use dawn_infra::{
    dap_requests::ExtendedRequestCommand,
    debugger::{Client, DebugAdapter, Server, State},
};
use nll::nll_todo::nll_todo;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::error;
use tvix_debugger::{
    backend::DebuggerState,
    commands::{default_capabilities, Command, CommandReply},
    config::Args,
};

impl<R, W> DebugAdapter for NixDebugAdapter<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    async fn handle_request(&mut self, seq: i64, command: ExtendedRequestCommand) {
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
    async fn handle_initialize(&mut self, seq: i64, _args: InitializeRequestArguments) {
        let capabilities = default_capabilities();
        let response = InitializeResponse { capabilities };

        let body = Some(ResponseBody::Initialize(response));
        self.client.set_state(State::Initializing);

        self.initialize_debugger();

        self.client
            .send(Either::Right(Response {
                request_seq: seq,
                success: true,
                message: None,
                body,
            }))
            .await;

        self.client.set_state(State::Initialized);

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
    async fn handle_launch(&mut self, seq: i64, args: ExtendedLaunchArguments) {
        let Some(_root_file) = args.inner.manifest.clone() else {
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
        let Some(_flake_attribute) = args.inner.expression.clone() else {
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

        println!("program is !! {}", args.program);
        // TODO: this should happen in the initialization
        println!("program initialized");

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
            server
                .shutdown
                .store(true, std::sync::atomic::Ordering::SeqCst);
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

    async fn initialize_debugger(&mut self) {
        let (cmd_sender, mut cmd_receiver) = tokio::sync::mpsc::channel::<Command>(32);
        let (reply_sender, reply_receiver) = tokio::sync::mpsc::channel::<CommandReply>(32);

        let shutdown_token = Arc::new(AtomicBool::new(false));
        let shutdown_token_clone = shutdown_token.clone();

        let child = std::thread::spawn(move || {
            let args = Args::default();
            let mut debugger = tvix_debugger::backend::TvixBackend::new(args);
            while debugger.get_state() < DebuggerState::ShutDown
                && !shutdown_token_clone.load(std::sync::atomic::Ordering::Relaxed)
            {
                if let Some(cmd) = cmd_receiver.blocking_recv() {
                    let reply = debugger.handle_command(cmd);
                    let _ = reply_sender.blocking_send(reply);
                } else {
                    break;
                }
            }
        });

        self.server = Some(Server {
            sender: cmd_sender,
            receiver: reply_receiver,
            debugger: child,
            shutdown: shutdown_token,
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
