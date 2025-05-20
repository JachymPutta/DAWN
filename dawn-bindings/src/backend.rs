use dawn_infra::backend::DebugBackend;
use debug_types::requests::{InitializeRequestArguments, LaunchRequestArguments};
use debug_types::types::Capabilities;
use tvix_eval::Evaluation;

/// tvix backend struct
pub struct TvixBackend {
    root_file: String,
}

impl TvixBackend {
    pub fn new() -> Self {
        TvixBackend {
            root_file: "".into(),
        }
    }
}

impl DebugBackend for TvixBackend {
    fn initialize(&mut self, _args: InitializeRequestArguments) -> Capabilities {
        Capabilities {
            supports_configuration_done_request: Some(true),
            ..default_capabilities()
        }
    }

    fn launch(&mut self, args: LaunchRequestArguments) {
        if let Some(expr) = args.expression {
            let eval_builder = Evaluation::builder_pure();
            let evaluator = eval_builder.build();
            let res = evaluator.evaluate(expr, None);
            println!("Evaluation result: {:?}", res.value.unwrap());
        } else {
            todo!()
        }
    }
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

#[cfg(test)]
mod tests {
    #[test]
    fn eval_simple_expr() {
        let code = "1 + 2";
        let eval_builder = tvix_eval::Evaluation::builder_pure();
        let eval = eval_builder.build();

        let tvix_result = eval
            .evaluate(code, None)
            .value
            .expect("tvix evaluation should succeed")
            .to_string()
            .parse::<i32>()
            .unwrap_or_default();
        assert_eq!(tvix_result, 3);
    }
}
