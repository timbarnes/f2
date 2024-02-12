// f2 engine
//
//

use crate::messages::Msg;
use crate::reader::Reader;

const DATA_SIZE: usize = 10000;
const TIB_START: usize = DATA_SIZE - 132;
const PAD_START: usize = TIB_START - 132;
const ALLOC_START: usize = PAD_START - 1;
const STACK_START: usize = DATA_SIZE / 2; // stack counts up
const RET_START: usize = STACK_START - 1; // return stack counts downwards
const WORD_START: usize = 0; // data area counts up from the bottom (builtins, words, variables etc.)
const TRUE: i32 = -1; // forth convention for true and false
const FALSE: i32 = 0;

const IMMEDIATE: u32 = 1 << 26;
const BUILTIN: u32 = 1 << 27;
const CONSTANT: u32 = 1 << 28;
const VARIABLE: u32 = 1 << 29;
const STRING: u32 = 1 << 30;
const WORD: u32 = 1 << 31;
const LEN_MASK: u32 = 0x03FF;

/// Memory management is handled with a single array of i32 data
///
///    DATA_SIZE -> top of data area
///                 TIB starts at DATA_SIZE - TIB_SIZE
///                 PAD starts PAD_SIZE below
///                 ALLOC space  starts below that, working downwards
///                 < -- space for expansion -- >
///  STACK_START -> STACK works up from here towards ALLOC space
///                 RET stack heads down from here towards WORDS
///            0 -> WORDS start from the bottom
///                 
pub struct F2 {
    data: [i32; DATA_SIZE], // WORD, TIB, PAD, STACK, RET and ALLOC space
    here_ptr: usize,        // address of index into the word data space "here"
    stack_ptr: usize,       // point to next spot  on the forth stack
    ret_ptr: usize,         // point to next spot on the return stack
    alloc_ptr: usize,       // point to the top of unused ALLOC space
    compile: bool,          // compile mode indicator
    tick_eval_ptr: usize,   // used by EVAL to select compile vs interpret modes
    base_ptr: usize,        // address of numeric base for I/O
    tmp_ptr: usize,         // used for numeric calculations
    span_ptr: usize,        // ??
    tib_in_ptr: usize,      // address of offset into TIB
    hld_ptr: usize,         // used by HOLD
    context_ptr: usize,     // name field of last word
    last_ptr: usize,        // CONTEXT after build is complete
    exit_flag: bool,
    reader: Reader, // for interactive input
    msg: Msg,
}

impl F2 {
    pub fn new() -> Result<F2, String> {
        if let Some(rdr) = Reader::new(None, Msg::new()) {
            Ok(F2 {
                data: [0; DATA_SIZE],
                here_ptr: WORD_START,
                stack_ptr: STACK_START,
                ret_ptr: RET_START,
                alloc_ptr: ALLOC_START,
                compile: false,
                // Address in data[] of system variables used by the engine
                tick_eval_ptr: 0, // points to $COMPILE or $INTERPRET for EVAL
                base_ptr: 0,      // numeric base for I/O
                tmp_ptr: 0,       // used for numeric calculations
                span_ptr: 0,      // ??
                tib_in_ptr: 0,    // offset into TIB
                hld_ptr: 0,       // used by HOLD
                context_ptr: 0,   // name field of last word
                last_ptr: 0,      // CONTEXT after build is complete
                // internals
                exit_flag: false,
                reader: rdr,
                msg: Msg::new(),
            })
        } else {
            Err("Interpreter could not create reader".to_owned())
        }
    }

    pub fn init(&mut self) {
        // Initialize the data area with primitives
        let mut ptr = self.insert_variables();
        println!("data_ptr is now {}", self.data[self.here_ptr]);
        ptr = self.insert_constants();
        println!("data_ptr is now {}", self.data[self.here_ptr]);
        ptr = self.insert_builtins();
        println!("data_ptr is now {}", self.data[self.here_ptr]);
        let index = self.find_word("'eval");
        println!("FIND found: {}", index);
        self.data[self.tick_eval_ptr] = index as i32;
    }

    pub fn run(&mut self) {
        // Bootstrap execution
        // Enter the read-eval-print loop
        // Return after "bye" or EOF
        self.exit_flag = true;
        loop {
            if self.should_exit() {
                break;
            } else {
            }
        }
    }

    pub fn should_exit(&self) -> bool {
        self.exit_flag
    }

    pub fn f2_load_file(&self, _filename: &str) -> bool {
        // load the file of forth code
        true
    }

    fn find_word(&self, word: &str) -> usize {
        // returns the address of a word on the stack, if it's defined
        // uses the data_ptr as starting point, and loops back
        // print!("Finding {}..", word);
        let word_len: usize = word.len();
        let mut mismatch = false;
        let mut scanner = self.data[self.here_ptr] as usize; // the newest used back pointer
        loop {
            if scanner == 0 {
                // no more words to search
                return 0;
            } else {
                let cur_len = (self.data[scanner + 1]) as u32 & LEN_MASK;
                if cur_len as usize == word_len {
                    // word length matches, so compare the names
                    for (w, &c) in word
                        .bytes()
                        .zip(self.data[scanner + 2..scanner + word_len + 2].iter())
                    {
                        if w != c as u8 {
                            // not a match, so jump out
                            mismatch = true;
                            break;
                        }
                    }
                    if mismatch {
                        mismatch = false;
                    } else {
                        println!("Found at index {}", scanner);
                        return scanner;
                    }
                }
                scanner = self.data[scanner] as usize; // get the next one
            }
        }
    }

    fn make_variable(&mut self, name: &str) -> usize {
        // Create a variable, returning the address and updating the data_ptr
        // build the header for a variable
        let variable_ptr = self.make_word(&name, VARIABLE, &[0]); // install the name
        variable_ptr
    }

    fn make_constant(&mut self, name: &str, val: i32) -> usize {
        // Create a constant
        // build the header for a constant
        let const_ptr = self.make_word(name, CONSTANT, &[val]); // install the name
        const_ptr
    }

    fn make_word(&mut self, name: &str, flags: u32, args: &[i32]) -> usize {
        // install a new word with provided name, flags and arguments
        // back link is already in place
        // place it HERE
        // update HERE and LAST
        // return HERE
        let back = self.data[self.here_ptr] as usize; // the top-of-stack back pointer's location
        let mut ptr = back + 1;
        self.data[ptr] = (name.len() as u32 | flags) as i32;
        for c in name.chars() {
            // install the name
            ptr += 1;
            self.data[ptr] = c as i32;
        }
        for val in args {
            ptr += 1;
            self.data[ptr] = *val;
        }
        ptr += 1;
        self.data[ptr] = back as i32; // the new back pointer
        self.data[self.here_ptr] = ptr as i32; // top of the stack = HERE
        self.data[self.context_ptr] = back as i32 + 1; // context is the len/flags field of this word
        ptr // updated HERE location
    }

    pub fn at_execute(&mut self, addr: u32) {
        // execute a builtin word, identified by an index into data space

        macro_rules! pop2_push1 {
            // Helper macro
            ($expression:expr) => {{
                let (top, next) = (self.s_pop(), self.s_pop());
                self.s_push($expression(next, top));
            }};
        }
        macro_rules! pop1_push1 {
            // Helper macro
            ($expression:expr) => {{
                let top = self.s_pop();
                self.s_push($expression(top));
            }};
        }
        macro_rules! pop1 {
            ($expression:expr) => {{
                let top = s_pop();
                $expression(top);
            }};
        }

        match addr {
            1 => {
                // ! store
                let addr = self.s_pop() as usize;
                let val = self.s_pop() as u32;
                self.d_store(addr, val as i32);
            }
            2 => {} // , allocate a cell
            3 => {
                // @ fetch from addr on stack
                let addr = self.s_pop() as u32;
                self.s_push(addr.try_into().unwrap());
            }
            4 => {
                // ACCEPT - read a line of up to n chars into TIB, setting #TIB and >N
            }
            5 => {
                // TEXT - take a delimiter off the stack and get text up to delim or \n from TIB
                // put in TAB, and leave the address on the stack
            }
            6 => {
                // WORD - get a token delimited by a char from the stack (usually a space), insert it HERE
            }
            7 => {}                                                                // >NUMBER
            8 => self.s_push(TIB_START as i32),                                    // TIB
            9 => self.s_push((DATA_SIZE - TIB_START) as i32),                      // #TIB
            10 => self.s_push(self.tib_in_ptr as i32),                             // >IN
            11 => self.s_push(self.here_ptr as i32),                               // HERE
            12 => {}                                                               // ABORT
            13 => {}                                                               // BYE
            14 => self.exit_flag = true,                                           // EXIT
            15 => pop2_push1!(|next, top| next + top),                             // +
            16 => pop2_push1!(|next, top| next - top),                             // -
            17 => pop2_push1!(|next, top| next * top),                             // *
            18 => pop2_push1!(|next, top| next / top),                             // /
            19 => pop1_push1!(|top| top << 1),                                     // 2*
            20 => pop1_push1!(|top| top >> 1),                                     // 2/
            21 => {}                                                               // */MOD
            22 => pop2_push1!(|next, top| next % top),                             // MOD
            23 => pop1_push1!(|top| if top == 0 { TRUE } else { FALSE }),          // 0=
            24 => pop2_push1!(|next, top| if next == top { TRUE } else { FALSE }), // =
            25 => pop2_push1!(|next, top| if next < top { TRUE } else { FALSE }),  // <
            26 => pop2_push1!(|next, top| if next > top { TRUE } else { FALSE }),  // >
            27 => pop2_push1!(|next, top| next & top),                             // AND
            28 => pop2_push1!(|next, top| next | top),                             // OR
            29 => pop1_push1!(|top: i32| !top),                                    // INVERT
            30 => pop2_push1!(|next, top| next ^ top),                             // XOR
            31 => self.s_push(TRUE),                                               // TRUE
            32 => self.s_push(FALSE),                                              // FALSE
            33 => pop2_push1!(|next, top| next << top),                            // LSHIFT
            34 => pop2_push1!(|next, top| next >> top),                            // RSHIFT
            35 => self.s_push(self.s_top()),                                       // DUP
            36 => self.stack_ptr -= 1,                                             // DROP
            37 => {
                // SWAP
                let top = self.s_pop();
                let next = self.s_pop();
                self.s_push(top);
                self.s_push(next);
            }
            38 => {
                // OVER
                let val = self.s_get(1);
                self.s_push(val);
            }
            39 => {
                // >R
                let val = self.s_pop();
                self.r_push(val);
            }
            40 => {
                // R>
                let val = self.r_pop();
                self.s_push(val);
            }
            41 => self.s_push(self.r_top()), // R@
            42 => {
                // ROT
                let n2 = self.s_pop();
                let n3 = self.s_pop();
                let n1 = self.s_pop();
                self.s_push(n3);
                self.s_push(n2);
                self.s_push(n1);
            }
            43 => {} // [
            44 => {} // $INTERPRET
            45 => {} // .OK
            46 => {} // EVAL
            47 => {} // PRESET
            48 => {} // QUIT
            49 => {} // QUERY
            50 => {} // TOKEN
            51 => {} // PARSE
            52 => {} // (PARSE)
            53 => {} // "
            54 => {} // ."
            55 => {
                // ' find a name in the dictionary if it's there
                // loop through the back pointers
                let name = "while";
                let addr = self.find_word(name);
                if addr > 0 {
                    self.s_push(addr as i32);
                }
            } // '         Find a definition and return its address
            56 => {} // EXECUTE   Execute a definition with its address on the stack
            57 => {} // :
            58 => {} // ;
            59 => {} // CONSTANT
            60 => {} // VARIABLE
            61 => {} // CREATE    Defines compile-time behavior of a word
            62 => {} // DOES>     Defines run-time behavior of a word
            63 => {
                // KEY
                let c_result = self.reader.key();
                match c_result {
                    Ok(c) => self.s_push(c as i32),
                    Err(s) => println!("{}", s),
                }
            }
            64 => {}                               // KEY?
            65 => print!("{}", self.s_pop()),      // EMIT
            66 => println!(""),                    // CR
            67 => {}                               // (
            68 => {}                               // \
            69 => {}                               // .S
            70 => {}                               // IMMEDIATE
            71 => {}                               // BRANCH
            72 => {}                               // BRANCH0
            73 => {}                               // IF
            74 => {}                               // ELSE
            75 => {}                               // THEN
            76 => {}                               // BEGIN
            77 => {}                               // WHILE
            78 => {}                               // AGAIN
            79 => {}                               // REPEAT
            80 => {}                               // UNTIL
            81 => {}                               // DO
            82 => {}                               // LOOP
            83 => {}                               // I
            84 => {}                               // J
            _ => println!("Unrecognized builtin"), // not a builtin
        }
    }

    fn insert_builtins(&mut self) -> usize {
        // place builtin data at the front of the data space
        // returns the updated data pointer
        // format is as follows:
        //      cell 1 is a back-pointer (0 to terminate)
        //      cell 2 is the name length ORed with the BUILTIN flag
        //      cell 3, 4.. as required are the name
        //
        // flags (used by EXECUTE) are as follows:
        //      1 << 26: immediate
        //      1 << 27: builtin
        //      1 << 28: constant
        //      1 << 29: variable
        //      1 << 30: string
        //      1 << 31: word
        //
        const BUILTIN_CONFIG: [(i32, &'static str); 84] = [
            (1, "!"),
            (2, ","),
            (3, "@"),
            (4, "accept"),
            (5, "text"),
            (6, "word"),
            (7, ">number"),
            (8, "tib"),
            (9, "#tib"),
            (10, ">in"),
            (11, "here"),
            (12, "abort"),
            (13, "bye"),
            (14, "exit"),
            (15, "+"),
            (16, "-"),
            (17, "*"),
            (18, "/"),
            (19, "2*"),
            (20, "2/"),
            (21, "*/mod"),
            (22, "mod"),
            (23, "0="),
            (24, "="),
            (25, "<"),
            (26, ">"),
            (27, "and"),
            (28, "or"),
            (29, "invert"),
            (30, "xor"),
            (31, "true"),
            (32, "false"),
            (33, ""),
            (34, "lshift"),
            (35, "rshift"),
            (36, "dup"),
            (37, "drop"),
            (38, "over"),
            (39, ">r"),
            (40, "r>"),
            (41, "r@"),
            (42, "rot"),
            (43, "["),
            (44, "$interpret"),
            (45, ".ok"),
            (46, "eval"),
            (47, "preset"),
            (48, "quit"),
            (49, "query"),
            (50, "token"),
            (51, "parse"),
            (52, "(parse)"),
            (53, "\""),
            (54, ".\""),
            (55, "'"),
            (56, "execute"),
            (57, ":"),
            (58, ";"),
            (59, "constant"),
            (60, "variable"),
            (61, "create"),
            (62, "does>"),
            (63, "key"),
            (64, "key?"),
            (65, "emit"),
            (66, "CR"),
            (67, "("),
            (68, "\\"),
            (69, ".s"),
            (70, "immediate"),
            (71, "branch"),
            (72, "branch0"),
            (73, "if"),
            (74, "else"),
            (75, "then"),
            (66, "begin"),
            (77, "while"),
            (78, "again"),
            (79, "repeat"),
            (80, "until"),
            (81, "do"),
            (82, "loop"),
            (83, "i"),
            (84, "j"),
        ];

        let mut ptr = self.data[self.here_ptr] as usize;
        for (code, name) in BUILTIN_CONFIG.iter() {
            ptr = self.make_word(&name, BUILTIN, &[*code]);
        }

        // record in the HERE variable
        self.data[self.here_ptr] = ptr as i32;
        ptr // now points to the new back pointer
    }

    fn insert_variables(&mut self) -> usize {
        // install system variables in data area
        // hand craft HERE, because it's needed by make_word
        self.data[0] = 0; // null pointer
        self.data[1] = (4 | VARIABLE) as i32; //
        for (i, c) in "here".char_indices() {
            self.data[i + 2] = c as i32;
        }
        self.data[6] = 7; // the value of HERE
        self.data[7] = 0; // back pointer
        self.here_ptr = 6; // the address of the HERE variable

        // hand craft CONTEXT, because it's needed by make_word
        self.data[8] = (7 | VARIABLE) as i32;
        for (i, c) in "context".char_indices() {
            self.data[i + 9] = c as i32;
        }
        self.data[16] = 8; // value of CONTEXT
        self.data[17] = 7; // back pointer
        self.context_ptr = 16;
        self.data[self.here_ptr] = 17;

        self.base_ptr = self.make_variable("base") - 1;
        self.data[self.base_ptr] = 10;
        self.tmp_ptr = self.make_variable("tmp") - 1;
        self.tib_in_ptr = self.make_variable(">in") - 1;
        self.data[self.tib_in_ptr] = TIB_START as i32;
        self.hld_ptr = self.make_variable("hld") - 1;
        self.tick_eval_ptr = self.make_variable("'eval"); // value needs to be set before calling EVAL
        self.last_ptr = self.make_variable("last") - 1;
        self.data[self.here_ptr] as usize
    }

    fn insert_constants(&mut self) -> usize {
        // install system constants in data area
        1099
    }

    fn s_push(&mut self, val: i32) {
        // Push one item on the Forth stack
        self.data[self.stack_ptr] = val;
        self.stack_ptr += 1;
    }

    fn s_pop(&mut self) -> i32 {
        // Pop one item off the Forth stack
        if self.stack_ptr > 0 {
            self.stack_ptr -= 1;
            self.data[self.stack_ptr]
        } else {
            self.msg.error("POP", "Stack underflow", None::<bool>);
            0
        }
    }

    fn s_top(&self) -> i32 {
        self.data[self.stack_ptr - 1]
    }

    fn s_get(&self, delta: usize) -> i32 {
        // arg of 0 means top of stack; 1 means 1 lower etc.
        self.data[self.stack_ptr - delta - 1]
    }

    fn r_push(&mut self, val: i32) {
        // Push one item on the Return stack
        self.data[self.ret_ptr] = val;
        self.ret_ptr -= 1;
    }

    fn r_pop(&mut self) -> i32 {
        // Pop one item off the Return stack
        if self.ret_ptr > RET_START {
            self.ret_ptr += 1;
            self.data[self.ret_ptr] as i32
        } else {
            self.msg.error("POP", "Stack underflow", None::<bool>);
            0
        }
    }

    fn r_top(&self) -> i32 {
        self.data[self.ret_ptr + 1]
    }

    fn r_get(&self, delta: usize) -> i32 {
        // arg of 0 means top of stack; 1 means 1 lower etc.
        self.data[self.ret_ptr - delta - 1]
    }

    fn d_get(&self, addr: usize) -> i32 {
        self.data[addr]
    }

    fn d_store(&mut self, addr: usize, val: i32) {
        self.data[addr] = val;
    }

    fn d_push(&mut self, val: i32) {
        // adds a new value at HERE (first empty cell), updating the pointer
        self.here_ptr += 1;
        self.data[self.here_ptr] = val;
    }
}
