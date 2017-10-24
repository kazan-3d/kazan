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
#ifndef SPIRV_TO_LLVM_SPIRV_TO_LLVM_H_
#define SPIRV_TO_LLVM_SPIRV_TO_LLVM_H_

#include "spirv/parser.h"
#include <stdexcept>
#include <memory>
#include <vector>
#include <string>
#include <cassert>
#include <type_traits>
#include <utility>
#include <cstddef>
#include <unordered_map>
#include "llvm_wrapper/llvm_wrapper.h"
#include "util/string_view.h"
#include "vulkan/vulkan.h"
#include "vulkan/remove_xlib_macros.h"
#include "vulkan/api_objects.h"
#include "util/bitset.h"

namespace kazan
{
namespace spirv_to_llvm
{
/// std::size_t is instruction_start_index
typedef std::unordered_map<std::size_t, spirv::Decoration_with_parameters> Spirv_decoration_set;

namespace spirv_types
{
class Void;
class Bool;
class Int;
class Float;
class Vector;
class Matrix;
class Image;
class Sampler;
class Sampled_image;
class Array;
class Runtime_array;
class Struct;
class Opaque;
class Pointer;
class Function;
class Event;

class Type : public std::enable_shared_from_this<Type>
{
public:
    enum class Kind
    {
        Void,
        Bool,
        Int,
        Float,
        Vector,
        Matrix,
        Image,
        Sampler,
        Sampled_image,
        Array,
        Runtime_array,
        Struct,
        Opaque,
        Pointer,
        Function,
        Event,
    };

private:
    static constexpr Kind get_kind_from_type_helper(Void *) noexcept
    {
        return Kind::Void;
    }
    static constexpr Kind get_kind_from_type_helper(Bool *) noexcept
    {
        return Kind::Bool;
    }
    static constexpr Kind get_kind_from_type_helper(Int *) noexcept
    {
        return Kind::Int;
    }
    static constexpr Kind get_kind_from_type_helper(Float *) noexcept
    {
        return Kind::Float;
    }
    static constexpr Kind get_kind_from_type_helper(Vector *) noexcept
    {
        return Kind::Vector;
    }
    static constexpr Kind get_kind_from_type_helper(Matrix *) noexcept
    {
        return Kind::Matrix;
    }
    static constexpr Kind get_kind_from_type_helper(Image *) noexcept
    {
        return Kind::Image;
    }
    static constexpr Kind get_kind_from_type_helper(Sampler *) noexcept
    {
        return Kind::Sampler;
    }
    static constexpr Kind get_kind_from_type_helper(Sampled_image *) noexcept
    {
        return Kind::Sampled_image;
    }
    static constexpr Kind get_kind_from_type_helper(Array *) noexcept
    {
        return Kind::Array;
    }
    static constexpr Kind get_kind_from_type_helper(Runtime_array *) noexcept
    {
        return Kind::Runtime_array;
    }
    static constexpr Kind get_kind_from_type_helper(Struct *) noexcept
    {
        return Kind::Struct;
    }
    static constexpr Kind get_kind_from_type_helper(Opaque *) noexcept
    {
        return Kind::Opaque;
    }
    static constexpr Kind get_kind_from_type_helper(Pointer *) noexcept
    {
        return Kind::Pointer;
    }
    static constexpr Kind get_kind_from_type_helper(Function *) noexcept
    {
        return Kind::Function;
    }
    static constexpr Kind get_kind_from_type_helper(Event *) noexcept
    {
        return Kind::Event;
    }

public:
    template <typename T>
    static constexpr decltype(Type::get_kind_from_type_helper(static_cast<T *>(nullptr)))
        get_kind_from_type() noexcept
    {
        return Type::get_kind_from_type_helper(static_cast<T *>(nullptr));
    }

private:
    const Kind kind;
    const std::size_t instruction_start_index;

public:
    virtual ~Type() = default;
    template <typename Child>
    explicit Type(std::size_t instruction_start_index) noexcept
        : kind(get_kind_from_type<Child>()),
          instruction_start_index(instruction_start_index)
    {
    }
    Kind get_kind() const noexcept
    {
        return kind;
    }
    std::size_t get_instruction_start_index() const noexcept
    {
        return instruction_start_index;
    }
    virtual std::shared_ptr<Type> get_type_with_decoration(const Spirv_decoration_set::value_type &decoration) = 0;
    virtual std::shared_ptr<Type> get_type_with_member_decoration(
        std::uint32_t member_index, const Spirv_decoration_set::value_type &decoration)
    {
        assert(!"type has no members to decorate");
        return shared_from_this();
    }
};
}

struct Jit_symbol_resolver
{
    typedef void (*Resolved_symbol)();
    Resolved_symbol resolve(util::string_view name);
    static std::uint64_t resolve(const char *name, void *user_data) noexcept
    {
        return reinterpret_cast<std::uint64_t>(
            static_cast<Jit_symbol_resolver *>(user_data)->resolve(name));
    }
    static std::uintptr_t resolve(const std::string &name, void *user_data) noexcept
    {
        return reinterpret_cast<std::uintptr_t>(
            static_cast<Jit_symbol_resolver *>(user_data)->resolve(name));
    }
};
}
}

#endif /* SPIRV_TO_LLVM_SPIRV_TO_LLVM_H_ */
