use rustyline::{error::ReadlineError, DefaultEditor};

use crate::commands::Command;

pub fn handle_input(read_line: &mut DefaultEditor) -> Command {
    let line = read_line.readline(">> ");

    match line {
        Ok(text) => match text.trim().parse::<Command>() {
            Ok(cmd) => cmd,
            Err(_) => {
                println!("Unknown command {}", text);
                Command::Unknown
            }
        },
        Err(ReadlineError::Interrupted) => Command::Exit,
        Err(ReadlineError::Eof) => Command::Exit,
        Err(err) => {
            println!("Error: {:?}", err);
            Command::Exit
        }
    }
}
