mod common;

use common::request::initialize_request;
use common::session::TestSession;

use debug_types::MessageKind;

#[test]
fn test_initialize_request_sync() {
    let mut session = TestSession::new();

    session.send(initialize_request());
    let response = session.recv();

    match response.message {
        MessageKind::Response(r) if r.success => { /* success */ }
        other => panic!("unexpected init response: {:?}", other),
    }

    session.shutdown();
}
