mod common;

use common::request::{initialize_request, launch_request_with_file};
use common::session::TestSession;

use dawn_infra::dap_requests::ExtendedMessageKind;

#[tokio::test]
async fn test_launch_request_file() {
    let mut session = TestSession::new().await;

    session.send(initialize_request()).await;
    let _capabilities = session.recv().await;
    println!("Checkpoint: capabilities done");
    let _initialized = session.recv().await;
    println!("Checkpoint: initialized done");

    let launch_request =
        launch_request_with_file("../tvix-debugger/tests/simple.nix", Some(".".into()));
    session.send(launch_request).await;
    println!("Checkpoint: launched");

    let response = session.recv().await;
    println!("Checkpoint: got reply from session");
    match response.message {
        ExtendedMessageKind::Response(r) if r.success => {}
        other => panic!("bad launch response: {:?}", other),
    }

    session.shutdown().await;
}
