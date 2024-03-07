//The tForth interpreter struct and implementation

use crate::internals::builtin::BuiltInFn;
use crate::messages::Msg;
use crate::reader::Reader;
use std::time::Instant;

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

/// The primary data structure for the Forth engine
///
///     Forth's main data structure is a fixed array of integers (overloaded with characters and unsigned values).
///     This holds all the program data - words, variables, constants, stack etc. used by Forth
///     Strings are kept in a separate array, which is simpler than packing ASCII characters into 64 bit words
///     The Rust side of the engine keeps track of some variables with names following a *_ptr pattern.
///     This allows these values to be easily used by both Rust and Forth.
///     A small reader module manages input from files and stdin. Unfortunatly there is no easy way to provide
///     unbuffered keystroke input without a special library.
///     A simple messaging system provides warnings and errors. Ultimately these should be restricted to Rust error conditions,
///     while Forth should use it's own methods to display and process errors and warnings.
///
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
    pub state_ptr: usize, // true if compiling a word
    pub pc_ptr: usize,    // program counter
    pub abort_ptr: usize, // true if abort has been called
    pub tib_ptr: usize,   // TIB
    pub tib_size_ptr: usize,
    pub tib_in_ptr: usize,
    pub exit_flag: bool, // set when the "bye" word is executed.
    pub msg: Msg,
    pub reader: Vec<Reader>, // allows for nested file processing
    pub show_stack: bool,    // show the stack at the completion of a line of interaction
    pub stepper_ptr: usize,
    pub timer: Instant, // for timing things
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
                state_ptr: 0,
                pc_ptr: 0,
                abort_ptr: 0,
                tib_ptr: 0,
                tib_size_ptr: 0,
                tib_in_ptr: 0,
                exit_flag: false,
                msg: Msg::new(),
                reader: Vec::new(),
                show_stack: true,
                stepper_ptr: 0,
                timer: Instant::now(),
            };
            interpreter.reader.push(reader);
            interpreter
        } else {
            panic!("unable to create reader");
        }
    }

    /// cold_start is where the interpreter begins, installing some variables and the builtin functions.
    pub fn cold_start(&mut self) {
        self.u_insert_variables();
        self.add_builtins();
        self.set_var(self.state_ptr, FALSE);
        self.u_insert_code(); // allows forth code to be run prior to presenting a prompt.
    }

    /// get_var returns the value of a defined variable from its pointer address
    ///
    pub fn get_var(&mut self, addr: usize) -> i64 {
        self.data[addr]
    }

    /// set_var returns the value of a defined variable from its pointer address
    pub fn set_var(&mut self, addr: usize, val: i64) {
        self.data[addr] = val;
    }

    /// get_compile_mode determines whether or not compile mode is active
    ///     Traditionally, a variable called 'EVAL stores the compile or the interpret functions
    ///     In this version, the STATE variable is used directly.
    ///
    pub fn get_compile_mode(&mut self) -> bool {
        if self.get_var(self.state_ptr) == FALSE {
            false
        } else {
            true
        }
    }

    /// set_compile_mode turns on compilation mode
    ///
    pub fn set_compile_mode(&mut self, value: bool) {
        self.set_var(self.state_ptr, if value { -1 } else { 0 });
    }

    /// set_abort_flag allows the abort condition to be made globally visible
    ///
    pub fn set_abort_flag(&mut self, v: bool) {
        self.set_var(self.abort_ptr, if v { -1 } else { 0 });
    }

    /// get_abort_flag returns the current value of the flag
    ///
    pub fn get_abort_flag(&mut self) -> bool {
        let val = self.get_var(self.abort_ptr);
        if val == FALSE {
            false
        } else {
            true
        }
    }

    /// should_exit determines whether or not the user has executed BYE
    ///
    pub fn should_exit(&self) -> bool {
        // Method to determine if we should exit
        self.exit_flag
    }

    // pack_string compresses strings to fit into 64 bit words
    //     Not used by the current implementation because strings are in their own data structure
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
