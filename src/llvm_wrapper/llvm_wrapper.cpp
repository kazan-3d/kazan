/*
 * Copyright 2017 Jacob Lifshay
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 */
#include "llvm_wrapper.h"
#include <llvm/Support/Host.h>
#include <llvm/ExecutionEngine/SectionMemoryManager.h>
#include <llvm-c/ExecutionEngine.h>
#include <llvm/IR/DataLayout.h>
#include <llvm/Target/TargetMachine.h>
#include <iostream>
#include <cstdlib>
#include <algorithm>

namespace vulkan_cpu
{
namespace llvm_wrapper
{
// implement the unwrap functions that aren't in public llvm headers
static llvm::TargetMachine *unwrap(::LLVMTargetMachineRef v) noexcept
{
    return reinterpret_cast<llvm::TargetMachine *>(v);
}

void Context::init_helper()
{
    if(!::LLVMIsMultithreaded())
        throw std::runtime_error("LLVM is not multithreaded");
    if(::LLVMInitializeNativeTarget() != 0)
        throw std::runtime_error("LLVMInitializeNativeTarget failed");
    if(::LLVMInitializeNativeAsmParser() != 0)
        throw std::runtime_error("LLVMInitializeNativeAsmParser failed");
    if(::LLVMInitializeNativeAsmPrinter() != 0)
        throw std::runtime_error("LLVMInitializeNativeAsmPrinter failed");
    if(::LLVMInitializeNativeDisassembler() != 0)
        throw std::runtime_error("LLVMInitializeNativeDisassembler failed");
}

LLVM_string Target::get_process_target_triple()
{
    Context::init();
    return LLVM_string::from(llvm::sys::getProcessTriple());
}

LLVM_string Target::get_host_cpu_name()
{
    Context::init();
    return LLVM_string::from(llvm::sys::getHostCPUName());
}

LLVM_string Target::get_host_cpu_features()
{
    Context::init();
    llvm::StringMap<bool> features{};
    if(!llvm::sys::getHostCPUFeatures(features))
        return LLVM_string::from("");
    std::string retval;
    std::vector<std::string> names;
    names.reserve(features.size());
    for(auto &entry : features)
        names.push_back(entry.first());
    std::sort(names.begin(), names.end());
    bool first = true;
    for(auto &name : names)
    {
        if(first)
            first = false;
        else
            retval += ',';
        if(features[name])
            retval += '+';
        else
            retval += '-';
        retval += name;
    }
    return LLVM_string::from(retval);
}

std::size_t Target_data::get_pointer_alignment(::LLVMTargetDataRef td) noexcept
{
    return llvm::unwrap(td)->getPointerABIAlignment(0);
}

Target_machine Target_machine::create_native_target_machine(::LLVMCodeGenOptLevel code_gen_level)
{
    auto target = Target::get_native_target();
    return Target_machine(::LLVMCreateTargetMachine(target.get(),
                                                    Target::get_process_target_triple().get(),
                                                    Target::get_host_cpu_name().get(),
                                                    Target::get_host_cpu_features().get(),
                                                    code_gen_level,
                                                    ::LLVMRelocDefault,
                                                    ::LLVMCodeModelJITDefault));
}

::LLVMCodeGenOptLevel Target_machine::get_code_gen_opt_level(::LLVMTargetMachineRef tm) noexcept
{
    switch(unwrap(tm)->getOptLevel())
    {
    case llvm::CodeGenOpt::Level::None:
        return ::LLVMCodeGenLevelNone;
    case llvm::CodeGenOpt::Level::Less:
        return ::LLVMCodeGenLevelLess;
    case llvm::CodeGenOpt::Level::Default:
        return ::LLVMCodeGenLevelDefault;
    case llvm::CodeGenOpt::Level::Aggressive:
        return ::LLVMCodeGenLevelAggressive;
    }
    return ::LLVMCodeGenLevelDefault;
}

void Module::set_target_machine(::LLVMModuleRef module, ::LLVMTargetMachineRef target_machine)
{
    ::LLVMSetTarget(module, Target_machine::get_target_triple(target_machine).get());
    ::LLVMSetModuleDataLayout(module,
                              Target_machine::create_target_data_layout(target_machine).get());
}

void Module::set_function_target_machine(::LLVMValueRef function,
                                         ::LLVMTargetMachineRef target_machine)
{
    ::LLVMAddTargetDependentFunctionAttr(
        function, "target-cpu", Target_machine::get_cpu(target_machine).get());
    ::LLVMAddTargetDependentFunctionAttr(
        function, "target-features", Target_machine::get_feature_string(target_machine).get());
}
}
}
