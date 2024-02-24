// Compiler and Interpreter

use crate::engine::{
    ADDRESS_MASK, BUILTIN, CONSTANT, DEFINITION, FALSE, IMMEDIATE_MASK, LITERAL, STACK_START,
    STRING, TF, TRUE, VARIABLE,
};
use crate::internals::general::u_is_integer;

macro_rules! stack_ok {
    ($self:ident, $n: expr, $caller: expr) => {
        if $self.stack_ptr <= STACK_START - $n {
            $self.f_dot_s();
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

impl TF {
    /// immediate - sets the immediate flag on the most recently defined word
    /// Context pointer links to the most recent name field
    pub fn f_immediate(&mut self) {
        let mut mask = self.data[self.context_ptr] as usize;
        mask |= IMMEDIATE_MASK;
        self.data[self.context_ptr] = mask as i64;
    }

    /// [  Install $INTERPRET in 'EVAL
    pub fn f_lbracket(&mut self) {
        self.set_compile_mode(false);
    }

    /// ]  Install $COMPILE in 'EVAL   
    pub fn f_rbracket(&mut self) {
        self.set_compile_mode(true);
    }

    pub fn f_abort(&mut self) {
        // empty the stack, reset any pending operations, and return to the prompt
        self.msg
            .warning("ABORT", "Terminating execution", None::<bool>);
        self.stack.clear();
        self.set_abort_flag(true);
    }

    pub fn f_quit(&mut self) {
        self.return_stack.clear();
        self.set_program_counter(0);
        self.f_abort();
        loop {
            if self.should_exit() {
                break;
            } else {
                self.f_query();
                self.f_dot_s();
                self.f_eval(); // interpret the contents of the line
                println!("ok");
            }
        }
    }

    /// EXECUTE ( cfa -- ) interpret a word with addr on the stack
    /// stack value is the address of an inner interpreter
    pub fn f_execute(&mut self) {
        if stack_ok!(self, 1, "execute") {
            // call the appropriate inner interpreter
            let xt = pop!(self);
            match self.data[xt as usize] {
                BUILTIN => self.i_builtin(xt + 1),
                VARIABLE => self.i_variable(xt + 1),
                CONSTANT => self.i_constant(xt + 1),
                LITERAL => self.i_literal(xt + 1),
                STRING => self.i_string(xt + 1),
                DEFINITION => self.i_definition(xt + 1),
                _ => self
                    .msg
                    .error("execute", "Unknown inner interpreter", Some(xt)),
            }
        }
    }

    /// EVAL ( -- ) Interprets a line of tokens from TIB
    //             self.f_find(); // (s -- nfa, cfa, T | s F )
    pub fn f_eval(&mut self) {
        loop {
            self.f_text(); //  ( -- b u ) get a token
            let len = pop!(self);
            if len == FALSE {
                // u = 0 means EOL
                pop!(self); // lose the text pointer
                break;
            } else {
                // we have a token
                if self.get_compile_mode() {
                    self.f_d_compile(); // ( s -- )
                } else {
                    self.f_d_interpret(); // ( s -- )
                }
            }
        }
    }

    /// $COMPILE ( s -- ) compiles a token whose string address is on the stack
    ///            If not a word, try to convert to a number
    ///            If not a number, ABORT.
    pub fn f_d_compile(&mut self) {
        if stack_ok!(self, 1, "$compile") {
            let s_addr = top!(self);
            if s_addr != 0 {
                self.f_find(); // (s -- nfa, cfa, T | s F )
                if pop!(self) == TRUE {
                    // we found a word
                    self.f_comma(); // uses the cfa on the stack
                    pop!(self); // throw away the nfa
                }
            } else {
                self.f_number_q();
                if pop!(self) == TRUE {
                    self.f_literal(); // compile the literal
                } else {
                    pop!(self); // lose the failed number
                    let word = &self.u_get_string(self.pad_ptr);
                    self.msg
                        .warning("$interpret", "token not recognized", Some(word));
                }
            }
        }
    }

    /// $INTERPRET ( s -- ) executes a token whose string address is on the stack.
    ///            If not a word, try to convert to a number
    ///            If not a number, ABORT.
    pub fn f_d_interpret(&mut self) {
        if stack_ok!(self, 1, "$interpret") {
            self.f_find(); // (s -- nfa, cfa, T | s F )
            if pop!(self) == TRUE {
                // we have a definition
                let cfa = pop!(self);
                let nfa = pop!(self);
                if (nfa as usize | IMMEDIATE_MASK) != 0 {
                    self.f_d_interpret();
                } else {
                    push!(self, cfa);
                    self.f_execute();
                }
            } else {
                // try number?
                self.f_number_q(); // ( s -- n T | a F )
                if pop!(self) == TRUE {
                    // leave the converted number on the stack
                } else {
                    pop!(self); // lose the failed number
                    let word = &self.u_get_string(self.pad_ptr);
                    self.msg
                        .warning("$interpret", "token not recognized", Some(word));
                }
            }
        }
    }

    /// FIND (s -- nfa, cfa, T | s F ) Search the dictionary for the token indexed through s.
    /// If not found, return the string address so NUMBER? can look at it
    pub fn f_find(&mut self) {
        let mut result = false;
        let source_addr = pop!(self) as usize;
        let mut link = self.data[self.here_ptr] as usize - 1;
        if stack_ok!(self, 1, "find") {
            link = self.data[link] as usize; // go back to the beginning of the top word
            while link > 0 {
                // name field is immediately after the link
                if self.strings[self.data[link + 1] as usize & ADDRESS_MASK] as u8
                    == self.strings[source_addr] as u8
                {
                    if self.u_str_equal(source_addr, self.data[link + 1] as usize) {
                        result = true;
                        break;
                    }
                }
                link = self.data[link] as usize;
            }
        }
        if result {
            push!(self, link as i64 + 1);
            push!(self, link as i64 + 2);
            push!(self, TRUE);
        } else {
            push!(self, source_addr as i64);
            push!(self, FALSE);
        }
    }

    /// number? ( s -- n T | a F ) tests a string to see if it's a number;
    /// leaves n and flag on the stack: true if number is ok.
    pub fn f_number_q(&mut self) {
        let buf_addr = pop!(self);
        let mut result = 0;
        let numtext = self.u_get_string(buf_addr as usize);
        if u_is_integer(&numtext.as_str()) {
            result = numtext.parse().unwrap();
            push!(self, result);
            push!(self, TRUE);
        } else {
            push!(self, buf_addr);
            push!(self, FALSE);
        }
    }

    /// f_comma ( cfa -- ) compile a code word into a definition
    pub fn f_comma(&mut self) {
        self.data[self.here_ptr] = pop!(self);
        self.here_ptr += 1;
    }

    /// f_literal ( n -- ) compile a literal number with it's inner interpreter code pointer
    pub fn f_literal(&mut self) {
        self.data[self.here_ptr] = LITERAL;
        self.data[self.here_ptr + 1] = pop!(self);
        self.here_ptr += 2;
    }

    /// UNIQUE? (s -- s )
    /// Checks the dictionary to see if the word pointed to is defined.
    pub fn f_q_unique(&mut self) {
        self.f_find();
        let result = pop!(self);
        if result == TRUE {
            self.msg
                .warning("unique?", "Overwriting existing definition", None::<bool>);
        }
        pop!(self);
        pop!(self);
    }

    /// ' (TICK) <name> ( -- a | FALSE )
    /// Looks for a (postfix) word in the dictionary
    /// places it's execution token / address on the stack
    /// Pushes 0 if not found
    pub fn f_tick(&mut self) {
        // *** get a string from the user (via TEXT?) and put addr on stack
        self.f_find(); // look for it
        if top!(self) == FALSE {
            // write an error message
            let mut msg = self.u_get_string(self.pad_ptr);
            msg = format!("Word not found: {}", msg);
            self.u_set_string(self.pad_ptr, &msg);
            push!(self, self.pad_ptr as i64);
            self.f_type(); // a warning message
        }
    }

    /// (parse) - ( b u c -- b u delta )
    /// Find a c-delimited token in the string buffer at b, buffer len u.
    /// Return the pointer to the buffer, the length of the token,
    /// and the offset from the start of the buffer to the start of the token.
    pub fn f_parse_p(&mut self) {
        if stack_ok!(self, 3, "(parse)") {
            let delim = pop!(self) as u8 as char;
            let buf_len = pop!(self);
            let in_p = pop!(self);
            // traverse the string, dropping leading delim characters
            // in_p points *into* a string, so no count field
            let start = in_p as usize;
            let end = start + buf_len as usize;
            let mut i = start as usize;
            let mut j = i;
            while self.strings[i] == delim && i < end {
                i += 1;
            }
            j = i;
            while j < end && self.strings[j] != delim {
                j += 1;
            }
            push!(self, in_p);
            push!(self, (j - i) as i64);
            push!(self, i as i64);
        }
    }

    /// TEXT ( -- b u ) Get a space-delimited token from the TIB, place in PAD
    pub fn f_text(&mut self) {
        push!(self, ' ' as u8 as i64);
        self.f_parse();
    }

    /// PARSE ( c -- b u ) Get a c-delimited token from TIB, and return counted string in PAD
    /// need to check if TIB is empty
    /// if delimiter = 1, get the rest of the TIB
    /// Update >IN as required, and set #TIB to zero if the line has been consumed
    pub fn f_parse(&mut self) {
        if stack_ok!(self, 1, "parse") {
            let delim: i64 = pop!(self);
            push!(
                // starting address in the string
                self,
                (self.data[self.tib_ptr] + self.data[self.tib_in_ptr])
            );
            if delim == 1 {
                self.data[self.tib_in_ptr] = 1;
                self.data[self.tib_size_ptr] = 0;
                push!(self, 0); // indicates nothing found, TIB is empty
                return;
            } else {
                push!(
                    // bytes available (length of input string)
                    self,
                    (self.data[self.tib_size_ptr] - self.data[self.tib_in_ptr] + 1)
                );
                push!(self, delim);
                self.f_parse_p();
                // check length, and copy to PAD if a token was found
                let delta = pop!(self);
                let length = pop!(self);
                let addr = pop!(self);
                if length > 0 {
                    // copy to pad
                    self.u_str_copy(
                        (addr + delta - 1) as usize,
                        self.data[self.pad_ptr] as usize,
                        length as usize,
                        false,
                    );
                }
                self.data[self.tib_in_ptr] += delta + length;
                push!(self, self.data[self.pad_ptr]);
                push!(self, length);
            }
        }
    }

    pub fn f_colon(&mut self) {
        self.set_compile_mode(true);
    }

    /*  fn f_d_pack(&mut self) {
        // pack the string in PAD and place it in the dictionary for a new word
        let data = self.f_string_at(addr);
        let packed = self.pack_string(&data);
        for c in packed {
            let here = self.data[self.here_ptr];
            self.data[]
        }
    }
    */

    /// u_interpret executes a line of code in the new interpreter
    /* pub fn u_interpret(&mut self, line: &str) {
        // put it in TIB and call interpret?
        self.u_save_string(line, self.tib_ptr);
        self.data[self.tib_size_ptr] = line.len() as i64;
        self.data[self.tib_in_ptr] = 1;
        push!(self, self.tib_ptr as i64);
        self.f_eval();
    } */

    /// u_write_word compiles a token into the current definition at HERE
    ///              updating HERE afterwards
    ///              the address of the (defined) word is on the stack
    ///              we compile a pointer to the word's inner interpreter
    pub fn u_write_word(&mut self, word_addr: i64) {
        if stack_ok!(self, 1, "u-interpret") {
            self.data[self.here_ptr] = word_addr;
        }
    }

    /// Return a string from a Forth string address
    pub fn u_get_string(&mut self, addr: usize) -> String {
        let str_addr = (addr & ADDRESS_MASK) + 1; //
        let last = str_addr + self.strings[addr] as usize;
        let mut result = String::new();
        for i in str_addr..last {
            result.push(self.strings[i]);
        }
        result
    }

    /// Save a counted string to a Forth string address
    pub fn u_set_string(&mut self, addr: usize, string: &str) {
        let str_addr = addr & ADDRESS_MASK;
        self.strings[str_addr] = string.len() as u8 as char; // count byte
        for (i, c) in string.chars().enumerate() {
            self.strings[str_addr + i + 1] = c;
        }
    }

    /// copy a string from a text buffer to a counted string
    /// Typically used to copy to PAD from TIB
    /// Can work with source strings counted or uncounted
    pub fn u_str_copy(&mut self, from: usize, to: usize, length: usize, counted: bool) {
        self.strings[to] = length as u8 as char; // write count byte
        let offset = if counted { 1 } else { 0 };
        for i in 0..length {
            self.strings[to + i + 1] = self.strings[from + i + offset];
        }
    }

    /// Compare two Forth (counted) strings
    /// First byte is the length, so we'll bail quickly if they don't match
    pub fn u_str_equal(&mut self, s_addr1: usize, s_addr2: usize) -> bool {
        for i in 0..self.strings[s_addr1] as usize {
            if self.strings[s_addr1 + i] != self.strings[s_addr2 + i] {
                return false;
            }
        }
        true
    }

    /// copy a string slice into string space
    pub fn u_save_string(&mut self, from: &str, to: usize) {
        self.strings[to] = from.len() as u8 as char; // count byte
        for (i, c) in from.chars().enumerate() {
            self.strings[i + 1] = c;
        }
    }
}
