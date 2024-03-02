/// Input-output words
use crate::engine::{FileMode, BUF_SIZE, STACK_START, TF};
use crate::messages::Msg;
use crate::reader::Reader;
use std::cmp::min;
use std::io::{self, Write};

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

impl TF {
    /// key ( -- c | 0 ) get a character and push on the stack, or zero if none available
    pub fn f_key(&mut self) {
        let reader = self.reader.last();
        match reader {
            Some(reader) => {
                let c = reader.read_char();
                match c {
                    Some(c) => {
                        push!(self, c as u8 as i64);
                    }
                    None => {
                        push!(self, 0);
                    }
                }
            }
            None => {}
        }
    }

    /// accept ( b u -- b u ) Read up to u characters, storing them at string address b and returning the actual length.
    ///     If the read fails, we assume EOF, and pop the reader. Returned length will be 0.
    ///
    ///     Return the start of the string, and the number of characters read.
    ///     Typically writes a counted string to the TIB, in which case,
    ///     it needs TIB_START and BUF_SIZE - 1 on the stack.
    ///
    pub fn f_accept(&mut self) {
        if stack_ok!(self, 2, "accept") {
            let max_len = pop!(self);
            let dest = top!(self) as usize;
            match self.reader.last_mut() {
                Some(reader) => {
                    let l = reader.get_line();
                    match l {
                        Some(line) => {
                            let length = min(line.len() - 1, max_len as usize) as usize;
                            let line_str = &line[..length];
                            self.u_save_string(line_str, dest); // write a counted string
                            push!(self, length as i64);
                        }
                        None => {
                            // EOF - there are no more lines to read
                            if self.reader.len() > 1 {
                                // Reader 0 is stdin
                                self.reader.pop();
                                push!(self, 0);
                            } else {
                                panic!("Reader error - EOF in stdin");
                            }
                        }
                    }
                }
                None => self
                    .msg
                    .error("accept", "No input source available", None::<bool>),
            }
        }
    }

    /// QUERY ( -- ) Load a new line of text into the TIB
    pub fn f_query(&mut self) {
        push!(self, self.data[self.tib_ptr]);
        push!(self, BUF_SIZE as i64 - 1);
        self.f_accept();
        self.data[self.tib_size_ptr] = pop!(self); // update the TIB size pointer
        self.data[self.tib_in_ptr] = 1; // set the starting point in the TIB
        pop!(self); // we don't need the address
    }

    // output

    pub fn f_emit(&mut self) {
        if stack_ok!(self, 1, "emit") {
            let c = pop!(self);
            if (0x20..=0x7f).contains(&c) {
                print!("{}", c as u8 as char);
            } else {
                self.msg.error("EMIT", "Arg out of range", Some(c));
            }
        }
    }

    pub fn f_flush(&mut self) {
        io::stdout().flush().unwrap();
    }

    pub fn f_dot(&mut self) {
        if stack_ok!(self, 1, ".") {
            let a = pop!(self);
            print!("{a} ");
        }
    }

    pub fn f_dot_s(&mut self) {
        print!("[ ");
        for i in (self.stack_ptr..STACK_START).rev() {
            print!("{} ", self.data[i]);
        }
        print!("] ");
    }

    pub fn f_cr(&mut self) {
        println!("");
    }

    /// s" ( -- ) get a string and place it in TMP
    pub fn f_s_quote(&mut self) {
        push!(self, self.data[self.tmp_ptr]);
        push!(self, '"' as i64);
        self.f_parse_to();
        pop!(self);
        pop!(self);
    }

    /// type (s -- ) - print a string, using the string address on the stack
    pub fn f_type(&mut self) {
        if stack_ok!(self, 1, "type") {
            let addr = pop!(self) as usize;
            let text = self.u_get_string(addr);
            print!("{text}");
        }
    }

    // file i/o

    pub fn f_r_w(&mut self) {
        self.file_mode = FileMode::ReadWrite;
    }
    pub fn f_r_o(&mut self) {
        self.file_mode = FileMode::ReadOnly;
    }

    /// include-file (s -- ) Pushes a new reader, pointing to the file named at s, calling ABORT if unsuccessful
    ///     The intent is that the standard loop will continue, now reading lines from the file
    ///     At the end of the file, the reader will be popped off the stack.
    ///     This allows for nested file reads.
    ///
    pub fn f_include_file(&mut self) {
        if stack_ok!(self, 1, "include-file") {
            let file_ptr = pop!(self) as usize;
            let file_name = self.u_get_string(file_ptr);
            let full_path = std::fs::canonicalize(file_name);
            match full_path {
                Ok(full_path) => {
                    let reader = Reader::new(Some(&full_path), Msg::new());
                    match reader {
                        Some(reader) => {
                            self.reader.push(reader);
                        }
                        None => {
                            self.msg
                                .error("loaded", "Failed to create new reader", None::<bool>);
                            self.f_abort();
                        }
                    }
                }
                Err(error) => {
                    self.msg
                        .warning("include-file", error.to_string().as_str(), None::<bool>);
                    self.f_abort();
                }
            }
        }
    }
}
