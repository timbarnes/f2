/// Inner Interpreters
///
/// Core functions to execute specific types of objects
///
use crate::engine::{
    ABORT, BRANCH, BRANCH0, BUILTIN, BUILTIN_MASK, CONSTANT, DEFINITION, EXIT, LITERAL, NEXT,
    RET_START, STRLIT, TF, VARIABLE,
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
        let op = &self.builtins[code as usize];
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
    ///    represented by the cfa of their definition, overlaid with a flag. The interpreter calls the builtin code.
    ///    For nested definitions, the inner interpreter pushes the program counter (PC) and continues.
    ///    When the end of a definition is found, the PC is restored from the previous caller.
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
            match code {
                BUILTIN => {
                    self.msg
                        .error("i_definition", "Found BUILTIN???", Some(code));
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
                    push!(self, self.data[pc] as i64); // the string address of the data
                    pc += 1;
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
                    // see if it's a builtin:
                    let mut builtin_flag = code as usize & BUILTIN_MASK;
                    if builtin_flag != 0 {
                        builtin_flag = code as usize & !BUILTIN_MASK;
                        push!(self, builtin_flag as i64);
                        self.i_builtin();
                        pc += 1;
                    } else {
                        push!(self, pc as i64 + 1); // the return address is the next object in the list
                        self.f_to_r(); // save it on the return stack
                        pc = code as usize;
                    }
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

    /// Leave the current word *** doesn't work, because there's no way to reset the program counter from here
    pub fn i_exit(&mut self) {
        self.f_r_from();
        // pc = pop!(self) as usize;
    }

    /// Continue to the next word
    pub fn i_next(&mut self) {}

    /// f_marker <name> ( -- ) sets a location for FORGET
    ///     It creates a definition called <name> that has the effect of resetting HERE and CONTEXT       
    pub fn f_marker(&mut self) {}
}
