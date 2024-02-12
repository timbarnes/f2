*** A tiny Forth in Rust and Forth, based on eForth

This implementation attempts to create a reasonable minimum system in Rust, with as much as possible implemented in Forth.

For convenience rather than efficiency, the data store is an array[i32], and it uses indirect threading.
Builtin functions are made visible in the data space, which also contains:
* The text input buffer `TIB`
* The text working buffer `PAD`
* A general area for use by `ALLOT`
* The Forth calculation `STACK`
* The return stack `RET`
* `WORD`, `VARIABLE`, and `CONSTANT` storage

