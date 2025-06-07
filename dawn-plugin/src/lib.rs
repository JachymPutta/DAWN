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

use dawn_infra::codec::DebugAdapterCodec;
use dawn_infra::debugger::{Client, DebugAdapter, State};
use debug_types::ProtocolMessage;
use nix_debugger::{NixDebugAdapter, NixDebugState};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, Command as TokioCommand};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::error;

use tvix_debugger::commands::{Command, CommandReply};

/// debugger
pub mod nix_debugger;

/// Top-level function initializes the adapter and the debugger
pub fn run_toplevel<R, W>(reader: R, writer: W)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
    W: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    #[expect(
        clippy::missing_panics_doc,
        reason = "tokio runtime won't fail to build"
    )]
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(10)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let (adapter_sender, adapter_receiver) = spawn_debugger_process().await;
        run_debug_adapter(reader, writer, adapter_sender, adapter_receiver).await;
    });

    println!("toplevel exited");
}

/// Launches the debugger subprocess and wires its stdin/stdout into async channels.
async fn spawn_debugger_process() -> (
    tokio::sync::mpsc::Sender<Command>,
    tokio::sync::mpsc::Receiver<CommandReply>,
) {
    use std::process::Stdio;
    use std::sync::mpsc;

    let mut child = TokioCommand::new("debugger-process")
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

    (cmd_sender, reply_receiver)
}

/// Runs the debug adapter loop using the provided async reader/writer.
async fn run_debug_adapter<R, W>(
    reader: R,
    writer: W,
    sender: tokio::sync::mpsc::Sender<Command>,
    receiver: tokio::sync::mpsc::Receiver<CommandReply>,
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
        println!("got a message {msg:?}");
        match msg.message {
            Request(request) => adapter.handle_request(msg.seq, request).await,
            Response(response) => {
                error!("Received response {response:?}. Shouldn't be possible!");
            }
            Event(e) => error!("Received event {e:?}. Shouldn't be possible!"),
        }
    }

    println!("Adapter exited");
}
