use dawn_plugin::run_debugger;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let writer = std::fs::File::create("./LOGLOG").unwrap();
    tracing_subscriber::fmt().with_writer(writer).init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    run_debugger(stdin, stdout).await;
}
