// General-purpose builtin words

use crate::engine::{FALSE, STACK_START, TF, TRUE};

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

macro_rules! pop2_push1 {
    // Helper macro
    ($self:ident, $word:expr, $expression:expr) => {
        if stack_ok!($self, 2, $word) {
            let j = pop!($self);
            let k = pop!($self);
            push!($self, $expression(k, j));
        }
    };
}
macro_rules! pop1_push1 {
    // Helper macro
    ($self:ident, $word:expr, $expression:expr) => {
        if stack_ok!($self, 1, $word) {
            let x = pop!($self);
            push!($self, $expression(x));
        }
    };
}

pub fn u_is_integer(s: &str) -> bool {
    s.parse::<i64>().is_ok()
}

pub fn u_is_float(s: &str) -> bool {
    s.parse::<f64>().is_ok()
}

impl TF {
    pub fn f_plus(&mut self) {
        if stack_ok!(self, 2, "+") {
            let a = pop!(self);
            let b = pop!(self);
            push!(self, a + b);
        };
    }

    pub fn f_minus(&mut self) {
        pop2_push1!(self, "-", |a, b| a - b);
    }

    pub fn f_times(&mut self) {
        pop2_push1!(self, "*", |a, b| a * b);
    }

    pub fn f_divide(&mut self) {
        pop2_push1!(self, "/", |a, b| a / b);
    }

    pub fn f_mod(&mut self) {
        pop2_push1!(self, "mod", |a, b| a % b);
    }

    pub fn f_less(&mut self) {
        pop2_push1!(self, "<", |a, b| if a < b { -1 } else { 0 });
    }

    pub fn f_true(&mut self) {
        push!(self, TRUE);
    }

    pub fn f_false(&mut self) {
        push!(self, FALSE);
    }

    pub fn f_equal(&mut self) {
        pop2_push1!(self, "=", |a, b| if a == b { -1 } else { 0 });
    }

    pub fn f_0equal(&mut self) {
        pop1_push1!(self, "0=", |a| if a == 0 { -1 } else { 0 });
    }

    pub fn f_0less(&mut self) {
        pop1_push1!(self, "0<", |a| if a < 0 { -1 } else { 0 });
    }

    pub fn f_clear(&mut self) {
        self.stack_ptr = STACK_START;
    }

    pub fn f_bye(&mut self) {
        self.set_exit_flag();
    }

    pub fn f_dup(&mut self) {
        if stack_ok!(self, 1, "dup") {
            let top = top!(self);
            push!(self, top);
        }
    }
    pub fn f_drop(&mut self) {
        if stack_ok!(self, 1, "drop") {
            pop!(self);
        }
    }
    pub fn f_swap(&mut self) {
        if stack_ok!(self, 2, "swap") {
            let a = pop!(self);
            let b = pop!(self);
            push!(self, a);
            push!(self, b);
        }
    }
    pub fn f_over(&mut self) {
        if stack_ok!(self, 2, "over") {
            let first = pop!(self);
            let second = pop!(self);
            push!(self, second);
            push!(self, first);
            push!(self, second);
        }
    }
    pub fn f_rot(&mut self) {
        if stack_ok!(self, 3, "rot") {
            let first = pop!(self);
            let second = pop!(self);
            let third = pop!(self);
            push!(self, second);
            push!(self, first);
            push!(self, third);
        }
    }

    pub fn f_and(&mut self) {
        if stack_ok!(self, 2, "and") {
            let a = pop!(self);
            let b = pop!(self);
            push!(self, a & b);
        }
    }

    pub fn f_or(&mut self) {
        if stack_ok!(self, 2, "or") {
            let a = pop!(self);
            let b = pop!(self);
            push!(self, (a as usize | b as usize) as i64);
        }
    }

    pub fn f_get(&mut self) {
        if stack_ok!(self, 1, "@") {
            let addr = pop!(self);
            push!(self, self.data[addr as usize]);
        }
    }

    pub fn f_store(&mut self) {
        if stack_ok!(self, 2, "!") {
            let addr = pop!(self);
            let value = pop!(self);
            self.data[addr as usize] = value;
        }
    }

    /// >r ( n -- ) Pops the stack, placing the value on the return stack
    pub fn f_to_r(&mut self) {
        if stack_ok!(self, 1, ">r") {
            let value = pop!(self);
            self.return_ptr -= 1;
            self.data[self.return_ptr] = value;
        }
    }

    /// r> ( -- n ) Pops the return stack, pushing the value to the calculation stack
    pub fn f_r_from(&mut self) {
        push!(self, self.data[self.return_ptr]);
        self.return_ptr += 1;
    }

    /// r@ ( -- n ) Gets the top value from the return stack, pushing the value to the calculation stack
    pub fn f_r_get(&mut self) {
        push!(self, self.data[self.return_ptr]);
    }

    /// i ( -- n ) Pushes the current loop index to the calculation stack
    pub fn f_i(&mut self) {
        push!(self, self.data[self.return_ptr]);
    }

    /// j ( -- n ) Pushes the second level (outer) loop index to the calculation stack
    pub fn f_j(&mut self) {
        push!(self, self.data[self.return_ptr + 1]);
    }

    /// recurse ( -- ) Branches to the beginning of the current word.
    ///     Branch distance needs to be calculated somehow... ***
    pub fn f_recurse(&mut self) {
        self.msg.error("recurse", "Not implemented", None::<bool>);
    }

    /// s-copy (s-from s-to -- s-to )
    pub fn f_s_copy(&mut self) {
        if stack_ok!(self, 3, "s-copy") {
            let dest = pop!(self) as usize;
            let result_ptr = dest as i64;
            let source = pop!(self) as usize;
            let length = self.strings[source] as u8 as usize + 1;
            let mut i = 0;
            while i < length {
                self.strings[dest + i] = self.strings[source + i];
                i += 1;
            }
            self.data[self.string_ptr] += length as i64;
            push!(self, result_ptr);
        }
    }
    /// s-create ( s-from -- s-to ) copies a counted string into the next empty space, updating the free space pointer
    pub fn f_s_create(&mut self) {
        if stack_ok!(self, 1, "s-create") {
            let source = top!(self) as usize;
            let length = self.strings[source] as usize;
            let dest = self.data[self.string_ptr];
            push!(self, dest); // destination
            self.f_s_copy();
            self.data[self.string_ptr] += length as i64 + 1;
            push!(self, dest);
        }
    }

    /// s-parse ( c -- ) Get a c-delimited string from TIB, and place in TMP
    pub fn f_s_parse(&mut self) {
        let delim = pop!(self);
        // need to call (parse) to get the string directly from the TIB
        push!(self, self.data[self.tib_ptr] + self.data[self.tib_in_ptr]);
        push!(
            self,
            self.data[self.tib_size_ptr] - self.data[self.tib_in_ptr] + 1
        );
        push!(self, delim);
        self.f_parse_p();
        let delta = pop!(self);
        let length = pop!(self);
        let addr = pop!(self);
        if length > 0 {
            self.u_str_copy(
                (addr + delta) as usize,
                self.data[self.tmp_ptr] as usize,
                length as usize,
                false,
            );
        }
    }

    /// .s" (s -- ) Print a string using the string address on the stack
    pub fn f_dot_s_quote(&mut self) {
        if stack_ok!(self, 1, ".s") {
            let addr = pop!(self) as usize;
            let length = self.strings[addr] as usize + 1;
            let mut i = 1;
            while i <= length {
                print!("{}", self.strings[addr + i]);
                i += 1;
            }
        }
    }
}
