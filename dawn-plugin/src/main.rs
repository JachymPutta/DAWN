use dawn_plugin::run_toplevel;

fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let log_writer = std::fs::File::create("./LOGLOG").unwrap();
    tracing_subscriber::fmt().with_writer(log_writer).init();

    run_toplevel(stdin, stdout);
}
