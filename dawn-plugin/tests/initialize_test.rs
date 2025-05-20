use std::time::Duration;

use dawn_bindings::backend::TvixBackend;
use dawn_infra::codec::DebugAdapterCodec;
use dawn_infra::debugger::{Client, DebugAdapter};
use dawn_plugin::nix_debugger::{NixDebugAdapter, NixDebugState};
use dawn_plugin::run_debugger;

use debug_types::requests::{InitializeRequestArguments, RequestCommand};
use debug_types::{MessageKind, ProtocolMessage};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::time::timeout;
use tokio_util::codec::{FramedRead, FramedWrite};

#[tokio::test]
async fn test_initialize_request() {
    let (client_input, adapter_output) = tokio::io::duplex(1024);
    let (adapter_input, client_output) = tokio::io::duplex(1024);

    let _task = tokio::spawn(run_debugger(adapter_input, adapter_output));

    let writer = FramedWrite::new(
        client_input,
        DebugAdapterCodec::<ProtocolMessage>::default(),
    );
    let reader = FramedRead::new(
        client_output,
        DebugAdapterCodec::<ProtocolMessage>::default(),
    );

    let init_req = RequestCommand::Initialize(InitializeRequestArguments {
        client_id: None,
        client_name: None,
        adapter_id: "Hello".to_string(),
        locale: None,
        lines_start_at1: None,
        columns_start_at1: None,
        path_format: None,
        supports_variable_type: None,
        supports_variable_paging: None,
        supports_run_in_terminal_request: None,
        supports_memory_references: None,
        supports_progress_reporting: None,
        supports_invalidated_event: None,
        supports_memory_event: None,
    });

    let client = Client::new(reader, writer);
    let mut adapter = NixDebugAdapter {
        client,
        state: NixDebugState::default(),
        debugger: TvixBackend::new(),
    };

    adapter.handle_request(1, init_req).await;
}

#[tokio::test]
async fn test_initialize_request_json() {
    let (client_input, adapter_input) = tokio::io::duplex(1024);
    let (adapter_output, client_output) = tokio::io::duplex(1024);

    // spawn the debugger loop
    let _task = tokio::spawn(run_debugger(adapter_input, adapter_output));

    let mut writer = FramedWrite::new(
        client_input,
        DebugAdapterCodec::<ProtocolMessage>::default(),
    );
    let mut reader = FramedRead::new(
        client_output,
        DebugAdapterCodec::<ProtocolMessage>::default(),
    );

    let init = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {
            "adapterID": "dawn",
            "linesStartAt1": true,
            "columnsStartAt1": true,
            "pathFormat": "path"
        }
    });
    let init_msg: ProtocolMessage = serde_json::from_value(init).unwrap();
    writer.send(init_msg).await.unwrap();

    let init_resp = timeout(Duration::from_secs(1), reader.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    match init_resp.message {
        MessageKind::Response(r) if r.success => {}
        other => panic!("bad init response: {:?}", other),
    }
}
