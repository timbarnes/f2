/// Interpreter for builtins
///
/// Set up a table of builtin functions, with names and code

#[allow(dead_code)]
use crate::engine::{BUILTIN_MASK, FALSE, STR_START, TF, TIB_START, VARIABLE};
use crate::engine::{PAD_START, TMP_START};

pub trait BuiltinCall {
    fn call(&mut self);
}

pub struct BuiltInFn {
    pub name: String,
    pub code: for<'a> fn(&'a mut TF),
    pub doc: String,
}

impl BuiltinCall for BuiltInFn {
    fn call(&mut self) {}
}

impl BuiltInFn {
    pub fn new(name: String, code: for<'a> fn(&'a mut TF), doc: String) -> BuiltInFn {
        BuiltInFn { name, code, doc }
    }
}

impl TF {
    pub fn u_insert_variables(&mut self) {
        // install system variables in data area
        // hand craft S-HERE (free string pointer) so write_string() can work
        self.data[0] = 0;
        self.data[1] = 0;
        self.data[2] = STR_START as i64; //
        self.strings[STR_START] = 6 as char; // length of "s-here"
        for (i, c) in "s-here".chars().enumerate() {
            self.strings[i + STR_START + 1] = c;
        }
        self.string_ptr = 4;
        self.data[3] = VARIABLE;
        self.data[4] = (STR_START + 7) as i64; // update the value of S-HERE
        self.data[5] = 1; // back pointer
                          // hand craft HERE, because it's needed by make_word
        let name_pointer = self.u_new_string("here");
        self.data[6] = name_pointer as i64;
        self.data[7] = VARIABLE;
        self.data[8] = 10; // the value of HERE
        self.data[9] = 5; // back pointer
        self.here_ptr = 8; // the address of the HERE variable

        // hand craft CONTEXT, because it's needed by make_word
        self.data[10] = self.u_new_string("context") as i64;
        self.data[11] = VARIABLE;
        self.data[12] = 10;
        self.data[13] = 9; // back pointer
        self.context_ptr = 12;
        self.data[self.here_ptr] = 14;

        self.pad_ptr = self.u_make_variable("pad");
        self.data[self.pad_ptr] = PAD_START as i64;
        self.base_ptr = self.u_make_variable("base");
        self.data[self.base_ptr] = 10; // decimal
        self.tmp_ptr = self.u_make_variable("tmp");
        self.data[self.tmp_ptr] = TMP_START as i64;
        self.tib_ptr = self.u_make_variable("'tib");
        self.data[self.tib_ptr] = 0;
        self.tib_size_ptr = self.u_make_variable("#tib");
        self.data[self.tib_size_ptr] = 0;
        self.tib_in_ptr = self.u_make_variable(">in");
        self.data[self.tib_in_ptr] = TIB_START as i64 + 1;
        self.hld_ptr = self.u_make_variable("hld");
        self.last_ptr = self.u_make_variable("last"); // points to nfa of new definition
        self.state_ptr = self.u_make_variable("'eval");
        self.abort_ptr = self.u_make_variable("abort?");
        self.state_ptr = self.u_make_variable("state");
        self.data[self.abort_ptr] = FALSE;
    }

    /// Insert Forth code into the dictionary
    pub fn u_insert_code(&mut self) {
        // self.u_interpret("2 2 + .");
    }

    /// u_write_string writes a new string into the next empty space, updating the free space pointer
    fn u_new_string(&mut self, string: &str) -> usize {
        // place a new str into string space and update the free pointer string_ptr
        let mut ptr = self.data[self.string_ptr] as usize;
        let result_ptr = ptr;
        self.strings[ptr] = string.len() as u8 as char;
        ptr += 1;
        for (i, c) in string.chars().enumerate() {
            self.strings[ptr + i] = c;
        }
        self.data[self.string_ptr] = (ptr + string.len()) as i64;
        result_ptr
    }

    /// make-variable Create a variable, returning the address of the variable's value
    fn u_make_variable(&mut self, name: &str) -> usize {
        let code_ptr = self.u_make_word(&name, &[VARIABLE, 0]); // install the name
        code_ptr + 1 // the location of the variable's value
    }

    /* fn u_make_constant(&mut self, name: &str, val: i64) -> usize {
           // Create a constant
           let code_ptr = self.u_make_word(name, &[val]); // install the name
           code_ptr + 1
       }
    */
    /// u_make_word Install a new word with provided name and arguments
    /// back link is already in place
    /// place it HERE
    /// update HERE and LAST
    /// return pointer to first parameter field - the code field pointer or cfa
    fn u_make_word(&mut self, name: &str, args: &[i64]) -> usize {
        let back = self.data[self.here_ptr] as usize - 1; // the top-of-stack back pointer's location
        let mut ptr = back + 1;
        self.data[ptr] = self.u_new_string(name) as i64;
        for val in args {
            ptr += 1;
            self.data[ptr] = *val;
        }
        ptr += 1;
        self.data[ptr] = back as i64; // the new back pointer
        self.data[self.here_ptr] = ptr as i64 + 1; // start of free space = HERE
        self.data[self.context_ptr] = back as i64 + 1; // context is the name_pointer field of this word
        back + 2 // address of first parameter field
    }

    fn u_add_builtin(&mut self, name: &str, code: for<'a> fn(&'a mut TF), doc: &str) {
        self.builtins
            .push(BuiltInFn::new(name.to_owned(), code, doc.to_string()));
        // now build the DATA space record
        let cfa = (self.builtins.len() - 1) | BUILTIN_MASK;
        self.u_make_word(name, &[cfa as i64]);
    }

    pub fn add_builtins(&mut self) {
        // Inner interpreters occupy the addresses lower than 100
        self.u_add_builtin(
            "_builtin",
            TF::i_builtin,
            "_builtin  opcode -- executes the builtin on the stack",
        );
        self.u_add_builtin(
            "_variable",
            TF::i_variable,
            "variable  opcode -- loads a variable's address on the stack",
        );
        self.u_add_builtin(
            "_constant",
            TF::i_constant,
            "_constant  opcode -- loads a constant's value on the stack",
        );
        self.u_add_builtin(
            "_literal",
            TF::i_literal,
            "_literal  opcode -- loads a number on the stack",
        );
        self.u_add_builtin(
            "_stringlit",
            TF::i_strlit,
            "_stringlit  opcode -- loads a string pointer on the stack",
        );
        self.u_add_builtin(
            "_definition",
            TF::i_builtin,
            "_definition  opcode -- executes a colon definition",
        );
        self.u_add_builtin(
            "_branch",
            TF::i_branch,
            "_branch  opcode -- executes an unconditional branch",
        );
        self.u_add_builtin(
            "_branch0",
            TF::i_branch0,
            "_branch0 opcode -- executes a branch if zero",
        );
        self.u_add_builtin("_abort", TF::f_abort, "abort ( opcode -- ) calls ABORT");
        self.u_add_builtin(
            "exit",
            TF::i_exit,
            "exit ( -- ) returns from the current definition",
        );
        self.u_add_builtin(
            "_next",
            TF::i_next,
            "_next opcode -- end of word - continue to the next one",
        );
        // Start of normal functions
        self.u_add_builtin(
            "f_marker",
            TF::f_marker,
            "marker <name> ( -- ) Places a named marker in the dictionary, to be used by FORGET",
        );
        self.u_add_builtin("+", TF::f_plus, "+ ( j k -- j+k ) Push j+k on the stack");
        self.u_add_builtin("-", TF::f_minus, "- ( j k -- j+k ) Push j-k on the stack");
        self.u_add_builtin("*", TF::f_times, "* ( j k -- j-k ) Push  -k on the stack");
        self.u_add_builtin("/", TF::f_divide, "/ ( j k -- j/k ) Push j/k on the stack");
        self.u_add_builtin("mod", TF::f_mod, "mod ( j k -- j/k ) Push j%k on the stack");
        self.u_add_builtin(
            "<",
            TF::f_less,
            "( j k -- j/k ) If j < k push true else false",
        );
        self.u_add_builtin(
            ".",
            TF::f_dot,
            ". ( n -- ) Pop the top of the stack and print it, followed by a space",
        );
        self.u_add_builtin(
            "true",
            TF::f_true,
            "true ( -- -1 ) Push the canonical true value on the stack.",
        );
        self.u_add_builtin(
            "false",
            TF::f_false,
            "false ( -- 0 ) Push the canonical false value on the stack",
        );
        self.u_add_builtin(
            "=",
            TF::f_equal,
            "= ( j k -- b ) If j == k push true else false",
        );
        self.u_add_builtin(
            "0=",
            TF::f_0equal,
            "0= ( j -- b ) If j == 0 push true else false",
        );
        self.u_add_builtin(
            "0<",
            TF::f_0less,
            "( j k -- j/k ) If j < 0 push true else false",
        );
        /*         self.u_add_builtin(
            ".s",
            TF::f_dot_s,
            ".s ( -- ) Print the contents of the calculation stack",
        ); */
        self.u_add_builtin("cr", TF::f_cr, "cr ( -- ) Print a newline");
        self.u_add_builtin(
            "show-stack",
            TF::f_show_stack,
            "show-stack ( -- ) Display the stack at the end of each line of console input",
        );
        self.u_add_builtin(
            "hide-stack",
            TF::f_hide_stack,
            "hide-stack ( -- ) Turn off automatic stack display",
        );
        self.u_add_builtin(
            "emit",
            TF::f_emit,
            "emit: ( c -- ) if printable, sends character c to the terminal",
        );
        self.u_add_builtin(
            "flush",
            TF::f_flush,
            "flush: forces pending output to appear on the terminal",
        );
        self.u_add_builtin("clear", TF::f_clear, "clear: resets the stack to empty");
        self.u_add_builtin(":", TF::f_colon, ": starts a new definition");
        self.u_add_builtin("bye", TF::f_bye, "bye: exits to the operating system");
        self.u_add_builtin(
            "dup",
            TF::f_dup,
            "dup ( n -- n n ) Push a second copy of the top of stack",
        );
        self.u_add_builtin(
            "drop",
            TF::f_drop,
            "drop ( n --  ) Pop the top element off the stack",
        );
        self.u_add_builtin(
            "swap",
            TF::f_swap,
            "swap ( m n -- n m ) Reverse the order of the top two stack elements",
        );
        self.u_add_builtin(
            "over",
            TF::f_over,
            "over ( m n -- m n m ) Push a copy of the second item on the stack on to",
        );
        self.u_add_builtin(
            "rot",
            TF::f_rot,
            "rot ( i j k -- j k i ) Move the third stack item to the top",
        );
        self.u_add_builtin(
            "and",
            TF::f_and,
            "and ( a b -- a & b ) Pop a and b, returning the logical and",
        );
        self.u_add_builtin(
            "or",
            TF::f_or,
            "or ( a b -- a | b ) Pop a and b, returning the logical or",
        );
        self.u_add_builtin("@", TF::f_get, "@: ( a -- v ) Pushes variable a's value");
        self.u_add_builtin("!", TF::f_store, "!: ( v a -- ) stores v at address a");
        self.u_add_builtin("i", TF::f_i, "Pushes the current FOR - NEXT loop index");
        self.u_add_builtin("j", TF::f_j, "Pushes the second-level (outer) loop index");
        self.u_add_builtin(
            "abort",
            TF::f_abort,
            "abort ( -- ) Ends execution of the current word and clears the stack",
        );
        self.u_add_builtin(
            "depth",
            TF::f_stack_depth,
            "depth: Pushes the current stack depth",
        );
        self.u_add_builtin(
            "key",
            TF::f_key,
            "key ( -- c | 0 ) get a character and push on the stack, or zero if none available",
        );
        self.u_add_builtin("r/w", TF::f_r_w, "");
        self.u_add_builtin("r/o", TF::f_r_o, "");
        self.u_add_builtin(
            "include-file",
            TF::f_include_file,
            "include-file ( a -- ) Taking the TOS as a pointer to 
        a filename (string), load a file of source code",
        );
        self.u_add_builtin("dbg", TF::f_dbg, "");
        self.u_add_builtin(
            "debuglevel",
            TF::f_debuglevel,
            "debuglevel ( -- ) Displays the current debug level",
        );
        self.u_add_builtin("step-on", TF::f_step_on, "");
        self.u_add_builtin("step-off", TF::f_step_off, "");
        self.u_add_builtin(
            ">r",
            TF::f_to_r,
            ">r ( n -- ) Pop stack and push value to return stack",
        );
        self.u_add_builtin(
            "r>",
            TF::f_r_from,
            "r> ( -- n ) Pop return stack and push value to calculation stack",
        );
        self.u_add_builtin(
            "r@",
            TF::f_r_get,
            "r@ ( -- n ) Push the value on the top of the return stack to the calculation stack",
        );
        self.u_add_builtin(
            "immediate",
            TF::f_immediate,
            "immediate sets the immediate flag on the most recently defined word",
        );
        self.u_add_builtin("[", TF::f_lbracket, "[ ( -- ) Exit compile mode");
        self.f_immediate();
        self.u_add_builtin("]", TF::f_rbracket, "] ( -- ) Enter compile mode");
        self.u_add_builtin(
            "quit",
            TF::f_quit,
            "quit ( -- ) Outer interpreter that repeatedly reads input lines and runs them",
        );
        self.u_add_builtin(
            "execute",
            TF::f_execute,
            "execute: interpret the word whose address is on the stack",
        );
        self.u_add_builtin(
            "interpret",
            TF::f_eval,
            "interpret: Interprets one line of Forth",
        );
        self.u_add_builtin(
            "number?",
            TF::f_number_q,
            "number? ( a -- n T | a F ) tests a string to see if it's a number;
            leaves n and flag on the stack: true if number is ok.",
        );
        self.u_add_builtin(
            "?unique",
            TF::f_q_unique,
            "?unique ( a -- b ) tests to see if the name TOS points to is in the dictionary",
        );
        self.u_add_builtin(
            "find",
            TF::f_find,
            "FIND (s -- a | F ) Search the dictionary for the token indexed through s. 
        Return it's address or FALSE if not found",
        );
        self.u_add_builtin(
            "(')",
            TF::f_tick_p,
            "(') <name> ( -- a ) searches the dictionary for a (postfix) word, returning its address",
        );
        self.u_add_builtin(
            "query",
            TF::f_query,
            "query ( -- ) Read a line from the console into TIB",
        );
        self.u_add_builtin(
            "accept",
            TF::f_accept,
            "accept ( b l1 -- b l2 ) Read up to l1 characters into the buffer at b.
        Return the pointer to the buffer and the actual number of characters read.",
        );
        self.u_add_builtin(
            "parse-to",
            TF::f_parse_to,
            "parse-to ( b c -- b u ) Get a c-delimited token from TIB, and return counted string in string buffer b",
        );
        self.u_add_builtin(
            "(parse)",
            TF::f_parse_p,
            "(parse) - b u c -- b u delta ) return the location of a delimited token in string space",
        );
        self.u_add_builtin(
            "s\"",
            TF::f_s_quote,
            "s\" Place the following string in the TMP string buffer",
        );
        self.u_add_builtin(
            "type",
            TF::f_type,
            "type: print a string using pointer on stack",
        );
        self.u_add_builtin(
            "variable",
            TF::f_variable,
            "variable <name> creates a new variable in the dictionary",
        );
        self.u_add_builtin(
            "constant",
            TF::f_constant,
            "constant <name> ( n -- ) creates and initializes a new constant in the dictionary",
        );
        self.u_add_builtin(
            "create",
            TF::f_create,
            "create <name> ( -- ) creates a name field in the dictionary",
        );
        self.u_add_builtin(
            "pack$",
            TF::f_smove,
            "pack$ ( src n dest -- ) copies a counted string to a new location",
        );
        self.u_add_builtin(
            "eval",
            TF::f_eval,
            "eval ( dest -- ) interprets a line of tokens from the TIB",
        );
        self.u_add_builtin(
            ",",
            TF::f_comma,
            ", ( n -- ) copies the top of the stack to the top of the dictionary",
        );
        self.u_add_builtin(
            ";",
            TF::f_semicolon,
            "; ( -- ) terminate a definition, resetting to interpret mode",
        );
        self.f_immediate();
        self.u_add_builtin(
            "immed?",
            TF::f_immediate_q,
            "immed? ( cfa -- T | F ) Determines if a word is immediate",
        );
        self.u_add_builtin("see", TF::f_see, "see <name> decompiles and prints a word");
        self.u_add_builtin(
            "s-create",
            TF::f_s_create,
            "s-create ( s1 -- s2 ) Copy a string to the head of free space and return its address",
        );
        /*         self.u_add_builtin(
            ".s\"",
            TF::f_dot_s_quote,
            ".s\" ( s -- ) Print a string from a string address",
        ); */
        /*         self.u_add_builtin(
            "s-parse",
            TF::f_s_parse,
            "s-parse ( c -- ) Read a delimited string into TMP",
        ); */
        self.u_add_builtin(
            "s-copy",
            TF::f_s_copy,
            "s-copy ( source dest -- ) Copy a counted string from source to dest",
        );
    }
}
