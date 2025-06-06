use debug_types::ProtocolMessage;
use serde_json::json;

/// Builds a standard initialize request.
pub fn initialize_request() -> ProtocolMessage {
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

/// Builds a launch request using an expression instead of a file.
pub fn launch_request_with_expression(expr: &str) -> ProtocolMessage {
    let val = json!({
        "seq": 1,
        "type": "request",
        "command": "launch",
        "arguments": {
            "no_debug": true,
            "manifest": ".",
            "name": "",
            "expression": expr,
        }
    });
    serde_json::from_value(val).expect("valid launch request (expression)")
}

/// Builds a launch request using a file path.
pub fn launch_request_with_file(program: &str, manifest: Option<String>) -> ProtocolMessage {
    let val = json!({
        "seq": 1,
        "type": "request",
        "command": "launch",
        "arguments": {
            "no_debug": true,
            "name": program,
            "manifest": manifest.unwrap_or_else(|| ".".into()),
            "expression": "",
        }
    });
    serde_json::from_value(val).expect("valid launch request (file)")
}

/// Builds a disconnect request.
pub fn disconnect_request() -> ProtocolMessage {
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
