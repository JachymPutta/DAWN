use dawn_infra::dap_requests::ExtendedProtocolMessage;
use serde_json::json;

/// Builds a standard initialize request.
pub fn initialize_request() -> ExtendedProtocolMessage {
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
    serde_json::from_value(init).expect("valid initialize request")
}

/// Builds a launch request using a file path.
pub fn launch_request_with_file(
    program: &str,
    manifest: Option<String>,
) -> ExtendedProtocolMessage {
    let val = json!({
        "seq": 1,
        "type": "request",
        "command": "launch",
        "arguments": {
            "no_debug": true,
            "manifest": manifest.unwrap_or_else(|| ".".into()),
            "expression": "",
            "program": program,
        }
    });
    serde_json::from_value(val).expect("valid launch request (file)")
}

/// Builds a disconnect request.
pub fn disconnect_request() -> ExtendedProtocolMessage {
    let val = json!({
        "seq": 1,
        "type": "request",
        "command": "disconnect",
        "arguments": {
            "restart": false
        }
    });
    serde_json::from_value(val).expect("valid disconnect request")
}
