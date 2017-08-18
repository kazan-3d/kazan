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
#include "orc_compile_stack.h"
#include <llvm/ExecutionEngine/ExecutionEngine.h>
#include <llvm/ExecutionEngine/RTDyldMemoryManager.h>
#include <llvm/ExecutionEngine/JITEventListener.h>
#include <llvm/ExecutionEngine/Orc/ObjectLinkingLayer.h>
#include <llvm/Target/TargetMachine.h>
#include <llvm/Config/llvm-config.h>

#if LLVM_VERSION_MAJOR != 4 || LLVM_VERSION_MINOR != 0
#error Orc compile stack is not yet implemented for this version of LLVM
#endif

namespace vulkan_cpu
{
namespace llvm_wrapper
{
namespace
{
// implement the unwrap functions that aren't in public llvm headers
llvm::TargetMachine *unwrap(::LLVMTargetMachineRef v) noexcept
{
    return reinterpret_cast<llvm::TargetMachine *>(v);
}
}

class Orc_compile_stack_implementation
{
    Orc_compile_stack_implementation(const Orc_compile_stack_implementation &) = delete;
    Orc_compile_stack_implementation(Orc_compile_stack_implementation &&) = delete;
    Orc_compile_stack_implementation &operator=(const Orc_compile_stack_implementation &) = delete;
    Orc_compile_stack_implementation &operator=(Orc_compile_stack_implementation &&) = delete;

private:
    std::unique_ptr<llvm::TargetMachine> target_machine;
    const llvm::DataLayout data_layout;
    llvm::orc::ObjectLinkingLayer<> object_linking_layer;

public:
    explicit Orc_compile_stack_implementation(Target_machine target_machine_in)
        : target_machine(unwrap(target_machine_in.release())),
          data_layout(target_machine->createDataLayout())
    {
#warning finish
        assert(!"finish");
    }
    void add_eagerly_compiled_ir(Module module,
                                 ::LLVMOrcSymbolResolverFn symbol_resolver_callback,
                                 void *symbol_resolver_user_data)
    {
#warning finish
        assert(!"finish");
    }
    std::uintptr_t get_symbol_address(const char *symbol_name)
    {
#warning finish
        assert(!"finish");
        return 0;
    }
};

void Orc_compile_stack_deleter::operator()(Orc_compile_stack_ref v) const noexcept
{
    delete v;
}

Orc_compile_stack Orc_compile_stack::create(Target_machine target_machine)
{
#warning finish
    assert(!"finish");
    return {};
}

void Orc_compile_stack::add_eagerly_compiled_ir(Orc_compile_stack_ref orc_compile_stack,
                                                Module module,
                                                ::LLVMOrcSymbolResolverFn symbol_resolver_callback,
                                                void *symbol_resolver_user_data)
{
    orc_compile_stack->add_eagerly_compiled_ir(
        std::move(module), symbol_resolver_callback, symbol_resolver_user_data);
}

std::uintptr_t Orc_compile_stack::get_symbol_address(Orc_compile_stack_ref orc_compile_stack,
                                                     const char *symbol_name)
{
    return orc_compile_stack->get_symbol_address(symbol_name);
}
}
}
