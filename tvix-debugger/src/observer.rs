use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

use smol_str::SmolStr;
use tvix_eval::{
    observer::RuntimeObserver,
    opcode::{CodeIdx, Op},
    value::Lambda,
    SourceCode,
};

use crate::commands::{Breakpoint, ObserverCommand, ObserverReply};

pub struct DebugObserver {
    code_path: PathBuf,
    source_code: SourceCode,
    breakpoints: Vec<Breakpoint>,
    receiver: Receiver<ObserverCommand>,
    sender: Sender<ObserverReply>,
    cur_cmd: ObserverCommand,
}

impl DebugObserver {
    pub fn new(
        code_path: PathBuf,
        source_code: SourceCode,
        breakpoints: Vec<Breakpoint>,
        receiver: Receiver<ObserverCommand>,
        sender: Sender<ObserverReply>,
    ) -> Self {
        DebugObserver {
            code_path,
            source_code,
            breakpoints,
            receiver,
            sender,
            cur_cmd: ObserverCommand::Wait,
        }
    }

    /// Handling the commands from the backend, can pause execution to wait for
    /// more user input
    fn await_command(&mut self, name: &Option<SmolStr>) {
        if self.cur_cmd != ObserverCommand::Continue || self.is_breakpoint(name) {
            let command = self.receiver.recv().unwrap();
            match command {
                ObserverCommand::Print(_smol_str) => (), // TODO: self.handle_print(smol_str, lambda),
                ObserverCommand::Break(smol_str) => self.breakpoints.push(smol_str),
                ObserverCommand::Continue | ObserverCommand::Step => self.cur_cmd = command,
                ObserverCommand::Wait => (),
            }

            //TODO: fix these replies
            // let reply = match self.cur_cmd {
            //     ObserverCommand::Wait => ObserverReply::State,
            //     ObserverCommand::Continue => ObserverReply::State,
            //     ObserverCommand::Step => ObserverReply::State,
            //     ObserverCommand::Break(_) => ObserverReply::State,
            //     ObserverCommand::Print(_) => ObserverReply::State,
            // };
            //
            // self.sender.send(reply).unwrap();
        }
    }

    fn is_breakpoint(&self, name: &Option<SmolStr>) -> bool {
        // println!("got name {:?} current name: {:?}", self.breakpoints, name);
        if let Some(name_val) = name {
            self.breakpoints.contains(name_val)
        } else {
            false
        }
    }

    fn handle_print(&self, var_name: SmolStr, lambda: &std::rc::Rc<Lambda>) {
        // TODO if the option is some, find that variable name in the lambda
        // else print all the variable names
    }
}

impl RuntimeObserver for DebugObserver {
    fn observe_enter_call_frame(
        &mut self,
        arg_count: usize,
        lambda: &std::rc::Rc<Lambda>,
        call_depth: usize,
    ) {
        self.await_command(&lambda.name);

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
        name: &str,
        _stack: &[tvix_eval::Value],
    ) {
        self.await_command(&Some(name.into()));
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
