// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
use llvm;
use shader_compiler::backend;
use std::cell::Cell;
use std::ffi::{CStr, CString};
use std::fmt;
use std::ops::Deref;
use std::os::raw::{c_char, c_uint};
use std::ptr::null_mut;
use std::ptr::NonNull;
use std::sync::{Once, ONCE_INIT};

fn to_bool(v: llvm::LLVMBool) -> bool {
    v != 0
}

#[derive(Clone)]
pub struct LLVM7CompilerConfig {
    pub variable_vector_length_multiplier: u32,
    pub optimization_mode: backend::OptimizationMode,
}

impl Default for LLVM7CompilerConfig {
    fn default() -> Self {
        backend::CompilerIndependentConfig::default().into()
    }
}

impl From<backend::CompilerIndependentConfig> for LLVM7CompilerConfig {
    fn from(v: backend::CompilerIndependentConfig) -> Self {
        let backend::CompilerIndependentConfig { optimization_mode } = v;
        Self {
            variable_vector_length_multiplier: 1,
            optimization_mode,
        }
    }
}

#[repr(transparent)]
struct LLVM7String(NonNull<c_char>);

impl Drop for LLVM7String {
    fn drop(&mut self) {
        unsafe {
            llvm::LLVMDisposeMessage(self.0.as_ptr());
        }
    }
}

impl Deref for LLVM7String {
    type Target = CStr;
    fn deref(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.0.as_ptr()) }
    }
}

impl Clone for LLVM7String {
    fn clone(&self) -> Self {
        Self::new(self)
    }
}

impl LLVM7String {
    fn new(v: &CStr) -> Self {
        unsafe { Self::from_ptr(llvm::LLVMCreateMessage(v.as_ptr())).unwrap() }
    }
    unsafe fn from_nonnull(v: NonNull<c_char>) -> Self {
        LLVM7String(v)
    }
    unsafe fn from_ptr(v: *mut c_char) -> Option<Self> {
        NonNull::new(v).map(|v| Self::from_nonnull(v))
    }
}

impl fmt::Debug for LLVM7String {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct LLVM7Type(llvm::LLVMTypeRef);

impl fmt::Debug for LLVM7Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let string =
                LLVM7String::from_ptr(llvm::LLVMPrintTypeToString(self.0)).ok_or(fmt::Error)?;
            f.write_str(&string.to_string_lossy())
        }
    }
}

impl<'a> backend::types::Type<'a> for LLVM7Type {
    type Context = LLVM7Context;
}

pub struct LLVM7TypeBuilder {
    context: llvm::LLVMContextRef,
    variable_vector_length_multiplier: u32,
}

impl<'a> backend::types::TypeBuilder<'a, LLVM7Type> for LLVM7TypeBuilder {
    fn build_bool(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm::LLVMInt1TypeInContext(self.context)) }
    }
    fn build_i8(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm::LLVMInt8TypeInContext(self.context)) }
    }
    fn build_i16(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm::LLVMInt16TypeInContext(self.context)) }
    }
    fn build_i32(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm::LLVMInt32TypeInContext(self.context)) }
    }
    fn build_i64(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm::LLVMInt64TypeInContext(self.context)) }
    }
    fn build_f32(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm::LLVMFloatTypeInContext(self.context)) }
    }
    fn build_f64(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm::LLVMDoubleTypeInContext(self.context)) }
    }
    fn build_pointer(&self, target: LLVM7Type) -> LLVM7Type {
        unsafe { LLVM7Type(llvm::LLVMPointerType(target.0, 0)) }
    }
    fn build_array(&self, element: LLVM7Type, count: usize) -> LLVM7Type {
        assert_eq!(count as u32 as usize, count);
        unsafe { LLVM7Type(llvm::LLVMArrayType(element.0, count as u32)) }
    }
    fn build_vector(&self, element: LLVM7Type, length: backend::types::VectorLength) -> LLVM7Type {
        use self::backend::types::VectorLength::*;
        let length = match length {
            Fixed { length } => length,
            Variable { base_length } => base_length
                .checked_mul(self.variable_vector_length_multiplier)
                .unwrap(),
        };
        assert_ne!(length, 0);
        unsafe { LLVM7Type(llvm::LLVMVectorType(element.0, length)) }
    }
    fn build_struct(&self, members: &[LLVM7Type]) -> LLVM7Type {
        assert_eq!(members.len() as c_uint as usize, members.len());
        unsafe {
            LLVM7Type(llvm::LLVMStructTypeInContext(
                self.context,
                members.as_ptr() as *mut llvm::LLVMTypeRef,
                members.len() as c_uint,
                false as llvm::LLVMBool,
            ))
        }
    }
    fn build_function(&self, arguments: &[LLVM7Type], return_type: Option<LLVM7Type>) -> LLVM7Type {
        assert_eq!(arguments.len() as c_uint as usize, arguments.len());
        unsafe {
            LLVM7Type(llvm::LLVMFunctionType(
                return_type
                    .unwrap_or_else(|| LLVM7Type(llvm::LLVMVoidTypeInContext(self.context)))
                    .0,
                arguments.as_ptr() as *mut llvm::LLVMTypeRef,
                arguments.len() as c_uint,
                false as llvm::LLVMBool,
            ))
        }
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct LLVM7Value(llvm::LLVMValueRef);

impl fmt::Debug for LLVM7Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let string =
                LLVM7String::from_ptr(llvm::LLVMPrintValueToString(self.0)).ok_or(fmt::Error)?;
            f.write_str(&string.to_string_lossy())
        }
    }
}

impl<'a> backend::Value<'a> for LLVM7Value {
    type Context = LLVM7Context;
}

#[derive(Clone)]
#[repr(transparent)]
pub struct LLVM7BasicBlock(llvm::LLVMBasicBlockRef);

impl fmt::Debug for LLVM7BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::backend::BasicBlock;
        unsafe {
            let string = LLVM7String::from_ptr(llvm::LLVMPrintValueToString(self.as_value().0))
                .ok_or(fmt::Error)?;
            f.write_str(&string.to_string_lossy())
        }
    }
}

impl<'a> backend::BasicBlock<'a> for LLVM7BasicBlock {
    type Context = LLVM7Context;
    fn as_value(&self) -> LLVM7Value {
        unsafe { LLVM7Value(llvm::LLVMBasicBlockAsValue(self.0)) }
    }
}

impl<'a> backend::BuildableBasicBlock<'a> for LLVM7BasicBlock {
    type Context = LLVM7Context;
    fn as_basic_block(&self) -> LLVM7BasicBlock {
        self.clone()
    }
}

pub struct LLVM7Function {
    context: llvm::LLVMContextRef,
    function: llvm::LLVMValueRef,
}

impl fmt::Debug for LLVM7Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let string = LLVM7String::from_ptr(llvm::LLVMPrintValueToString(self.function))
                .ok_or(fmt::Error)?;
            f.write_str(&string.to_string_lossy())
        }
    }
}

impl<'a> backend::Function<'a> for LLVM7Function {
    type Context = LLVM7Context;
    fn as_value(&self) -> LLVM7Value {
        LLVM7Value(self.function)
    }
    fn append_new_basic_block(&mut self, name: Option<&str>) -> LLVM7BasicBlock {
        let name = CString::new(name.unwrap_or("")).unwrap();
        unsafe {
            LLVM7BasicBlock(llvm::LLVMAppendBasicBlockInContext(
                self.context,
                self.function,
                name.as_ptr(),
            ))
        }
    }
}

pub struct LLVM7Context {
    context: llvm::LLVMContextRef,
    modules: Cell<Vec<llvm::LLVMModuleRef>>,
    config: LLVM7CompilerConfig,
}

impl Drop for LLVM7Context {
    fn drop(&mut self) {
        unsafe {
            for module in self.modules.get_mut().drain(..) {
                llvm::LLVMDisposeModule(module);
            }
            llvm::LLVMContextDispose(self.context);
        }
    }
}

impl<'a> backend::Context<'a> for LLVM7Context {
    type Value = LLVM7Value;
    type BasicBlock = LLVM7BasicBlock;
    type BuildableBasicBlock = LLVM7BasicBlock;
    type Function = LLVM7Function;
    type Type = LLVM7Type;
    type TypeBuilder = LLVM7TypeBuilder;
    type Module = LLVM7Module;
    type VerifiedModule = LLVM7Module;
    type AttachedBuilder = LLVM7Builder;
    type DetachedBuilder = LLVM7Builder;
    fn create_module(&self, name: &str) -> LLVM7Module {
        let name = CString::new(name).unwrap();
        let mut modules = self.modules.take();
        modules.reserve(1); // so we don't unwind without freeing the new module
        unsafe {
            let module = llvm::LLVMModuleCreateWithNameInContext(name.as_ptr(), self.context);
            modules.push(module);
            self.modules.set(modules);
            LLVM7Module {
                context: self.context,
                module,
            }
        }
    }
    fn create_builder(&self) -> LLVM7Builder {
        unsafe { LLVM7Builder(llvm::LLVMCreateBuilderInContext(self.context)) }
    }
    fn create_type_builder(&self) -> LLVM7TypeBuilder {
        LLVM7TypeBuilder {
            context: self.context,
            variable_vector_length_multiplier: self.config.variable_vector_length_multiplier,
        }
    }
}

#[repr(transparent)]
pub struct LLVM7Builder(llvm::LLVMBuilderRef);

impl Drop for LLVM7Builder {
    fn drop(&mut self) {
        unsafe {
            llvm::LLVMDisposeBuilder(self.0);
        }
    }
}

impl<'a> backend::AttachedBuilder<'a> for LLVM7Builder {
    type Context = LLVM7Context;
    fn current_basic_block(&self) -> LLVM7BasicBlock {
        unsafe { LLVM7BasicBlock(llvm::LLVMGetInsertBlock(self.0)) }
    }
    fn build_return(self, value: Option<LLVM7Value>) -> LLVM7Builder {
        unsafe {
            match value {
                Some(value) => llvm::LLVMBuildRet(self.0, value.0),
                None => llvm::LLVMBuildRetVoid(self.0),
            };
            llvm::LLVMClearInsertionPosition(self.0);
        }
        self
    }
}

impl<'a> backend::DetachedBuilder<'a> for LLVM7Builder {
    type Context = LLVM7Context;
    fn attach(self, basic_block: LLVM7BasicBlock) -> LLVM7Builder {
        unsafe {
            llvm::LLVMPositionBuilderAtEnd(self.0, basic_block.0);
        }
        self
    }
}

pub struct LLVM7Module {
    context: llvm::LLVMContextRef,
    module: llvm::LLVMModuleRef,
}

impl fmt::Debug for LLVM7Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let string = LLVM7String::from_ptr(llvm::LLVMPrintModuleToString(self.module))
                .ok_or(fmt::Error)?;
            f.write_str(&string.to_string_lossy())
        }
    }
}

impl<'a> backend::Module<'a> for LLVM7Module {
    type Context = LLVM7Context;
    fn set_source_file_name(&mut self, source_file_name: &str) {
        unsafe {
            llvm::LLVMSetSourceFileName(
                self.module,
                source_file_name.as_ptr() as *const c_char,
                source_file_name.len(),
            )
        }
    }
    fn add_function(&mut self, name: &str, ty: LLVM7Type) -> LLVM7Function {
        let name = CString::new(name).unwrap();
        unsafe {
            LLVM7Function {
                context: self.context,
                function: llvm::LLVMAddFunction(self.module, name.as_ptr(), ty.0),
            }
        }
    }
    fn verify(self) -> Result<LLVM7Module, backend::VerificationFailure<'a, LLVM7Module>> {
        unsafe {
            let mut message = null_mut();
            match to_bool(llvm::LLVMVerifyModule(
                self.module,
                llvm::LLVMReturnStatusAction,
                &mut message,
            )) {
                broken if broken != false => {
                    let message = LLVM7String::from_ptr(message).unwrap();
                    let message = message.to_string_lossy();
                    Err(backend::VerificationFailure::new(self, message.as_ref()))
                }
                _ => Ok(self),
            }
        }
    }
    unsafe fn to_verified_module_unchecked(self) -> LLVM7Module {
        self
    }
}

impl<'a> backend::VerifiedModule<'a> for LLVM7Module {
    type Context = LLVM7Context;
    fn into_module(self) -> LLVM7Module {
        self
    }
}

struct LLVM7TargetMachine(llvm::LLVMTargetMachineRef);

impl Drop for LLVM7TargetMachine {
    fn drop(&mut self) {
        unsafe {
            llvm::LLVMDisposeTargetMachine(self.0);
        }
    }
}

impl LLVM7TargetMachine {
    fn take(mut self) -> llvm::LLVMTargetMachineRef {
        let retval = self.0;
        self.0 = null_mut();
        retval
    }
}

struct LLVM7OrcJITStack(llvm::LLVMOrcJITStackRef);

impl Drop for LLVM7OrcJITStack {
    fn drop(&mut self) {
        unsafe {
            match llvm::LLVMOrcDisposeInstance(self.0) {
                llvm::LLVMOrcErrSuccess => {}
                _ => {
                    panic!("LLVMOrcDisposeInstance failed");
                }
            }
        }
    }
}

fn initialize_native_target() {
    static ONCE: Once = ONCE_INIT;
    ONCE.call_once(|| unsafe {
        llvm::LLVM_InitializeNativeTarget();
        llvm::LLVM_InitializeNativeAsmParser();
    });
}

extern "C" fn symbol_resolver_fn<Void>(name: *const c_char, _lookup_context: *mut Void) -> u64 {
    let name = unsafe { CStr::from_ptr(name) };
    panic!("symbol_resolver_fn is unimplemented: name = {:?}", name)
}

#[derive(Copy, Clone)]
pub struct LLVM7Compiler;

impl backend::Compiler for LLVM7Compiler {
    type Config = LLVM7CompilerConfig;
    fn name(self) -> &'static str {
        "LLVM 7"
    }
    fn run<U: backend::CompilerUser>(
        self,
        user: U,
        config: LLVM7CompilerConfig,
    ) -> Result<Box<dyn backend::CompiledCode<U::FunctionKey>>, U::Error> {
        unsafe {
            initialize_native_target();
            let context = LLVM7Context {
                context: llvm::LLVMContextCreate(),
                modules: Vec::new().into(),
                config: config.clone(),
            };
            let backend::CompileInputs {
                module,
                callable_functions,
            } = user.run(&context)?;
            for callable_function in callable_functions.values() {
                assert_eq!(
                    llvm::LLVMGetGlobalParent(callable_function.function),
                    module.module
                );
            }
            let target_triple = LLVM7String::from_ptr(llvm::LLVMGetDefaultTargetTriple()).unwrap();
            let mut target = null_mut();
            let mut error = null_mut();
            let success = !to_bool(llvm::LLVMGetTargetFromTriple(
                target_triple.as_ptr(),
                &mut target,
                &mut error,
            ));
            if !success {
                let error = LLVM7String::from_ptr(error).unwrap();
                return Err(U::create_error(error.to_string_lossy().into()));
            }
            if !to_bool(llvm::LLVMTargetHasJIT(target)) {
                return Err(U::create_error(format!(
                    "target {:?} doesn't support JIT",
                    target_triple
                )));
            }
            let host_cpu_name = LLVM7String::from_ptr(llvm::LLVMGetHostCPUName()).unwrap();
            let host_cpu_features = LLVM7String::from_ptr(llvm::LLVMGetHostCPUFeatures()).unwrap();
            let target_machine = LLVM7TargetMachine(llvm::LLVMCreateTargetMachine(
                target,
                target_triple.as_ptr(),
                host_cpu_name.as_ptr(),
                host_cpu_features.as_ptr(),
                match config.optimization_mode {
                    backend::OptimizationMode::NoOptimizations => llvm::LLVMCodeGenLevelNone,
                    backend::OptimizationMode::Normal => llvm::LLVMCodeGenLevelDefault,
                },
                llvm::LLVMRelocDefault,
                llvm::LLVMCodeModelJITDefault,
            ));
            assert!(!target_machine.0.is_null());
            let orc_jit_stack =
                LLVM7OrcJITStack(llvm::LLVMOrcCreateInstance(target_machine.take()));
            let mut orc_module_handle = 0;
            llvm::LLVMOrcAddEagerlyCompiledIR(
                orc_jit_stack.0,
                &mut orc_module_handle,
                module.module,
                Some(symbol_resolver_fn),
                null_mut(),
            );
            unimplemented!()
        }
    }
}
