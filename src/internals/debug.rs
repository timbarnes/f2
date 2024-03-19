// Debugging help

use crate::engine::{ADDRESS_MASK, BUILTIN_MASK, FALSE, RET_START, STACK_START, TF,
    VARIABLE, CONSTANT, LITERAL, STRLIT, DEFINITION, BRANCH, BRANCH0, ABORT, EXIT, BREAK};
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
        //$self.data[$self.stack_ptr] = 999999;
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
    ///     STEPPER >0 => trace mode, printing the stack and current word before each operation.
    ///                   value of stepper indicates how many levels deep to trace
    /// 
    ///     pc is the program counter, which represents the address of the cell being executed.
    ///
    pub fn u_step(&mut self, pc: usize) {
        let mode = self.data[self.stepper_ptr];
        if mode == 0 { return };

        let mut contents = self.data[pc] as usize;
        let is_builtin = if contents & BUILTIN_MASK != 0 { true } else { false };
        contents &= ADDRESS_MASK;
        let mut c;

        // Indent based on return stack depth
        let depth = RET_START - self.return_ptr;
        if depth > mode as usize { return; }
        print!("{depth}");
        for _i in 1..depth { print!(" "); }  
        self.f_dot_s();   
        match mode {
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
        match c {
            't' => self.data[self.stepper_ptr] = 1,
            's' => {
                match contents as i64 {
                    VARIABLE | CONSTANT | DEFINITION => println!(" {} ", self.u_get_string(self.data[pc - 1] as usize)),
                    LITERAL => println!(" {} ", self.data[pc + 1]),
                    STRLIT => println!(" {} ", self.u_get_string(self.data[pc + 1] as usize)),
                    BRANCH => println!(" BRANCH:{}", self.data[pc + 1]),
                    BRANCH0 => println!(" BRANCH0:{}", self.data[pc + 1]),
                    ABORT => println!(" ABORT "),
                    EXIT => println!(" EXIT "),
                    BREAK => println!(" BREAK "),
                    _ => {
                        if is_builtin {
                            println!(" {} ", &self.builtins[contents].name);
                        } else {
                            println!(" ->{}", self.u_get_string(self.data[contents - 1] as usize));
                        }
                    }
                } 
            }
            'o' => self.data[self.stepper_ptr] = 0,
            '\n' | _ => println!("Stepper: 's' for show, 't' for trace, 'o' for off."),
        }
    }
}
