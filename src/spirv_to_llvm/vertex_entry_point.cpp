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

namespace vulkan_cpu
{
namespace spirv_to_llvm
{
using namespace spirv;

::LLVMValueRef Spirv_to_llvm::generate_vertex_entry_function(Op_entry_point_state &entry_point,
                                                             ::LLVMValueRef main_function)
{
    typedef std::uint32_t Vertex_index_type;
    auto llvm_vertex_index_type = llvm_wrapper::Create_llvm_type<Vertex_index_type>()(context);
    typedef void (*Vertex_shader_function)(Vertex_index_type vertex_start_index,
                                           Vertex_index_type vertex_end_index,
                                           std::uint32_t instance_id,
                                           void *output_buffer);
    constexpr std::size_t arg_vertex_start_index = 0;
    constexpr std::size_t arg_vertex_end_index = 1;
    constexpr std::size_t arg_instance_id = 2;
    constexpr std::size_t arg_output_buffer = 3;
    static_assert(std::is_same<Vertex_shader_function,
                               pipeline::Graphics_pipeline::Vertex_shader_function>::value,
                  "vertex shader function signature mismatch");
    auto function_type = llvm_wrapper::Create_llvm_type<Vertex_shader_function>()(context);
    auto entry_function = ::LLVMAddFunction(
        module.get(), get_prefixed_name("vertex_entry_point", true).c_str(), function_type);
    llvm_wrapper::Module::set_function_target_machine(entry_function, target_machine);
    ::LLVMSetValueName(::LLVMGetParam(entry_function, arg_vertex_start_index),
                       "vertex_start_index");
    ::LLVMSetValueName(::LLVMGetParam(entry_function, arg_vertex_end_index), "vertex_end_index");
    ::LLVMSetValueName(::LLVMGetParam(entry_function, arg_instance_id), "instance_id");
    ::LLVMSetValueName(::LLVMGetParam(entry_function, arg_output_buffer), "output_buffer_");
    auto entry_block = ::LLVMAppendBasicBlockInContext(context, entry_function, "entry");
    auto loop_block = ::LLVMAppendBasicBlockInContext(context, entry_function, "loop");
    auto exit_block = ::LLVMAppendBasicBlockInContext(context, entry_function, "exit");
    ::LLVMPositionBuilderAtEnd(builder.get(), entry_block);
    auto io_struct_type = io_struct->get_or_make_type();
    auto io_struct_pointer = ::LLVMBuildAlloca(builder.get(), io_struct_type.type, "io_struct");
    auto inputs_struct_pointer =
        ::LLVMBuildAlloca(builder.get(), inputs_struct->get_or_make_type().type, "inputs");
    ::LLVMSetAlignment(
        ::LLVMBuildStore(builder.get(), ::LLVMConstNull(io_struct_type.type), io_struct_pointer),
        io_struct_type.alignment);
    auto inputs_pointer =
        ::LLVMBuildStructGEP(builder.get(),
                             io_struct_pointer,
                             io_struct->get_members(true)[inputs_member].llvm_member_index,
                             "inputs_pointer");
    ::LLVMBuildStore(builder.get(), inputs_struct_pointer, inputs_pointer);
    auto start_output_buffer =
        ::LLVMBuildBitCast(builder.get(),
                           ::LLVMGetParam(entry_function, arg_output_buffer),
                           outputs_struct_pointer_type->get_or_make_type().type,
                           "start_output_buffer");
    auto start_loop_condition =
        ::LLVMBuildICmp(builder.get(),
                        ::LLVMIntULT,
                        ::LLVMGetParam(entry_function, arg_vertex_start_index),
                        ::LLVMGetParam(entry_function, arg_vertex_end_index),
                        "start_loop_condition");
    ::LLVMBuildCondBr(builder.get(), start_loop_condition, loop_block, exit_block);
    ::LLVMPositionBuilderAtEnd(builder.get(), loop_block);
    auto vertex_index = ::LLVMBuildPhi(builder.get(),
                                       llvm_wrapper::Create_llvm_type<Vertex_index_type>()(context),
                                       "vertex_index");
    auto output_buffer = ::LLVMBuildPhi(
        builder.get(), outputs_struct_pointer_type->get_or_make_type().type, "output_buffer");
    auto next_vertex_index = ::LLVMBuildNUWAdd(builder.get(),
                                               vertex_index,
                                               ::LLVMConstInt(llvm_vertex_index_type, 1, false),
                                               "next_vertex_index");
    constexpr std::size_t vertex_index_incoming_count = 2;
    ::LLVMValueRef vertex_index_incoming_values[vertex_index_incoming_count] = {
        next_vertex_index, ::LLVMGetParam(entry_function, arg_vertex_start_index),
    };
    ::LLVMBasicBlockRef vertex_index_incoming_blocks[vertex_index_incoming_count] = {
        loop_block, entry_block,
    };
    ::LLVMAddIncoming(vertex_index,
                      vertex_index_incoming_values,
                      vertex_index_incoming_blocks,
                      vertex_index_incoming_count);
    ::LLVMValueRef next_output_buffer;
    {
        constexpr std::size_t index_count = 1;
        ::LLVMValueRef indexes[index_count] = {
            ::LLVMConstInt(llvm_wrapper::Create_llvm_type<std::ptrdiff_t>()(context), 1, true)};
        next_output_buffer = ::LLVMBuildGEP(
            builder.get(), output_buffer, indexes, index_count, "next_output_buffer");
    }
    constexpr std::size_t output_buffer_incoming_count = 2;
    ::LLVMValueRef output_buffer_incoming_values[output_buffer_incoming_count] = {
        next_output_buffer, start_output_buffer,
    };
    ::LLVMBasicBlockRef output_buffer_incoming_blocks[output_buffer_incoming_count] = {
        loop_block, entry_block,
    };
    ::LLVMAddIncoming(output_buffer,
                      output_buffer_incoming_values,
                      output_buffer_incoming_blocks,
                      output_buffer_incoming_count);
    auto &&members = io_struct->get_members(true);
    for(std::size_t member_index = 0; member_index < members.size(); member_index++)
    {
        auto &member = members[member_index];
        if(member_index == inputs_member)
        {
            for(auto &input_member : inputs_struct->get_members(true))
            {
                auto input_pointer = ::LLVMBuildStructGEP(
                    builder.get(), inputs_struct_pointer, input_member.llvm_member_index, "input");
                ::LLVMDumpType(::LLVMTypeOf(input_pointer));
                util::optional<Built_in> built_in;
                static_cast<void>(input_pointer);
                for(auto &decoration : input_member.decorations)
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
                        if(built_in)
                            throw Parser_error(
                                0, 0, "multiple BuiltIn decorations on the same variable");
                        built_in =
                            util::get<Decoration_built_in_parameters>(decoration.parameters)
                                .built_in;
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
                    throw Parser_error(
                        0,
                        0,
                        "unimplemented member decoration on shader input variable: "
                            + std::string(get_enumerant_name(decoration.value)));
                }
                if(!built_in)
                    throw Parser_error(
                        0, 0, "non-built-in shader input variables are not implemented");
                do
                {
                    switch(*built_in)
                    {
                    case Built_in::position:
#warning finish implementing Built_in::position
                        break;
                    case Built_in::point_size:
#warning finish implementing Built_in::point_size
                        break;
                    case Built_in::clip_distance:
#warning finish implementing Built_in::clip_distance
                        break;
                    case Built_in::cull_distance:
#warning finish implementing Built_in::cull_distance
                        break;
                    case Built_in::vertex_id:
#warning finish implementing Built_in::vertex_id
                        break;
                    case Built_in::instance_id:
#warning finish implementing Built_in::instance_id
                        break;
                    case Built_in::primitive_id:
#warning finish implementing Built_in::primitive_id
                        break;
                    case Built_in::invocation_id:
#warning finish implementing Built_in::invocation_id
                        break;
                    case Built_in::layer:
#warning finish implementing Built_in::layer
                        break;
                    case Built_in::viewport_index:
#warning finish implementing Built_in::viewport_index
                        break;
                    case Built_in::tess_level_outer:
#warning finish implementing Built_in::tess_level_outer
                        break;
                    case Built_in::tess_level_inner:
#warning finish implementing Built_in::tess_level_inner
                        break;
                    case Built_in::tess_coord:
#warning finish implementing Built_in::tess_coord
                        break;
                    case Built_in::patch_vertices:
#warning finish implementing Built_in::patch_vertices
                        break;
                    case Built_in::frag_coord:
#warning finish implementing Built_in::frag_coord
                        break;
                    case Built_in::point_coord:
#warning finish implementing Built_in::point_coord
                        break;
                    case Built_in::front_facing:
#warning finish implementing Built_in::front_facing
                        break;
                    case Built_in::sample_id:
#warning finish implementing Built_in::sample_id
                        break;
                    case Built_in::sample_position:
#warning finish implementing Built_in::sample_position
                        break;
                    case Built_in::sample_mask:
#warning finish implementing Built_in::sample_mask
                        break;
                    case Built_in::frag_depth:
#warning finish implementing Built_in::frag_depth
                        break;
                    case Built_in::helper_invocation:
#warning finish implementing Built_in::helper_invocation
                        break;
                    case Built_in::num_workgroups:
#warning finish implementing Built_in::num_workgroups
                        break;
                    case Built_in::workgroup_size:
#warning finish implementing Built_in::workgroup_size
                        break;
                    case Built_in::workgroup_id:
#warning finish implementing Built_in::workgroup_id
                        break;
                    case Built_in::local_invocation_id:
#warning finish implementing Built_in::local_invocation_id
                        break;
                    case Built_in::global_invocation_id:
#warning finish implementing Built_in::global_invocation_id
                        break;
                    case Built_in::local_invocation_index:
#warning finish implementing Built_in::local_invocation_index
                        break;
                    case Built_in::work_dim:
#warning finish implementing Built_in::work_dim
                        break;
                    case Built_in::global_size:
#warning finish implementing Built_in::global_size
                        break;
                    case Built_in::enqueued_workgroup_size:
#warning finish implementing Built_in::enqueued_workgroup_size
                        break;
                    case Built_in::global_offset:
#warning finish implementing Built_in::global_offset
                        break;
                    case Built_in::global_linear_id:
#warning finish implementing Built_in::global_linear_id
                        break;
                    case Built_in::subgroup_size:
#warning finish implementing Built_in::subgroup_size
                        break;
                    case Built_in::subgroup_max_size:
#warning finish implementing Built_in::subgroup_max_size
                        break;
                    case Built_in::num_subgroups:
#warning finish implementing Built_in::num_subgroups
                        break;
                    case Built_in::num_enqueued_subgroups:
#warning finish implementing Built_in::num_enqueued_subgroups
                        break;
                    case Built_in::subgroup_id:
#warning finish implementing Built_in::subgroup_id
                        break;
                    case Built_in::subgroup_local_invocation_id:
#warning finish implementing Built_in::subgroup_local_invocation_id
                        break;
                    case Built_in::vertex_index:
                    {
                        if(::LLVMGetElementType(::LLVMTypeOf(input_pointer))
                           != llvm_vertex_index_type)
                            throw Parser_error(
                                0, 0, "invalid type for vertex index built-in variable");
                        ::LLVMBuildStore(builder.get(), vertex_index, input_pointer);
                        continue;
                    }
                    case Built_in::instance_index:
#warning finish implementing Built_in::instance_index
                        break;
                    case Built_in::subgroup_eq_mask_khr:
#warning finish implementing Built_in::subgroup_eq_mask_khr
                        break;
                    case Built_in::subgroup_ge_mask_khr:
#warning finish implementing Built_in::subgroup_ge_mask_khr
                        break;
                    case Built_in::subgroup_gt_mask_khr:
#warning finish implementing Built_in::subgroup_gt_mask_khr
                        break;
                    case Built_in::subgroup_le_mask_khr:
#warning finish implementing Built_in::subgroup_le_mask_khr
                        break;
                    case Built_in::subgroup_lt_mask_khr:
#warning finish implementing Built_in::subgroup_lt_mask_khr
                        break;
                    case Built_in::base_vertex:
#warning finish implementing Built_in::base_vertex
                        break;
                    case Built_in::base_instance:
#warning finish implementing Built_in::base_instance
                        break;
                    case Built_in::draw_index:
#warning finish implementing Built_in::draw_index
                        break;
                    case Built_in::device_index:
#warning finish implementing Built_in::device_index
                        break;
                    case Built_in::view_index:
#warning finish implementing Built_in::view_index
                        break;
                    case Built_in::viewport_mask_nv:
#warning finish implementing Built_in::viewport_mask_nv
                        break;
                    case Built_in::secondary_position_nv:
#warning finish implementing Built_in::secondary_position_nv
                        break;
                    case Built_in::secondary_viewport_mask_nv:
#warning finish implementing Built_in::secondary_viewport_mask_nv
                        break;
                    case Built_in::position_per_view_nv:
#warning finish implementing Built_in::position_per_view_nv
                        break;
                    case Built_in::viewport_mask_per_view_nv:
#warning finish implementing Built_in::viewport_mask_per_view_nv
                        break;
                    }
                    throw Parser_error(0,
                                              0,
                                              "unimplemented built in shader input variable: "
                                                  + std::string(get_enumerant_name(*built_in)));
                } while(false);
            }
        }
        else if(member_index == outputs_member)
        {
            auto outputs_struct_pointer = output_buffer;
            ::LLVMBuildStore(
                builder.get(),
                outputs_struct_pointer,
                ::LLVMBuildStructGEP(
                    builder.get(), io_struct_pointer, member.llvm_member_index, "outputs_pointer"));
            for(auto &output_member : outputs_struct->get_members(true))
            {
                auto output_pointer = ::LLVMBuildStructGEP(builder.get(),
                                                           outputs_struct_pointer,
                                                           output_member.llvm_member_index,
                                                           "output");
                static_cast<void>(output_pointer);
                for(auto &decoration : output_member.decorations)
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
                    throw Parser_error(
                        0,
                        0,
                        "unimplemented member decoration on shader output variable: "
                            + std::string(get_enumerant_name(decoration.value)));
                }
            }
        }
        else
        {
            throw Parser_error(0, 0, "internal error: unhandled Io_struct member");
        }
    }
    {
        constexpr std::size_t arg_count = 1;
        assert(implicit_function_arguments.size() == arg_count);
        assert(implicit_function_arguments[0]->get_or_make_type().type
               == ::LLVMTypeOf(io_struct_pointer));
        ::LLVMValueRef args[arg_count] = {
            io_struct_pointer,
        };
        assert(::LLVMGetReturnType(::LLVMGetElementType(::LLVMTypeOf(main_function)))
               == llvm_wrapper::Create_llvm_type<void>()(context));
        ::LLVMBuildCall(builder.get(), main_function, args, arg_count, "");
    }
#warning add output copy
    auto next_iteration_condition =
        ::LLVMBuildICmp(builder.get(),
                        ::LLVMIntULT,
                        next_vertex_index,
                        ::LLVMGetParam(entry_function, arg_vertex_end_index),
                        "next_iteration_condition");
    ::LLVMBuildCondBr(builder.get(), next_iteration_condition, loop_block, exit_block);
    ::LLVMPositionBuilderAtEnd(builder.get(), exit_block);
    static_assert(std::is_same<decltype(std::declval<Vertex_shader_function>()(0, 0, 0, nullptr)),
                               void>::value,
                  "");
    ::LLVMBuildRetVoid(builder.get());
    return entry_function;
}
}
}
