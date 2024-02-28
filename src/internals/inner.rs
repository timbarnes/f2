/// Inner Interpreters
///
/// Core functions to execute specific types of objects
///
use crate::engine::{
    ABORT, BRANCH, BRANCH0, BUILTIN, CONSTANT, DEFINITION, EXIT, LITERAL, NEXT, RET_START, STRLIT,
    TF, VARIABLE,
};

macro_rules! pop {
    ($self:ident) => {{
        let r = $self.data[$self.stack_ptr];
        $self.data[$self.stack_ptr] = 999999;
        $self.stack_ptr += 1;
        r
    }};
}
macro_rules! top {
    ($self:ident) => {{
        $self.data[$self.stack_ptr]
    }};
}
macro_rules! push {
    ($self:ident, $val:expr) => {
        $self.stack_ptr -= 1;
        $self.data[$self.stack_ptr] = $val;
    };
}

impl TF {
    /// Executes the builtin at the next address in DATA
    ///
    ///    [ index of i_builtin ] [ index of builtin ] in a compiled word
    ///
    pub fn i_builtin(&mut self) {
        let code = pop!(self);
        let index = self.data[code as usize] as usize;
        let op = &self.builtins[index];
        let func = op.code;
        func(self);
    }

    /// Places the address of the adjacent variable on the stack
    ///
    ///    [ index of i_variable ] [ index of builtin ] in a compiled word
    ///
    pub fn i_variable(&mut self) {
        let val = pop!(self);
        push!(self, val); // address of the value
    }

    /// Places the value of the adjacent constant on the stack
    ///
    ///    [ index of i_constant ] [ constant value ] in a compiled word
    ///
    pub fn i_constant(&mut self) {
        let val = pop!(self);
        push!(self, self.data[val as usize]);
    }

    /// Places the number in data[d] on the stack
    ///
    ///    [ index of i_literal ] [ number ] in a compiled word
    ///
    pub fn i_literal(&mut self) {} // Number is already on the stack

    /// Places the address (in string space) of the adjacent string on the stack
    ///
    ///    [ i_string ] [ index into string space ] in a compiled word
    ///
    pub fn i_strlit(&mut self) {} // Address is already on the stack

    /// i_definition ( cfa -- ) Loops through the adjacent definition, running their inner interpreters
    ///
    ///    [ index of i_definition ] [ sequence of compiled words ]
    ///
    ///    A program counter is used to step through the entries in the definition.
    ///    Each entry is one or two cells, and may be an inner interpreter code, with or without an argument,
    ///    or a defined word. For space efficiency, builtin words and user defined (colon) words are
    ///    represented by the cfa of their definition. The interpreter calls the builtin code.
    ///    For nested definitions, the inner interpreter pushes the program counter (PC) and continues.
    ///    When the end of a definition is found, the PC is restored from the previous caller.
    ///    Not sure how we know we're done and ready to return to the command line. ***
    ///
    ///    Most data is represented by an address, so self.data[pc] is the cfa of the word referenced.
    ///    Each operation advances the pc to the next token.
    ///
    pub fn i_definition(&mut self) {
        let mut pc = pop!(self) as usize; // This is the start of the definition: first word after the inner interpreter opcode
        push!(self, 0); // this is how we know when we're done
        self.f_to_r();
        loop {
            // each time round the loop should be one word
            if pc == 0 || self.get_abort_flag() {
                self.return_ptr = RET_START; // clear the return stack
                return; // we've completed the last exit or encountered an error
            }
            let code = self.data[pc];
            // println!("{code}");
            match code {
                BUILTIN => {
                    let index = self.data[pc + 1] as usize;
                    let op = &self.builtins[index];
                    let func = op.code;
                    func(self);
                    // return
                    self.f_r_from();
                    pc = pop!(self) as usize;
                }
                VARIABLE => {
                    // this means we've pushed into a variable and are seeing the inner interpreter
                    pc += 1;
                    push!(self, pc as i64); // the address of the variable's data
                    self.f_r_from();
                    pc = pop!(self) as usize;
                }
                CONSTANT => {
                    pc += 1;
                    push!(self, self.data[pc]); // the value of the constant
                    self.f_r_from();
                    pc = pop!(self) as usize;
                }
                LITERAL => {
                    pc += 1;
                    push!(self, self.data[pc]); // the data stored in the current definition
                    pc += 1;
                }
                STRLIT => {
                    pc += 1;
                    push!(self, pc as i64); // the string address of the data
                    self.f_r_from();
                    pc = pop!(self) as usize;
                }
                DEFINITION => {
                    pc += 1;
                    // Continue to work through the definition
                    // at the end, EXIT will pop back to the previous definition
                }
                BRANCH => {
                    // Unconditional jump based on self.data[pc + 1]
                    pc += 1;
                    let offset = self.data[pc];
                    if offset < 0 {
                        pc -= offset.abs() as usize;
                    } else {
                        pc += offset as usize;
                    }
                    // pc += 1; // skip over the offset
                }
                BRANCH0 => {
                    pc += 1;
                    if pop!(self) == 0 {
                        let offset = self.data[pc];
                        if offset < 0 {
                            pc -= offset.abs() as usize;
                        } else {
                            pc += offset as usize;
                        }
                    } else {
                        pc += 1; // skip over the offset
                    }
                }
                ABORT => {
                    self.f_abort();
                    break;
                }
                EXIT => {
                    // Current definition is finished, so pop the PC from the return stack
                    self.f_r_from();
                    pc = pop!(self) as usize;
                }
                NEXT => self.i_next(),
                _ => {
                    // we have a word address
                    push!(self, pc as i64 + 1); // the return address is the next object in the list
                    self.f_to_r(); // save it on the return stack
                    pc = code as usize;
                }
            }
        }
    }

    /// Unconditional branch, used by condition and loop structures
    pub fn i_branch(&mut self) {}

    /// Branch if zero, used by condition and loop structures
    pub fn i_branch0(&mut self) {}

    /// Force an abort
    pub fn i_abort(&mut self) {}

    /// Leave the current word
    pub fn i_exit(&mut self) {}

    /// Continue to the next word
    pub fn i_next(&mut self) {}

    /// f_marker <name> ( -- ) sets a location for FORGET
    ///     It creates a definition called <name> that has the effect of resetting HERE and CONTEXT       
    pub fn f_marker(&mut self) {}

    /// f_if ( b -- ) if the top of stack is TRUE, continue, otherwise branch to ELSE; IMMEDIATE
    ///     Implemented by compiling BRANCH0 and an offset word, putting the offset word's address on the return stack.
    pub fn f_if(&mut self) {
        // need to know the current address where the definition is happening. ***
        // We should be able to use HERE
        push!(self, BRANCH0);
        self.f_comma();
        push!(self, self.data[self.here_ptr]);
        self.f_to_r();
        push!(self, 0); // placeholder
        self.f_comma();
    }

    /// f_else ( -- ) branch to THEN; IMMEDIATE
    ///     Compile time: Compiles BRANCH0 and an offset word, putting the offset word's address on the return stack.
    ///     Resolves the address on the return stack and stores into IF's branch offset.
    pub fn f_else(&mut self) {
        let here = self.data[self.here_ptr];
        self.f_r_from();
        let there = pop!(self);
        self.data[there as usize] = here - there + 2; // to skip the branch
        push!(self, BRANCH);
        self.f_comma();
        push!(self, self.data[self.here_ptr]);
        self.f_to_r();
        push!(self, 0);
        self.f_comma();
    }

    /// f_then ( -- ) no execution semantics; IMMEDIATE
    ///     Compile time: Resolves the address on the stack, storing it into IF or ELSE's branch offset.
    ///                   Compiles a BRANCH and pushes it's offset address on the return stack.
    pub fn f_then(&mut self) {
        let here = self.data[self.here_ptr];
        self.f_r_from();
        let there = pop!(self);
        // println!("Here:{} There:{}", here, there);
        self.data[there as usize] = here - there;
    }

    /// f_for ( -- ) no execution semantics; IMMEDIATE
    ///     Compile time: Compiles a >R and puts the pc on the compute stack.
    pub fn f_for(&mut self) {
        push!(self, self.data[self.here_ptr]); // so NEXT can calculate the BRANCH0
        push!(self, 278); // *** hardwired address of >R !!! Not good !!!
        self.f_comma();
    }

    /// f_next ( -- ) decrement loop counter; if <= 0, continue; otherwise push loop counter and branch back; IMMEDIATE
    ///     Compile time: Resolves the address on the stack, storing it into FOR's branch offset.
    pub fn f_next(&mut self) {
        push!(self, 282); // R>
        self.f_comma();
        push!(self, 190); // DUP
        self.f_comma();
        push!(self, LITERAL);
        self.f_comma();
        push!(self, 1);
        self.f_comma();
        push!(self, 102); // -
        self.f_comma();
        push!(self, BRANCH0);
        self.f_comma();
        let here = self.data[self.here_ptr];
        let there = pop!(self);
        push!(self, there - here);
        self.f_comma();
    }
}
