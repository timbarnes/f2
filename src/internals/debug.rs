// Debugging help

use crate::engine::{FALSE, STACK_START, TF};
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
    /// show-stack ( -- ) turns on stack printing at the time the prompt is issued
    ///
    pub fn f_show_stack(&mut self) {
        self.show_stack = true;
    }

    /// hide-stack ( -- ) turns off stack printing at the time the prompt is issued
    ///
    pub fn f_hide_stack(&mut self) {
        self.show_stack = false;
    }

    /// DEPTH - print the number of items on the stack
    ///
    pub fn f_stack_depth(&mut self) {
        let depth = STACK_START - self.stack_ptr;
        push!(self, depth as i64);
    }

    /// dbg ( n -- ) sets the current debug level used by the message module
    ///
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

    /// u_step provides the step / trace functionality
    ///     called from inside the definition interpreter
    ///     it is driven by the STEPPER variable:
    ///     STEPPER = 0 => stepping is off
    ///     STEPPER = -1 => single step
    ///     STEPPER = 1 => trace mode, printing the stack and current word before each operation
    ///
    pub fn u_step(&mut self, address: usize, is_builtin: bool) {
        let mode = self.data[self.stepper_ptr];
        let mut c;
        match mode {
            0 => return, // stepper is off
            -1 => {
                // step mode: get a character
                print!("Step> ");
                self.f_flush();
                loop {
                    self.f_key();
                    c = pop!(self) as u8 as char;
                    if c != '\n' {
                        break;
                    }
                }
            }
            _ => {
                // trace mode
                c = 's';
            }
        }
        print!("Step> ");
        self.f_flush();
        match c {
            's' => {
                self.f_dot_s();
                if is_builtin {
                    println!(" {} ", &self.builtins[address].name);
                } else {
                    println!(" {} ", self.u_get_string(self.data[address - 1] as usize));
                }
            }
            'c' => self.data[self.stepper_ptr] = FALSE,
            '\n' | _ => println!("Stepper: 's' for show, 'c' for continue."),
        }
    }
}
