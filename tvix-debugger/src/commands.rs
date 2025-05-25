use smol_str::SmolStr;
use std::str::FromStr;
use strum::Display;
use strum_macros::EnumString;

pub(crate) type Breakpoint = SmolStr; //TODO: change to Either<SmolStr, i32> to set breakpoints on a line
                                      //number or function

#[derive(Debug, Display)]
pub enum Command {
    Exit,
    Unknown,
    Initialize,
    Continue,
    Launch,
    Step,
    Break(SmolStr),
    Print(SmolStr),
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
            "initialize" | "init" | "i" => Ok(Command::Initialize),
            "launch" | "l" => Ok(Command::Launch),
            "step" | "s" => Ok(Command::Step),
            "break" | "b" => {
                if let Some(target) = arg {
                    Ok(Command::Break(target.into()))
                } else {
                    println!("Err: break missing argument -- provide function name");
                    Err(()) // Or Command::Unknown if you prefer
                }
            }
            _ => Ok(Command::Unknown),
        }
    }
}
#[derive(Debug, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum CommandReply {
    ExitReply,
    UnknownReply,
    InitializeReply,
    LaunchReply,
    StepReply,
    BreakReply,
    ContinueReply,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ObserverCommand {
    Wait,
    Continue,
    Step,
    Break(Breakpoint),
    Print(SmolStr),
}

#[derive(Debug)]
pub enum ObserverReply {
    State,
    Done,
}
