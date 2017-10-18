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
#include <algorithm>
#include "llvm_wrapper/llvm_wrapper.h"
#include "util/string_view.h"
#include "vulkan/vulkan.h"
#include "vulkan/remove_xlib_macros.h"
#include "vulkan/api_objects.h"
#include "util/bitset.h"

namespace kazan
{
namespace pipeline
{
struct Instantiated_pipeline_layout;
}

namespace spirv_to_llvm
{
struct LLVM_type_and_alignment
{
    ::LLVMTypeRef type;
    std::size_t alignment;
    constexpr LLVM_type_and_alignment() noexcept : type(nullptr), alignment(0)
    {
    }
    constexpr LLVM_type_and_alignment(::LLVMTypeRef type, std::size_t alignment) noexcept
        : type(type),
          alignment(alignment)
    {
    }
};

struct Shader_interface_position
{
    std::size_t value;
    static constexpr int component_index_bit_width = 2;
    static constexpr std::size_t component_index_count = 1ULL << component_index_bit_width;
    static constexpr std::size_t component_index_mask = component_index_count - 1;
    static constexpr std::size_t location_mask = ~component_index_mask;
    static constexpr std::size_t location_shift_amount = component_index_bit_width;
    constexpr std::uint32_t get_location() const noexcept
    {
        return (value & location_mask) >> location_shift_amount;
    }
    constexpr std::uint32_t get_component_index() const noexcept
    {
        return value & component_index_mask;
    }
    constexpr std::uint32_t get_components_left_in_current_location() const noexcept
    {
        return component_index_count - get_component_index();
    }
    constexpr bool is_aligned_to_location() const noexcept
    {
        return get_component_index() == 0;
    }
    constexpr Shader_interface_position get_aligned_location_rounding_up() const noexcept
    {
        if(is_aligned_to_location())
            return *this;
        return Shader_interface_position(get_location() + 1);
    }
    constexpr Shader_interface_position get_position_after_components(std::uint32_t count) const
        noexcept
    {
        std::uint32_t result_component_index = get_component_index() + count;
        std::uint32_t result_location =
            get_location() + result_component_index / component_index_count;
        result_component_index %= component_index_count;
        return Shader_interface_position(result_location, result_component_index);
    }
    constexpr Shader_interface_position(std::uint32_t location,
                                        std::uint8_t component_index) noexcept
        : value((location << location_shift_amount) | component_index)
    {
        assert(location == get_location() && component_index == get_component_index());
    }
    constexpr explicit Shader_interface_position(std::uint32_t location) noexcept
        : Shader_interface_position(location, 0)
    {
    }
    constexpr Shader_interface_position() noexcept : value(0)
    {
    }
    Shader_interface_position(
        spirv::Decoration_location_parameters location,
        util::optional<spirv::Decoration_component_parameters> component) noexcept
        : Shader_interface_position(location.location, component ? component->component : 0)
    {
    }
    explicit Shader_interface_position(
        const std::vector<spirv::Decoration_with_parameters> &decorations)
        : Shader_interface_position()
    {
        util::optional<spirv::Decoration_location_parameters> location;
        util::optional<spirv::Decoration_component_parameters> component;
        for(auto &decoration : decorations)
        {
            switch(decoration.value)
            {
            case spirv::Decoration::location:
                location = util::get<spirv::Decoration_location_parameters>(decoration.parameters);
                break;
            case spirv::Decoration::component:
                component =
                    util::get<spirv::Decoration_component_parameters>(decoration.parameters);
                break;
            default:
                break;
            }
        }
        if(!location)
            throw spirv::Parser_error(0, 0, "missing Location decoration");
        *this = Shader_interface_position(*location, component);
    }
    friend constexpr bool operator==(Shader_interface_position a,
                                     Shader_interface_position b) noexcept
    {
        return a.value == b.value;
    }
    friend constexpr bool operator!=(Shader_interface_position a,
                                     Shader_interface_position b) noexcept
    {
        return a.value != b.value;
    }
    friend constexpr bool operator<(Shader_interface_position a,
                                    Shader_interface_position b) noexcept
    {
        return a.value < b.value;
    }
    friend constexpr bool operator>(Shader_interface_position a,
                                    Shader_interface_position b) noexcept
    {
        return a.value > b.value;
    }
    friend constexpr bool operator<=(Shader_interface_position a,
                                     Shader_interface_position b) noexcept
    {
        return a.value <= b.value;
    }
    friend constexpr bool operator>=(Shader_interface_position a,
                                     Shader_interface_position b) noexcept
    {
        return a.value >= b.value;
    }
};

/** represents the range [begin_position, end_position) */
struct Shader_interface_range
{
    Shader_interface_position begin_position;
    Shader_interface_position end_position;
    constexpr bool empty() const noexcept
    {
        return end_position == begin_position;
    }
    constexpr bool overlaps(Shader_interface_range other) const noexcept
    {
        if(begin_position >= other.end_position)
            return false;
        if(other.begin_position >= end_position)
            return false;
        return true;
    }
};

class Type_descriptor;

class Shader_interface
{
public:
    /** uses a single type for both signed and unsigned integer variants */
    enum class Component_type
    {
        Int8,
        Int16,
        Int32,
        Int64,
        Float16,
        Float32,
        Float64,
    };
    static constexpr std::uint32_t get_type_component_count(
        Component_type type, std::size_t vector_element_count) noexcept
    {
        std::size_t size_in_bytes = 0;
        switch(type)
        {
        case Component_type::Int8:
            size_in_bytes = sizeof(std::uint8_t);
            break;
        case Component_type::Int16:
            size_in_bytes = sizeof(std::uint16_t);
            break;
        case Component_type::Int32:
            size_in_bytes = sizeof(std::uint32_t);
            break;
        case Component_type::Int64:
            size_in_bytes = sizeof(std::uint64_t);
            break;
        case Component_type::Float16:
            size_in_bytes = sizeof(std::uint16_t);
            break;
        case Component_type::Float32:
            size_in_bytes = sizeof(float);
            break;
        case Component_type::Float64:
            size_in_bytes = sizeof(double);
            break;
        }
        assert(size_in_bytes != 0);
        assert(vector_element_count >= 1 && vector_element_count <= 4);
        size_in_bytes *= vector_element_count;
        constexpr std::size_t component_size_in_bytes = sizeof(float);
        static_assert(component_size_in_bytes == 4, "");
        return (size_in_bytes + component_size_in_bytes - 1) / component_size_in_bytes;
    }
    static util::optional<Component_type> get_component_type_for_llvm_scalar_type(
        ::LLVMTypeRef type)
    {
        util::optional<Shader_interface::Component_type> component_type;
        switch(::LLVMGetTypeKind(type))
        {
        case ::LLVMHalfTypeKind:
            return Shader_interface::Component_type::Float16;
        case ::LLVMFloatTypeKind:
            return Shader_interface::Component_type::Float32;
        case ::LLVMDoubleTypeKind:
            return Shader_interface::Component_type::Float64;
        case ::LLVMIntegerTypeKind:
        {
            auto bit_width = ::LLVMGetIntTypeWidth(type);
            switch(bit_width)
            {
            case 8:
                return Shader_interface::Component_type::Int8;
            case 16:
                return Shader_interface::Component_type::Int16;
            case 32:
                return Shader_interface::Component_type::Int32;
            case 64:
                return Shader_interface::Component_type::Int64;
            default:
                break;
            }
            break;
        }
        case ::LLVMVoidTypeKind:
        case ::LLVMX86_FP80TypeKind:
        case ::LLVMFP128TypeKind:
        case ::LLVMPPC_FP128TypeKind:
        case ::LLVMLabelTypeKind:
        case ::LLVMFunctionTypeKind:
        case ::LLVMStructTypeKind:
        case ::LLVMArrayTypeKind:
        case ::LLVMPointerTypeKind:
        case ::LLVMVectorTypeKind:
        case ::LLVMMetadataTypeKind:
        case ::LLVMX86_MMXTypeKind:
        case ::LLVMTokenTypeKind:
            break;
        }
        return {};
    }
    enum class Interpolation_kind
    {
        Perspective,
        Linear,
        Flat,
    };
    struct Variable
    {
        Component_type type;
        Interpolation_kind interpolation_kind;
        Shader_interface_range range;
        std::vector<std::size_t> indexes;
        std::shared_ptr<Type_descriptor> base_type;
        Variable() noexcept : type(), interpolation_kind(), range(), indexes(), base_type()
        {
        }
        Variable(Component_type type,
                 Interpolation_kind interpolation_kind,
                 Shader_interface_range range,
                 std::vector<std::size_t> indexes,
                 std::shared_ptr<Type_descriptor> base_type) noexcept
            : type(type),
              interpolation_kind(interpolation_kind),
              range(range),
              indexes(std::move(indexes)),
              base_type(std::move(base_type))
        {
        }
        explicit operator bool() const noexcept
        {
            return !range.empty();
        }
    };

private:
    std::vector<Variable> variables;
    bool is_sorted;

private:
    void sort_variables() noexcept
    {
        std::stable_sort(variables.begin(),
                         variables.end(),
                         [](const Variable &a, const Variable &b) noexcept
                         {
                             return a.range.begin_position < b.range.begin_position;
                         });
        is_sorted = true;
    }

public:
    Shader_interface() noexcept : variables()
    {
    }
    explicit Shader_interface(std::vector<Variable> variables) noexcept
        : variables(std::move(variables)),
          is_sorted(false)
    {
    }
    const std::vector<Variable> &get_sorted_variables() noexcept
    {
        if(!is_sorted)
            sort_variables();
        return variables;
    }
    void add(const Variable &variable)
    {
        variables.push_back(variable);
        is_sorted = false;
    }
};

class Simple_type_descriptor;
class Vector_type_descriptor;
class Matrix_type_descriptor;
class Row_major_matrix_type_descriptor;
class Array_type_descriptor;
class Pointer_type_descriptor;
class Function_type_descriptor;
class Struct_type_descriptor;
class Type_descriptor : public std::enable_shared_from_this<Type_descriptor>
{
    Type_descriptor(const Type_descriptor &) = delete;
    Type_descriptor &operator=(const Type_descriptor &) = delete;

public:
    struct Type_visitor
    {
        virtual ~Type_visitor() = default;
        virtual void visit(Simple_type_descriptor &type) = 0;
        virtual void visit(Vector_type_descriptor &type) = 0;
        virtual void visit(Matrix_type_descriptor &type) = 0;
        virtual void visit(Row_major_matrix_type_descriptor &type) = 0;
        virtual void visit(Array_type_descriptor &type) = 0;
        virtual void visit(Pointer_type_descriptor &type) = 0;
        virtual void visit(Function_type_descriptor &type) = 0;
        virtual void visit(Struct_type_descriptor &type) = 0;
    };

public:
    const std::vector<spirv::Decoration_with_parameters> decorations;

public:
    explicit Type_descriptor(std::vector<spirv::Decoration_with_parameters> decorations) noexcept
        : decorations(std::move(decorations))
    {
    }
    virtual ~Type_descriptor() = default;
    virtual LLVM_type_and_alignment get_or_make_type() = 0;
    virtual void visit(Type_visitor &type_visitor) = 0;
    virtual std::shared_ptr<Type_descriptor> get_row_major_type(::LLVMTargetDataRef target_data)
    {
        return shared_from_this();
    }
    virtual std::shared_ptr<Type_descriptor> get_column_major_type(::LLVMTargetDataRef target_data)
    {
        return shared_from_this();
    }
    virtual util::optional<std::size_t> get_matrix_stride(::LLVMTargetDataRef target_data) const
    {
        return {};
    }
    void visit(Type_visitor &&type_visitor)
    {
        visit(type_visitor);
    }
    template <typename Fn>
    typename std::enable_if<!std::is_convertible<Fn &&, const Type_visitor &>::value, void>::type
        visit(Fn &&fn)
    {
        struct Visitor final : public Type_visitor
        {
            Fn &fn;
            virtual void visit(Simple_type_descriptor &type) override
            {
                std::forward<Fn>(fn)(type);
            }
            virtual void visit(Vector_type_descriptor &type) override
            {
                std::forward<Fn>(fn)(type);
            }
            virtual void visit(Matrix_type_descriptor &type) override
            {
                std::forward<Fn>(fn)(type);
            }
            virtual void visit(Row_major_matrix_type_descriptor &type) override
            {
                std::forward<Fn>(fn)(type);
            }
            virtual void visit(Array_type_descriptor &type) override
            {
                std::forward<Fn>(fn)(type);
            }
            virtual void visit(Pointer_type_descriptor &type) override
            {
                std::forward<Fn>(fn)(type);
            }
            virtual void visit(Function_type_descriptor &type) override
            {
                std::forward<Fn>(fn)(type);
            }
            virtual void visit(Struct_type_descriptor &type) override
            {
                std::forward<Fn>(fn)(type);
            }
            explicit Visitor(Fn &fn) noexcept : fn(fn)
            {
            }
        };
        visit(Visitor(fn));
    }
    class Recursion_checker;
    class Recursion_checker_state
    {
        friend class Recursion_checker;

    private:
        std::size_t recursion_count = 0;
    };
    class Recursion_checker
    {
        Recursion_checker(const Recursion_checker &) = delete;
        Recursion_checker &operator=(const Recursion_checker &) = delete;

    private:
        Recursion_checker_state &state;

    public:
        explicit Recursion_checker(Recursion_checker_state &state,
                                   std::size_t instruction_start_index)
            : state(state)
        {
            state.recursion_count++;
            if(state.recursion_count > 5)
                throw spirv::Parser_error(instruction_start_index,
                                          instruction_start_index,
                                          "too many recursions making type");
        }
        ~Recursion_checker()
        {
            state.recursion_count--;
        }
        std::size_t get_recursion_count() const noexcept
        {
            return state.recursion_count;
        }
        bool is_nested_recursion() const noexcept
        {
            return get_recursion_count() > 1;
        }
    };
    enum class Load_store_implementation_kind
    {
        Simple,
        Transpose_matrix,
    };
    virtual Load_store_implementation_kind get_load_store_implementation_kind()
    {
        return Load_store_implementation_kind::Simple;
    }
    util::optional<spirv::Decoration_with_parameters> find_decoration(
        spirv::Decoration decoration_id) const
    {
        for(auto &decoration : decorations)
            if(decoration.value == decoration_id)
                return decoration;
        return {};
    }
    struct Shader_interface_index_list_item
    {
        const Shader_interface_index_list_item *prev;
        std::size_t index;
    };
    static std::vector<std::size_t> shader_interface_index_list_to_vector(
        const Shader_interface_index_list_item *index_list)
    {
        std::size_t size = 0;
        for(auto *p = index_list; p; p = p->prev)
            size++;
        std::vector<std::size_t> retval(size);
        std::size_t i = size - 1;
        for(auto *p = index_list; p; p = p->prev)
            retval[i--] = p->index;
        return retval;
    }
    virtual void add_to_shader_interface(
        Shader_interface &shader_interface,
        util::optional<Shader_interface_position> &current_position,
        Shader_interface::Interpolation_kind interpolation_kind,
        const Shader_interface_index_list_item *parent_index_list,
        const std::shared_ptr<Type_descriptor> &base_type) = 0;
    void add_to_shader_interface(Shader_interface &shader_interface)
    {
        util::optional<Shader_interface_position> current_position;
        add_to_shader_interface(shader_interface,
                                current_position,
                                Shader_interface::Interpolation_kind::Perspective,
                                nullptr,
                                shared_from_this());
    }
};

class Simple_type_descriptor final : public Type_descriptor
{
private:
    LLVM_type_and_alignment type;

public:
    explicit Simple_type_descriptor(std::vector<spirv::Decoration_with_parameters> decorations,
                                    LLVM_type_and_alignment type) noexcept
        : Type_descriptor(std::move(decorations)),
          type(type)
    {
    }
    virtual LLVM_type_and_alignment get_or_make_type() override
    {
        return type;
    }
    virtual void visit(Type_visitor &type_visitor) override
    {
        type_visitor.visit(*this);
    }
    using Type_descriptor::add_to_shader_interface;
    virtual void add_to_shader_interface(
        Shader_interface &shader_interface,
        util::optional<Shader_interface_position> &current_position,
        Shader_interface::Interpolation_kind interpolation_kind,
        const Shader_interface_index_list_item *parent_index_list,
        const std::shared_ptr<Type_descriptor> &base_type) override
    {
        auto component_type = Shader_interface::get_component_type_for_llvm_scalar_type(type.type);
        if(!component_type)
            throw spirv::Parser_error(0, 0, "invalid type in shader interface");
        if(!current_position)
            throw spirv::Parser_error(
                0, 0, "no Location decoration specified for shader interface");
        auto component_count = Shader_interface::get_type_component_count(*component_type, 1);
        if(component_count > current_position->get_components_left_in_current_location()
           && current_position->get_component_index() != 0)
            throw spirv::Parser_error(0, 0, "Component decoration too big for type");
        Shader_interface_range range = {
            .begin_position = *current_position,
            .end_position = current_position->get_position_after_components(component_count),
        };
        current_position = range.end_position.get_aligned_location_rounding_up();
        shader_interface.add(
            Shader_interface::Variable(*component_type,
                                       interpolation_kind,
                                       range,
                                       shader_interface_index_list_to_vector(parent_index_list),
                                       base_type));
    }
};

class Vector_type_descriptor final : public Type_descriptor
{
private:
    LLVM_type_and_alignment type;
    std::shared_ptr<Simple_type_descriptor> element_type;
    std::size_t element_count;

public:
    explicit Vector_type_descriptor(std::vector<spirv::Decoration_with_parameters> decorations,
                                    std::shared_ptr<Simple_type_descriptor> element_type,
                                    std::size_t element_count,
                                    ::LLVMTargetDataRef target_data) noexcept
        : Type_descriptor(std::move(decorations)),
          type(make_vector_type(element_type, element_count, target_data)),
          element_type(std::move(element_type)),
          element_count(element_count)
    {
    }
    static LLVM_type_and_alignment make_vector_type(
        const std::shared_ptr<Simple_type_descriptor> &element_type,
        std::size_t element_count,
        ::LLVMTargetDataRef target_data)
    {
        auto llvm_element_type = element_type->get_or_make_type();
        auto type = ::LLVMVectorType(llvm_element_type.type, element_count);
        std::size_t alignment = ::LLVMPreferredAlignmentOfType(target_data, type);
        constexpr std::size_t max_abi_alignment = alignof(std::max_align_t);
        if(alignment > max_abi_alignment)
            alignment = max_abi_alignment;
        return {type, alignment};
    }
    virtual LLVM_type_and_alignment get_or_make_type() override
    {
        return type;
    }
    virtual void visit(Type_visitor &type_visitor) override
    {
        type_visitor.visit(*this);
    }
    const std::shared_ptr<Simple_type_descriptor> &get_element_type() const noexcept
    {
        return element_type;
    }
    std::size_t get_element_count() const noexcept
    {
        return element_count;
    }
    using Type_descriptor::add_to_shader_interface;
    virtual void add_to_shader_interface(
        Shader_interface &shader_interface,
        util::optional<Shader_interface_position> &current_position,
        Shader_interface::Interpolation_kind interpolation_kind,
        const Shader_interface_index_list_item *parent_index_list,
        const std::shared_ptr<Type_descriptor> &base_type) override
    {
        auto component_type = Shader_interface::get_component_type_for_llvm_scalar_type(
            ::LLVMGetElementType(type.type));
        if(!component_type)
            throw spirv::Parser_error(0, 0, "invalid type in shader interface");
        if(!current_position)
            throw spirv::Parser_error(
                0, 0, "no Location decoration specified for shader interface");
        auto component_count =
            Shader_interface::get_type_component_count(*component_type, element_count);
        if(component_count > current_position->get_components_left_in_current_location()
           && current_position->get_component_index() != 0)
            throw spirv::Parser_error(0, 0, "Component decoration too big for type");
        Shader_interface_range range = {
            .begin_position = *current_position,
            .end_position = current_position->get_position_after_components(component_count),
        };
        current_position = range.end_position.get_aligned_location_rounding_up();
        shader_interface.add(
            Shader_interface::Variable(*component_type,
                                       interpolation_kind,
                                       range,
                                       shader_interface_index_list_to_vector(parent_index_list),
                                       base_type));
    }
};

class Array_type_descriptor final : public Type_descriptor
{
private:
    LLVM_type_and_alignment type;
    std::shared_ptr<Type_descriptor> element_type;
    std::size_t element_count;
    std::size_t instruction_start_index;
    Recursion_checker_state recursion_checker_state;
    std::weak_ptr<Type_descriptor> column_major_type;
    std::weak_ptr<Type_descriptor> row_major_type;

public:
    explicit Array_type_descriptor(std::vector<spirv::Decoration_with_parameters> decorations,
                                   std::shared_ptr<Type_descriptor> element_type,
                                   std::size_t element_count,
                                   std::size_t instruction_start_index) noexcept
        : Type_descriptor(std::move(decorations)),
          type(),
          element_type(std::move(element_type)),
          element_count(element_count),
          instruction_start_index(instruction_start_index)
    {
    }
    virtual LLVM_type_and_alignment get_or_make_type() override
    {
        if(!type.type)
        {
            Recursion_checker recursion_checker(recursion_checker_state, instruction_start_index);
            auto llvm_element_type = element_type->get_or_make_type();
            type = LLVM_type_and_alignment(::LLVMArrayType(llvm_element_type.type, element_count),
                                           llvm_element_type.alignment);
        }
        return type;
    }
    virtual void visit(Type_visitor &type_visitor) override
    {
        type_visitor.visit(*this);
    }
    virtual std::shared_ptr<Type_descriptor> get_row_major_type(
        ::LLVMTargetDataRef target_data) override
    {
        auto retval = row_major_type.lock();
        if(retval)
            return retval;
        auto row_major_element_type = element_type->get_row_major_type(target_data);
        if(row_major_element_type == element_type)
            retval = shared_from_this();
        else
            retval = std::make_shared<Array_type_descriptor>(decorations,
                                                             std::move(row_major_element_type),
                                                             element_count,
                                                             instruction_start_index);
        row_major_type = retval;
        return retval;
    }
    virtual std::shared_ptr<Type_descriptor> get_column_major_type(
        ::LLVMTargetDataRef target_data) override
    {
        auto retval = column_major_type.lock();
        if(retval)
            return retval;
        auto column_major_element_type = element_type->get_column_major_type(target_data);
        if(column_major_element_type == element_type)
            retval = shared_from_this();
        else
            retval = std::make_shared<Array_type_descriptor>(decorations,
                                                             std::move(column_major_element_type),
                                                             element_count,
                                                             instruction_start_index);
        column_major_type = retval;
        return retval;
    }
    virtual util::optional<std::size_t> get_matrix_stride(
        ::LLVMTargetDataRef target_data) const override
    {
        return element_type->get_matrix_stride(target_data);
    }
    const std::shared_ptr<Type_descriptor> &get_element_type() const noexcept
    {
        return element_type;
    }
    std::size_t get_element_count() const noexcept
    {
        return element_count;
    }
    using Type_descriptor::add_to_shader_interface;
    virtual void add_to_shader_interface(
        Shader_interface &shader_interface,
        util::optional<Shader_interface_position> &current_position,
        Shader_interface::Interpolation_kind interpolation_kind,
        const Shader_interface_index_list_item *parent_index_list,
        const std::shared_ptr<Type_descriptor> &base_type) override
    {
        if(!current_position)
            throw spirv::Parser_error(
                0, 0, "no Location decoration specified for shader interface");
        if(current_position->get_component_index() != 0)
            throw spirv::Parser_error(0, 0, "Component decoration not allowed on array");
        for(std::size_t i = 0; i < element_count; i++)
        {
            const Shader_interface_index_list_item index_list[1] = {{
                .prev = parent_index_list, .index = i,
            }};
            element_type->add_to_shader_interface(
                shader_interface, current_position, interpolation_kind, index_list, base_type);
        }
    }
};

class Matrix_type_descriptor final : public Type_descriptor
{
    friend class Row_major_matrix_type_descriptor;

private:
    LLVM_type_and_alignment type;
    std::shared_ptr<Vector_type_descriptor> column_type;
    std::size_t column_count;
    std::weak_ptr<Type_descriptor> row_major_type;
    std::shared_ptr<Type_descriptor> make_row_major_type(::LLVMTargetDataRef target_data);

public:
    explicit Matrix_type_descriptor(std::vector<spirv::Decoration_with_parameters> decorations,
                                    std::shared_ptr<Vector_type_descriptor> column_type,
                                    std::size_t column_count) noexcept
        : Type_descriptor(std::move(decorations)),
          type(::LLVMArrayType(column_type->get_or_make_type().type, column_count),
               column_type->get_or_make_type().alignment),
          column_type(std::move(column_type)),
          column_count(column_count),
          row_major_type()
    {
    }
    virtual LLVM_type_and_alignment get_or_make_type() override
    {
        return type;
    }
    virtual void visit(Type_visitor &type_visitor) override
    {
        type_visitor.visit(*this);
    }
    const std::shared_ptr<Vector_type_descriptor> &get_column_type() const noexcept
    {
        return column_type;
    }
    std::size_t get_column_count() const noexcept
    {
        return column_count;
    }
    std::size_t get_row_count() const noexcept
    {
        return column_type->get_element_count();
    }
    const std::shared_ptr<Simple_type_descriptor> &get_element_type() const noexcept
    {
        return column_type->get_element_type();
    }
    virtual std::shared_ptr<Type_descriptor> get_row_major_type(
        ::LLVMTargetDataRef target_data) override
    {
        auto retval = row_major_type.lock();
        if(retval)
            return retval;
        retval = make_row_major_type(target_data);
        row_major_type = retval;
        return retval;
    }
    virtual util::optional<std::size_t> get_matrix_stride(
        ::LLVMTargetDataRef target_data) const override
    {
        return ::LLVMABISizeOfType(target_data, column_type->get_or_make_type().type);
    }
    using Type_descriptor::add_to_shader_interface;
    virtual void add_to_shader_interface(
        Shader_interface &shader_interface,
        util::optional<Shader_interface_position> &current_position,
        Shader_interface::Interpolation_kind interpolation_kind,
        const Shader_interface_index_list_item *parent_index_list,
        const std::shared_ptr<Type_descriptor> &base_type) override
    {
        if(!current_position)
            throw spirv::Parser_error(
                0, 0, "no Location decoration specified for shader interface");
        if(current_position->get_component_index() != 0)
            throw spirv::Parser_error(0, 0, "Component decoration not allowed on matrix");
        for(std::size_t i = 0; i < column_count; i++)
        {
            const Shader_interface_index_list_item index_list[1] = {{
                .prev = parent_index_list, .index = i,
            }};
            column_type->add_to_shader_interface(
                shader_interface, current_position, interpolation_kind, index_list, base_type);
        }
    }
};

class Row_major_matrix_type_descriptor final : public Type_descriptor
{
    friend class Matrix_type_descriptor;

private:
    LLVM_type_and_alignment type;
    std::shared_ptr<Vector_type_descriptor> row_type;
    std::size_t row_count;
    std::shared_ptr<Matrix_type_descriptor> column_major_type;

public:
    explicit Row_major_matrix_type_descriptor(
        std::vector<spirv::Decoration_with_parameters> decorations,
        std::shared_ptr<Vector_type_descriptor> row_type,
        std::size_t row_count) noexcept
        : Type_descriptor(std::move(decorations)),
          type(::LLVMArrayType(row_type->get_or_make_type().type, row_count),
               row_type->get_or_make_type().alignment),
          row_type(std::move(row_type)),
          row_count(row_count),
          column_major_type()
    {
    }
    virtual LLVM_type_and_alignment get_or_make_type() override
    {
        return type;
    }
    virtual void visit(Type_visitor &type_visitor) override
    {
        type_visitor.visit(*this);
    }
    const std::shared_ptr<Vector_type_descriptor> &get_row_type() const noexcept
    {
        return row_type;
    }
    std::size_t get_row_count() const noexcept
    {
        return row_count;
    }
    std::size_t get_column_count() const noexcept
    {
        return row_type->get_element_count();
    }
    const std::shared_ptr<Simple_type_descriptor> &get_element_type() const noexcept
    {
        return row_type->get_element_type();
    }
    virtual std::shared_ptr<Type_descriptor> get_column_major_type(
        ::LLVMTargetDataRef target_data) override
    {
        if(column_major_type)
            return column_major_type;
        auto column_type = std::make_shared<Vector_type_descriptor>(
            std::vector<spirv::Decoration_with_parameters>{},
            row_type->get_element_type(),
            row_count,
            target_data);
        column_major_type = std::make_shared<Matrix_type_descriptor>(
            decorations, std::move(column_type), row_type->get_element_count());
        column_major_type->row_major_type =
            std::static_pointer_cast<Row_major_matrix_type_descriptor>(shared_from_this());
        return column_major_type;
    }
    virtual Load_store_implementation_kind get_load_store_implementation_kind() override
    {
        return Load_store_implementation_kind::Transpose_matrix;
    }
    using Type_descriptor::add_to_shader_interface;
    virtual void add_to_shader_interface(
        Shader_interface &shader_interface,
        util::optional<Shader_interface_position> &current_position,
        Shader_interface::Interpolation_kind interpolation_kind,
        const Shader_interface_index_list_item *parent_index_list,
        const std::shared_ptr<Type_descriptor> &base_type) override
    {
        if(!current_position)
            throw spirv::Parser_error(
                0, 0, "no Location decoration specified for shader interface");
        if(current_position->get_component_index() != 0)
            throw spirv::Parser_error(0, 0, "Component decoration not allowed on matrix");
        for(std::size_t i = 0; i < row_count; i++)
        {
            const Shader_interface_index_list_item index_list[1] = {{
                .prev = parent_index_list, .index = i,
            }};
            row_type->add_to_shader_interface(
                shader_interface, current_position, interpolation_kind, index_list, base_type);
        }
    }
};

inline std::shared_ptr<Type_descriptor> Matrix_type_descriptor::make_row_major_type(
    ::LLVMTargetDataRef target_data)
{
    auto row_type =
        std::make_shared<Vector_type_descriptor>(std::vector<spirv::Decoration_with_parameters>{},
                                                 column_type->get_element_type(),
                                                 column_count,
                                                 target_data);
    auto retval = std::make_shared<Row_major_matrix_type_descriptor>(
        decorations, std::move(row_type), column_type->get_element_count());
    retval->column_major_type =
        std::static_pointer_cast<Matrix_type_descriptor>(shared_from_this());
    return retval;
}

class Pointer_type_descriptor final : public Type_descriptor
{
private:
    std::shared_ptr<Type_descriptor> base;
    std::size_t instruction_start_index;
    LLVM_type_and_alignment type;
    Recursion_checker_state recursion_checker_state;

public:
    Pointer_type_descriptor(std::vector<spirv::Decoration_with_parameters> decorations,
                            std::shared_ptr<Type_descriptor> base,
                            std::size_t instruction_start_index,
                            ::LLVMTargetDataRef target_data) noexcept
        : Type_descriptor(std::move(decorations)),
          base(std::move(base)),
          instruction_start_index(instruction_start_index),
          type(nullptr, llvm_wrapper::Target_data::get_pointer_alignment(target_data))
    {
    }
    const std::shared_ptr<Type_descriptor> &get_base_type() const noexcept
    {
        return base;
    }
    void set_base_type(std::shared_ptr<Type_descriptor> new_base) noexcept
    {
        assert(!base);
        assert(new_base);
        base = std::move(new_base);
    }
    explicit Pointer_type_descriptor(std::vector<spirv::Decoration_with_parameters> decorations,
                                     std::size_t instruction_start_index,
                                     ::LLVMTargetDataRef target_data) noexcept
        : Type_descriptor(std::move(decorations)),
          base(nullptr),
          instruction_start_index(instruction_start_index),
          type(nullptr, llvm_wrapper::Target_data::get_pointer_alignment(target_data))
    {
    }
    virtual LLVM_type_and_alignment get_or_make_type() override
    {
        if(!type.type)
        {
            Recursion_checker recursion_checker(recursion_checker_state, instruction_start_index);
            if(!base)
                throw spirv::Parser_error(
                    instruction_start_index,
                    instruction_start_index,
                    "attempting to create type from pointer forward declaration");
            auto base_type = base->get_or_make_type();
            constexpr unsigned default_address_space = 0;
            type.type = ::LLVMPointerType(base_type.type, default_address_space);
        }
        return type;
    }
    virtual void visit(Type_visitor &type_visitor) override
    {
        type_visitor.visit(*this);
    }
    using Type_descriptor::add_to_shader_interface;
    virtual void add_to_shader_interface(
        Shader_interface &shader_interface,
        util::optional<Shader_interface_position> &current_position,
        Shader_interface::Interpolation_kind interpolation_kind,
        const Shader_interface_index_list_item *parent_index_list,
        const std::shared_ptr<Type_descriptor> &base_type) override
    {
        throw spirv::Parser_error(0, 0, "pointers not allowed shader interface");
    }
};

class Function_type_descriptor final : public Type_descriptor
{
private:
    std::shared_ptr<Type_descriptor> return_type;
    std::vector<std::shared_ptr<Type_descriptor>> args;
    LLVM_type_and_alignment type;
    Recursion_checker_state recursion_checker_state;
    std::size_t instruction_start_index;
    bool valid_for_entry_point;
    bool is_var_arg;

public:
    explicit Function_type_descriptor(std::vector<spirv::Decoration_with_parameters> decorations,
                                      std::shared_ptr<Type_descriptor> return_type,
                                      std::vector<std::shared_ptr<Type_descriptor>> args,
                                      std::size_t instruction_start_index,
                                      ::LLVMTargetDataRef target_data,
                                      bool valid_for_entry_point,
                                      bool is_var_arg) noexcept
        : Type_descriptor(std::move(decorations)),
          return_type(std::move(return_type)),
          args(std::move(args)),
          type(nullptr, llvm_wrapper::Target_data::get_pointer_alignment(target_data)),
          instruction_start_index(instruction_start_index),
          valid_for_entry_point(valid_for_entry_point),
          is_var_arg(is_var_arg)
    {
    }
    virtual LLVM_type_and_alignment get_or_make_type() override
    {
        if(!type.type)
        {
            Recursion_checker recursion_checker(recursion_checker_state, instruction_start_index);
            std::vector<::LLVMTypeRef> llvm_args;
            llvm_args.reserve(args.size());
            auto llvm_return_type = return_type->get_or_make_type();
            for(auto &arg : args)
                llvm_args.push_back(arg->get_or_make_type().type);
            type.type = ::LLVMFunctionType(
                llvm_return_type.type, llvm_args.data(), llvm_args.size(), is_var_arg);
        }
        return type;
    }
    virtual void visit(Type_visitor &type_visitor) override
    {
        type_visitor.visit(*this);
    }
    bool is_valid_for_entry_point() const noexcept
    {
        return valid_for_entry_point;
    }
    using Type_descriptor::add_to_shader_interface;
    virtual void add_to_shader_interface(
        Shader_interface &shader_interface,
        util::optional<Shader_interface_position> &current_position,
        Shader_interface::Interpolation_kind interpolation_kind,
        const Shader_interface_index_list_item *parent_index_list,
        const std::shared_ptr<Type_descriptor> &base_type) override
    {
        throw spirv::Parser_error(0, 0, "function pointers not allowed shader interface");
    }
};

class Struct_type_descriptor final : public Type_descriptor
{
public:
    struct Member
    {
        std::vector<spirv::Decoration_with_parameters> decorations;
        std::size_t llvm_member_index = -1;
        std::shared_ptr<Type_descriptor> type;
        explicit Member(std::vector<spirv::Decoration_with_parameters> decorations,
                        std::shared_ptr<Type_descriptor> type) noexcept
            : decorations(std::move(decorations)),
              type(std::move(type))
        {
        }
        util::optional<spirv::Decoration_with_parameters> find_decoration(
            spirv::Decoration decoration_id) const
        {
            for(auto &decoration : decorations)
                if(decoration.value == decoration_id)
                    return decoration;
            return {};
        }
    };
    enum class Layout_kind
    {
        Default,
        Shader_interface,
    };

private:
    std::vector<Member> members;
    util::Enum_map<spirv::Built_in, std::size_t> builtin_members;
    std::vector<std::size_t> non_built_in_members;
    LLVM_type_and_alignment type;
    bool is_complete;
    Recursion_checker_state recursion_checker_state;
    std::size_t instruction_start_index;
    ::LLVMContextRef context;
    ::LLVMTargetDataRef target_data;
    const Layout_kind layout_kind;
    void complete_type();
    void on_add_member(std::size_t added_member_index) noexcept
    {
        assert(!is_complete);
        auto &member = members[added_member_index];
        bool is_built_in = false;
        for(auto &decoration : member.decorations)
        {
            if(decoration.value == spirv::Decoration::built_in)
            {
                builtin_members[util::get<spirv::Decoration_built_in_parameters>(
                                    decoration.parameters)
                                    .built_in] = added_member_index;
                is_built_in = true;
            }
        }
        if(!is_built_in)
            non_built_in_members.push_back(added_member_index);
    }

public:
    std::size_t add_member(Member member)
    {
        std::size_t index = members.size();
        members.push_back(std::move(member));
        on_add_member(index);
        return index;
    }
    const std::vector<Member> &get_members(bool need_llvm_member_indexes)
    {
        if(need_llvm_member_indexes)
            get_or_make_type();
        return members;
    }
    Layout_kind get_layout_kind() const noexcept
    {
        return layout_kind;
    }
    explicit Struct_type_descriptor(std::vector<spirv::Decoration_with_parameters> decorations,
                                    ::LLVMContextRef context,
                                    ::LLVMTargetDataRef target_data,
                                    const char *name,
                                    std::size_t instruction_start_index,
                                    Layout_kind layout_kind,
                                    std::vector<Member> members = {})
        : Type_descriptor(std::move(decorations)),
          members(std::move(members)),
          builtin_members{},
          type(::LLVMStructCreateNamed(context, name), 0),
          is_complete(false),
          instruction_start_index(instruction_start_index),
          context(context),
          target_data(target_data),
          layout_kind(layout_kind)
    {
        for(std::size_t member_index = 0; member_index < members.size(); member_index++)
            on_add_member(member_index);
    }
    virtual LLVM_type_and_alignment get_or_make_type() override
    {
        if(!is_complete)
        {
            Recursion_checker recursion_checker(recursion_checker_state, instruction_start_index);
            if(!recursion_checker.is_nested_recursion())
                complete_type();
        }
        return type;
    }
    virtual void visit(Type_visitor &type_visitor) override
    {
        type_visitor.visit(*this);
    }
    using Type_descriptor::add_to_shader_interface;
    virtual void add_to_shader_interface(
        Shader_interface &shader_interface,
        util::optional<Shader_interface_position> &current_position,
        Shader_interface::Interpolation_kind interpolation_kind,
        const Shader_interface_index_list_item *parent_index_list,
        const std::shared_ptr<Type_descriptor> &base_type) override
    {
        if(find_decoration(spirv::Decoration::location))
            current_position = Shader_interface_position(decorations);
        if(!current_position)
            throw spirv::Parser_error(
                0, 0, "no Location decoration specified for shader interface");
        if(current_position->get_component_index() != 0)
            throw spirv::Parser_error(0, 0, "Component decoration not allowed on struct");
        for(auto &member : get_members(true))
        {
            if(member.find_decoration(spirv::Decoration::location))
                current_position = Shader_interface_position(member.decorations);
            auto member_interpolation_kind = Shader_interface::Interpolation_kind::Perspective;
            if(member.find_decoration(spirv::Decoration::flat))
                member_interpolation_kind = Shader_interface::Interpolation_kind::Flat;
            else if(member.find_decoration(spirv::Decoration::no_perspective))
                member_interpolation_kind = Shader_interface::Interpolation_kind::Linear;
            const Shader_interface_index_list_item index_list[1] = {{
                .prev = parent_index_list, .index = member.llvm_member_index,
            }};
            member.type->add_to_shader_interface(shader_interface,
                                                 current_position,
                                                 member_interpolation_kind,
                                                 index_list,
                                                 base_type);
        }
    }
};

class Constant_descriptor
{
    Constant_descriptor(const Constant_descriptor &) = delete;
    Constant_descriptor &operator=(const Constant_descriptor &) = delete;

public:
    const std::shared_ptr<Type_descriptor> type;

public:
    explicit Constant_descriptor(std::shared_ptr<Type_descriptor> type) noexcept
        : type(std::move(type))
    {
    }
    ~Constant_descriptor() = default;
    virtual ::LLVMValueRef get_or_make_value() = 0;
};

class Simple_constant_descriptor final : public Constant_descriptor
{
private:
    ::LLVMValueRef value;

public:
    explicit Simple_constant_descriptor(std::shared_ptr<Type_descriptor> type,
                                        ::LLVMValueRef value) noexcept
        : Constant_descriptor(std::move(type)),
          value(value)
    {
    }
    virtual ::LLVMValueRef get_or_make_value() override
    {
        return value;
    }
};

struct Converted_module
{
    llvm_wrapper::Module module;
    std::string entry_function_name;
    std::shared_ptr<Struct_type_descriptor> inputs_struct;
    std::shared_ptr<Struct_type_descriptor> built_in_inputs_struct;
    std::shared_ptr<Struct_type_descriptor> outputs_struct;
    std::shared_ptr<Struct_type_descriptor> built_in_outputs_struct;
    spirv::Execution_model execution_model;
    std::unique_ptr<Shader_interface> output_shader_interface;
    std::unique_ptr<Shader_interface> built_in_output_shader_interface;
    std::shared_ptr<Struct_type_descriptor> combined_outputs_struct;
    static std::shared_ptr<Struct_type_descriptor> make_combined_outputs_struct(
        ::LLVMContextRef context,
        ::LLVMTargetDataRef target_data,
        const char *name,
        const std::shared_ptr<Struct_type_descriptor> &outputs_struct,
        const std::shared_ptr<Struct_type_descriptor> &built_in_outputs_struct)
    {
        return std::make_shared<Struct_type_descriptor>(
            std::vector<spirv::Decoration_with_parameters>{},
            context,
            target_data,
            name,
            0,
            Struct_type_descriptor::Layout_kind::Default,
            std::vector<Struct_type_descriptor::Member>{
                Struct_type_descriptor::Member({}, built_in_outputs_struct),
                Struct_type_descriptor::Member({}, outputs_struct),
            });
    }
    Converted_module() = default;
    explicit Converted_module(
        llvm_wrapper::Module module,
        std::string entry_function_name,
        std::shared_ptr<Struct_type_descriptor> inputs_struct,
        std::shared_ptr<Struct_type_descriptor> built_in_inputs_struct,
        std::shared_ptr<Struct_type_descriptor> outputs_struct,
        std::shared_ptr<Struct_type_descriptor> built_in_outputs_struct,
        spirv::Execution_model execution_model,
        std::unique_ptr<Shader_interface> output_shader_interface,
        std::unique_ptr<Shader_interface> built_in_output_shader_interface,
        std::shared_ptr<Struct_type_descriptor> combined_outputs_struct) noexcept
        : module(std::move(module)),
          entry_function_name(std::move(entry_function_name)),
          inputs_struct(std::move(inputs_struct)),
          built_in_inputs_struct(std::move(built_in_inputs_struct)),
          outputs_struct(std::move(outputs_struct)),
          built_in_outputs_struct(std::move(built_in_outputs_struct)),
          execution_model(execution_model),
          output_shader_interface(std::move(output_shader_interface)),
          built_in_output_shader_interface(std::move(built_in_output_shader_interface)),
          combined_outputs_struct(std::move(combined_outputs_struct))
    {
    }
};

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

class Spirv_to_llvm;

Converted_module spirv_to_llvm(
    ::LLVMContextRef context,
    ::LLVMTargetMachineRef target_machine,
    const spirv::Word *shader_words,
    std::size_t shader_size,
    std::uint64_t shader_id,
    spirv::Execution_model execution_model,
    util::string_view entry_point_name,
    const VkPipelineVertexInputStateCreateInfo *vertex_input_state,
    pipeline::Instantiated_pipeline_layout &pipeline_layout,
    const Shader_interface *previous_stage_output_shader_interface,
    const Shader_interface *previous_stage_built_in_output_shader_interface);
}
}

#endif /* SPIRV_TO_LLVM_SPIRV_TO_LLVM_H_ */
