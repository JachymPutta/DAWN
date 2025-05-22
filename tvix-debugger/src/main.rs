use clap::Parser;

use tvix_debugger::config::Args;
use tvix_debugger::run_debugger;

fn main() {
    let mut args = Args::parse();
    args.validate();
    run_debugger(args);
}
