use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::mpsc::{Receiver, Sender},
};

use codemap::Span;
use smol_str::SmolStr;
use tvix_eval::{
    chunk::SourceSpan,
    observer::RuntimeObserver,
    opcode::{CodeIdx, Op},
    value::{self, Lambda},
    SourceCode, Value,
};

use crate::commands::{Breakpoint, ObserverCommand, ObserverReply};

// TODO: this doesn't maintain anything, need to maintain the mappings
// by myself --> hashmap<Name, Value>
struct ProgramState {
    lambda: Option<std::rc::Rc<Lambda>>,
    stack: Vec<tvix_eval::Value>,
}

struct BreakPoints {
    breakpoints: HashSet<usize>,
    source_id: HashMap<SourceSpan, usize>,
}

impl Default for BreakPoints {
    fn default() -> Self {
        BreakPoints {
            breakpoints: HashSet::default(),
            source_id: HashMap::default(),
        }
    }
}

impl Display for ProgramState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = write!(f, "cur lambda:\n {:?}\n", self.lambda);
        let _ = write!(f, "cur stack:\n {:?}\n", self.stack);
        Ok(())
    }
}

pub struct DebugObserver {
    code: SourceCode,
    breakpoints: BreakPoints,
    receiver: Receiver<ObserverCommand>,
    _sender: Sender<ObserverReply>,
    cur_cmd: ObserverCommand,
    cur_state: ProgramState,
}

impl DebugObserver {
    pub fn new(
        code: SourceCode,
        receiver: Receiver<ObserverCommand>,
        _sender: Sender<ObserverReply>,
    ) -> Self {
        let breakpoints = BreakPoints::default();
        DebugObserver {
            code,
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
    pub fn handle_command(&mut self) {
        // Only stop when we hit a breakpoint || step through the program
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

    // fn print_current_source(&self) {
    //     let lambda = self.cur_state.lambda.as_ref().unwrap();
    //     let spans = &lambda.chunk.spans;
    //
    //     println!("Lambda chunks spans");
    //     for span in spans {
    //         let snippet = self.code.source_slice(span.span);
    //         let line = self.code.get_line(span.span);
    //
    //         println!("snippet: {}, on line {}", snippet, line);
    //     }
    //
    //     // println!("Constants");
    //     // if let Some(source_span) = spans
    //     //     .iter()
    //     //     .filter(|s| s.start <= self.cur_state.ip)
    //     //     .max_by_key(|s| s.start)
    //     // {
    //     //     let span = source_span.span;
    //     //     let codemap = self.code.codemap();
    //     //
    //     //     // Look up the file containing the span
    //     //     let file = codemap.find_file(span.low());
    //     //
    //     //     // Ask the file to extract the source snippet for the span
    //     //     let snippet = file.source_slice(span);
    //     //
    //     //     println!("IP: {}, Cur snippet: {}", self.cur_state.ip, snippet);
    //     // }
    // }

    fn is_breakpoint(&mut self, cur_span: &Span) -> bool {
        // println!(
        //     "Looking for span {:?}\n all spans: {:?}\n",
        //     cur_span, &self.breakpoints.breakpoints
        // );
        let line = &self.code.get_line(*cur_span);
        let found = self.breakpoints.breakpoints.contains(line);
        if found {
            self.breakpoints.breakpoints.remove(&line);
            self.cur_cmd = ObserverCommand::Step;
        }
        found
    }

    fn handle_launch(&mut self) {
        self.cur_cmd = ObserverCommand::Step;
    }

    fn handle_continue(&mut self) {
        self.cur_cmd = ObserverCommand::Continue;
    }

    fn handle_step(&mut self) {
        self.cur_cmd = ObserverCommand::Step;
        // self.print_current_source();
        // println!("{}", self.cur_state);
    }

    fn handle_print(&self, var_name: SmolStr) {
        // TODO if the option is some, find that variable name in the lambda
        // else print all the variable names
        if let Some(lambda) = &self.cur_state.lambda {
            if let Some(name) = &lambda.name {
                if *name == var_name {
                    // TODO: get the value here
                    // println!("FOUND IT {:?}", lambda.chunk);
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
                            // println!("FOUND IT {}", val.explain());
                        }
                    }
                }
                // _ => println!("looking at: {}", val.explain()),
                _ => continue,
            }
        }

        // println!("looking for {}", var_name);
    }

    fn handle_break(&mut self, breakpoint: Breakpoint) {
        let line = match breakpoint {
            Breakpoint::Line(line) => line,
            Breakpoint::FileLine { file: _, line } => line,
        };

        let _ = self.breakpoints.breakpoints.insert(line);
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
    }

    fn observe_exit_call_frame(&mut self, _frame_at: usize, stack: &[tvix_eval::Value]) {
        // println!("{}", self.cur_state);
        self.cur_state.stack = stack.to_owned();
    }

    fn observe_suspend_call_frame(&mut self, _frame_at: usize, _stack: &[tvix_eval::Value]) {}

    fn observe_enter_generator(
        &mut self,
        _frame_at: usize,
        _name: &str,
        stack: &[tvix_eval::Value],
    ) {
        self.cur_state.stack = stack.to_owned();
    }

    fn observe_exit_generator(
        &mut self,
        _frame_at: usize,
        _name: &str,
        stack: &[tvix_eval::Value],
    ) {
        self.cur_state.stack = stack.to_owned();
    }

    fn observe_suspend_generator(
        &mut self,
        _frame_at: usize,
        _name: &str,
        stack: &[tvix_eval::Value],
    ) {
        self.cur_state.stack = stack.to_owned();
    }

    fn observe_generator_request(&mut self, _name: &str, _msg: &tvix_eval::generators::VMRequest) {}

    fn observe_tail_call(&mut self, _frame_at: usize, _: &std::rc::Rc<Lambda>) {}

    fn observe_enter_builtin(&mut self, _name: &'static str) {}

    fn observe_exit_builtin(&mut self, _name: &'static str, _stack: &[tvix_eval::Value]) {}

    fn observe_execute_op(&mut self, span: &Span, _ip: CodeIdx, op: &Op, _: &[tvix_eval::Value]) {
        let pause = if self.cur_cmd != ObserverCommand::Continue || self.is_breakpoint(span) {
            true
        } else {
            false
        };

        if self.cur_cmd == ObserverCommand::Step {
            // let snippet = self.code.source_slice(*span);
            // let line = self.code.get_line(*span);
            //
            // println!("snippet: {}, on line {}", snippet, line);
            let name = if let Some(lambda) = &self.cur_state.lambda {
                lambda.name.clone().unwrap_or("anonymous".into())
            } else {
                "anonymous lambda".into()
            };
            println!("Lambda: {}, Cur_op: {:?}", name, op);
        }
        if pause {
            self.handle_command();
        }
    }
}
