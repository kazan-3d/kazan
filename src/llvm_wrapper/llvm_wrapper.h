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
#include <string>
#include <cassert>
#include "util/string_view.h"

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
    explicit operator T() const noexcept
    {
        return value;
    }
    explicit operator bool() const noexcept
    {
        return value != nullptr;
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

struct LLVM_string_deleter
{
    void operator()(char *str)
    {
        ::LLVMDisposeMessage(str);
    }
};

class LLVM_string : public Wrapper<char *, LLVM_string_deleter>
{
public:
    constexpr LLVM_string() noexcept : Wrapper()
    {
    }
    static LLVM_string wrap(char *value) noexcept
    {
        LLVM_string retval;
        retval.reset(value);
        return retval;
    }
    static LLVM_string from(const char *value)
    {
        return wrap(::LLVMCreateMessage(value));
    }
    static LLVM_string from(const std::string &value)
    {
        return from(value.c_str());
    }
    static LLVM_string from(util::string_view value)
    {
        return from(std::string(value));
    }
    operator util::string_view() const
    {
        assert(*this);
        return util::string_view(get());
    }
    explicit operator std::string() const
    {
        assert(*this);
        return get();
    }
    explicit operator char *() const // override non-explicit operator
    {
        return get();
    }
};

inline LLVM_string print_type_to_string(::LLVMTypeRef type)
{
    return LLVM_string::wrap(::LLVMPrintTypeToString(type));
}

struct Builder_deleter
{
    void operator()(::LLVMBuilderRef v) noexcept
    {
        return ::LLVMDisposeBuilder(v);
    }
};

struct Builder : public Wrapper<::LLVMBuilderRef, Builder_deleter>
{
    using Wrapper::Wrapper;
    static Builder create(::LLVMContextRef context)
    {
        return Builder(::LLVMCreateBuilderInContext(context));
    }
};
}
}

#endif /* LLVM_WRAPPER_LLVM_WRAPPER_H_ */
