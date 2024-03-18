# Documentation for built-in functions.

This is not intended to be full documentation, but a reference to the stack behavior and basic functions of some important words to help with debugging. f2 supports most of the standard stack and arithmetic words.

## System Variables

| WORD | Notes                                                                                                           |
| ---- | --------------------------------------------------------------------------------------------------------------- |
| 'tib | The address of the text input buffer. TIB is not counted, but the first byte is reserved and not used for text. |
| #tib | The length of the line currently in TIB.                                                                         |
| >in  | Pointer to the first unconsumed character in TIB.                                                                |
| pad  | Address of the temporary string buffer PAD. PAD is a counted string, used by the parser to hold the current token during interpretation.                                           |
tmp | Address of a second temporary string buffer used by string functions to stage new strings.
| here | The location of the top of the dictionary, where new elements will be added.                                   |
s-here | The location of the top of string space, where new strings will be added.
context | Holds the address of the most recent word's name field
last | Holds the address of the name field of the word being defined.
| base | Radix for numberic I/O. Defaults to 10.    
state | Set to TRUE if compile mode is active, otherwise FALSE.
stepper | Controls the stepper / debugger. 0 => off, 1 => trace, -1 => single step.                                                                     |

## System Commands
| WORD | SIGNATURE  |  NOTES |
| ------ | ------- | -------- |
system" \<shell command>" | ( -- ) | Runs a shell command and returns the output to stdout, printed into the output stream. For example, `system" ls -l"` will pass `ls -l` to sh for execution. `system"` blocks until the command is complete.
(system) | ( s -- ) | Takes a string pointer on the stack and passes the string to `sh` for execution. Used by `system"`.

## I/O

| WORD          | SIGNATURE      | NOTES                                                                                      |
| ------------- | -------------- | ------------------------------------------------------------------------------------------ |
| query         | ( -- )         | Read a line of Forth from the terminal. Store in TIB and set #TIB and >IN variables        |
| accept        | ( b u -- b u ) | Read up to u characters, placing them in b. Return the number of characters actually read. |
| emit          | ( c -- )       | Print a character, if it's in the printable range from space to 0x7F.                      |
| flush         | ( -- )         | Force the output buffer to be flushed to the terminal.                                     |
space | ( -- ) | Prints a single space.
spaces | ( u -- ) | Prints u spaces.
| .s            | ( -- )         | Print the contents of the stack. Does not consume stack elements.                          |
| .             | ( v -- )       | Print the top of the stack as an integer.                                                  |
u. | ( u -- ) | Print the top of the stack as an unsigned value
u.r | ( u w -- ) | Print unsigned u right-justified in a field w wide. If w is too small, print the full number anyway
.r | ( n w -- ) | Print integer n right-justified in a field w wide. If w is too small, print the full number anyway
| cr            | ( -- )         | Print a newline.                                                                           |
| s" \<string>" | ( -- )         | Print the inline string                                                                    |
| type          | ( s -- )       | Print a string, using the top of stack as a pointer to the string.                         |
ltype | ( s w -- ) | Print a string left justified in a field w characters wide. If w is too small, print the entire string anyway.
rtype | ( s w -- ) | Print a string right justified in a field w characters wide. If w is too small, print the entire string anyway.
tell | ( s u -- ) | Print the string at s, of length u
ltell | ( s u w -- ) | Print a string of length u left justified in a field w characters wide. If w is too small, print the entire string anyway.
rtell | ( s u w -- ) | Print a string of length u right justified in a field w characters wide. If w is too small, print the entire string anyway.
| r/w           | ( -- )         | Set file mode to read/write, for file operations.                                          |
| r/o           | ( -- )         | Set file mode to read only, for file operations.                                           |
w/o | ( -- ) | Set file mode to write-only, for file operations.
open-file | ( s u fam -- file-id ior ) | Open the file named at `s`, string length `u`, with file access mode `fam`. The file-id is an index into a vector of open files, within which the information for the file is kept. This can be accessed by other operations like `file-size` and `file-position`. ior is an i/o system result provided by the operating system. 0 means success. 
close-file | ( file-id -- ior ) | Close the file associated with file-id, returning a code indicating success or failure.
read-line | ( s u file-id -- u flag ior ) | Read up to `u` characters from a file, stopping at the first linefeed, or at the max length `u`. Returns the number of characters read, a flag indicating success or failure, and an io result code.
write-line | ( s u file-id -- ior ) | Write `u` characters from `s` to a file, returning an i/o result code `ior`.

## Text interpreter and Compiler

| WORD       | SIGNATURE                 | NOTES                                                                                                                                                                                                                                 |
| ---------- | ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
words |  ( -- ) | Prints a list of all dictionary entries, whether words, builtins, variables or constants. Each word is preceded by its address in the dictionary for debugging purposes.
abort | ( -- ) | Ends execution of the current word, clears the stack, and returns to the interpreter's top level
abort" \<message>" | ( -- ) | Print the message and call abort
| quit       | ( -- )                    | Interpreter outer loop: gets a line of input, processes it. Calls `query` and `eval` to do the work.                                                                                                                                  |
| eval       | ( -- )                    | Interprets a line of input from the `TIB`. Exits when the line is finished, or if `abort` is called.                                                                                                                                  |
| text       | ( -- b u )                | Gets a space-delimited token from the `TIB`, starting at offset `>IN`. Places it in `PAD`. Returns the address of `PAD` and the number of characters in the token, or 0 if no token could be ready (typically end of line condition). |
| \\         | ( -- )                    | Inline comment. Causes the remainder of the line to be ignored.                                                                                                                                                                       |
| (          | ( -- )                    | Text from the left paren to its maching closing paren is ignored. Used for documenting stack signatures in word definitions.                                                                                                          |
| parse      | ( c -- b u )              | Gets a token from `PAD` delimited by `c`. Returns `PAD` address and count.                                                                                                                                                            |
| (parse)    | ( b u c -- b u delta )    | Find a `c`-delimited token in the string buffer at `b`, of length `u`. Return the pointer to the buffer, the length of the token, and the offset from the start of the buffer to the start of the token.
[char] | ( -- c )                             | Place the first character of the next token on the stack. Consumes the entire token.
| find       | ( s -- cfa T \| s FALSE ) | Search the dictionary for the token with string at s. Used by $interpret and $compile to identify the current token.                                                                                                                  |
| ' \<name>  | ( -- cfa \| FALSE )       | Looks for the (postfix) name in the dictionary. Returns its code field address if found, otherwise FALSE (= 0). If the word is not found, it displays an error message.                                                               |
| unique?    | ( s -- s )                | Checks to see if the given string is already defined. If so, returns quietly; otherwise returns `FALSE`.                                                                                                                              |
| :          | ( -- )                    | Sets compile mode to start a definition                                                                                                                                                                                               |
| [          | ( -- )                    | Immediate: set  state to interpret mode. Used to force interpretation inside a definition.                                                                                                                                            |
| ]          | ( -- )                    | Set state to compile mode.  Used inside a definition to undo the effect of a previous `[`.                                                                                                                                            |
| number?    | (s -- n T \| s F )        | Attempts to convert the string at s to a number. If successful, push the number and a `TRUE` flag. If not successful, leave the string address on the stack, and push `FALSE`. Used inside `$compile` and `$interpret`.               |
| literal    | ( n -- )                  | Takes a number from the stack and compiles it into the current definition.                                                                                                                                                            |
| $interpret | ( s -- )                  | Called from `eval` to interpret the string at s, either as a word or a number. If neither, `abort`.                                                                                                                                   |
| $compile   | ( s -- )                  | Called from `eval` to compile the string at s as a word or number. If neither, `abort`.        |
, (comma) | ( v -- ) | Compiles the value on the stack into the dictionary and updates `here`.
create \<name> | ( -- ) | Takes a postfix name, and creates a new name field in the dictionary
immediate | ( -- ) | Marks the most recent definition as immediate by setting a flag on the name field. Immediate words are executed even when compile mode is set. They are most often used to compile control structures that need some level of computation at compile time.
immed? ( cfa -- T | F ) | Tests the word with code field address on the stack, and returns TRUE if it's an immediate word, otherwise FALSE.
[compile] | \<name> | Delays the compilation of an immediate word. Typically used in the definition of control structures and compiler customization.
forget-last | ( -- ) | Delete the last definition from the dictionary. 
forget | \<name> | Delete word `<name>` and any words defined more recently than `<name>`.

## Timing and Delay
To time a function, precede it with `now` and follow it with `millis` or `micros`, which will place the elapsed time on the stack.

| WORD       | SIGNATURE                 | NOTES                                                                                                                                                                                                                                 |
| ---------- | ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
now | ( -- ) | Captures the current time using Rust's `std::time::Instant` capability
millis | ( -- n ) | Places the number of milliseconds since `now` was called on the stack
micros  | ( -- n ) | Places the number of microseconds since `now` was called on the stack
ms | ( n -- ) | Sleep for `n` milliseconds
sec | ( n -- ) | Sleep for `n` seconds

## Debugging
A single stepper and trace capability allows for viewing interpreted functions as they execute. When active, it prints a visual indication of the depth of the return stack, the contents of the stack, and the word being executed.

The single stepper responds to single character commands (followed by Enter):
* `s` => take a single step
* `t` => shift to trace mode
* `o` => turn the stepper off

WORD | SIGNATURE | NOTES
--- | --- | ---
step-on | ( -- ) | Turns on single stepping.
step-off | ( -- ) | Turns off single stepping.
trace-on | ( -- ) | Turns on tracing.
trace-off | ( -- ) | Turns off tracing.