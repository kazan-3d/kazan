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
#ifndef LLVM_WRAPPER_LLVM_WRAPPER_H_
#define LLVM_WRAPPER_LLVM_WRAPPER_H_

#include <llvm-c/Core.h>
#include <memory>
#include <type_traits>
#include <utility>

namespace vulkan_cpu
{
namespace llvm_wrapper
{
template <typename T, typename Deleter>
class Wrapper
{
    static_assert(std::is_pointer<T>::value, "");

private:
    T value;

public:
    constexpr Wrapper() noexcept : value(nullptr)
    {
    }
    constexpr explicit Wrapper(T value) noexcept : value(value)
    {
    }
    Wrapper(Wrapper &&rt) noexcept : value(rt.value)
    {
        rt.value = nullptr;
    }
    Wrapper &operator=(Wrapper rt) noexcept
    {
        swap(rt);
        return *this;
    }
    ~Wrapper() noexcept
    {
        if(value)
            Deleter()(value);
    }
    void swap(Wrapper &other) noexcept
    {
        using std::swap;
        swap(value, other.value);
    }
    T get() const noexcept
    {
        return value;
    }
    operator T() const noexcept
    {
        return value;
    }
    T release() noexcept
    {
        auto retval = value;
        value = nullptr;
        return retval;
    }
    void reset(T value) noexcept
    {
        *this = Wrapper(value);
    }
};

struct Context_deleter
{
    void operator()(::LLVMContextRef context) noexcept
    {
        ::LLVMContextDispose(context);
    }
};

struct Context : public Wrapper<::LLVMContextRef, Context_deleter>
{
    using Wrapper::Wrapper;
    static Context create();
};

struct Module_deleter
{
    void operator()(::LLVMModuleRef module) noexcept
    {
        ::LLVMDisposeModule(module);
    }
};

struct Module : public Wrapper<::LLVMModuleRef, Module_deleter>
{
    using Wrapper::Wrapper;
    static Module create(const char *id, ::LLVMContextRef context)
    {
        return Module(::LLVMModuleCreateWithNameInContext(id, context));
    }
};
}
}

#endif /* LLVM_WRAPPER_LLVM_WRAPPER_H_ */
