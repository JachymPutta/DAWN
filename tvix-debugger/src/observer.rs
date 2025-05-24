use std::sync::mpsc::{self, Receiver, Sender};

use tvix_eval::{
    observer::RuntimeObserver,
    opcode::{CodeIdx, Op},
    value::Lambda,
};

use crate::commands::{ObserverCommand, ObserverReply};

type Breakpoint = i32;

pub struct DebugObserver {
    breakpoints: Vec<Breakpoint>,
    receiver: Receiver<ObserverCommand>,
    sender: Sender<ObserverReply>,
}

impl DebugObserver {
    pub fn new(
        breakpoints: Vec<Breakpoint>,
        receiver: Receiver<ObserverCommand>,
        sender: Sender<ObserverReply>,
    ) -> Self {
        DebugObserver {
            breakpoints,
            receiver,
            sender,
        }
    }
}

impl RuntimeObserver for DebugObserver {
    fn observe_enter_call_frame(
        &mut self,
        arg_count: usize,
        lambda: &std::rc::Rc<Lambda>,
        call_depth: usize,
    ) {
        let cmd = self.receiver.recv();

        let prefix = if arg_count == 0 {
            "=== entering thunk "
        } else {
            "=== entering closure "
        };

        let name_str = if let Some(name) = &lambda.name {
            format!("'{}' ", name)
        } else {
            String::new()
        };

        println!(
            "{}{}in frame[{}] @ {:p} ===",
            prefix, name_str, call_depth, *lambda
        );
    }

    fn observe_exit_call_frame(&mut self, _frame_at: usize, _stack: &[tvix_eval::Value]) {
        println!("{:?}", _stack);

        // if frame in breakpoints
        // user_input(stack)
    }

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
