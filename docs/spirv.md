# spirv library
library for SPIR-V parsing

## `spirv/literal_string.h`

### `spirv::Literal_string`
`string_view` for SPIR-V strings -- does endian translation on the fly.

## `spirv/word.h`
Contains the definition for `spirv::Word`

## `spirv/spirv.h`
Main SPIR-V header, contains the definitions for constants and types from the SPIR-V specification. Generated from `khronos-spirv`.

## `spirv/parser.h`
SPIR-V parser. Generated from `khronos-spirv`.

### `spirv::Parser_error`
Members:
- `error_index` -- the index of the `Word` in the shader that caused the error.
- `instruction_start_index` -- the index of the beginning of the SPIR-V instruction that caused the error, or `0` if not in an instruction.

### `spirv::Parser_callbacks`
Parser callbacks, must be implemented by the user.  
Members:
- `void handle_header()` -- called with the values parsed from the SPIR-V shader's header
- `virtual void handle_instruction_*(Instruction_type instruction, std::size_t instruction_start_index)` -- corresponding function called for each instruction in the SPIR-V shader.  
`handle_instruction_op_ext_inst` is only called for unrecognized extension instructions, otherwise the `handle_instruction_*` function for that instruction is called.

### `spirv::parse`
Entry point for the SPIR-V parser. Calls the corresponding functions in `spirv::Parser_callbacks` during parsing. Throws `spirv::Parser_error` on parse error.
