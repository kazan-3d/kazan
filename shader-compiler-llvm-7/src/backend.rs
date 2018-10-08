// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
use llvm_sys;
use shader_compiler::backend::*;
use std::ffi::CString;
use std::os::raw::c_char;

#[repr(transparent)]
pub struct LLVM7Context(llvm_sys::prelude::LLVMContextRef);

impl Drop for LLVM7Context {
    fn drop(&mut self) {
        unsafe {
            llvm_sys::core::LLVMContextDispose(self.0);
        }
    }
}

unsafe impl Send for LLVM7Context {}

impl<'a> Context<'a> for LLVM7Context {
    type Module = LLVM7Module;
    fn create_module(&self, name: &str) -> LLVM7Module {
        let name = CString::new(name).unwrap();
        unsafe {
            LLVM7Module(llvm_sys::core::LLVMModuleCreateWithNameInContext(
                name.as_ptr(),
                self.0,
            ))
        }
    }
}

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
    fn name() -> &'static str {
        "LLVM 7"
    }
    fn run_with_user<SCU: ShaderCompilerUser>(shader_compiler_user: SCU) -> SCU::ReturnType {
        let context = unsafe { LLVM7Context(llvm_sys::core::LLVMContextCreate()) };
        shader_compiler_user.run_with_context(&context)
    }
}
