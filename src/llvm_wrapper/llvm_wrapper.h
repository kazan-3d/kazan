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
#include <llvm-c/Target.h>
#include <llvm-c/TargetMachine.h>
#include <llvm-c/ExecutionEngine.h>
#include <llvm-c/Analysis.h>
#include <memory>
#include <type_traits>
#include <utility>
#include <string>
#include <cassert>
#include "util/string_view.h"
#include "util/variant.h"

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

struct Target_deleter
{
    void operator()(::LLVMTargetRef target) noexcept
    {
        static_cast<void>(target);
    }
};

struct Target : public Wrapper<::LLVMTargetRef, Target_deleter>
{
    using Wrapper::Wrapper;
    static LLVM_string get_default_target_triple()
    {
        return LLVM_string::wrap(::LLVMGetDefaultTargetTriple());
    }
    static LLVM_string get_process_target_triple();
    static LLVM_string get_host_cpu_name();
    static LLVM_string get_host_cpu_features();
    typedef util::variant<Target, LLVM_string> Target_or_error_message;
    static Target_or_error_message get_target_from_target_triple(const char *triple)
    {
        ::LLVMTargetRef target = nullptr;
        char *error_message = nullptr;
        if(::LLVMGetTargetFromTriple(triple, &target, &error_message) == 0)
            return Target(target);
        return LLVM_string::wrap(error_message);
    }
    static Target get_native_target()
    {
        auto native_triple = get_process_target_triple();
        auto retval = get_target_from_target_triple(native_triple.get());
        auto *target = util::get_if<Target>(&retval);
        if(!target)
            throw std::runtime_error(
                "can't find target for native triple (" + std::string(native_triple) + "): "
                + util::get<LLVM_string>(retval).get());
        return std::move(*target);
    }
};

struct Target_data_deleter
{
    void operator()(::LLVMTargetDataRef v) noexcept
    {
        ::LLVMDisposeTargetData(v);
    }
};

struct Target_data : public Wrapper<::LLVMTargetDataRef, Target_data_deleter>
{
    using Wrapper::Wrapper;
    static LLVM_string to_string(::LLVMTargetDataRef td)
    {
        return LLVM_string::wrap(::LLVMCopyStringRepOfTargetData(td));
    }
    LLVM_string to_string() const
    {
        return to_string(get());
    }
    static Target_data from_string(const char *str)
    {
        return Target_data(::LLVMCreateTargetData(str));
    }
};

struct Target_machine_deleter
{
    void operator()(::LLVMTargetMachineRef tm) noexcept
    {
        ::LLVMDisposeTargetMachine(tm);
    }
};

struct Target_machine : public Wrapper<::LLVMTargetMachineRef, Target_machine_deleter>
{
    using Wrapper::Wrapper;
    static Target_machine create_native_target_machine();
    static Target get_target(::LLVMTargetMachineRef tm)
    {
        return Target(::LLVMGetTargetMachineTarget(tm));
    }
    Target get_target() const
    {
        return get_target(get());
    }
    static LLVM_string get_target_triple(::LLVMTargetMachineRef tm)
    {
        return LLVM_string::wrap(::LLVMGetTargetMachineTriple(tm));
    }
    LLVM_string get_target_triple() const
    {
        return get_target_triple(get());
    }
    static Target_data create_target_data_layout(::LLVMTargetMachineRef tm)
    {
        return Target_data(::LLVMCreateTargetDataLayout(tm));
    }
    Target_data create_target_data_layout() const
    {
        return create_target_data_layout(get());
    }
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
    static Module create_native(const char *id, ::LLVMContextRef context)
    {
        Module retval = create(id, context);
        retval.set_target_to_native();
        return retval;
    }
    static void set_target_to_native(::LLVMModuleRef module);
    void set_target_to_native()
    {
        set_target_to_native(get());
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

inline ::LLVMTypeRef get_scalar_or_vector_element_type(::LLVMTypeRef type)
{
    if(::LLVMGetTypeKind(type) == ::LLVMTypeKind::LLVMVectorTypeKind)
        return ::LLVMGetElementType(type);
    return type;
}
}
}

#endif /* LLVM_WRAPPER_LLVM_WRAPPER_H_ */
