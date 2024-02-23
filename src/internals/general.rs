// General-purpose builtin words

use crate::engine::{STACK_START, TF};

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
        $self.stack_ptr += 1;
        $self.data[$self.stack_ptr - 1]
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
            push!(self, $expression(x));
        }
    };
}
/* macro_rules! pop1 {
    ($self:ident, $word:expr, $code:expr) => {
        if let Some(x) = $self.pop_one(&$word) {
            $code(x);
        }
    };
} */

impl TF {
    pub fn f_plus(&mut self) {
        pop2_push1!(self, "+", |a, b| a + b);
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
        self.stack.push(-1);
    }

    pub fn f_false(&mut self) {
        self.stack.push(0);
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
        self.stack.clear()
    }

    pub fn f_bye(&mut self) {
        self.set_exit_flag();
    }

    pub fn f_dup(&mut self) {
        if let Some(top) = self.stack.last() {
            self.stack.push(*top);
        } else {
            self.msg
                .warning("DUP", "Error - DUP: Stack is empty.", None::<bool>);
        }
    }
    pub fn f_drop(&mut self) {
        pop!(self);
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

    pub fn f_to_r(&mut self) {
        if stack_ok!(self, 1, ">r") {
            let value = pop!(self);
            self.return_ptr -= 1;
            self.data[self.return_ptr] = value;
        }
    }

    pub fn f_r_from(&mut self) {
        if let Some(n) = self.return_stack.pop() {
            self.stack.push(n);
        } else {
            self.msg.error("r>", "Return stack underflow", None::<bool>);
        }
    }

    pub fn f_r_get(&mut self) {
        if self.return_stack.len() > 0 {
            self.stack.push(*self.return_stack.last().unwrap());
        } else {
            self.msg.error("r@", "Return stack underflow", None::<bool>);
        }
    }

    pub fn f_i(&mut self) {
        // print the index of the current top-level loop
        if self.return_stack.is_empty() {
            self.msg.warning(
                "I",
                "Can only be used inside a DO .. LOOP structure",
                None::<bool>,
            );
        } else {
            self.stack
                .push(self.return_stack[self.return_stack.len() - 1]);
        }
    }

    pub fn f_j(&mut self) {
        // print the index of the current second-level (outer) loop
        if self.return_stack.len() < 2 {
            self.msg.warning(
                "I",
                "Can only be used inside a nested DO .. LOOP structure",
                None::<bool>,
            );
        } else {
            self.stack
                .push(self.return_stack[self.return_stack.len() - 2]);
        }
    }

    pub fn f_recurse(&mut self) {
        self.set_program_counter(0);
    }
}
