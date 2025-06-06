use std::thread::{self, JoinHandle};
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::io::{duplex, DuplexStream};
use tokio::runtime::Runtime;
use tokio_util::codec::{FramedRead, FramedWrite};

use dawn_infra::codec::DebugAdapterCodec;
use debug_types::ProtocolMessage;

use super::request::disconnect_request;

/// Holds the full state of a test session.
pub struct TestSession {
    runtime: Runtime,
    writer: FramedWrite<DuplexStream, DebugAdapterCodec<ProtocolMessage>>,
    reader: FramedRead<DuplexStream, DebugAdapterCodec<ProtocolMessage>>,
    adapter_handle: JoinHandle<()>,
}

impl TestSession {
    /// Creates a new session, spawns the adapter, and sets up runtime + I/O.
    pub fn new() -> Self {
        let (client_input, adapter_input) = duplex(1024);
        let (adapter_output, client_output) = duplex(1024);

        let adapter_handle = thread::spawn(move || {
            dawn_plugin::run_toplevel(adapter_input, adapter_output);
        });

        let runtime = Runtime::new().expect("failed to create tokio runtime");

        let writer = FramedWrite::new(client_input, DebugAdapterCodec::default());
        let reader = FramedRead::new(client_output, DebugAdapterCodec::default());

        Self {
            runtime,
            writer,
            reader,
            adapter_handle,
        }
    }

    /// Sends a `ProtocolMessage` to the adapter.
    pub fn send(&mut self, msg: ProtocolMessage) {
        self.runtime
            .block_on(self.writer.send(msg))
            .expect("failed to send message");
    }

    /// Reads a message from the adapter with a timeout.
    pub fn recv(&mut self) -> ProtocolMessage {
        self.runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(1), self.reader.next())
                .await
                .expect("read timeout")
                .expect("stream closed")
                .expect("decode error")
        })
    }

    /// Gracefully shutdown the adapter thread.
    pub fn shutdown(mut self) {
        // TODO: send terminate, then disconnect
        self.send(disconnect_request());
        let _ = self.adapter_handle.join();
    }
}
