/// Inner Interpreters
///
/// Core functions to execute specific types of objects
///
use crate::engine::TF;

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
    pub fn i_builtin(&mut self, code: i64) {
        let index = self.data[code as usize] as usize;
        let op = &self.builtins[index];
        let func = op.code;
        func(self);
    }

    /// Places the address of the adjacent variable on the stack
    ///
    ///    [ index of i_variable ] [ index of builtin ] in a compiled word
    ///
    pub fn i_variable(&mut self, val: i64) {
        push!(self, val); // address of the value
    }

    /// Places the value of the adjacent constant on the stack
    ///
    ///    [ index of i_constant ] [ constant value ] in a compiled word
    ///
    pub fn i_constant(&mut self, val: i64) {
        push!(self, self.data[val as usize]);
    }

    /// Places the number in data[d] on the stack
    ///
    ///    [ index of i_literal ] [ number ] in a compiled word
    ///
    pub fn i_literal(&mut self, lit: i64) {
        push!(self, self.data[lit as usize]);
    }

    /// Places the address (in string space) of the adjacent string on the stack
    ///
    ///    [ i_string ] [ index into string space ] in a compiled word
    ///
    pub fn i_string(&mut self, ptr: i64) {
        push!(self, ptr);
    }

    /// Loops through the adjacent definition, running their inner interpreters
    ///
    ///    [ index of i_definition ] [ sequence of compiled words ]
    ///
    pub fn i_definition(&mut self, def: i64) {}

    /// Unconditional branch, used by condition and loop structures
    pub fn i_branch(&mut self, dist: i64) {}

    /// Branch if zero, used by condition and loop structures
    pub fn i_branch0(&mut self, dist: i64) {}

    /// Force an abort
    pub fn i_abort(&mut self, dist: i64) {}

    /// Leave the current word
    pub fn i_exit(&mut self, dist: i64) {}

    /// Continue to the next word
    pub fn i_next(&mut self, dist: i64) {}
}
