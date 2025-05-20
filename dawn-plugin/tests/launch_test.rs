use std::time::Duration;

use dawn_infra::codec::DebugAdapterCodec;
use dawn_plugin::run_debugger;

use debug_types::{MessageKind, ProtocolMessage};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::time::timeout;
use tokio_util::codec::{FramedRead, FramedWrite};

#[tokio::test]
async fn test_launch_request() {
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

    let lauch = json!({
        "seq": 1,
        "type": "request",
        "command": "launch",
        "arguments": {
            "no_debug": true,
            "manifest": Some(".".to_string()),
            "expression": Some("3 + 1".to_string()),
        }
    });
    let init_msg: ProtocolMessage = serde_json::from_value(lauch).unwrap();
    writer.send(init_msg).await.unwrap();

    let launch_resp = timeout(Duration::from_secs(1), reader.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    match launch_resp.message {
        MessageKind::Response(r) if r.success => {}
        other => panic!("bad init response: {:?}", other),
    }
}
