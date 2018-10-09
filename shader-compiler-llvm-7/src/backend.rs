// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
use llvm_sys;
use shader_compiler::backend::*;
use std::ffi::{CStr, CString};
use std::fmt;
use std::ops::Deref;
use std::os::raw::{c_char, c_uint};
use std::ptr::NonNull;

#[derive(Clone)]
pub struct LLVM7ShaderCompilerConfig {
    pub variable_vector_length_multiplier: u32,
}

impl Default for LLVM7ShaderCompilerConfig {
    fn default() -> Self {
        Self {
            variable_vector_length_multiplier: 1,
        }
    }
}

#[repr(transparent)]
struct LLVM7String(NonNull<c_char>);

impl Drop for LLVM7String {
    fn drop(&mut self) {
        unsafe {
            llvm_sys::core::LLVMDisposeMessage(self.0.as_ptr());
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
        unsafe { Self::from_ptr(llvm_sys::core::LLVMCreateMessage(v.as_ptr())).unwrap() }
    }
    unsafe fn from_nonnull(v: NonNull<c_char>) -> Self {
        LLVM7String(v)
    }
    unsafe fn from_ptr(v: *mut c_char) -> Option<Self> {
        NonNull::new(v).map(LLVM7String)
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct LLVM7Type(llvm_sys::prelude::LLVMTypeRef);

impl fmt::Debug for LLVM7Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let string = LLVM7String::from_ptr(llvm_sys::core::LLVMPrintTypeToString(self.0))
                .ok_or(fmt::Error)?;
            f.write_str(&string.to_string_lossy())
        }
    }
}

impl<'a> Type<'a> for LLVM7Type {}

pub struct LLVM7TypeBuilder {
    context: llvm_sys::prelude::LLVMContextRef,
    variable_vector_length_multiplier: u32,
}

impl<'a> TypeBuilder<'a, LLVM7Type> for LLVM7TypeBuilder {
    fn build_bool(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm_sys::core::LLVMInt1TypeInContext(self.context)) }
    }
    fn build_i8(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm_sys::core::LLVMInt8TypeInContext(self.context)) }
    }
    fn build_i16(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm_sys::core::LLVMInt16TypeInContext(self.context)) }
    }
    fn build_i32(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm_sys::core::LLVMInt32TypeInContext(self.context)) }
    }
    fn build_i64(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm_sys::core::LLVMInt64TypeInContext(self.context)) }
    }
    fn build_f32(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm_sys::core::LLVMFloatTypeInContext(self.context)) }
    }
    fn build_f64(&self) -> LLVM7Type {
        unsafe { LLVM7Type(llvm_sys::core::LLVMDoubleTypeInContext(self.context)) }
    }
    fn build_pointer(&self, target: LLVM7Type) -> LLVM7Type {
        unsafe { LLVM7Type(llvm_sys::core::LLVMPointerType(target.0, 0)) }
    }
    fn build_array(&self, element: LLVM7Type, count: usize) -> LLVM7Type {
        assert_eq!(count as u32 as usize, count);
        unsafe { LLVM7Type(llvm_sys::core::LLVMArrayType(element.0, count as u32)) }
    }
    fn build_vector(&self, element: LLVM7Type, length: VectorLength) -> LLVM7Type {
        let length = match length {
            VectorLength::Fixed { length } => length,
            VectorLength::Variable { base_length } => base_length
                .checked_mul(self.variable_vector_length_multiplier)
                .unwrap(),
        };
        assert_ne!(length, 0);
        unsafe { LLVM7Type(llvm_sys::core::LLVMVectorType(element.0, length)) }
    }
    fn build_struct(&self, members: &[LLVM7Type]) -> LLVM7Type {
        assert_eq!(members.len() as c_uint as usize, members.len());
        unsafe {
            LLVM7Type(llvm_sys::core::LLVMStructTypeInContext(
                self.context,
                members.as_ptr() as *mut llvm_sys::prelude::LLVMTypeRef,
                members.len() as c_uint,
                false as llvm_sys::prelude::LLVMBool,
            ))
        }
    }
    fn build_function(&self, arguments: &[LLVM7Type], return_type: Option<LLVM7Type>) -> LLVM7Type {
        assert_eq!(arguments.len() as c_uint as usize, arguments.len());
        unsafe {
            LLVM7Type(llvm_sys::core::LLVMFunctionType(
                return_type
                    .unwrap_or_else(|| {
                        LLVM7Type(llvm_sys::core::LLVMVoidTypeInContext(self.context))
                    })
                    .0,
                arguments.as_ptr() as *mut llvm_sys::prelude::LLVMTypeRef,
                arguments.len() as c_uint,
                false as llvm_sys::prelude::LLVMBool,
            ))
        }
    }
}

pub struct LLVM7Context {
    context: llvm_sys::prelude::LLVMContextRef,
    config: LLVM7ShaderCompilerConfig,
}

impl Drop for LLVM7Context {
    fn drop(&mut self) {
        unsafe {
            llvm_sys::core::LLVMContextDispose(self.context);
        }
    }
}

impl<'a> Context<'a> for LLVM7Context {
    type Type = LLVM7Type;
    type TypeBuilder = LLVM7TypeBuilder;
    type Module = LLVM7Module;
    type Builder = LLVM7Builder;
    fn create_module(&self, name: &str) -> LLVM7Module {
        let name = CString::new(name).unwrap();
        unsafe {
            LLVM7Module(llvm_sys::core::LLVMModuleCreateWithNameInContext(
                name.as_ptr(),
                self.context,
            ))
        }
    }
    fn create_builder(&self) -> LLVM7Builder {
        unsafe { LLVM7Builder(llvm_sys::core::LLVMCreateBuilderInContext(self.context)) }
    }
    fn create_type_builder(&self) -> LLVM7TypeBuilder {
        LLVM7TypeBuilder {
            context: self.context,
            variable_vector_length_multiplier: self.config.variable_vector_length_multiplier,
        }
    }
}

#[repr(transparent)]
pub struct LLVM7Builder(llvm_sys::prelude::LLVMBuilderRef);

impl Drop for LLVM7Builder {
    fn drop(&mut self) {
        unsafe {
            llvm_sys::core::LLVMDisposeBuilder(self.0);
        }
    }
}

impl<'a> Builder<'a> for LLVM7Builder {}

#[repr(transparent)]
pub struct LLVM7Module(llvm_sys::prelude::LLVMModuleRef);

impl Drop for LLVM7Module {
    fn drop(&mut self) {
        unsafe {
            llvm_sys::core::LLVMDisposeModule(self.0);
        }
    }
}

impl<'a> Module<'a> for LLVM7Module {
    fn set_source_file_name(&mut self, source_file_name: &str) {
        unsafe {
            llvm_sys::core::LLVMSetSourceFileName(
                self.0,
                source_file_name.as_ptr() as *const c_char,
                source_file_name.len(),
            )
        }
    }
}

pub struct LLVM7ShaderCompiler;

impl ShaderCompiler for LLVM7ShaderCompiler {
    type Config = LLVM7ShaderCompilerConfig;
    fn name() -> &'static str {
        "LLVM 7"
    }
    fn run_with_user<SCU: ShaderCompilerUser>(
        shader_compiler_user: SCU,
        config: LLVM7ShaderCompilerConfig,
    ) -> SCU::ReturnType {
        let context = unsafe {
            LLVM7Context {
                context: llvm_sys::core::LLVMContextCreate(),
                config,
            }
        };
        shader_compiler_user.run_with_context(&context)
    }
}
