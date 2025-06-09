use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::io::{duplex, DuplexStream};
use tokio_util::codec::{FramedRead, FramedWrite};

use dawn_infra::{
    codec::DebugAdapterCodec,
    dap_requests::{ExtendedMessageKind, ExtendedProtocolMessage},
};

use super::request::disconnect_request;

/// Holds the full state of a test session.
pub struct TestSession {
    writer: FramedWrite<DuplexStream, DebugAdapterCodec<ExtendedProtocolMessage>>,
    reader: FramedRead<DuplexStream, DebugAdapterCodec<ExtendedProtocolMessage>>,
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

    /// Sends a `ExtendedProtocolMessage` to the adapter.
    pub async fn send(&mut self, msg: ExtendedProtocolMessage) {
        self.writer.send(msg).await.expect("Failed to send message")
    }

    /// Reads a message from the adapter with a timeout.
    pub async fn recv(&mut self) -> ExtendedProtocolMessage {
        tokio::time::timeout(Duration::from_secs(3), self.reader.next())
            .await
            .expect("read timeout")
            .expect("stream closed")
            .expect("decode error")
    }

    /// Gracefully shutdown the adapter thread.
    pub async fn shutdown(mut self) {
        // TODO: send terminate with a timeout, then disconnect
        // currently just forces disconnect immediately

        self.send(disconnect_request()).await;

        // let response = self.recv().await;
        // match response.message {
        //     ExtendedMessageKind::Response(r) if r.success => {}
        //     other => panic!("bad disconnect response: {:?}", other),
        // }
    }
}
