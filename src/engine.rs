//The tForth interpreter struct and implementation

use crate::internals::builtin::BuiltInFn;
// use crate::internals::compiler::*;
use crate::messages::Msg;
use crate::reader::Reader;
//use crate::tokenizer::{ForthToken, Tokenizer};

// DATA AREA constants
pub const DATA_SIZE: usize = 10000;
pub const STRING_SIZE: usize = 5000;
pub const BUF_SIZE: usize = 132;
pub const ALLOC_START: usize = DATA_SIZE / 2;
pub const STACK_START: usize = ALLOC_START - 1; // stack counts up
pub const RET_START: usize = DATA_SIZE - 1; // return stack counts downwards
pub const WORD_START: usize = 0; // data area counts up from the bottom (builtins, words, variables etc.)

// STRING AREA constants
pub const TIB_START: usize = 0; // Text input buffer, used by readers
pub const PAD_START: usize = TIB_START + BUF_SIZE; // Scratchpad buffer, used by PARSE and friends
pub const TMP_START: usize = PAD_START + BUF_SIZE; // Temporary buffer, used for string input
pub const STR_START: usize = TMP_START + BUF_SIZE; // Free space for additional strings

// GENERAL constants
pub const TRUE: i64 = -1; // forth convention for true and false
pub const FALSE: i64 = 0;
pub const ADDRESS_MASK: usize = 0x00FFFFFFFFFFFFFF; // to get rid of flags
pub const IMMEDIATE_MASK: usize = 0x4000000000000000; // the immediate flag bit
pub const BUILTIN_MASK: usize = 0x2000000000000000; // the builtin flag bit

// Indices into builtins to drive execution of each data type
pub const BUILTIN: i64 = 1000;
pub const VARIABLE: i64 = 1001;
pub const CONSTANT: i64 = 1002;
pub const LITERAL: i64 = 1003;
pub const STRLIT: i64 = 1004;
pub const DEFINITION: i64 = 1005;
pub const BRANCH: i64 = 1006;
pub const BRANCH0: i64 = 1007;
pub const ABORT: i64 = 1008;
pub const EXIT: i64 = 1009;
pub const NEXT: i64 = 1010;

//#[derive(Debug)]
pub struct TF {
    pub data: [i64; DATA_SIZE],
    pub strings: [char; STRING_SIZE], // storage for strings
    pub builtins: Vec<BuiltInFn>,     // the dictionary of builtins
    //pub return_stack: Vec<i64>,       // for do loops etc.
    pub here_ptr: usize,
    pub stack_ptr: usize,  // top of the linear space stack
    pub return_ptr: usize, // top of the return stack
    pub context_ptr: usize,
    pub eval_ptr: usize, // used to turn compile mode on and off
    pub base_ptr: usize,
    pub pad_ptr: usize,    // string buffer for parser
    pub tmp_ptr: usize,    // temporary string buffer
    pub string_ptr: usize, // points to the beginning of free string space
    pub last_ptr: usize,   // points to name of top word
    pub hld_ptr: usize,    // for numeric string work
    pub file_mode: FileMode,
    pub compile_ptr: usize, // true if compiling a word
    pub pc_ptr: usize,      // program counter
    pub abort_ptr: usize,   // true if abort has been called
    pub tib_ptr: usize,     // TIB
    pub tib_size_ptr: usize,
    pub tib_in_ptr: usize,
    exit_flag: bool, // set when the "bye" word is executed.
    pub msg: Msg,
    pub reader: Vec<Reader>, // allows for nested file processing
    pub show_stack: bool,    // show the stack at the completion of a line of interaction
    pub step_mode: bool,
}

#[derive(Debug)]
pub enum FileMode {
    // used for file I/O
    ReadWrite,
    ReadOnly,
    Unset,
}

impl TF {
    // ForthInterpreter struct implementations
    pub fn new() -> TF {
        if let Some(reader) = Reader::new(None, Msg::new()) {
            let mut interpreter = TF {
                data: [0; DATA_SIZE],
                strings: [' '; STRING_SIZE],
                builtins: Vec::new(),
                //return_stack: Vec::new(),
                here_ptr: WORD_START,
                stack_ptr: STACK_START,
                return_ptr: RET_START,
                string_ptr: 0,
                context_ptr: 0,
                eval_ptr: 0,
                base_ptr: 0,
                pad_ptr: 0,
                tmp_ptr: 0,
                last_ptr: 0,
                hld_ptr: 0,
                file_mode: FileMode::Unset,
                compile_ptr: 0,
                pc_ptr: 0,
                abort_ptr: 0,
                tib_ptr: 0,
                tib_size_ptr: 0,
                tib_in_ptr: 0,
                exit_flag: false,
                msg: Msg::new(),
                reader: Vec::new(),
                show_stack: true,
                step_mode: false,
            };
            interpreter.reader.push(reader);
            interpreter
        } else {
            panic!("unable to create reader");
        }
    }

    pub fn cold_start(&mut self) {
        self.u_insert_variables();
        //self.f_insert_builtins();
        self.add_builtins();
        self.set_var(self.compile_ptr, FALSE);
        self.u_insert_code(); // allows forth code to be run prior to presenting a prompt.
    }

    /// get_var returns the value of a defined variable from its pointer address
    pub fn get_var(&mut self, addr: usize) -> i64 {
        self.data[addr]
    }

    /// set_var returns the value of a defined variable from its pointer address
    pub fn set_var(&mut self, addr: usize, val: i64) {
        self.data[addr] = val;
    }

    /// get_compile_mode *** needs to work with 'EVAL contents
    pub fn get_compile_mode(&mut self) -> bool {
        if self.get_var(self.compile_ptr) == FALSE {
            false
        } else {
            true
        }
    }

    /// get_compile_mode *** needs to work with 'EVAL contents
    pub fn set_compile_mode(&mut self, value: bool) {
        self.set_var(self.compile_ptr, if value { -1 } else { 0 });
    }

    pub fn set_abort_flag(&mut self, v: bool) {
        self.set_var(self.abort_ptr, if v { -1 } else { 0 });
    }
    pub fn get_abort_flag(&mut self) -> bool {
        let val = self.get_var(self.abort_ptr);
        if val == FALSE {
            false
        } else {
            true
        }
    }

    pub fn set_program_counter(&mut self, val: usize) {
        self.set_var(self.pc_ptr, val as i64);
    }
    fn get_program_counter(&mut self) -> usize {
        self.get_var(self.pc_ptr) as usize
    }
    fn increment_program_counter(&mut self, val: usize) {
        let new = self.get_program_counter() + val;
        self.set_var(self.pc_ptr, (new) as i64);
    }
    fn decrement_program_counter(&mut self, val: usize) {
        let new = self.get_program_counter() - val;
        self.set_var(self.pc_ptr, (new) as i64);
    }

    pub fn set_exit_flag(&mut self) {
        // Method executed by "bye"
        self.exit_flag = true;
    }

    pub fn should_exit(&self) -> bool {
        // Method to determine if we should exit
        self.exit_flag
    }

    fn step(&mut self) {
        // controls step / debug functions
        if self.step_mode {
            /*             match &self.token_ptr.1 {
                          ForthToken::Integer(num) => print!("{num}: Step> "),
                          ForthToken::Float(num) => print!("f{num}: Step> "),
                          ForthToken::Operator(op) => print!("{op}: Step> "),
                          ForthToken::Jump(name, offset) => {
                              print!("{name}:{}: Step> ", offset);
                          }
                          ForthToken::Forward(info) => {
                              print!("{}{}: Step> ", info.word, info.tail);
                          }
                          ForthToken::Builtin(name, code) => print!("{}:{:?}", name, code),
                          ForthToken::Definition(name, _def) => print!("{name} "),
                          ForthToken::Empty => print!("ForthToken::Empty: Step> "),
                          ForthToken::Variable(n, v) => print!("{}={}", n, v),
                          _ => print!("variable or constant???"),
                      }
                      io::stdout().flush().unwrap();
                      match self.parser.reader.read_char() {
                          Some('s') => {
                              self.print_stack();
                              self.print_return_stack();
                          }
                          // Some('v') => self.print_variables(),
                          Some('a') => {
                              self.print_stack();
                              //self.print_variables();
                          }
                          Some('c') => self.step_mode = false,
                          Some(_) | None => {}
                      }
            */
        }
    }

    /* pub fn pack_string(&self, input: &str) -> Vec<usize> {
        // tries to pack a string
        let mut output = Vec::new();
        let mut tmp = input.len();
        println!("{:#x}", tmp);

        let words: usize = input.len() / 7 + 1;
        //println!("Words:{words}");
        let mut i = 0;
        for c in input.chars() {
            i += 1;
            if i % 8 == 0 {
                output.push(tmp);
                tmp = 0;
            }
            let shift = i % 8;
            let new = (c as u8 as usize) << 8 * shift;
            println!("{shift} {:#x}", new);
            tmp |= (c as u8 as usize) << 8 * shift;
            //println!("tmp{:#x}", tmp);
        }
        output.push(tmp);
        //println!("Finished packing");
        output
    } */
}
