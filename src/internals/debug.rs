// Debugging help

use crate::engine::{STACK_START, TF};
use crate::messages::DebugLevel;

macro_rules! stack_ok {
    ($self:ident, $n: expr, $caller: expr) => {
        if $self.stack_ptr <= STACK_START - $n {
            true
        } else {
            $self.msg.error($caller, "Stack underflow", None::<bool>);
            $self.f_abort();
            false
        }
    };
}
macro_rules! pop {
    ($self:ident) => {{
        let r = $self.data[$self.stack_ptr];
        $self.data[$self.stack_ptr] = 999999;
        $self.stack_ptr += 1;
        r
    }};
}

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

    /// DEPTH - print the number of items on the stack
    pub fn f_stack_depth(&mut self) {
        let depth = STACK_START - self.stack_ptr;
        push!(self, depth as i64);
    }

    pub fn f_dbg(&mut self) {
        if stack_ok!(self, 1, "dbg") {
            match pop!(self) {
                0 => self.msg.set_level(DebugLevel::Error),
                1 => self.msg.set_level(DebugLevel::Warning),
                2 => self.msg.set_level(DebugLevel::Info),
                _ => self.msg.set_level(DebugLevel::Debug),
            }
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
