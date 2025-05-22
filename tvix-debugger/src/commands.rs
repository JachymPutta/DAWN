use strum::Display;
use strum_macros::EnumString;

#[derive(Debug, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Command {
    #[strum(serialize = "exit", serialize = "e")]
    Exit,
    Unknown,
    Initialize,
    Launch,
}

#[derive(Debug, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum CommandReply {
    ExitReply,
    UnknownReply,
    InitializeReply,
    LaunchReply,
}
