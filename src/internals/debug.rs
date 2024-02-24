// Debugging help

use crate::engine::{STACK_START, TF};
use crate::messages::DebugLevel;

macro_rules! push {
    ($self:ident, $val:expr) => {
        $self.stack_ptr -= 1;
        $self.data[$self.stack_ptr] = $val;
    };
}

impl TF {
    pub fn f_show_stack(&mut self) {
        self.show_stack = true;
    }

    pub fn f_hide_stack(&mut self) {
        self.show_stack = false;
    }

    /// WORDS ( -- ) Print a list of all defined words
    ///              Includes definitions, builtins, variables and constants
    pub fn f_words(&mut self) {
        // walk the definition linked list and print each entry's name
        println!("words - not implemented");
    }

    /// SEE-ALL ( -- ) Show decompiled versions of all the defined words
    pub fn f_see_all(&mut self) {
        println!("see-all - not implemented")
    }

    /// DEPTH - print the number of items on the stack
    pub fn f_stack_depth(&mut self) {
        let depth = STACK_START - self.stack_ptr;
        push!(self, depth as i64);
    }

    pub fn f_dbg(&mut self) {
        match self.stack.pop() {
            Some(0) => self.msg.set_level(DebugLevel::Error),
            Some(1) => self.msg.set_level(DebugLevel::Warning),
            Some(2) => self.msg.set_level(DebugLevel::Info),
            _ => self.msg.set_level(DebugLevel::Debug),
        }
    }

    pub fn f_debuglevel(&mut self) {
        println!("DebugLevel is {:?}", self.msg.get_level());
    }

    pub fn f_step_on(&mut self) {
        self.step_mode = true;
    }

    pub fn f_step_off(&mut self) {
        self.step_mode = false;
    }
}
