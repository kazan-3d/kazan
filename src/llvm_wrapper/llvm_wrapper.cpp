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
#include <stdexcept>
#include <iostream>
#include <cstdlib>

namespace vulkan_cpu
{
namespace llvm_wrapper
{
Context Context::create()
{
    if(!::LLVMIsMultithreaded())
        throw std::runtime_error("LLVM is not multithreaded");
    ::LLVMLinkInMCJIT();
    if(::LLVMInitializeNativeTarget() != 0)
        throw std::runtime_error("LLVMInitializeNativeTarget failed");
    return Context(::LLVMContextCreate());
}

LLVM_string Target::get_process_target_triple()
{
    return LLVM_string::from(llvm::sys::getProcessTriple());
}

LLVM_string Target::get_host_cpu_name()
{
    return LLVM_string::from(llvm::sys::getHostCPUName());
}

LLVM_string Target::get_host_cpu_features()
{
    llvm::StringMap<bool> features{};
    if(!llvm::sys::getHostCPUFeatures(features))
        return LLVM_string::from("");
    std::string retval;
    bool first = true;
    for(auto &entry : features)
    {
        if(first)
            first = false;
        else
            retval += ',';
        if(entry.second)
            retval += '+';
        else
            retval += '-';
        retval += entry.first();
    }
    return LLVM_string::from(retval);
}

Target_machine Target_machine::create_native_target_machine()
{
    auto target = Target::get_native_target();
    return Target_machine(::LLVMCreateTargetMachine(target.get(),
                                                    Target::get_process_target_triple().get(),
                                                    Target::get_host_cpu_name().get(),
                                                    Target::get_host_cpu_features().get(),
                                                    ::LLVMCodeGenLevelDefault,
                                                    ::LLVMRelocDefault,
                                                    ::LLVMCodeModelJITDefault));
}

void Module::set_target_to_native(::LLVMModuleRef module)
{
    auto target_machine = Target_machine::create_native_target_machine();
    ::LLVMSetTarget(module, target_machine.get_target_triple().get());
    auto target_data_string = target_machine.create_target_data_layout().to_string();
    ::LLVMSetModuleDataLayout(module, target_machine.create_target_data_layout().get());
}
}
}
