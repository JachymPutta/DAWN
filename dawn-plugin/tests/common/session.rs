use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::io::{duplex, DuplexStream};
use tokio_util::codec::{FramedRead, FramedWrite};

use dawn_infra::codec::DebugAdapterCodec;
use debug_types::ProtocolMessage;

use super::request::disconnect_request;

/// Holds the full state of a test session.
pub struct TestSession {
    writer: FramedWrite<DuplexStream, DebugAdapterCodec<ProtocolMessage>>,
    reader: FramedRead<DuplexStream, DebugAdapterCodec<ProtocolMessage>>,
}

impl TestSession {
    /// Creates a new session, spawns the adapter, and sets up runtime + I/O.
    pub async fn new() -> Self {
        let (client_input, adapter_input) = duplex(1024);
        let (adapter_output, client_output) = duplex(1024);

        let _adapter_handle = tokio::spawn(dawn_plugin::run_debug_adapter(
            adapter_input,
            adapter_output,
        ));

        let writer = FramedWrite::new(client_input, DebugAdapterCodec::default());
        let reader = FramedRead::new(client_output, DebugAdapterCodec::default());

        Self { writer, reader }
    }

    /// Sends a `ProtocolMessage` to the adapter.
    pub async fn send(&mut self, msg: ProtocolMessage) {
        self.writer.send(msg).await.expect("Failed to send message")
    }

    /// Reads a message from the adapter with a timeout.
    pub async fn recv(&mut self) -> ProtocolMessage {
        tokio::time::timeout(Duration::from_secs(3), self.reader.next())
            .await
            .expect("read timeout")
            .expect("stream closed")
            .expect("decode error")
    }

    /// Gracefully shutdown the adapter thread.
    pub async fn shutdown(mut self) {
        // TODO: send terminate with a timeout, then disconnect
        self.send(disconnect_request()).await;
    }
}
