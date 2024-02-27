# Documentation for built-in functions.

This is not intended to be full documentation, but a reference to the stack behavior and basic functions of some important words to help with debugging.

## System Variables

| WORD | Notes                                                                                                           |
| ---- | --------------------------------------------------------------------------------------------------------------- |
| 'TIB | The address of the text input buffer. TIB is not counted, but the first byte is reserved and not used for text. |
| #TIB | The length of the line currently in TIB                                                                         |
| >IN  | Pointer to the first unconsumed character in TIB                                                                |
| PAD  | Address of the temporary string buffer PAD. PAD is a counted string                                             |
| HERE | The location of the top of the dictionary, where new elements will be added                                     |
| BASE | Radix for numberic I/O. Defaults to 10.                                                                         |

## I/O

| WORD          | SIGNATURE      | NOTES                                                                                      |
| ------------- | -------------- | ------------------------------------------------------------------------------------------ |
| query         | ( -- )         | Read a line of Forth from the terminal. Store in TIB and set #TIB and >IN variables        |
| accept        | ( b u -- b u ) | Read up to u characters, placing them in b. Return the number of characters actually read. |
| emit          | ( c -- )       | Print a character, if it's in the printable range from space to 0x7F.                      |
| flush         | ( -- )         | Force the output buffer to be flushed to the terminal.                                     |
| .s            | ( -- )         | Print the contents of the stack.                                                           |
| .             | ( v -- )       | Print the top of the stack as an integer.                                                  |
| cr            | ( -- )         | Print a newline.                                                                           |
| s" \<string>" | ( -- )         | Print the inline string                                                                    |
| type          | ( s -- )       | Print a string, using the top of stack as a pointer to the string.                         |
| r/w           | ( -- )         | Set file mode to read/write, for file operations.                                          |
| r/o           | ( -- )         | Set file mode to read only, for file operations.                                           |

## Text interpreter and Compiler

| WORD       | SIGNATURE                     | NOTES                                                                                                                                                                                                                                 |
| ---------- | ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| quit       | ( -- )                        | Interpreter outer loop: gets a line of input, processes it. Calls `query` and `eval` to do the work.                                                                                                                                  |
| eval       | ( -- )                        | Interprets a line of input from the `TIB`. Exits when the line is finished, or if `abort` is called.                                                                                                                                  |
| text       | ( -- b u )                    | Gets a space-delimited token from the `TIB`, starting at offset `>IN`. Places it in `PAD`. Returns the address of `PAD` and the number of characters in the token, or 0 if no token could be ready (typically end of line condition). |
| \\         | ( -- )                        | Inline comment. Causes the remainder of the line to be ignored.                                                                                                                                                                       |
| (          | ( -- )                        | Text from the left paren to its maching closing paren is ignored. Used for documenting stack signatures in word definitions.                                                                                                          |
| parse      | ( c -- b u )                  | Gets a token from `PAD` delimited by `c`. Returns `PAD` address and count.                                                                                                                                                            |
| (parse)    | ( b u c -- b u delta )        | Find a `c`-delimited token in the string buffer at `b`, of length `u`. Return the pointer to the buffer, the length of the token, and the offset from the start of the buffer to the start of the token.                              |
| find       | ( s -- nfa cfa T \| s FALSE ) | Search the dictionary for the token with string at s. Used by $interpret and $compile to identify the current token.                                                                                                                  |
| ' \<name>  | ( -- cfa \| FALSE )           | Looks for the (postfix) name in the dictionary. Returns its code field address if found, otherwise FALSE (= 0). If the word is not found, it displays an error message.                                                               |
| unique?    | ( s -- s )                    | Checks to see if the given string is already defined. If so, returns quietly; otherwise returns `FALSE`.                                                                                                                              |
| :          | ( -- )                        | Sets compile mode to start a definition                                                                                                                                                                                               |
| [          | ( -- )                        | Immediate: set  state to interpret mode. Used to force interpretation inside a definition.                                                                                                                                            |
| ]          | ( -- )                        | Set state to compile mode.  Used inside a definition to undo the effect of a previous `[`.                                                                                                                                            |
| number?    | (s -- n T \| s F )            | Attempts to convert the string at s to a number. If successful, push the number and a `TRUE` flag. If not successful, leave the string address on the stack, and push `FALSE`. Used inside `$compile` and `$interpret`.               |
| literal    | ( n -- )                      | Takes a number from the stack and compiles it into the current definition.                                                                                                                                                            |
| $interpret | ( s -- )                      | Called from `eval` to interpret the string at s, either as a word or a number. If neither, `abort`.                                                                                                                                   |
| $compile   | ( s -- )                      | Called from `eval` to compile the string at s as a word or number. If neither, `abort`.                                                                                                                                               |