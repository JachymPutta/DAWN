mod common;

use common::request::initialize_request;
use common::session::TestSession;

use debug_types::MessageKind;

#[tokio::test]
async fn test_initialize_request_sync() {
    let mut session = TestSession::new().await;

    session.send(initialize_request()).await;
    let response = session.recv().await;

    match response.message {
        MessageKind::Response(r) if r.success => { /* success */ }
        other => panic!("unexpected init response: {:?}", other),
    }

    session.shutdown().await;
}
