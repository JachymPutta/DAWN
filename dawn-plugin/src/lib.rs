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
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::error;
use tvix_debugger::backend::TvixBackend;

/// debugger
pub mod nix_debugger;

/// Runs the debug adapter loop using the provided async reader/writer.
pub async fn run_debug_adapter<R, W>(reader: R, writer: W)
where
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
        debugger: TvixBackend::new(),
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
