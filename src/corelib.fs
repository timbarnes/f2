: parse pad @ swap parse-to ;                       
: \ 1 parse drop drop ; immediate                   \ Implements comments to end of line 
: ( 41 parse drop drop ; immediate                  \ Implements in-line comments

1 dbg \ set debuglevel to warnings and errors

0 constant FALSE
-1 constant TRUE

\ Constants referring to inner interpreter opcodes, which are typically compiled into definitions
1000 constant BUILTIN
1001 constant VARIABLE
1002 constant CONSTANT
1003 constant LITERAL
1004 constant STRLIT
1005 constant DEFINITION
1006 constant BRANCH
1007 constant BRANCH0
1008 constant ABORT
1009 constant EXIT

72057594037927935 constant ADDRESS_MASK                      \ wipes any flags

\ ASCII symbols that are useful for text processing
32 constant BL
34 constant '"'
39 constant '''
41 constant ')'
45 constant '-'
58 constant ':'
61 constant '='
91 constant '['
93 constant ']'

\ create fn               \ make a definition for the word fn
\ DEFINITION , 
\ LITERAL , 1 ,


: text BL parse ;                                   \ Parser shortcut for space-delimited tokens
: s-parse tmp @ swap parse-to ;                     \ Same as text, but loads to tmp instead of pad
: s" ( -- s u ") tmp @ '"' parse-to ;         \ Places a double-quoted string in tmp

( File reader functions )
: included tmp @ include-file ; \ include-file uses a string pointer on the stack to load a file
: include tmp @ 32 parse-to included ;

: [ FALSE state ! ; immediate                                \ Turns compile mode off
: ] TRUE state ! ;                                  \ Turns compile mode on

\ Untested implementation of recursion support
: recurse ( -- ) \ Simply compiles the cfa of the word being defined
    last @ 1 + , ; immediate \ last points to the latest nfa, so increment

: nip ( a b -- b ) swap drop ;
: tuck ( a b -- b a b ) swap over ;

: if BRANCH0 , here @  0 , ; immediate
: else BRANCH , here @ 0 , swap dup here @ swap - swap ! ; immediate
: then dup here @ swap - swap ! ; immediate

: ' (') dup @ dup DEFINITION = if drop else nip then ; \ searches for a (postfix) word and returns its cfa or FALSE

: ['] LITERAL , ' , ; immediate                        \ compiles a word's cfa into a definition as a literal
: cfa>nfa 1 - ;                                        \ converts an cfa to an nfa
: nfa>cfa 1 + ;                                        \ converts an nfa to a cfa
: bp>nfa 1 + ;                                         \ from preceding back pointer to nfa
: bp>cfa 2 + ; 

: for here @ ['] >r , ; immediate
: next ['] r> , 
    LITERAL , 1 , 
    ['] - , ['] dup , 
    ['] 0= , BRANCH0 , 
    here @ - , 
    ['] drop , ; immediate

: begin here @ ; immediate
: until BRANCH0 , here @ - , ; immediate
: again BRANCH , here @ - , ; immediate
: while BRANCH0 , here @ 0 , ;  immediate
: repeat BRANCH , swap here @ - , dup here @ swap - swap ! ; immediate

: 1- ( n -- n-1 ) 1 - ;
: 1+ ( n -- n+1 ) 1 + ;
: negate ( n -- -n ) 0 swap - ;
: not 0= ;
: pop ( a -- ) drop ;
: 2dup ( a b -- a b a b ) over over ;
: ?dup dup 0= if dup else then ;
: > < if false else true then ;
: <> ( n -- n ) = 0= ;
: min ( m n -- m | n ) 2dup < if drop else nip then ;
: max ( m n -- m | n ) 2dup > if drop else nip then ;
: abs ( n -- n | -n ) dup 0 < if -1 * then ;

: space ( -- ) BL emit ;
: spaces ( n -- ) for space next ;

: type ( s -- ) ADDRESS_MASK and dup c@ dup rot + swap for dup i - 1+ c@ emit next drop BL emit ;

: tell ( s l -- ) swap ADDRESS_MASK and swap for dup i - 1+ c@ emit next drop BL emit ;

: .tmp tmp @ type ;
: .pad pad @ type ;
: ." state @
    if
        STRLIT , 
        s" drop s-create ,
        ['] type ,
    else
        s" drop type
    then ; immediate

\ Implementation of word
: .word ( bp -- bp ) dup dup '[' emit . 1+ @ type ']' emit space @ ;             \ prints a word name, given the preceding back pointer
: words ( -- ) here @ 1- @ begin .word dup not until ;   \ loops through the words in the dictionary

\ Stepper controls
: step-on -1 stepper ! ;
: step-off 0 stepper ! ;
: trace-on 1 stepper ! ;
: trace-off 0 stepper ! ;

: dbg-debug 3 dbg ;
: dbg-info 2 dbg ;
: dbg-warning 1 dbg ;
: dbg-quiet 0 dbg ;
: debug show-stack step-on ;

\ : ?stack depth 0= if ." Stack underflow" abort then ;


: +! ( n addr -- ) dup @ rot + swap ! ;
: ? ( addr -- ) @ . ;

: kkey ( -- c ) >in @ c@ 1 >in +! ;                     \ Get the next character from the TIB
: ?key ( -- c T | F )                                   \ If there's a character in TIB, push it and TRUE
    #tib @ >in @ < if FALSE else key TRUE then ;        \ otherwise push FALSE
: strlen ( s -- n ) c@ ;                                \ return the count byte from the string
                                                
\ s" src/regression.fs" drop drop
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

 cr cr ." Library loaded." cr
