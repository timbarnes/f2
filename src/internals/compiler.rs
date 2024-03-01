// Compiler and Interpreter

use crate::engine::{
    ABORT, ADDRESS_MASK, BRANCH, BRANCH0, BUILTIN, BUILTIN_MASK, CONSTANT, DEFINITION, EXIT, FALSE,
    IMMEDIATE_MASK, LITERAL, NEXT, STACK_START, STRLIT, TF, TRUE, VARIABLE,
};
use crate::internals::general::u_is_integer;

macro_rules! stack_ok {
    ($self:ident, $n: expr, $caller: expr) => {
        if $self.stack_ptr <= STACK_START - $n {
            // $self.f_dot_s();
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
    /// immediate ( -- ) sets the immediate flag on the most recently defined word
    /// Context pointer links to the most recent name field
    pub fn f_immediate(&mut self) {
        let mut str_addr = self.data[self.data[self.context_ptr] as usize] as usize;
        str_addr |= IMMEDIATE_MASK;
        self.data[self.data[self.context_ptr] as usize] = str_addr as i64;
    }

    /// immediate? ( cfa -- T | F ) Determines if a word is immediate or not
    pub fn f_immediate_q(&mut self) {
        if stack_ok!(self, 1, "immediate?") {
            let cfa = pop!(self) as usize;
            let name_ptr = self.data[cfa - 1] as usize;
            let immed = name_ptr & IMMEDIATE_MASK;
            let result = if immed == 0 { FALSE } else { TRUE };
            push!(self, result);
        }
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
        self.f_clear();
        self.set_abort_flag(true);
    }

    pub fn f_quit(&mut self) {
        self.set_program_counter(0);
        self.f_abort();
        print!(" ok ");
        loop {
            if self.should_exit() {
                break;
            } else {
                self.set_abort_flag(false);
                self.f_query();
                self.f_eval(); // interpret the contents of the line
                if self.show_stack {
                    self.f_dot_s();
                }
                print!(" ok ");
                self.f_flush();
            }
        }
    }

    /// EXECUTE ( cfa -- ) interpret a word with addr on the stack
    /// stack value is the address of an inner interpreter
    pub fn f_execute(&mut self) {
        if stack_ok!(self, 1, "execute") {
            // call the appropriate inner interpreter
            let xt = pop!(self);
            push!(self, xt + 1);
            match self.data[xt as usize] {
                BUILTIN => self.msg.error("f_execute", "BUILTIN found", Some(xt)), //self.i_builtin(),
                VARIABLE => self.i_variable(),
                CONSTANT => self.i_constant(),
                LITERAL => self.i_literal(),
                STRLIT => self.i_strlit(),
                DEFINITION => self.i_definition(),
                BRANCH => self.i_branch(),
                BRANCH0 => self.i_branch0(),
                ABORT => self.i_abort(),
                EXIT => self.i_exit(),
                NEXT => self.i_next(),
                _ => {
                    pop!(self);
                    let cfa = self.data[xt as usize] as usize & !BUILTIN_MASK;
                    push!(self, cfa as i64);
                    self.i_builtin();
                }
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
            self.f_find();
            if pop!(self) == TRUE {
                let cfa = top!(self);
                // we found a word
                // if it's immediate, we need to execute it; otherwise continue compiling
                self.f_immediate_q();
                if pop!(self) == TRUE {
                    // call the interpreter for this word
                    push!(self, self.data[self.pad_ptr] as i64);
                    self.f_d_interpret();
                } else {
                    // check if it's a builtin, and compile appropriately
                    let indirect = self.data[cfa as usize] as usize;
                    if indirect & BUILTIN_MASK != 0 {
                        push!(self, indirect as i64);
                    } else {
                        push!(self, cfa);
                    }
                    self.f_comma(); // uses the cfa on the stack
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
                    self.f_abort();
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
                self.f_execute();
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

    /// FIND (s -- cfa T | s F ) Search the dictionary for the token indexed through s.
    /// If not found, return the string address so NUMBER? can look at it
    pub fn f_find(&mut self) {
        if stack_ok!(self, 1, "find") {
            let mut result = false;
            let source_addr = pop!(self) as usize;
            let mut link = self.data[self.context_ptr] as usize - 1;
            // link = self.data[link] as usize; // go back to the beginning of the top word
            while link > 0 {
                // name field is immediately after the link
                let nfa_val = self.data[link + 1];
                let str_addr = nfa_val as usize & ADDRESS_MASK;
                if self.strings[str_addr] as u8 == self.strings[source_addr] as u8 {
                    if self.u_str_equal(source_addr, str_addr as usize) {
                        result = true;
                        break;
                    }
                }
                link = self.data[link] as usize;
            }
            if result {
                push!(self, link as i64 + 2);
                push!(self, TRUE);
            } else {
                push!(self, source_addr as i64);
                push!(self, FALSE);
            }
        } else {
            // stack error
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

    /// f_comma ( n -- ) compile a value into a definition
    pub fn f_comma(&mut self) {
        self.data[self.data[self.here_ptr] as usize] = pop!(self);
        self.data[self.here_ptr] += 1;
    }

    /// f_literal ( n -- ) compile a literal number with it's inner interpreter code pointer
    pub fn f_literal(&mut self) {
        push!(self, LITERAL);
        self.f_comma();
        self.f_comma();
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
    }

    /// ' (TICK) <name> ( -- a | FALSE ) Searches for a word, places cfa on stack if found; otherwise FALSE
    /// Looks for a (postfix) word in the dictionary
    /// places it's execution token / address on the stack
    /// Pushes 0 if not found
    pub fn f_tick(&mut self) {
        self.f_text(); // ( -- b u )
        pop!(self); // don't need the delim
        self.f_find(); // look for the token
        if pop!(self) == FALSE {
            // write an error message
            let mut msg = self.u_get_string(self.data[self.pad_ptr] as usize);
            msg = format!("Word not found: {} ", msg);
            self.u_set_string(self.data[self.pad_ptr] as usize, &msg);
            self.f_type(); // a warning message
            push!(self, FALSE);
        } else {
            // we found it, so leave the cfa on the stack
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
            if buf_len > 0 {
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
                push!(self, i as i64 - in_p);
            } else {
                // nothing left to read
                push!(self, in_p);
                push!(self, 0);
                push!(self, 0);
            }
        }
    }

    /// TEXT ( -- b u ) Get a space-delimited token from the TIB, place in PAD
    pub fn f_text(&mut self) {
        push!(self, ' ' as u8 as i64);
        self.f_parse();
    }

    /// \ (backslash)  <text> \n Inline comment: ignores the remainder of the line
    pub fn f_backslash(&mut self) {
        push!(self, 1 as u8 as i64);
        self.f_parse();
        pop!(self); // throw away stack values left by f_parse
        pop!(self);
    }

    /// ( <text> ) Used for stack signature documentation. Ignores everything up to the right paren.
    pub fn f_l_paren(&mut self) {
        push!(self, ')' as u8 as i64);
        self.f_parse();
        pop!(self); // throw away stack values left by f_parse, causing the text to be abandoned
        pop!(self);
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
                self.data[self.tib_ptr] + self.data[self.tib_in_ptr]
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
                    self.data[self.tib_size_ptr] - self.data[self.tib_in_ptr] + 1
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
                        (addr + delta) as usize,
                        self.data[self.pad_ptr] as usize,
                        length as usize,
                        false,
                    );
                }
                self.data[self.tib_in_ptr] += delta + length + 1;
                push!(self, self.data[self.pad_ptr]);
                push!(self, length);
            }
        }
    }

    pub fn f_colon(&mut self) {
        self.set_compile_mode(true);
        self.f_create(); // gets the name and makes a new dictionary entry
        push!(self, DEFINITION);
        self.f_comma();
    }

    /// ; terminates a definition, writing the cfa for EXIT, and resetting to interpret mode
    ///   It has to write the exit code word, and add a back pointer
    ///   It also has to update HERE and CONTEXT.
    ///   Finally it switches out of compile mode
    pub fn f_semicolon(&mut self) {
        push!(self, EXIT);
        self.f_comma();
        self.data[self.data[self.here_ptr] as usize] = self.data[self.last_ptr] - 1; // write the back pointer
        self.data[self.here_ptr] += 1; // over EXIT and back pointer
        self.data[self.context_ptr] = self.data[self.last_ptr]; // adds the new definition to FIND
        self.set_compile_mode(false);
    }

    /// f_create makes a new dictionary entry, using a postfix name
    /// References HERE, and assumes back pointer is in place already
    pub fn f_create(&mut self) {
        self.f_text(); // get the word's name
        pop!(self); // throw away the length, keep the text pointer
        self.f_q_unique(); // issue a warning if it's already defined
        let length = self.strings[self.data[self.pad_ptr] as usize] as u8 as i64;
        push!(self, length);
        push!(self, self.data[self.string_ptr]);
        self.f_smove(); // make a new string with the name from PAD
        self.data[self.data[self.here_ptr] as usize] = pop!(self); // the string header
        self.data[self.string_ptr] += length + 1; // update the free string pointer
        self.data[self.last_ptr] = self.data[self.here_ptr];
        self.data[self.here_ptr] += 1;
    }

    /// variable <name> ( -- ) Creates a new variable in the dictionary
    pub fn f_variable(&mut self) {
        self.f_create(); // gets a name and makes a name field in the dictionary
        push!(self, VARIABLE);
        self.f_comma(); // ( n -- )
        push!(self, 0); // default initial value
        self.f_comma();
        self.data[self.data[self.here_ptr] as usize] = self.data[self.last_ptr] - 1; // write the back pointer
        self.data[self.here_ptr] += 1; // over EXIT and back pointer
        self.data[self.context_ptr] = self.data[self.last_ptr]; // adds the new definition to FIND
    }

    /// constant <name> ( n -- ) Creates and initializez a new constant in the dictionary
    pub fn f_constant(&mut self) {
        if stack_ok!(self, 1, "constant") {
            self.f_create();
            push!(self, CONSTANT);
            self.f_comma();
            self.f_comma(); // write the value from the stack
            self.data[self.data[self.here_ptr] as usize] = self.data[self.last_ptr] - 1; // write the back pointer
            self.data[self.here_ptr] += 1; // over EXIT and back pointer
            self.data[self.context_ptr] = self.data[self.last_ptr]; // adds the new definition to FIND
        }
    }

    /// string <name> ( s -- ) Creates and initializez a new string from the PAD
    pub fn f_string(&mut self) {}

    /// f_pack_d ( source len dest -- dest ) builds a new counted string from an existing counted string.
    pub fn f_smove(&mut self) {
        let dest = pop!(self) as usize;
        let length = pop!(self) as usize;
        let source = pop!(self) as usize;
        // assuming both are counted, we begin with the count byte. Length should match the source count byte
        for i in (0..=length) {
            self.strings[dest + i] = self.strings[source + i];
        }
        push!(self, dest as i64);
    }

    /// see <name> ( -- ) prints the definition of a word
    pub fn f_see(&mut self) {
        self.f_tick(); // finds the address of the word
        let cfa = pop!(self);
        if cfa == FALSE {
            self.msg.warning("see", "Word not found", None::<bool>);
        } else {
            let nfa = self.data[cfa as usize - 1] as usize;
            let is_immed = nfa & IMMEDIATE_MASK;
            let xt = self.data[cfa as usize] as usize;
            let is_builtin = xt & BUILTIN_MASK;
            if is_builtin != 0 {
                println!(
                    "Builtin: {}",
                    self.builtins[xt as usize & !BUILTIN_MASK].doc
                );
            } else {
                // It's a definition of some kind
                //xt &= ADDRESS_MASK; // get rid of any special bits
                match xt as i64 {
                    DEFINITION => {
                        print!(": ");
                        let name = self.u_get_string(nfa);
                        print!("{name} ");
                        let mut index = cfa as usize + 1; // skip the inner interpreter
                        loop {
                            let xt = self.data[index];
                            match xt {
                                LITERAL => {
                                    print!("{} ", self.data[index as usize + 1]);
                                    index += 1;
                                }
                                STRLIT => {} // print string contents
                                BRANCH => {
                                    print!("branch:{} ", self.data[index as usize + 1]);
                                    index += 1;
                                }
                                BRANCH0 => {
                                    print!("branch0:{} ", self.data[index as usize + 1]);
                                    index += 1;
                                }
                                ABORT => println!("abort "),
                                EXIT => {
                                    print!("; ");
                                    if is_immed != 0 {
                                        println!("immediate");
                                    } else {
                                        println!();
                                    }
                                    break;
                                }
                                _ => {
                                    // it's a definition or a builtin
                                    let mut cfa = self.data[index] as usize;
                                    let mut mask = cfa & BUILTIN_MASK;
                                    if mask == 0 {
                                        let word = self.data[self.data[index] as usize - 1]; // nfa address
                                        let name = self.u_get_string(word as usize);
                                        print!("{name} ");
                                    } else {
                                        mask = !BUILTIN_MASK;
                                        cfa &= mask;
                                        let name = &self.builtins[cfa].name;
                                        print!("{name} ");
                                    }
                                }
                            }
                            index += 1;
                        }
                    }
                    CONSTANT => println!(
                        "Constant: {} = {}",
                        self.u_get_string(self.data[cfa as usize - 1] as usize),
                        self.data[cfa as usize]
                    ),
                    VARIABLE => println!(
                        "Variable: {} = {}",
                        self.u_get_string(self.data[cfa as usize - 1] as usize),
                        self.data[cfa as usize + 1]
                    ),
                    _ => self.msg.error("see", "Unrecognized type", None::<bool>),
                }
            }
        }
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
        for i in 0..=self.strings[s_addr1] as usize {
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
