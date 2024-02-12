( Core word definitions )

: ?dup ( w -- w w | 0 ) dup if dup then ;
: negate ( n -- -n ) if -1 else 0 then ;
: nip ( a b -- b ) swap drop ;
: tuck ( a b -- b a b ) swap over ;
: pop ( a -- ) drop ;
: 2dup ( a b -- a b a b ) over over ;
: 2drop ( a b -- ) drop drop ;
: not -1 xor ;
: negate not 1+ ;
: > < if false else true then ;
: <> ( n -- n ) = 0= ;
: min ( m n -- m | n ) 2dup < if drop else nip then ;
: max ( m n -- m | n ) 2dup > if drop else nip then ;
: abs ( n -- n | -n ) dup 0 < if negate then ;
: within ( n l h -- b ) over >r - r> < ;
: >char ( c -- c ) 127 and dup 127 bl within if drop 95 then ;
: dbg-debug 3 dbg ;
: dbg-info 2 dbg ;
: dbg-warning 1 dbg ;
: dbg-quiet 0 dbg ;
: debug show-stack step-on ;
: bl 32 ; ( puts the character code for a space on the stack )
: space ( -- ) bl emit ;
: spaces ( n -- ) 0 do space loop ;
: 1- ( n -- n-1 ) 1 - ;
: 1+ ( n -- n+1 ) 1 + ;
: endif then ; ( synonym for then, to allow if - else - endif conditionals )
: +! ( n addr -- ) dup @ rot + swap ! ;
: ? ( addr -- ) @ . ;

\ numeric
variable base 10 base !
: decimal 10 base ! ;
: hex 16 base ! ;
variable hld   \ used for numeric conversions
: digit ( n -- c ) 9 over < 7 and + 48 + ;
: extract ( n base -- n c ) 0 swap mod digit ;
: <# ( -- ) 0 hld ! ;
: hold (c -- ) hld @ 1- dup hld ! ! ;
: # (u -- u ) base @ extract hold ;
: #s (u -- 0 ) begin # dup while repeat ;
: sign ( n -- ) 0< if 45 hold then ;
: #> ( w -- b u ) drop hld @ 0 over - ;
: str ( n -- b u ) dup >r abs <# #s r> sign #> ;
: .r ( n +n -- ) >r str r> over - spaces type ; 

: run-regression clear s" src/regression.fs" loaded ;


( Application functions )
: fac ( n -- n! )   \ Calculates factorial of a non-negative integer. No checks for stack or calculation overflow.
    dup 
        if 
            1 swap _fac 
        else 
            drop 1 
        then ;

: _fac ( r n -- r )   \ Helper function that does most of the work.
    dup if 
        tuck * swap 1 - _fac 
    else 
        drop 
    then ;

." Library loaded."
