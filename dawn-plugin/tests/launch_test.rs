mod common;

use common::request::{initialize_request, launch_request_with_file};
use common::session::TestSession;

use debug_types::MessageKind;

// #[test]
// fn test_launch_request_expression() {
//     let mut session = TestSession::new();
//
//     let launch_request = launch_request_with_expression("3 + 1");
//     session.send(launch_request);
//
//     let response = session.recv();
//     match response.message {
//         MessageKind::Response(r) if r.success => {}
//         other => panic!("bad launch response: {:?}", other),
//     }
//
//     session.shutdown();
// }

#[tokio::test]
async fn test_launch_request_file() {
    let mut session = TestSession::new().await;

    session.send(initialize_request()).await;
    let _capabilities = session.recv().await;
    let _initialized = session.recv().await;

    let launch_request =
        launch_request_with_file("../tvix-debugger/tests/simple.nix", Some(".".into()));
    session.send(launch_request).await;

    let response = session.recv().await;
    match response.message {
        MessageKind::Response(r) if r.success => {}
        other => panic!("bad launch response: {:?}", other),
    }

    session.shutdown().await;
}
