use std::sync::mpsc::{Receiver, Sender};

use tvix_eval::{
    observer::RuntimeObserver,
    opcode::{CodeIdx, Op},
    value::Lambda,
};

use crate::commands::{Breakpoint, ObserverCommand, ObserverReply};

pub struct DebugObserver {
    breakpoints: Vec<Breakpoint>,
    receiver: Receiver<ObserverCommand>,
    sender: Sender<ObserverReply>,
    cur_cmd: ObserverCommand,
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
            cur_cmd: ObserverCommand::Wait,
        }
    }

    fn handle_command(&mut self, command: ObserverCommand) {
        match command {
            ObserverCommand::Wait => (),
            ObserverCommand::Break(smol_str) => self.breakpoints.push(smol_str),
            ObserverCommand::Continue | ObserverCommand::Step => self.cur_cmd = command,
        }
    }

    fn is_breakpoint(&self, lambda: &std::rc::Rc<Lambda>) -> bool {
        // TODO: implement this
        // look up the span
        // check if it's in breakpoints
        println!(
            "got name {:?} current name: {:?}",
            self.breakpoints, lambda.name
        );
        if let Some(name) = &lambda.name {
            self.breakpoints.contains(name)
        } else {
            false
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
        if self.cur_cmd != ObserverCommand::Continue || self.is_breakpoint(lambda) {
            let cmd = self.receiver.recv().unwrap();
            self.handle_command(cmd);

            //TODO: fix these replies
            let reply = match self.cur_cmd {
                ObserverCommand::Wait => ObserverReply::State,
                ObserverCommand::Continue => ObserverReply::State,
                ObserverCommand::Step => ObserverReply::State,
                ObserverCommand::Break(_) => ObserverReply::State,
            };

            self.sender.send(reply).unwrap();
        }

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
