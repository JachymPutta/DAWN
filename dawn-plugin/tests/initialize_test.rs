mod common;

use common::request::initialize_request;
use common::session::TestSession;

use dawn_infra::dap_requests::ExtendedMessageKind;

#[tokio::test]
async fn test_initialize_request_sync() {
    let mut session = TestSession::new().await;

    session.send(initialize_request()).await;
    let response = session.recv().await;

    match response.message {
        ExtendedMessageKind::Response(r) if r.success => { /* success */ }
        other => panic!("unexpected init response: {:?}", other),
    }

    session.shutdown().await;
}
