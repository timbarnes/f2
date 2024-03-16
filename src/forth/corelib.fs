: parse pad @ swap parse-to ;                       
: \ 1 parse drop drop ; immediate                  
: ( 41 parse drop drop ; immediate                  \ Implements in-line comments
: [char] BL parse drop drop pad @ 1 + c@ ;          \ Place the first char of the next token on the stack

1 dbg \ set debuglevel to warnings and errors

\ here points to the slot where the new back pointer goes
\ last and context point to the previous word's name field address

: (close) ( -- )                      \ terminate a definition, writing a back pointer and updating context, last, and here
        last @ 1 - here @ !           \ write the new back pointer
        here @ 1 + here !             \ update HERE 
        last @ context !              \ update CONTEXT
        ;

: constant ( n -- ) create 100002 , ,      \ v constant <name> creates a constant with value v
    (close) ;  

0 constant FALSE
-1 constant TRUE

\ Constants referring to inner interpreter opcodes, which are typically compiled into definitions
100000 constant BUILTIN
100001 constant VARIABLE
100002 constant CONSTANT
100003 constant LITERAL
100004 constant STRLIT
100005 constant DEFINITION
100006 constant BRANCH
100007 constant BRANCH0
100008 constant ABORT
100009 constant EXIT
100010 constant BREAK

72057594037927935 constant ADDRESS_MASK                      \ wipes any flags

\ ASCII symbols that are useful for text processing
10 constant '\n'
32 constant BL
34 constant '"'
39 constant '''
41 constant ')'
45 constant '-'
48 constant '0'
58 constant ':'
61 constant '='
65 constant 'A'
91 constant '['
93 constant ']'

\ For file I/O
-1 constant R/W
 0 constant R/O
 1 constant W/O

 : variable ( -- ) create VARIABLE , 0 ,    \ variable <name> creates a variable, initialized to zero
    (close) ;  

: decimal 10 base ! ;
: hex 16 base ! ;


: text              BL parse ;                                \ Parser shortcut for space-delimited tokens
: s-parse           tmp @ swap parse-to ;                     \ Same as text, but loads to tmp instead of pad
: s" ( -- s u ")    tmp @ '"' parse-to ;                      \ Places a double-quoted string in tmp

( File reader functions )
: included          tmp @ include-file ; \ include-file uses a string pointer on the stack to load a file
: include           tmp @ 32 parse-to included ;

: [ FALSE state ! ; immediate                       \ Turns compile mode off
: ] TRUE state ! ;                                  \ Turns compile mode on

: recurse ( -- ) \ Simply compiles the cfa of the word being defined
                    last @ 1 + , ; immediate \ last points to the latest nfa, so increment

: nip ( a b -- b )  swap drop ;
: tuck ( a b -- b a b ) swap over ;

: if                BRANCH0 , here @  0 , ; immediate
: else              BRANCH , here @ 0 , swap dup here @ swap - swap ! ; immediate
: then dup here @ swap - swap ! ; immediate

: ' (')             dup @ dup DEFINITION = if drop else nip then ; \ searches for a (postfix) word and returns its cfa or FALSE

: [']               LITERAL , ' , ; immediate                        \ compiles a word's cfa into a definition as a literal
: cfa>nfa           1 - ;                                        \ converts an cfa to an nfa
: nfa>cfa           1 + ;                                        \ converts an nfa to a cfa
: bp>nfa            1 + ;                                         \ from preceding back pointer to nfa
: bp>cfa            2 + ; 

: for               here @ ['] >r , ; immediate
: next              ['] r> , 
                    LITERAL , 1 , 
                    ['] - , ['] dup , 
                    ['] 0= , BRANCH0 , 
                    here @ - , 
                    ['] drop , ; immediate

: begin             here @ ; immediate
: until             BRANCH0 , here @ - , ; immediate
: again             BRANCH , here @ - , ; immediate
: while             BRANCH0 , here @ 0 , ;  immediate
: repeat            BRANCH , swap here @ - , dup here @ swap - swap ! ; immediate

: 1- ( n -- n-1 )   1 - ;
: 1+ ( n -- n+1 )   1 + ;
: negate ( n -- -n ) 0 swap - ;
: not               0= ;
: pop ( a -- )      drop ;
: 2dup ( a b -- a b a b ) over over ;
: 2drop ( a b -- )  drop drop ;
: ?dup              dup 0= if else dup then ;
: rdrop ( -- )      r> drop ;                           \ Pop a return address off the stack
: exit ( -- )       BREAK , ; immediate                 \ Pop out of the current definition and reset the Program Counter
: >                 swap < ;
: <> ( n -- n )     = 0= ;
: 0>                0 > ;
: 0<>               0= 0= ;
: min ( m n -- m | n ) 2dup < if drop else nip then ;
: max ( m n -- m | n ) 2dup > if drop else nip then ;
: abs ( n -- n | -n ) dup 0 < if -1 * then ;

: space ( -- )      BL emit ;
: spaces ( n -- )   dup 0> if for space next else drop then ;
: cr ( -- )         '\n' emit ;

: tell ( s l -- )                               \ like type, but length is provided: useful for substrings
                    swap ADDRESS_MASK and
                    swap 2dup + rot drop swap 
                    for 
                        dup i - c@ emit 
                    next 
                        drop ;

: type ( s -- )                                 \ Print from the string pointer on the stack
                    ADDRESS_MASK and                            \ Wipe out any flags
                    dup c@ swap 1+ swap                          \ Get the length to drive the for loop
                    tell ;

: rtell ( s l w -- )                             \ Right justify a string of length l in a field of w characters
                    over - 1 max 
                    spaces tell ;

: ltell ( s l w -- ) 2dup swap -
                    nip rot rot
                    tell spaces ;

: rtype ( s w -- )  swap ADDRESS_MASK and dup c@ 
                    rot swap - spaces type ;

: ltype             swap ADDRESS_MASK and dup c@ 
                    rot swap - swap type spaces ;

: .tmp              tmp @ type ;                             \ Print the tmp buffer
: .pad              pad @ type ;                             \ Print the pad buffer
: ."  ( -- )        state @                                    \ Compile or print a string
                    if
                        STRLIT ,                                \ Compilation section
                        s" drop s-create ,
                        ['] type ,
                    else
                        s" drop type                            \ Execution (print) section
                    then ; immediate

\ mumeric functions

: /mod              2dup mod rot rot / ;

: .d ( n -- )       dup 10 < 
                    if '0' else 10 - 'A' then
                    + emit ;    \ print a single digit

: .- ( -- )         '-' emit ;       \ print a minus sign

: u.    ( u -- )    base @ /mod
                    ?dup if recurse then .d ;

: uwidth ( u -- n ) \ returns the number of digits in an unsigned number
                    base @ / 
                    ?dup if recurse 1+ else  1 then ;

: u.r ( u width -- )
                    swap dup uwidth rot swap -
                    spaces u. ;

: .r ( n width -- )
                    swap dup 0< 
                    if 
                        negate 1 swap rot 1-
                    else
                        0 swap rot
                    then
                    swap dup uwidth rot swap -
                    spaces swap
                    if '-' emit then
                    u. ;

: . 0 .r space ;

: +! ( n addr -- )  dup @ rot + swap ! ;
: ? ( addr -- )     @ . ;

\ Implementation of word
variable word-counter

: .word ( bp -- bp )                            \ prints a word name, given the preceding back pointer
                    dup dup 4 u.r space 1+ @ 12 ltype 
                    1 word-counter +! 
                    word-counter @ 8 mod
                    if space else cr then @ ;   

: words ( -- )
                    0 word-counter !
                    here @ 1- @                                 \ Get the starting point: the top back pointer
                    begin                                       \ loops through the words in the dictionary
                        .word dup not                           \ print a word and test the next pointer
                    until 
                        drop ;   

\ Stepper controls
: step-on           -1 stepper ! ;
: step-off          0 stepper ! ;
: trace-on          1 stepper ! ;
: trace-off         0 stepper ! ;

: dbg-debug         3 dbg ;
: dbg-info          2 dbg ;
: dbg-warning       1 dbg ;
: dbg-quiet         0 dbg ;
: debug             show-stack step-on ;

\ : abort" ." drop type space abort ;

: forget-last ( -- )                            \ delete the most recent definition
                    here @ 1- @ dup 1+ here !                   \ resets HERE to the previous back pointer
                    @ 1+ dup context ! last !                   \ resets CONTEXT and LAST
                    ;

: forget ( <name> )                             \ delete <name> and any words since
                    ' dup  
                    if 
                        1- dup dup here ! @ s-here !            \ move to nfa and set HERE and S-HERE
                        1- @ 1+ dup context ! last !            \ go back a link and set CONTEXT and LAST
                    else
                        drop ;

\ : ?stack depth 0= if abort" Stack underflow" then ;

: kkey ( -- c )     >in @ c@ 1 >in +! ;                     \ Get the next character from the TIB
: ?key ( -- c T | F )                                   \ If there's a character in TIB, push it and TRUE
                    #tib @ >in @ < if FALSE else key TRUE then ;        \ otherwise push FALSE
: strlen ( s -- n ) c@ ;                                \ return the count byte from the string
                                                
\ s" src/regression.fs" drop drop
\ : run-regression include ;


( Application functions )

: _fac ( r n -- r )   \ Helper function that does most of the work.
                    dup 
                    if 
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

\ include src/numbers.fs

clear
\ cr ." Library loaded." cr
