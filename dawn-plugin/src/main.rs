use dawn_plugin::run_debug_adapter;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let log_writer = std::fs::File::create("./LOGLOG").unwrap();
    tracing_subscriber::fmt().with_writer(log_writer).init();

    run_debug_adapter(stdin, stdout).await
}
