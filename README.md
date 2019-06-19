# rust-reduce

`rust-reduce` will try to make the source file smaller by interpreting it as valid Rust code and intelligently removing parts of the code. After each removal, the given command will be run but passing a path to a file containing the reduced code. The command should return 0 if run on the original input, and also if the reduced code is interesting, non-0 otherwise.

The original file will be overwritten with the smallest interesting reduced version, if found. This happens while `rust-reduce` is running. The original file will be backed up with the `.orig` suffix. If `rustfmt` is found, it will be used to clean up the output.

A common way to use `rust-reduce` is to write a short shell script that runs `rustc` and greps the compiler output for a particular error message. NB. you will want to look for a specific error message because while `rust-reduce` will generate syntactically correct code, it's not guaranteed to compile.

The original file may refer to modules in different files, these will be inlined and reduced along with the main file.

## C-reduce

This project is inspired by, and should be used in conjuniction with [C-reduce](http://embed.cs.utah.edu/creduce/).

Although Rust and C syntax are different, they are similar enough that running C-reduce on Rust source code can be very effective! However, C-reduce only works with single input files, whereas Rust has a module system. `rust-reduce` can be run on an entire crate and will produce a single reduced output file. `rust-reduce` only implements a few reduction passes that are designed to remove large chunks of code, after which C-reduce can take over the reduction.

When using C-reduce and `rust-reduce` in the same project, please take note that `rust-reduce` will change the command line of the test command whereas C-reduce won't.

## Passes

Take a look at `src/transforms` to see the kind of reductions `rust-reduce` can do.

## Examples

Take a look at the test suite in `tests/suite` for example usage.
