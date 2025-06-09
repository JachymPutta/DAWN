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
    Initialize, //FIXME: Initialize seems to be adapter only, if not, it's here
    Continue,
    Launch(SerSmolStr),
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
            "initialize" | "init" | "i" => Ok(Command::Initialize),
            "launch" | "l" => {
                if let Some(target) = arg {
                    // TODO: don't explode in case of invalid string
                    Ok(Command::Launch(target.into()))
                } else {
                    println!("Err: break missing argument -- provide function name");
                    Err(())
                }
            }
            "step" | "s" => Ok(Command::Step),
            "break" | "b" => {
                if let Some(target) = arg {
                    // TODO: don't explode in case of invalid string
                    Ok(Command::Break(target.parse().unwrap()))
                } else {
                    println!("Err: break missing argument -- provide function name");
                    Err(())
                }
            }
            "print" | "p" => {
                if let Some(target) = arg {
                    Ok(Command::Print(target.into()))
                } else {
                    println!("Err: print missing argument -- provide variable name");
                    Err(())
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
    Launch(SerSmolStr),
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

// FIXME why does capabilities not implement default?
/// "sane" capabilities: disable everything!
#[must_use]
pub fn default_capabilities() -> Capabilities {
    Capabilities {
        supports_configuration_done_request: None,
        supports_function_breakpoints: None,
        supports_step_in_targets_request: None,
        support_terminate_debuggee: None,
        supports_loaded_sources_request: None,
        supports_data_breakpoints: None,
        supports_breakpoint_locations_request: None,
        supports_conditional_breakpoints: None,
        supports_hit_conditional_breakpoints: None,
        supports_evaluate_for_hovers: None,
        exception_breakpoint_filters: None,
        supports_step_back: None,
        supports_set_variable: None,
        supports_restart_frame: None,
        supports_goto_targets_request: None,
        supports_completions_request: None,
        completion_trigger_characters: None,
        supports_modules_request: None,
        additional_module_columns: None,
        supported_checksum_algorithms: None,
        supports_restart_request: None,
        supports_exception_options: None,
        supports_value_formatting_options: None,
        supports_exception_info_request: None,
        support_suspend_debuggee: None,
        supports_delayed_stack_trace_loading: None,
        supports_log_points: None,
        supports_terminate_threads_request: None,
        supports_set_expression: None,
        supports_terminate_request: None,
        supports_read_memory_request: None,
        supports_write_memory_request: None,
        supports_disassemble_request: None,
        supports_cancel_request: None,
        supports_clipboard_context: None,
        supports_stepping_granularity: None,
        supports_instruction_breakpoints: None,
        supports_exception_filter_options: None,
        supports_single_thread_execution_requests: None,
    }
}
