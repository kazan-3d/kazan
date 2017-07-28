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
#include <stdexcept>
#include <iostream>
#include <cstdlib>
#include <algorithm>

namespace vulkan_cpu
{
namespace llvm_wrapper
{
void Context::init_helper()
{
    if(!::LLVMIsMultithreaded())
        throw std::runtime_error("LLVM is not multithreaded");
    ::LLVMLinkInMCJIT();
    if(::LLVMInitializeNativeTarget() != 0)
        throw std::runtime_error("LLVMInitializeNativeTarget failed");
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

Shared_memory_manager Shared_memory_manager::create()
{
    Context::init();
    auto memory_manager = std::make_shared<llvm::SectionMemoryManager>();
    std::unique_ptr<std::shared_ptr<llvm::SectionMemoryManager>> memory_manager_wrapper;
    memory_manager_wrapper.reset(new std::shared_ptr<llvm::SectionMemoryManager>(memory_manager));
    MCJIT_memory_manager mcjit_memory_manager(::LLVMCreateSimpleMCJITMemoryManager(
        static_cast<void *>(memory_manager_wrapper.get()),
        [](void *user_data,
           std::uintptr_t size,
           unsigned alignment,
           unsigned section_id,
           const char *section_name) -> std::uint8_t *
        {
            auto &memory_manager =
                *static_cast<std::shared_ptr<llvm::SectionMemoryManager> *>(user_data);
            return memory_manager->allocateCodeSection(size, alignment, section_id, section_name);
        },
        [](void *user_data,
           std::uintptr_t size,
           unsigned alignment,
           unsigned section_id,
           const char *section_name,
           LLVMBool is_read_only) -> std::uint8_t *
        {
            auto &memory_manager =
                *static_cast<std::shared_ptr<llvm::SectionMemoryManager> *>(user_data);
            return memory_manager->allocateDataSection(
                size, alignment, section_id, section_name, is_read_only);
        },
        [](void *user_data, char **error_message_out) -> LLVMBool
        {
            auto &memory_manager =
                *static_cast<std::shared_ptr<llvm::SectionMemoryManager> *>(user_data);
            if(!error_message_out)
                return memory_manager->finalizeMemory(nullptr);
            std::string error_message;
            bool failed = memory_manager->finalizeMemory(&error_message);
            if(failed)
                *error_message_out = LLVM_string::from(error_message).release();
            return failed;
        },
        [](void *user_data) noexcept
        {
            delete static_cast<std::shared_ptr<llvm::SectionMemoryManager> *>(user_data);
        }));
    memory_manager_wrapper.release();
    return Shared_memory_manager(std::move(mcjit_memory_manager), std::move(memory_manager));
}
}
}
