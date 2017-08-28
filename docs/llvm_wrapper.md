# llvm_wrapper library

Wraps the LLVM API, exposing LLVM's C API through `llvm_wrapper.h` and wrapping the portions of LLVM that are not exposed through the C API.

## `llvm_wrapper/llvm_wrapper.h`

### `llvm_wrapper::Wrapper<T, Deleter>`
Template designed to behave the same as `std::unique_ptr` except that `T` must be the held pointer type instead of the type pointed to:  
`Wrapper<A *, A_deleter>` instead of `std::unique_ptr<A, A_deleter>`.  
Useful because all the object handle typedefs used by LLVM's C API already are pointers instead of being typedefs for the pointed-to object, avoiding needing to dereference the pointer type before we can use it with `std::unique_ptr`.

### `llvm_wrapper::LLVM_string`
Wrapper for `LLVMCreateMessage` and `LLVMDisposeMessage`.

### `llvm_wrapper::Context`
Wrapper for `LLVMContextRef`.  
Members:
- `init`: initialize the LLVM library. Called by `Context::create`, and `Target`'s static functions.
- `create`: create a LLVM context.

### `llvm_wrapper::Target`
Wrapper for `LLVMTargetRef`.  
Members:
- `get_default_target_triple`: wrapper for `LLVMGetDefaultTargetTriple`
- `get_process_target_triple`: wrapper for `llvm::sys::getProcessTriple`
- `get_host_cpu_name`: wrapper for `llvm::sys::getHostCPUName`
- `get_host_cpu_name`: wrapper for `llvm::sys::getHostCPUName`
- `get_host_cpu_features`: wrapper for `llvm::sys::getHostCPUFeatures`. Returns the feature list in the format used for the `target-features` LLVM function annotation.
- `get_target_from_target_triple`: wrapper for `LLVMGetTargetFromTriple`. returns the `Target` if successful, otherwise returns an `LLVM_string` containing the error message.
- `get_native_target`: returns the `Target` corresponding to the value returned by `Target::get_process_target_triple`.

### `llvm_wrapper::Target_data`
Wrapper for `LLVMTargetDataRef`  
Members:
- `to_string`: wrapper for `LLVMCopyStringRepOfTargetData`
- `from_string`: wrapper for `LLVMCreateTargetData`
- `get_pointer_alignment`: wrapper for `llvm::TargetData::getPointerABIAlignment`

### `llvm_wrapper::Target_machine`
Wrapper for `LLVMTargetMachineRef`  
Members:
- `create_native_target_machine`: creates a target machine for JIT compiling
- `get_target`: wrapper for `LLVMGetTargetMachineTarget`
- `get_target_triple`: wrapper for `LLVMGetTargetMachineTriple`
- `create_target_data_layout`: wrapper for `LLVMCreateTargetDataLayout`
- `get_cpu`: wrapper for `LLVMGetTargetMachineCPU`
- `get_feature_string`: wrapper for `LLVMGetTargetMachineFeatureString`
- `get_code_gen_opt_level`: wrapper for `llvm::TargetMachine::getOptLevel`
- `get_biggest_vector_register_bit_width`: gets the bit-width for the largest supported vector register. Use to determine how much to vectorize by.

### `llvm_wrapper::Module`
Wrapper for `LLVMModuleRef`  
- `create`: wrapper for `LLVMModuleCreateWithNameInContext`
- `create_with_target_machine`: calls `create` then `set_target_machine`.
- `set_target_machine`: sets the target-specific parts of a LLVM `Module` to the values in the provided `Target_machine`.
- `set_function_target_machine`: sets the target-specific parts of a LLVM `Function` to the values in the provided `Target_machine`.

### `llvm_wrapper::print_type_to_string`
Wrapper for `LLVMPrintTypeToString`

### `llvm_wrapper::Builder`
Wrapper for `LLVMBuilderRef`  
Members:
- `create`: creates a `Builder`.
- `build_smod`: builds the code needed to implement the SPIR-V `OpSMod` instruction.

### `llvm_wrapper::Pass_manager`
Wrapper for `LLVMPassManagerRef`  
Members:
- `create_module_pass_manager`
- `create_function_pass_manager`

### `llvm_wrapper::get_scalar_or_vector_element_type`
Utility function that helps with vector type manipulation.  
If the passed in type is a vector type, returns the element type of that vector type,  
otherwise, returns the passed in type.

### `llvm_wrapper::Orc_jit_stack`
Wrapper for `LLVMOrcJITStackRef`.
Note that the code that was using this has been switched to using `llvm_wrapper::Orc_compile_stack` instead.  
Members:
- `create`: create a ORC JIT compiler stack.
- `add_eagerly_compiled_ir`: wrapper for `LLVMOrcAddEagerlyCompiledIR`.
- `get_symbol_address`: wrapper for `LLVMOrcGetSymbolAddress`

### `llvm_wrapper::Create_llvm_type<T>`
Template Functor that returns the LLVM type for `T`  
Members:
- `::LLVMTypeRef operator()(::LLVMContextRef context) const`: return the LLVM type for `T`.

## `llvm_wrapper/orc_compile_stack.h`

### `llvm_wrapper::Orc_compile_stack`
Custom ORC-based LLVM JIT compiler stack. Replaces `llvm_wrapper::Orc_jit_stack`  
Members:
- `create`: create a new compiler stack.
- `add_eagerly_compiled_ir`: optimize IR code then compile it to machine code, setting it up to run in-process.
- `get_symbol_address`: return the address of a JIT compiled symbol.
