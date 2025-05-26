use std::sync::mpsc::{Receiver, Sender};

use smol_str::SmolStr;
use tvix_eval::{
    observer::RuntimeObserver,
    opcode::{CodeIdx, Op},
    value::Lambda,
    Value,
};

use crate::commands::{Breakpoint, ObserverCommand, ObserverReply};

// TODO: this doesn't maintain anything, need to maintain the mappings
// by myself --> hashmap<Name, Value>
struct ProgramState {
    lambda: Option<std::rc::Rc<Lambda>>,
    stack: Vec<tvix_eval::Value>,
}

pub struct DebugObserver {
    breakpoints: Vec<Breakpoint>,
    receiver: Receiver<ObserverCommand>,
    _sender: Sender<ObserverReply>,
    cur_cmd: ObserverCommand,
    cur_state: ProgramState,
}

impl DebugObserver {
    pub fn new(
        breakpoints: Vec<Breakpoint>,
        receiver: Receiver<ObserverCommand>,
        _sender: Sender<ObserverReply>,
    ) -> Self {
        DebugObserver {
            breakpoints,
            receiver,
            _sender,
            cur_cmd: ObserverCommand::Wait,
            cur_state: ProgramState {
                lambda: None,
                stack: vec![],
            },
        }
    }

    pub fn set_cmd(&mut self, command: ObserverCommand) {
        self.cur_cmd = command;
    }

    /// Handling the commands from the backend, can pause execution to wait for
    /// more user input
    pub fn await_command(&mut self, name: &Option<SmolStr>) {
        // Only stop when we hit a breakpoint || step through the program
        if self.cur_cmd != ObserverCommand::Continue || self.is_breakpoint(name) {
            let command = self.receiver.recv().unwrap();

            if self.cur_cmd == ObserverCommand::Wait && command != ObserverCommand::Launch {
                println!("Program is not running! Launch first");
                return;
            }

            match &command {
                ObserverCommand::Print(smol_str) => self.handle_print(smol_str.clone()),
                ObserverCommand::Break(smol_str) => self.handle_break(smol_str.clone()),
                ObserverCommand::Continue => self.handle_continue(),
                ObserverCommand::Step => self.handle_step(),
                ObserverCommand::Launch => self.handle_launch(),
                ObserverCommand::Wait => (),
                ObserverCommand::Done => (),
            };
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

    fn handle_launch(&mut self) {
        self.cur_cmd = ObserverCommand::Step;
    }

    fn handle_continue(&mut self) {
        self.cur_cmd = ObserverCommand::Continue;
    }

    fn handle_step(&mut self) {
        self.cur_cmd = ObserverCommand::Step;
    }

    fn handle_print(&self, var_name: SmolStr) {
        // TODO if the option is some, find that variable name in the lambda
        // else print all the variable names
        //
        if let Some(lambda) = &self.cur_state.lambda {
            if let Some(name) = &lambda.name {
                if *name == var_name {
                    // TODO: get the value here
                    println!("FOUND IT {:?}", lambda.chunk);
                }
            }
        }
        for val in &self.cur_state.stack {
            match val {
                // TODO: calling val on suspended thunks results in error
                Value::Thunk(_) => continue,
                Value::Closure(closure) => {
                    // println!("looking at: {}", val.explain());
                    if let Some(fn_name) = &closure.lambda.name {
                        if *fn_name == var_name {
                            println!("FOUND IT {}", val.explain());
                        }
                    }
                }
                // _ => println!("looking at: {}", val.explain()),
                _ => continue,
            }
        }

        println!("looking for {}", var_name);
    }

    fn handle_break(&mut self, breakpoint_name: SmolStr) {
        self.breakpoints.push(breakpoint_name);
    }
}

impl RuntimeObserver for DebugObserver {
    fn observe_enter_call_frame(
        &mut self,
        _arg_count: usize,
        lambda: &std::rc::Rc<Lambda>,
        _call_depth: usize,
    ) {
        // println!(
        //     "entering call frame: {}",
        //     (lambda.name.clone()).unwrap_or("hello".into())
        // );
        self.cur_state.lambda = Some(lambda.to_owned());
        // self.await_command(&lambda.name);
    }

    fn observe_exit_call_frame(&mut self, _frame_at: usize, stack: &[tvix_eval::Value]) {
        self.cur_state.stack = stack.to_owned();
        self.await_command(&None);
    }

    fn observe_suspend_call_frame(&mut self, _frame_at: usize, _stack: &[tvix_eval::Value]) {}

    fn observe_enter_generator(
        &mut self,
        _frame_at: usize,
        _name: &str,
        stack: &[tvix_eval::Value],
    ) {
        self.cur_state.stack = stack.to_owned();
        // self.await_command(&Some(name.into()));
    }

    fn observe_exit_generator(&mut self, _frame_at: usize, name: &str, stack: &[tvix_eval::Value]) {
        self.cur_state.stack = stack.to_owned();
        self.await_command(&Some(name.into()));
    }

    fn observe_suspend_generator(
        &mut self,
        _frame_at: usize,
        name: &str,
        stack: &[tvix_eval::Value],
    ) {
        self.cur_state.stack = stack.to_owned();
        self.await_command(&Some(name.into()));
    }

    fn observe_generator_request(&mut self, _name: &str, _msg: &tvix_eval::generators::VMRequest) {}

    fn observe_tail_call(&mut self, _frame_at: usize, _: &std::rc::Rc<Lambda>) {}

    fn observe_enter_builtin(&mut self, _name: &'static str) {}

    fn observe_exit_builtin(&mut self, _name: &'static str, _stack: &[tvix_eval::Value]) {}

    fn observe_execute_op(&mut self, _ip: CodeIdx, _: &Op, _: &[tvix_eval::Value]) {}
}
