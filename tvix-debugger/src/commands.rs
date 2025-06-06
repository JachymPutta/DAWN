use debug_types::types::Capabilities;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::Display;

use crate::serde_smolstr::SerSmolStr;

// TODO: support breakpoints on variable names
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Breakpoint {
    Line(usize),
    FileLine { file: SerSmolStr, line: usize },
}

impl FromStr for Breakpoint {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((file, line)) = s.split_once(':') {
            line.parse::<usize>()
                .map(|l| Breakpoint::FileLine {
                    file: file.into(),
                    line: l,
                })
                .map_err(|_| ())
        } else {
            s.parse::<usize>().map(Breakpoint::Line).map_err(|_| ())
        }
    }
}

#[derive(Debug, Display, Serialize, Deserialize)]
pub enum Command {
    Exit,
    Unknown,
    // Initialize, //FIXME: Initialize seems to be adapter only, if not, it's here
    Continue,
    Launch(Option<String>),
    Step,
    Break(Breakpoint),
    Print(SerSmolStr),
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        // split into command and optional argument
        let mut parts = trimmed.splitn(2, ' ');
        let cmd = parts.next().unwrap_or("").to_lowercase();
        let arg = parts.next().map(str::trim);

        match cmd.as_str() {
            "exit" | "e" => Ok(Command::Exit),
            "continue" | "c" => Ok(Command::Continue),
            // "initialize" | "init" | "i" => Ok(Command::Initialize),
            "launch" | "l" => Ok(Command::Launch(None)),
            "step" | "s" => Ok(Command::Step),
            "break" | "b" => {
                if let Some(target) = arg {
                    // TODO: don't explode in case of invalid string
                    Ok(Command::Break(target.parse().unwrap()))
                } else {
                    println!("Err: break missing argument -- provide function name");
                    Err(()) // Or Command::Unknown if you prefer
                }
            }
            "print" | "p" => {
                if let Some(target) = arg {
                    Ok(Command::Print(target.into()))
                } else {
                    println!("Err: print missing argument -- provide variable name");
                    Err(()) // Or Command::Unknown if you prefer
                }
            }
            _ => Ok(Command::Unknown),
        }
    }
}
#[derive(Debug, Display, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum CommandReply {
    ExitReply,
    UnknownReply,
    InitializeReply(Capabilities),
    LaunchReply,
    StepReply,
    BreakReply,
    PrintReply,
    ContinueReply,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ObserverCommand {
    Exit,
    Wait,
    Done,
    Launch,
    Continue,
    Step,
    Break(Breakpoint),
    Print(SerSmolStr),
}

#[derive(Debug)]
pub enum ObserverReply {
    State,
    Done,
}
