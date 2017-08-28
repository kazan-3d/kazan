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
