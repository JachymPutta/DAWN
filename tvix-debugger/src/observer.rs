use tvix_eval::{
    observer::RuntimeObserver,
    opcode::{CodeIdx, Op},
    value::Lambda,
};

pub struct DebugObserver;

impl DebugObserver {
    pub fn new() -> Self {
        DebugObserver
    }
}

impl RuntimeObserver for DebugObserver {
    fn observe_enter_call_frame(
        &mut self,
        _arg_count: usize,
        _: &std::rc::Rc<Lambda>,
        _call_depth: usize,
    ) {
    }

    fn observe_exit_call_frame(&mut self, _frame_at: usize, _stack: &[tvix_eval::Value]) {}

    fn observe_suspend_call_frame(&mut self, _frame_at: usize, _stack: &[tvix_eval::Value]) {}

    fn observe_enter_generator(
        &mut self,
        _frame_at: usize,
        _name: &str,
        _stack: &[tvix_eval::Value],
    ) {
    }

    fn observe_exit_generator(
        &mut self,
        _frame_at: usize,
        _name: &str,
        _stack: &[tvix_eval::Value],
    ) {
    }

    fn observe_suspend_generator(
        &mut self,
        _frame_at: usize,
        _name: &str,
        _stack: &[tvix_eval::Value],
    ) {
    }

    fn observe_generator_request(&mut self, _name: &str, _msg: &tvix_eval::generators::VMRequest) {}

    fn observe_tail_call(&mut self, _frame_at: usize, _: &std::rc::Rc<Lambda>) {}

    fn observe_enter_builtin(&mut self, _name: &'static str) {}

    fn observe_exit_builtin(&mut self, _name: &'static str, _stack: &[tvix_eval::Value]) {}

    fn observe_execute_op(&mut self, _ip: CodeIdx, _: &Op, _: &[tvix_eval::Value]) {}
}
