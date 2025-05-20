use debug_types::requests::{InitializeRequestArguments, LaunchRequestArguments};
use debug_types::types::Capabilities;

/// dap server mt
pub trait DebugBackend {
    /// Initialize the back-end
    fn initialize(&mut self, args: InitializeRequestArguments) -> Capabilities;
    /// Launch the target program
    fn launch(&mut self, args: LaunchRequestArguments);
    // TODO: other methods
}
