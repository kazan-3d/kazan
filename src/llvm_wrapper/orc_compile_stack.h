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
#ifndef LLVM_WRAPPER_ORC_COMPILE_STACK_H_
#define LLVM_WRAPPER_ORC_COMPILE_STACK_H_

#include "llvm_wrapper.h"
#include <string>
#include <utility>
#include <functional>

namespace kazan
{
namespace llvm_wrapper
{
class Orc_compile_stack_implementation;

typedef Orc_compile_stack_implementation *Orc_compile_stack_ref;

struct Orc_compile_stack_deleter
{
    void operator()(Orc_compile_stack_ref v) const noexcept;
};

struct Orc_compile_stack : public Wrapper<Orc_compile_stack_ref, Orc_compile_stack_deleter>
{
    using Wrapper::Wrapper;
    typedef std::uintptr_t (*Symbol_resolver_callback)(const std::string &name, void *user_data);
    typedef std::uint64_t Module_handle;
    typedef std::function<Module(Module, ::LLVMTargetMachineRef target_machine)> Optimize_function;
    static Orc_compile_stack create(Target_machine target_machine,
                                    Optimize_function optimize_function = nullptr);
    static Module_handle add_eagerly_compiled_ir(Orc_compile_stack_ref orc_compile_stack,
                                                 Module module,
                                                 Symbol_resolver_callback symbol_resolver_callback,
                                                 void *symbol_resolver_user_data);
    Module_handle add_eagerly_compiled_ir(Module module,
                                          Symbol_resolver_callback symbol_resolver_callback,
                                          void *symbol_resolver_user_data)
    {
        return add_eagerly_compiled_ir(
            get(), std::move(module), symbol_resolver_callback, symbol_resolver_user_data);
    }
    static std::uintptr_t get_symbol_address(Orc_compile_stack_ref orc_compile_stack,
                                             const std::string &symbol_name);
    template <typename T>
    static T *get_symbol(Orc_compile_stack_ref orc_compile_stack, const std::string &symbol_name)
    {
        return reinterpret_cast<T *>(get_symbol_address(orc_compile_stack, symbol_name));
    }
    std::uintptr_t get_symbol_address(const std::string &symbol_name)
    {
        return get_symbol_address(get(), symbol_name);
    }
    template <typename T>
    T *get_symbol(const std::string &symbol_name)
    {
        return get_symbol<T>(get(), symbol_name);
    }
};
}
}

#endif // LLVM_WRAPPER_ORC_COMPILE_STACK_H_
