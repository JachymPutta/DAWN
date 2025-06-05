use std::sync::mpsc::{Receiver, Sender};

use debug_types::{
    events::EventBody,
    requests::{BreakpointLocationsArguments, InitializeRequestArguments, LaunchRequestArguments},
    responses::{BreakpointLocationsResponse, InitializeResponse, Response, ResponseBody},
    types::BreakpointLocation,
};
use either::Either;

use dawn_infra::debugger::{Client, DebugAdapter, State};
use debug_types::requests::RequestCommand::{
    BreakpointLocations, ConfigurationDone, Disconnect, Initialize, Launch,
};
use nll::nll_todo::nll_todo;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::error;
use tvix_debugger::{
    backend::TvixBackend,
    commands::{Command, CommandReply},
};

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
        self.sender
            .send(tvix_debugger::commands::Command::Initialize)
            .unwrap();
        let capabilities =
            if let CommandReply::InitializeReply(capabilities) = self.receiver.recv().unwrap() {
                capabilities
            } else {
                panic!("Error: initializing backend")
            };

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
        // self.debugger.launch(args);
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
    /// channel to send commands to the debugger
    pub sender: Sender<Command>,
    /// channel for replies from the debugger
    pub receiver: Receiver<CommandReply>,
}

/// the debug state
#[derive(Default, Debug, Clone)]
pub struct NixDebugState {
    // root_file: std::io
}
