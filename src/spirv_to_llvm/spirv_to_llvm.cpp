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
#include "spirv_to_llvm_implementation.h"

namespace kazan
{
namespace spirv_to_llvm
{
using namespace spirv;

namespace
{
constexpr bool is_power_of_2(std::uint64_t v) noexcept
{
    return (v & (v - 1)) == 0 && v != 0;
}

constexpr std::size_t get_padding_size(std::size_t current_position,
                                       std::size_t needed_alignment) noexcept
{
    assert(is_power_of_2(needed_alignment));
    return -current_position & (needed_alignment - 1);
}
}

void Struct_type_descriptor::complete_type()
{
    for(auto &decoration : decorations)
    {
        switch(decoration.value)
        {
        case Decoration::relaxed_precision:
#warning finish implementing Decoration::relaxed_precision
            break;
        case Decoration::spec_id:
#warning finish implementing Decoration::spec_id
            break;
        case Decoration::block:
#warning finish implementing Decoration::block
            continue;
        case Decoration::buffer_block:
#warning finish implementing Decoration::buffer_block
            break;
        case Decoration::row_major:
#warning finish implementing Decoration::row_major
            break;
        case Decoration::col_major:
#warning finish implementing Decoration::col_major
            break;
        case Decoration::array_stride:
#warning finish implementing Decoration::array_stride
            break;
        case Decoration::matrix_stride:
#warning finish implementing Decoration::matrix_stride
            break;
        case Decoration::glsl_shared:
#warning finish implementing Decoration::glsl_shared
            break;
        case Decoration::glsl_packed:
#warning finish implementing Decoration::glsl_packed
            break;
        case Decoration::c_packed:
#warning finish implementing Decoration::c_packed
            break;
        case Decoration::built_in:
#warning finish implementing Decoration::built_in
            break;
        case Decoration::no_perspective:
#warning finish implementing Decoration::no_perspective
            break;
        case Decoration::flat:
#warning finish implementing Decoration::flat
            break;
        case Decoration::patch:
#warning finish implementing Decoration::patch
            break;
        case Decoration::centroid:
#warning finish implementing Decoration::centroid
            break;
        case Decoration::sample:
#warning finish implementing Decoration::sample
            break;
        case Decoration::invariant:
#warning finish implementing Decoration::invariant
            break;
        case Decoration::restrict:
#warning finish implementing Decoration::restrict
            break;
        case Decoration::aliased:
#warning finish implementing Decoration::aliased
            break;
        case Decoration::volatile_:
#warning finish implementing Decoration::volatile_
            break;
        case Decoration::constant:
#warning finish implementing Decoration::constant
            break;
        case Decoration::coherent:
#warning finish implementing Decoration::coherent
            break;
        case Decoration::non_writable:
#warning finish implementing Decoration::non_writable
            break;
        case Decoration::non_readable:
#warning finish implementing Decoration::non_readable
            break;
        case Decoration::uniform:
#warning finish implementing Decoration::uniform
            break;
        case Decoration::saturated_conversion:
#warning finish implementing Decoration::saturated_conversion
            break;
        case Decoration::stream:
#warning finish implementing Decoration::stream
            break;
        case Decoration::location:
#warning finish implementing Decoration::location
            break;
        case Decoration::component:
#warning finish implementing Decoration::component
            break;
        case Decoration::index:
#warning finish implementing Decoration::index
            break;
        case Decoration::binding:
#warning finish implementing Decoration::binding
            break;
        case Decoration::descriptor_set:
#warning finish implementing Decoration::descriptor_set
            break;
        case Decoration::offset:
#warning finish implementing Decoration::offset
            break;
        case Decoration::xfb_buffer:
#warning finish implementing Decoration::xfb_buffer
            break;
        case Decoration::xfb_stride:
#warning finish implementing Decoration::xfb_stride
            break;
        case Decoration::func_param_attr:
#warning finish implementing Decoration::func_param_attr
            break;
        case Decoration::fp_rounding_mode:
#warning finish implementing Decoration::fp_rounding_mode
            break;
        case Decoration::fp_fast_math_mode:
#warning finish implementing Decoration::fp_fast_math_mode
            break;
        case Decoration::linkage_attributes:
#warning finish implementing Decoration::linkage_attributes
            break;
        case Decoration::no_contraction:
#warning finish implementing Decoration::no_contraction
            break;
        case Decoration::input_attachment_index:
#warning finish implementing Decoration::input_attachment_index
            break;
        case Decoration::alignment:
#warning finish implementing Decoration::alignment
            break;
        case Decoration::max_byte_offset:
#warning finish implementing Decoration::max_byte_offset
            break;
        case Decoration::alignment_id:
#warning finish implementing Decoration::alignment_id
            break;
        case Decoration::max_byte_offset_id:
#warning finish implementing Decoration::max_byte_offset_id
            break;
        case Decoration::override_coverage_nv:
#warning finish implementing Decoration::override_coverage_nv
            break;
        case Decoration::passthrough_nv:
#warning finish implementing Decoration::passthrough_nv
            break;
        case Decoration::viewport_relative_nv:
#warning finish implementing Decoration::viewport_relative_nv
            break;
        case Decoration::secondary_viewport_relative_nv:
#warning finish implementing Decoration::secondary_viewport_relative_nv
            break;
        }
        throw Parser_error(instruction_start_index,
                           instruction_start_index,
                           "unimplemented decoration on OpTypeStruct: "
                               + std::string(get_enumerant_name(decoration.value)));
    }
    struct Member_descriptor
    {
        std::size_t alignment;
        std::size_t size;
        ::LLVMTypeRef type;
        explicit Member_descriptor(std::size_t alignment,
                                   std::size_t size,
                                   ::LLVMTypeRef type) noexcept : alignment(alignment),
                                                                  size(size),
                                                                  type(type)
        {
        }
    };
    std::vector<Member_descriptor> member_descriptors;
    member_descriptors.reserve(members.size());
    std::size_t total_alignment = 1;
    for(auto &member : members)
    {
        for(auto &decoration : member.decorations)
        {
            switch(decoration.value)
            {
            case Decoration::relaxed_precision:
#warning finish implementing Decoration::relaxed_precision
                break;
            case Decoration::spec_id:
#warning finish implementing Decoration::spec_id
                break;
            case Decoration::block:
#warning finish implementing Decoration::block
                break;
            case Decoration::buffer_block:
#warning finish implementing Decoration::buffer_block
                break;
            case Decoration::row_major:
#warning finish implementing Decoration::row_major
                break;
            case Decoration::col_major:
#warning finish implementing Decoration::col_major
                break;
            case Decoration::array_stride:
#warning finish implementing Decoration::array_stride
                break;
            case Decoration::matrix_stride:
#warning finish implementing Decoration::matrix_stride
                break;
            case Decoration::glsl_shared:
#warning finish implementing Decoration::glsl_shared
                break;
            case Decoration::glsl_packed:
#warning finish implementing Decoration::glsl_packed
                break;
            case Decoration::c_packed:
#warning finish implementing Decoration::c_packed
                break;
            case Decoration::built_in:
#warning finish implementing Decoration::built_in
                continue;
            case Decoration::no_perspective:
#warning finish implementing Decoration::no_perspective
                break;
            case Decoration::flat:
#warning finish implementing Decoration::flat
                break;
            case Decoration::patch:
#warning finish implementing Decoration::patch
                break;
            case Decoration::centroid:
#warning finish implementing Decoration::centroid
                break;
            case Decoration::sample:
#warning finish implementing Decoration::sample
                break;
            case Decoration::invariant:
#warning finish implementing Decoration::invariant
                break;
            case Decoration::restrict:
#warning finish implementing Decoration::restrict
                break;
            case Decoration::aliased:
#warning finish implementing Decoration::aliased
                break;
            case Decoration::volatile_:
#warning finish implementing Decoration::volatile_
                break;
            case Decoration::constant:
#warning finish implementing Decoration::constant
                break;
            case Decoration::coherent:
#warning finish implementing Decoration::coherent
                break;
            case Decoration::non_writable:
#warning finish implementing Decoration::non_writable
                break;
            case Decoration::non_readable:
#warning finish implementing Decoration::non_readable
                break;
            case Decoration::uniform:
#warning finish implementing Decoration::uniform
                break;
            case Decoration::saturated_conversion:
#warning finish implementing Decoration::saturated_conversion
                break;
            case Decoration::stream:
#warning finish implementing Decoration::stream
                break;
            case Decoration::location:
                continue;
            case Decoration::component:
#warning finish implementing Decoration::component
                break;
            case Decoration::index:
#warning finish implementing Decoration::index
                break;
            case Decoration::binding:
#warning finish implementing Decoration::binding
                break;
            case Decoration::descriptor_set:
#warning finish implementing Decoration::descriptor_set
                break;
            case Decoration::offset:
#warning finish implementing Decoration::offset
                break;
            case Decoration::xfb_buffer:
#warning finish implementing Decoration::xfb_buffer
                break;
            case Decoration::xfb_stride:
#warning finish implementing Decoration::xfb_stride
                break;
            case Decoration::func_param_attr:
#warning finish implementing Decoration::func_param_attr
                break;
            case Decoration::fp_rounding_mode:
#warning finish implementing Decoration::fp_rounding_mode
                break;
            case Decoration::fp_fast_math_mode:
#warning finish implementing Decoration::fp_fast_math_mode
                break;
            case Decoration::linkage_attributes:
#warning finish implementing Decoration::linkage_attributes
                break;
            case Decoration::no_contraction:
#warning finish implementing Decoration::no_contraction
                break;
            case Decoration::input_attachment_index:
#warning finish implementing Decoration::input_attachment_index
                break;
            case Decoration::alignment:
#warning finish implementing Decoration::alignment
                break;
            case Decoration::max_byte_offset:
#warning finish implementing Decoration::max_byte_offset
                break;
            case Decoration::alignment_id:
#warning finish implementing Decoration::alignment_id
                break;
            case Decoration::max_byte_offset_id:
#warning finish implementing Decoration::max_byte_offset_id
                break;
            case Decoration::override_coverage_nv:
#warning finish implementing Decoration::override_coverage_nv
                break;
            case Decoration::passthrough_nv:
#warning finish implementing Decoration::passthrough_nv
                break;
            case Decoration::viewport_relative_nv:
#warning finish implementing Decoration::viewport_relative_nv
                break;
            case Decoration::secondary_viewport_relative_nv:
#warning finish implementing Decoration::secondary_viewport_relative_nv
                break;
            }
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "unimplemented member decoration on OpTypeStruct: "
                                   + std::string(get_enumerant_name(decoration.value)));
        }
        auto member_type = member.type->get_or_make_type();
        std::size_t size = ::LLVMABISizeOfType(target_data, member_type.type);
        struct Member_type_visitor : public Type_descriptor::Type_visitor
        {
            LLVM_type_and_alignment &member_type;
            std::size_t &size;
            Struct_type_descriptor *this_;
            virtual void visit(Simple_type_descriptor &type) override
            {
#warning finish implementing member type
            }
            virtual void visit(Vector_type_descriptor &type) override
            {
#warning finish implementing member type
            }
            virtual void visit(Matrix_type_descriptor &type) override
            {
#warning finish implementing member type
                throw Parser_error(this_->instruction_start_index,
                                   this_->instruction_start_index,
                                   "unimplemented member type");
            }
            virtual void visit(Row_major_matrix_type_descriptor &type) override
            {
#warning finish implementing member type
                throw Parser_error(this_->instruction_start_index,
                                   this_->instruction_start_index,
                                   "unimplemented member type");
            }
            virtual void visit(Array_type_descriptor &type) override
            {
#warning finish implementing member type
            }
            virtual void visit(Pointer_type_descriptor &type) override
            {
#warning finish implementing member type
            }
            virtual void visit(Function_type_descriptor &type) override
            {
#warning finish implementing member type
            }
            virtual void visit(Struct_type_descriptor &type) override
            {
#warning finish implementing member type
                if(::LLVMIsOpaqueStruct(member_type.type))
                    throw Parser_error(this_->instruction_start_index,
                                       this_->instruction_start_index,
                                       "recursive struct has infinite size");
            }
            explicit Member_type_visitor(LLVM_type_and_alignment &member_type,
                                         std::size_t &size,
                                         Struct_type_descriptor *this_) noexcept
                : member_type(member_type),
                  size(size),
                  this_(this_)
            {
            }
        };
        member.type->visit(Member_type_visitor(member_type, size, this));
        if(::LLVMGetTypeKind(member_type.type) == ::LLVMStructTypeKind
           && ::LLVMIsOpaqueStruct(member_type.type))
        {
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "struct can't have opaque struct members");
        }
        assert(is_power_of_2(member_type.alignment));
        if(member_type.alignment > total_alignment)
            total_alignment = member_type.alignment;
        member_descriptors.push_back(
            Member_descriptor(member_type.alignment, size, member_type.type));
    }
    assert(member_descriptors.size() == members.size());
    assert(is_power_of_2(total_alignment));
    std::size_t current_offset = 0;
    std::vector<::LLVMTypeRef> member_types;
    member_types.reserve(members.size() * 2);
    if(!members.empty())
    {
        for(std::size_t member_index = 0; member_index < members.size(); member_index++)
        {
            members[member_index].llvm_member_index = member_types.size();
#warning finish Struct_type_descriptor::complete_type
            member_types.push_back(member_descriptors[member_index].type);
            current_offset += member_descriptors[member_index].size;
            std::size_t next_alignment = member_index + 1 < members.size() ?
                                             member_descriptors[member_index + 1].alignment :
                                             total_alignment;
            auto padding_size = get_padding_size(current_offset, next_alignment);
            if(padding_size != 0)
            {
                member_types.push_back(
                    ::LLVMArrayType(::LLVMInt8TypeInContext(context), padding_size));
                current_offset += padding_size;
            }
        }
    }
    else
    {
        member_types.push_back(::LLVMInt8TypeInContext(context)); // so it isn't empty
    }
    constexpr bool is_packed = true;
    ::LLVMStructSetBody(type.type, member_types.data(), member_types.size(), is_packed);
    type.alignment = total_alignment;
    is_complete = true;
}

void Spirv_to_llvm::handle_header(unsigned version_number_major,
                                  unsigned version_number_minor,
                                  Word generator_magic_number,
                                  Word id_bound,
                                  [[gnu::unused]] Word instruction_schema)
{
    if(stage == util::Enum_traits<Stage>::values[0])
    {
        input_version_number_major = version_number_major;
        input_version_number_minor = version_number_minor;
        input_generator_magic_number = generator_magic_number;
        id_states.resize(id_bound - 1);
    }
}

namespace
{
#define DECLARE_LIBRARY_SYMBOL(name) \
    static void library_symbol_##name() __attribute__((weakref(#name)));
#ifdef __ARM_EABI__
DECLARE_LIBRARY_SYMBOL(__aeabi_unwind_cpp_pr0)
#endif
}

Jit_symbol_resolver::Resolved_symbol Jit_symbol_resolver::resolve(util::string_view name)
{
#define RESOLVE_LIBRARY_SYMBOL(symbol_name)                                            \
    if(#symbol_name == name)                                                           \
    {                                                                                  \
        if(library_symbol_##symbol_name == nullptr)                                    \
            throw std::runtime_error(                                                  \
                "JIT symbol resolve error: library symbol not linked: " #symbol_name); \
        return library_symbol_##symbol_name;                                           \
    }

#ifdef __ARM_EABI__
    RESOLVE_LIBRARY_SYMBOL(__aeabi_unwind_cpp_pr0)
#endif
#warning finish implementing
    return nullptr;
}
}

spirv_to_llvm::Converted_module spirv_to_llvm::spirv_to_llvm(
    ::LLVMContextRef context,
    ::LLVMTargetMachineRef target_machine,
    const spirv::Word *shader_words,
    std::size_t shader_size,
    std::uint64_t shader_id,
    spirv::Execution_model execution_model,
    util::string_view entry_point_name,
    const VkPipelineVertexInputStateCreateInfo *vertex_input_state)
{
    return Spirv_to_llvm(context,
                         target_machine,
                         shader_id,
                         execution_model,
                         entry_point_name,
                         vertex_input_state)
        .run(shader_words, shader_size);
}
}
