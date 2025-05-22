use backend::TvixBackend;
use commands::Command;
use config::Args;
use input::handle_input;
use rustyline::DefaultEditor;

/// dap server
pub mod backend;
pub mod commands;
pub mod config;
pub mod input;
pub mod observer;

pub fn run_debugger(args: Args) {
    let mut backend = TvixBackend::new(args);
    let mut read_line = DefaultEditor::new().expect("rl: failure creating editor");

    // The main repl loop
    loop {
        let command = handle_input(&mut read_line);
        match command {
            Command::Unknown => continue,
            Command::Exit => break,
            _ => {
                let reply = backend.handle_command(command);
                println!("{}", reply);
            }
        }
    }
}
