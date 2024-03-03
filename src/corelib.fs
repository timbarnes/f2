: parse pad @ swap parse-to ;                       
: \ 1 parse drop drop ; immediate                   \ Implements comments to end of line 
: ( 41 parse drop drop ; immediate                  \ Implements in-line comments

\ Constants referring to inner interpreter opcodes, which are typically compiled into definitions
1000 constant BUILTIN
1001 constant VARIABLE
1002 constant CONSTANT
1003 constant LITERAL
1004 constant STRLIT
1005 constant definition
1006 constant BRANCH
1007 constant BRANCH0
1008 constant ABORT
1009 constant EXIT

\ ASCII symbols that are useful for text processing
32 constant BL
34 constant DQUOTE
39 constant SQUOTE
41 constant RPAREN

: text BL parse ;                                   \ Parser shortcut for space-delimited tokens
: s-parse tmp @ swap parse-to ;                     \ Same as text, but loads to tmp instead of pad
: s" ( delim -- s u ") tmp @ DQUOTE parse-to ;      \ Places a double-quoted string in tmp

1 dbg \ set debuglevel to warnings and errors

( File reader functions )
: included tmp @ include-file ; \ include-file uses a string pointer on the stack to load a file
: include tmp @ 32 parse-to included ;

( inner interpreter op codes, used by control structures )

\ Untested implementation of recursion support
: recurse ( -- ) \ Simply compiles the cfa of the word being defined
    last @ 1 + , ; immediate \ last points to the latest nfa, so increment

: .tmp tmp @ type ;
: .pad pad @ type ;
: ." s" .tmp ;
: ' (') @ ;
: ['] LITERAL , ' , ; immediate                     \ compiles a word's cfa into a definition as a literal
: >nfa 1 - ;                                        \ converts an cfa to an nfa
: >cfa 1 + ;                                        \ converts an nfa to a cfa

 : if BRANCH0 , here @  0 , ; immediate
 : else BRANCH , here @ 0 , swap dup here @ swap - swap ! ; immediate
 : then dup here @ swap - swap ! ; immediate

: for here @ ['] >r , ; immediate
: next ['] r> , LITERAL , 1 , ['] - , ['] dup , ['] 0= , BRANCH0 , here @ - , ['] drop , ; immediate

: begin here @ ; immediate
: until BRANCH0 , here @ - , ; immediate
: while BRANCH0 , here @ 0 , ;  immediate
: repeat BRANCH , swap here @ - , dup here @ swap - swap ! ; immediate

: 1- ( n -- n-1 ) 1 - ;
: 1+ ( n -- n+1 ) 1 + ;
: negate ( n -- -n ) 0 swap - ;
: not 0= ;
: nip ( a b -- b ) swap drop ;
: tuck ( a b -- b a b ) swap over ;
: pop ( a -- ) drop ;
: 2dup ( a b -- a b a b ) over over ;
: ?dup dup 0= if dup else then ;
: > < if false else true then ;
: <> ( n -- n ) = 0= ;
: min ( m n -- m | n ) 2dup < if drop else nip then ;
: max ( m n -- m | n ) 2dup > if drop else nip then ;
: abs ( n -- n | -n ) dup 0 < if -1 * then ;

: space ( -- ) BL emit ;
: spaces ( n -- ) 1- for space next ;

\ Implementation of word
: .word ( bp -- bp ) dup 1+ @ type space @ ;                \ prints a word name, given the preceding back pointer
: words ( -- ) here @ 1- @ begin .word dup not until ;   \ loops through the words in the dictionary

: dbg-debug 3 dbg ;
: dbg-info 2 dbg ;
: dbg-warning 1 dbg ;
: dbg-quiet 0 dbg ;
: debug show-stack step-on ;

\ : ?stack depth 0= if ." Stack underflow" abort then ;


: +! ( n addr -- ) dup @ rot + swap ! ;
: ? ( addr -- ) @ . ;

\ s" src/regression.fs" 
\ : run-regression include ;


( Application functions )

: _fac ( r n -- r )   \ Helper function that does most of the work.
    dup if 
        tuck * swap 1 - recurse 
    else 
        drop 
    then ;

: fac ( n -- n! )   \ Calculates factorial of a non-negative integer. No checks for stack or calculation overflow.
    dup 
        if 
            1 swap _fac 
        else 
            drop 1 
        then ;

 ." Library loaded."
