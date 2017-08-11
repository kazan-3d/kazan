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
#include <llvm-c/OrcBindings.h>
#include <llvm-c/Analysis.h>
#include <memory>
#include <type_traits>
#include <utility>
#include <string>
#include <cassert>
#include <stdexcept>
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

private:
    static void init_helper();

public:
    static void init()
    {
        static int v = (init_helper(), 0);
        static_cast<void>(v);
    }
    static Context create()
    {
        init();
        return Context(::LLVMContextCreate());
    }
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
        Context::init();
        return LLVM_string::wrap(::LLVMGetDefaultTargetTriple());
    }
    static LLVM_string get_process_target_triple();
    static LLVM_string get_host_cpu_name();
    static LLVM_string get_host_cpu_features();
    typedef util::variant<Target, LLVM_string> Target_or_error_message;
    static Target_or_error_message get_target_from_target_triple(const char *triple)
    {
        Context::init();
        ::LLVMTargetRef target = nullptr;
        char *error_message = nullptr;
        if(::LLVMGetTargetFromTriple(triple, &target, &error_message) == 0)
            return Target(target);
        return LLVM_string::wrap(error_message);
    }
    static Target get_native_target()
    {
        Context::init();
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
    static std::size_t get_pointer_alignment(::LLVMTargetDataRef td) noexcept;
    std::size_t get_pointer_alignment() const noexcept
    {
        return get_pointer_alignment(get());
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
    static LLVM_string get_cpu(::LLVMTargetMachineRef tm)
    {
        return LLVM_string::wrap(::LLVMGetTargetMachineCPU(tm));
    }
    LLVM_string get_cpu() const
    {
        return get_cpu(get());
    }
    static LLVM_string get_feature_string(::LLVMTargetMachineRef tm)
    {
        return LLVM_string::wrap(::LLVMGetTargetMachineFeatureString(tm));
    }
    LLVM_string get_feature_string() const
    {
        return get_feature_string(get());
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
    static Module create_with_target_machine(const char *id,
                                             ::LLVMContextRef context,
                                             ::LLVMTargetMachineRef target_machine)
    {
        Module retval = create(id, context);
        retval.set_target_machine(target_machine);
        return retval;
    }
    static void set_target_machine(::LLVMModuleRef module, ::LLVMTargetMachineRef target_machine);
    static void set_function_target_machine(::LLVMValueRef function,
                                            ::LLVMTargetMachineRef target_machine);
    void set_target_machine(::LLVMTargetMachineRef target_machine)
    {
        set_target_machine(get(), target_machine);
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
    static ::LLVMValueRef build_smod(::LLVMBuilderRef builder,
                                     ::LLVMValueRef lhs,
                                     ::LLVMValueRef rhs,
                                     const char *result_name)
    {
        auto srem_result = ::LLVMBuildSRem(builder, lhs, rhs, "");
        auto zero_constant = ::LLVMConstInt(::LLVMTypeOf(lhs), 0, false);
        auto different_signs = ::LLVMBuildICmp(
            builder, ::LLVMIntSLT, ::LLVMBuildXor(builder, lhs, rhs, ""), zero_constant, "");
        auto imperfectly_divides =
            ::LLVMBuildICmp(builder, ::LLVMIntNE, srem_result, zero_constant, "");
        auto adjustment =
            ::LLVMBuildSelect(builder,
                              ::LLVMBuildAnd(builder, different_signs, imperfectly_divides, ""),
                              rhs,
                              zero_constant,
                              "");
        return ::LLVMBuildAdd(builder, srem_result, adjustment, result_name);
    }
    ::LLVMValueRef build_smod(::LLVMValueRef lhs, ::LLVMValueRef rhs, const char *result_name) const
    {
        return build_smod(get(), lhs, rhs, result_name);
    }
};

inline ::LLVMTypeRef get_scalar_or_vector_element_type(::LLVMTypeRef type)
{
    if(::LLVMGetTypeKind(type) == ::LLVMTypeKind::LLVMVectorTypeKind)
        return ::LLVMGetElementType(type);
    return type;
}

// TODO: add CMake tests to determine which Orc version we need
#if 0
// added error code return from LLVMOrcAddEagerlyCompiledIR
#define LLVM_WRAPPER_ORC_REVISION_NUMBER 307350
#elif 0
// added shared modules
#define LLVM_WRAPPER_ORC_REVISION_NUMBER 306182
#else
// initial revision
#define LLVM_WRAPPER_ORC_REVISION_NUMBER 251482
#endif

#if LLVM_WRAPPER_ORC_REVISION_NUMBER >= 306166
struct Orc_shared_module_ref_deleter
{
    void operator()(::LLVMSharedModuleRef v) noexcept
    {
        ::LLVMOrcDisposeSharedModuleRef(v);
    }
};

struct Orc_shared_module_ref : public Wrapper<::LLVMSharedModuleRef, Orc_shared_module_ref_deleter>
{
    using Wrapper::Wrapper;
    static Orc_shared_module_ref make(Module module)
    {
        return Orc_shared_module_ref(::LLVMOrcMakeSharedModule(module.release()));
    }
};
#endif

struct Orc_jit_stack_deleter
{
    void operator()(::LLVMOrcJITStackRef v) noexcept
    {
        ::LLVMOrcDisposeInstance(v);
    }
};

struct Orc_jit_stack : public Wrapper<::LLVMOrcJITStackRef, Orc_jit_stack_deleter>
{
    using Wrapper::Wrapper;
    static Orc_jit_stack create(Target_machine target_machine)
    {
        return Orc_jit_stack(::LLVMOrcCreateInstance(target_machine.release()));
    }
    static ::LLVMOrcModuleHandle add_eagerly_compiled_ir(
        ::LLVMOrcJITStackRef orc_jit_stack,
        Module module,
        ::LLVMOrcSymbolResolverFn symbol_resolver_callback,
        void *symbol_resolver_user_data)
    {
        ::LLVMOrcModuleHandle retval{};
#if LLVM_WRAPPER_ORC_REVISION_NUMBER >= 307350
        if(::LLVMOrcErrorSuccess
           != ::LLVMOrcAddEagerlyCompiledIR(orc_jit_stack,
                                            &retval,
                                            Orc_shared_module_ref::make(std::move(module)).get(),
                                            symbol_resolver_callback,
                                            symbol_resolver_user_data))
            throw std::runtime_error(std::string("LLVM Orc Error: ")
                                     + ::LLVMOrcGetErrorMsg(orc_jit_stack));
#elif LLVM_WRAPPER_ORC_REVISION_NUMBER >= 306182
        retval = ::LLVMOrcAddEagerlyCompiledIR(orc_jit_stack,
                                               Orc_shared_module_ref::make(std::move(module)).get(),
                                               symbol_resolver_callback,
                                               symbol_resolver_user_data);
#elif LLVM_WRAPPER_ORC_REVISION_NUMBER >= 251482
        retval = ::LLVMOrcAddEagerlyCompiledIR(
            orc_jit_stack, module.release(), symbol_resolver_callback, symbol_resolver_user_data);
#else
#error unsupported LLVM_WRAPPER_ORC_REVISION_NUMBER
#endif
        return retval;
    }
    ::LLVMOrcModuleHandle add_eagerly_compiled_ir(
        Module module,
        ::LLVMOrcSymbolResolverFn symbol_resolver_callback,
        void *symbol_resolver_user_data)
    {
        return add_eagerly_compiled_ir(
            get(), std::move(module), symbol_resolver_callback, symbol_resolver_user_data);
    }
    static std::uintptr_t get_symbol_address(::LLVMOrcJITStackRef orc_jit_stack,
                                             const char *symbol_name)
    {
        return ::LLVMOrcGetSymbolAddress(orc_jit_stack, symbol_name);
    }
    template <typename T>
    static T *get_symbol(::LLVMOrcJITStackRef orc_jit_stack, const char *symbol_name)
    {
        return reinterpret_cast<T *>(get_symbol_address(orc_jit_stack, symbol_name));
    }
    std::uintptr_t get_symbol_address(const char *symbol_name)
    {
        return get_symbol_address(get(), symbol_name);
    }
    template <typename T>
    T *get_symbol(const char *symbol_name)
    {
        return get_symbol<T>(get(), symbol_name);
    }
};
}
}

#endif /* LLVM_WRAPPER_LLVM_WRAPPER_H_ */
