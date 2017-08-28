# spirv_to_llvm library

## `spirv_to_llvm/spirv_to_llvm.h`

### `spirv_to_llvm::Type_descriptor`
Type representing a SPIR-V Type. Does custom type layout instead of letting LLVM layout types to avoid creating types that need a stricter alignment than `::operator new` provides.  
Members:
- `decorations`: the SPIR-V decorations applied to this type.
- `get_or_make_type`: returns the LLVM type that `this` translates to, generating it if needed. Also returns the alignment used to store the returned LLVM type.
- `visit`
- `Recursion_checker`: helper type used to prevent infinite recursion when generating LLVM types.  
See `spirv_to_llvm::Struct_type_descriptor::get_or_make_type` to see how to use it.
- `Recursion_checker_state`: state used by `Recursion_checker`.

### `spirv_to_llvm::Simple_type_descriptor`
Wrap a fundamental type (not based on any other SPIR-V type).

### `spirv_to_llvm::Vector_type_descriptor`
Wrap a vector type.

### `spirv_to_llvm::Matrix_type_descriptor`
Wrap a matrix type.

### `spirv_to_llvm::Array_type_descriptor`
Wrap an array type.

### `spirv_to_llvm::Pointer_type_descriptor`
Wrap a pointer type. If `base == nullptr`, then `this` is a forward declaration.

### `spirv_to_llvm::Function_type_descriptor`
Wrap a function type.

### `spirv_to_llvm::Struct_type_descriptor`
Wrap a struct type.  
This can be in one of two states:  
- Incomplete: The default state upon construction. Members can be added in this state.
- Complete: The state transitioned to by calling `get_members(true)` or `get_or_make_type()`. Members can no longer be added.

### `spirv_to_llvm::Constant_descriptor`
A SPIR-V constant.  
Members:
- `get_or_make_value`: returns the LLVM value that `this` translates to, generating it if needed.

### `spirv_to_llvm::Simple_constant_descriptor`
A simple SPIR-V constant. Wraps a LLVM value.

### `spirv_to_llvm::Converted_module`
The results of converting a SPIR-V shader from SPIR-V to LLVM IR.

### `spirv_to_llvm::Jit_symbol_resolver`
Resolve built-in symbols for the JIT. This is where to add new built-in functions that are called by LLVM IR.

### `spirv_to_llvm::spirv_to_llvm`
Convert the provided SPIR-V shader to LLVM IR. Throws `spirv::Parser_error` on error.

## `spirv_to_llvm/spirv_to_llvm_implementation.h`

### `spirv_to_llvm::Spirv_to_llvm`
class implementing conversion from SPIR-V to LLVM IR.  
Members:
- `Id_state`: Per-Id state for each SPIR-V Id  
Members:
 - `op_string`
 - `op_ext_inst_import`
 - `name`
 - `type`: State for result of `Op_type_*`
 - `decorations`: State for target of `Op_decorate`
 - `member_decorations`: State for target of `Op_member_decorate`
 - `member_names`: State for target of `Op_member_name`
 - `variable`
 - `constant`
 - `function`
 - `label`
 - `value`
- `Op_string_state`: State for result of `Op_string`
- `Op_ext_inst_import_state`: State for result of `Op_ext_inst_import`
- `Op_entry_point_state`: State for result of `Op_entry_point`
- `Name`: State for target of `Op_name`
- `Input_variable_state`: State for input variables
- `Output_variable_state`: State for output variables
- `Variable_state`: State for variables
- `Function_state`: State for functions  
Members:
  - `Entry_block`: state for entry basic block
- `Label_state`: State for result of `Op_label`
- `Value`: State for a SPIR-V value.
- `Last_merge_instruction`: holder for merge instructions
- `next_name_index`: the next index to generate a name with
- `id_states`: the `vector` of Per-Id states. For an `Id` `id`, the state for `id` is stored at `id_states[id - 1]`.
- `input_version_number_major`
- `input_version_number_minor`
- `input_generator_magic_number`
- `enabled_capabilities`: set of capabilities enabled by the shader
- `context`: the LLVM context that code is generated into.
- `target_machine`: the LLVM target machine that code is generated for.
- `target_data`: the LLVM target data layout that code is generated for.
- `shader_id`: the id of this shader, passed into `spirv_to_llvm::spirv_to_llvm`.
- `name_prefix_string`: the prefix applied to all module-level names to prevent name collisions, generated from `shader_id`.
- `module`: the LLVM module that code is being generated into.
- `io_struct`: the `Struct_type_descriptor` that members are added to for communicating with the translated shader.
- `io_struct_argument_index`: the index in `implicit_function_arguments` of `io_struct`.
- `implicit_function_arguments`: the list of arguments that are implicitly added to the beginning of all SPIR-V functions.
- `inputs_member`: the member index in `io_struct` of the pointer to `inputs_struct`
- `inputs_struct`: the struct that represents all of the SPIR-V input variables.
- `outputs_member`: the member index in `io_struct` of the pointer to `outputs_struct`
- `outputs_struct`: the struct that represents all of the SPIR-V output variables.
- `outputs_struct_pointer_type`: the type of the pointer to `outputs_struct`.
- `stage`: the stage of translation
- `current_function_id`: the id of the function currently being translated, otherwise `0`
- `current_basic_block_id`: the id of the label of the basic block currently being translated, otherwise `0`
- `builder`: the LLVM builder used to build the LLVM IR. Set to point at the end of the basic block currently being translated, otherwise, the position it's set to is not specified.
- `last_merge_instruction`: the last merge instruction. Used to access the corresponding merge instruction from the following branches.
- `function_entry_block_handlers`: list of functions to be called when the entry block of a function is created. Used to set up pointers to global variables and other stuff.
- `execution_model`: the execution model of the entry point that should be used.
- `entry_point_name`: the name of the entry point that should be used.
- `entry_point_state_pointer`: the pointer to the entry point state specified by `execution_model` and `entry_point_name`. Used as a cache by `get_entry_point_state`.
- `get_id_state`: get the `Id_state` corresponding to the passed-in `Id`.
- `get_type`: get and cast the value of `Id_state::type` corresponding to the passed-in `Id`.
- `get_unsigned_integer_constant`: gets the value of the unsigned integer constant instruction specified by the passed-in `Id`.
- `get_signed_integer_constant`: gets the value of the signed integer constant instruction specified by the passed-in `Id`.
- `get_name`: gets the name of the passed-in `Id`, otherwise returns the empty string.
- `get_or_make_label`: gets or creates the LLVM basic block corresponding to the passed-in `Op_label` `Id`.
- `get_prefixed_name`: return a name ensured to not conflict at module scope, otherwise returns the empty string.
- `get_or_make_prefixed_name`: return a name ensured to not conflict at module scope, otherwise generates a new numbered name.
- `get_entry_point_state`: get the entry point state specified by `execution_model` and `entry_point_name`. Uses `entry_point_state_pointer` as a cache.
- `Spirv_to_llvm`: create a `Spirv_to_llvm` object
- `generate_entry_function`: generates the entry-point function. This is the JIT compiled function whose signature matches the function pointers in `pipeline::Pipeline`'s child classes. This generated function calls the function specified by `Op_entry_point`.
- `run`: main function that runs all the different stages of translation into LLVM IR.
