use dawn_plugin::run_toplevel;

fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    run_toplevel(stdin, stdout);
}
