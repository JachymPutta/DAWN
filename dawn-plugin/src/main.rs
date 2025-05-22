use dawn_plugin::run_debug_adapter;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let writer = std::fs::File::create("./LOGLOG").unwrap();
    tracing_subscriber::fmt().with_writer(writer).init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    run_debug_adapter(stdin, stdout).await;
}
