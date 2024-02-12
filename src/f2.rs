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
    data_ptr: usize,        // index into the data space "here"
    tib_in: usize,          // index into TIB
    last: usize,            // header of the last word defined
    stack_ptr: usize,       // point to next spot  on the forth stack
    ret_ptr: usize,         // point to next spot on the return stack
    alloc_ptr: usize,       // point to the top of unused ALLOC space
    compile: bool,          // compile mode indicator
    exit_flag: bool,
    reader: Reader, // for interactive input
    msg: Msg,
}

impl F2 {
    pub fn new() -> Result<F2, String> {
        if let Some(rdr) = Reader::new(None, Msg::new()) {
            Ok(F2 {
                data: [0; DATA_SIZE],
                data_ptr: WORD_START,
                tib_in: TIB_START,
                last: 0, // first word will start here with it's zero back pointer
                stack_ptr: STACK_START,
                ret_ptr: RET_START,
                alloc_ptr: ALLOC_START,
                compile: false,
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
        self.data_ptr = self.insert_builtins();
        println!("data_ptr is now {}", self.data_ptr);
        let index = self.f2_find("bye");
        println!("FIND found: {}", index);

        self.insert_variables();
        self.insert_constants();
    }

    pub fn run(&mut self) {
        // Bootstrap execution
        // Enter the read-eval-print loop
        // Return after "bye" or EOF
        self.exit_flag = true;
    }

    pub fn should_exit(&self) -> bool {
        self.exit_flag
    }

    pub fn f2_load_file(&self, _filename: &str) -> bool {
        // load the file of forth code
        true
    }

    fn f2_find(&self, word: &str) -> usize {
        // returns the address of a word on the stack, if it's defined
        // uses the data_ptr as starting point, and loops back
        print!("Finding {}..", word);
        let word_len: usize = word.len();
        let mut mismatch = false;
        let mut scanner = self.data[self.data_ptr] as usize; // the newest used back pointer
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

    fn f2_variable(&mut self) -> i32 {
        // Create a variable, placing the address on the stack, and returning the address
        // get the name from the TIB
        // build the header for a variable
        let word_addr = self.f2_word(' '); // install the name
        self.s_push(word_addr);
        word_addr
    }

    fn f2_constant(&mut self) {
        // Create a constant, taking the value from the stack
        // get the name from the TIB
        // build the header for a constant
        let word_addr = self.f2_word(' '); // install the name
        let val = self.s_pop();
        self.d_push(val);
    }

    fn f2_word(&mut self, delim: char) -> i32 {
        // get a word from the buffer, delimited by delim
        // place it HERE, returning the address on the stack
        let word_addr = 0;
        word_addr
    }

    pub fn builtin(&mut self, addr: u32) {
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
            10 => self.s_push(self.tib_in as i32),                                 // >IN
            11 => self.s_push(self.data_ptr as i32),                               // HERE
            12 => {}                                                               // CALIGNED
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
            43 => {} // IF
            44 => {} // ELSE
            45 => {} // THEN
            46 => {} // BEGIN
            47 => {} // WHILE
            48 => {} // AGAIN
            49 => {} // REPEAT
            50 => {} // UNTIL
            51 => {} // DO
            52 => {} // LOOP
            53 => {} // I
            54 => {} // J
            55 => {
                // ' find a name in the dictionary if it's there
                // loop through the back pointers
                let name = "while";
                let addr = self.f2_find(name);
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
        const BUILTIN_CONFIG: [(i32, &'static str); 72] = [
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
            (12, "caligned"),
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
            (43, "if"),
            (44, "else"),
            (45, "then"),
            (46, "begin"),
            (47, "while"),
            (48, "again"),
            (49, "repeat"),
            (50, "until"),
            (51, "do"),
            (52, "loop"),
            (53, "i"),
            (54, "j"),
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
        ];
        self.data[WORD_START] = 0; // initial cell is the end of the line
        let mut ptr = WORD_START; // the starting point
                                  // self.last = 0;

        for (code, name) in BUILTIN_CONFIG.iter() {
            let name = *name;

            self.last = ptr; // update it to this definition's back pointer
            ptr += 1; // pointing to length / flag word
            self.data[ptr] = ((*name).len() as u32 | BUILTIN) as i32; // All need this flag
            for c in name.chars() {
                ptr += 1; // next free cell for a character
                self.data[ptr] = c as i32;
            }
            ptr += 1; // now the code word (builtin index)
            self.data[ptr] = *code; // install the code word
            ptr += 1; // move ahead to the next address / back pointer
            self.data[ptr] = self.last as i32; // install the back pointer for the next word
        }
        ptr // now points to the new back pointer
    }

    fn insert_variables(&mut self) -> usize {
        // install system variables in data area
        999
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
        self.data_ptr += 1;
        self.data[self.data_ptr] = val;
    }
}
