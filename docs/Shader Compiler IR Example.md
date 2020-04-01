The IR and
SPIR-V to IR translator are being written simultaneously, since that allows
more easily finding the things that need to be represented in the shader
compiler IR. Because writing both of the IR and SPIR-V translator together is
such a big task, we decided to pick an arbitrary point ([translating a totally
trivial shader into the IR](http://bugs.libre-riscv.org/show_bug.cgi?id=177))
and split it into tasks at that point so Jacob would be able to get paid
after several months of work.

The IR uses structured control-flow inspired by WebAssembly's control-flow
constructs as well as
[SSA](https://en.wikipedia.org/wiki/Static_single_assignment_form) but, instead
of using traditional phi instructions, it uses block and loop parameters and
return values (inspired by [Cranelift's EBB
parameters](https://github.com/bytecodealliance/wasmtime/blob/master/cranelift/docs/ir.md#static-single-assignment-form)
as well as both of the [Rust](https://www.rust-lang.org/) and [Lua](https://www.lua.org/) programming languages).

The IR has a single pointer type for all data pointers (`data_ptr`), unlike LLVM IR where pointer types have a type they point to (like `* i32`, where `i32` is the type the pointer points to).

Because having a serialized form of the IR is important for any good IR, like
LLVM IR, it has a user-friendly textual form that can be both read and
written without losing any information (assuming the IR is valid, comments are
ignored). A binary form may be added later.

Some example code (the IR is likely to change somewhat):

```
# this is a comment, comments go from the `#` character
# to the end of the line.

fn function1[] -> ! {
    # declares a function named function1 that takes
    # zero parameters and doesn't return
    # (the return type is !, taken from Rust).
    # If the function could return, there would instead be
    # a list of return types:
    # fn my_fn[] -> [i32, i64] {...}
    # my_fn returns an i32 and an i64. The multiple
    # returned values is inspired by Lua's multiple return values.

    # the hints for this function
    hints {
        # there are no inlining hints for this function
        inlining_hint: none,
        # this function doesn't have a side-effect hint
        side_effects: normal,
    }

    # function local variables
    {
        # the local variable is an i32 with an
        # alignment of 4 bytes
        i32, align: 0x4 -> local_var1: data_ptr;
        # the pointer to the local variable is
        # assigned to local_var1 which has the type data_ptr
    }

    # the function body is a single block -- block1.
    # block1's return types are instead attached to the
    # function signature above
    # (the `-> !` in the `fn function1[] -> !`).
    block1 {
        # the first instruction is a loop named loop1.
        # the initial value of loop_var is the_const,
        # which is a named constant.
        # the value of the_const is the address of the
        # function `function1`.
        loop loop1[the_const: fn function1] -> ! {
            # loop1 takes 1 parameter, which is assigned
            # to loop_var. the type of loop_var is a pointer to a
            # function which takes no parameters and doesn't
            # return.
            -> [loop_var: fn[] -> !];

            # the loop body is a single block -- block2.
            # block2's return value definitions are instead
            # attached to the loop instruction above
            # (the `-> !` in the `loop loop1[...] -> !`).
            block2 {

                # block3 is a block instruction, it returns
                # two values, which are assigned to a and b.
                # Both of a and b have type i32.
                block block3 -> [a: i32, b: i32] {
                    # the only way a block can return is by
                    # being broken out of using the break
                    # instruction. It is invalid for execution
                    # to reach the end of a block.

                    # this break instruction breaks out of
                    # block3, making block3 return the
                    # constants 1 and 2, both of type i32.
                    break block3[1i32, 2i32];
                };

                # an add instruction. The instruction adds
                # the value `a` (returned by block3 above) to
                # the constant `increment` (which is an i32
                # with the value 0x1), and stores the
                # result in the value `"a"1`. The source-code
                # location for the add instruction is specified
                # as being line 12, column 34, in the file
                # `source_file.vertex`.
                add [a, increment: 0x1i32]
                    -> ["a"1: i32] @ "source_file.vertex":12:34;

                # The `"a"1` name is stored as just `a` in
                # the IR, where the 1 is a numerical name
                # suffix to differentiate between the two
                # values with name `a`. This allows robustly
                # handling duplicate names, by using the
                # numerical name suffix to disambiguate.
                #
                # If a name is specified without the numerical
                # name suffix, the suffix is assumed to be the
                # number 0. This also allows handling names that
                # have unusual characters or are just the empty
                # string by using the form with the numerical
                # suffix:
                # `""0` (empty string)
                # `"\n"0` (a newline)
                # `"\u{12345}"0` (the unicode scalar value 0x12345)


                # this continue instruction jumps back to
                # the beginning of loop1, supplying the new
                # values of the loop parameters. In this case,
                # we just supply loop_var as the value for
                # the parameter, which just gets assigned to
                # loop_var in the next iteration.
                continue loop1[loop_var];
            }
        };
    }
}
```
