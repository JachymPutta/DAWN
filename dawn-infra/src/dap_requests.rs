use debug_types::{events::Event, requests::*, responses::Response, types::Capabilities};
use serde::{Deserialize, Serialize};

/// Mirror of the RequestCommand debug_types enum, with some modifications for custom args
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "command", content = "arguments")]
pub enum ExtendedRequestCommand {
    Initialize(InitializeRequestArguments),
    ConfigurationDone,
    Launch(ExtendedLaunchArguments),
    Attach(AttachRequestArguments),
    Restart(LaunchRequestArguments),
    Disconnect(DisconnectArguments),
    Terminate(TerminateArguments),
    BreakpointLocations(BreakpointLocationsArguments),
    SetBreakpoints(SetBreakpointsArguments),
    SetFunctionBreakpoints(SetFunctionBreakpointsArguments),
    SetExceptionBreakpoints(SetExceptionBreakpointsArguments),
    DataBreakpointInfo(DataBreakpointInfoArguments),
    SetDataBreakpoints(SetDataBreakpointsArguments),
    SetInstructionBreakpoints(SetInstructionBreakpointsArguments),
    Continue(ContinueArguments),
    Next(NextArguments),
    StepIn(StepInArguments),
    StepOut(StepOutArguments),
    StepBack(StepBackArguments),
    ReverseContinue(ReverseContinueArguments),
    RestartFrame(RestartFrameArguments),
    Goto(GotoArguments),
    Pause(PauseArguments),
    StackTrace(StackTraceArguments),
    Scopes(ScopesArguments),
    Variables(VariablesArguments),
    SetVariable(SetVariableArguments),
    Source(SourceArguments),
    Threads,
    TerminateThreads(TerminateThreadsArguments),
    Modules(ModulesArguments),
    LoadedSources,
    Evaluate(EvaluateArguments),
    SetExpression(SetExpressionArguments),
    StepInTargets(StepInTargetsArguments),
    GotoTargets(GotoTargetsArguments),
    Completions(CompletionsArguments),
    ExceptionInfo(ExceptionInfoArguments),
    ReadMemory(ReadMemoryArguments),
    WriteMemory(WriteMemoryArguments),
    Disassemble(DisassembleArguments),
}

/// dap doesn't specify how to pass the debugee, we extend init args to include
/// the program, this seems to be standard across various daps (vs-code and others)
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtendedLaunchArguments {
    /// base init req
    #[serde(flatten)]
    pub inner: LaunchRequestArguments,
    /// debuggee
    pub program: String,
}

/// Mirror of the ProtocolMessage debug_types enum, with some modifications for custom args
#[allow(missing_docs)]
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct ExtendedProtocolMessage {
    pub seq: i64,
    #[serde(flatten)]
    pub message: ExtendedMessageKind,
}

/// Mirror of the MessageKind debug_type enum, with some modifications for custom args
#[allow(missing_docs)]
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum ExtendedMessageKind {
    Request(ExtendedRequestCommand),
    Response(Response),
    Event(Event),
}
