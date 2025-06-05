#![warn(
    clippy::all,
    clippy::pedantic,
    rust_2018_idioms,
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::panic
)]
#![allow(clippy::unused_async, clippy::module_name_repetitions)]
//! nix debugger implementation

use std::sync::mpsc::{self, Receiver, Sender};

use dawn_infra::codec::DebugAdapterCodec;
use dawn_infra::debugger::{Client, DebugAdapter, State};
use debug_types::ProtocolMessage;
use nix_debugger::{NixDebugAdapter, NixDebugState};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::error;
use tvix_debugger::backend::TvixBackend;
use tvix_debugger::commands::{Command, CommandReply};

/// debugger
pub mod nix_debugger;

/// Top-level function initializes the adapter and the debugger
pub fn run_toplevel<R, W>(reader: R, writer: W)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
    W: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (adapter_sender, debugger_reciever) = mpsc::channel::<Command>();
    let (debugger_sender, adapter_reciever) = mpsc::channel::<CommandReply>();
    let debugger_handle = run_debugger(debugger_reciever, debugger_sender);
    let _ = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(10)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let log_writer = std::fs::File::create("./LOGLOG").unwrap();
            tracing_subscriber::fmt().with_writer(log_writer).init();

            run_debug_adapter(reader, writer, adapter_sender, adapter_reciever).await;
        });
    let _ = debugger_handle.join();
}

/// Runs the debug adapter loop using the provided async reader/writer.
async fn run_debug_adapter<R, W>(
    reader: R,
    writer: W,
    sender: Sender<Command>,
    receiver: Receiver<CommandReply>,
) where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
    W: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let reader = FramedRead::new(reader, DebugAdapterCodec::<ProtocolMessage>::default());
    let writer = FramedWrite::new(writer, DebugAdapterCodec::<ProtocolMessage>::default());
    println!("Framed reader and writer initialized");

    let client = Client::new(reader, writer);
    let mut adapter = NixDebugAdapter {
        client,
        state: NixDebugState::default(),
        sender,
        receiver,
    };
    println!("Adapter initialized, entering message loop");

    while adapter.client.get_state() < State::ShutDown {
        use debug_types::MessageKind::{Event, Request, Response};
        let msg = adapter.client.next_msg().await;
        println!("got a message {:?}", msg);
        match msg.message {
            Request(request) => adapter.handle_request(msg.seq, request).await,
            Response(response) => {
                error!("Received response {response:?}. Shouldn't be possible!");
            }
            Event(e) => error!("Received event {e:?}. Shouldn't be possible!"),
        }
    }
}

/// Initialize the tvix debugger
// TODO: add the program as an argument here, run only after we get the program
fn run_debugger(
    receiver: Receiver<Command>,
    sender: Sender<CommandReply>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let debugger_args = tvix_debugger::config::Args {
            program: "tvix-debugger/tests/simple_fn_call.nix".into(),
        };
        let mut debugger = TvixBackend::new(debugger_args);
        loop {
            let cmd = receiver.recv().unwrap();
            let reply = debugger.handle_command(cmd);
            sender.send(reply).unwrap();
        }
    })
}
