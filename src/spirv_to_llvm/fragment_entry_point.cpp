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

::LLVMValueRef Spirv_to_llvm::generate_fragment_entry_function(Op_entry_point_state &entry_point,
                                                               ::LLVMValueRef main_function)
{
    typedef std::uint32_t Pixel_type;
    auto llvm_pixel_type = llvm_wrapper::Create_llvm_type<Pixel_type>()(context);
    auto llvm_float_type = llvm_wrapper::Create_llvm_type<float>()(context);
    auto llvm_u8_type = llvm_wrapper::Create_llvm_type<std::uint8_t>()(context);
    auto llvm_vec4_type = ::LLVMVectorType(llvm_float_type, 4);
    auto llvm_u8vec4_type = ::LLVMVectorType(llvm_u8_type, 4);
    static_cast<void>(llvm_pixel_type);
    typedef void (*Fragment_shader_function)(Pixel_type *color_attachment_pixel);
    constexpr std::size_t arg_color_attachment_pixel = 0;
    static_assert(std::is_same<Fragment_shader_function,
                               pipeline::Graphics_pipeline::Fragment_shader_function>::value,
                  "vertex shader function signature mismatch");
    auto function_type = llvm_wrapper::Create_llvm_type<Fragment_shader_function>()(context);
    auto entry_function = ::LLVMAddFunction(
        module.get(), get_prefixed_name("fragment_entry_point", true).c_str(), function_type);
    llvm_wrapper::Module::set_function_target_machine(entry_function, target_machine);
    auto color_attachment_pixel = ::LLVMGetParam(entry_function, arg_color_attachment_pixel);
    ::LLVMSetValueName(color_attachment_pixel, "color_attachment_pixel");
    auto entry_block = ::LLVMAppendBasicBlockInContext(context, entry_function, "entry");
    ::LLVMPositionBuilderAtEnd(builder.get(), entry_block);
    auto io_struct_type = io_struct->get_or_make_type();
    auto io_struct_pointer = ::LLVMBuildAlloca(builder.get(), io_struct_type.type, "io_struct");
    auto inputs_struct_pointer =
        ::LLVMBuildAlloca(builder.get(), inputs_struct->get_or_make_type().type, "inputs");
    auto outputs_struct_pointer =
        ::LLVMBuildAlloca(builder.get(), outputs_struct->get_or_make_type().type, "outputs");
    ::LLVMSetAlignment(
        ::LLVMBuildStore(builder.get(), ::LLVMConstNull(io_struct_type.type), io_struct_pointer),
        io_struct_type.alignment);
    auto inputs_pointer =
        ::LLVMBuildStructGEP(builder.get(),
                             io_struct_pointer,
                             io_struct->get_members(true)[inputs_member].llvm_member_index,
                             "inputs_pointer");
    ::LLVMBuildStore(builder.get(), inputs_struct_pointer, inputs_pointer);
    auto outputs_pointer =
        ::LLVMBuildStructGEP(builder.get(),
                             io_struct_pointer,
                             io_struct->get_members(true)[outputs_member].llvm_member_index,
                             "outputs_pointer");
    ::LLVMBuildStore(builder.get(), outputs_struct_pointer, outputs_pointer);
    ::LLVMValueRef output_color = nullptr;
    std::vector<std::function<void()>> after_call_callbacks;
    auto &&members = io_struct->get_members(true);
    for(std::size_t member_index = 0; member_index < members.size(); member_index++)
    {
        auto &member = members[member_index];
        static_cast<void>(member);
        if(member_index == inputs_member)
        {
            for(auto &input_member : inputs_struct->get_members(true))
            {
                auto input_pointer = ::LLVMBuildStructGEP(
                    builder.get(), inputs_struct_pointer, input_member.llvm_member_index, "input");
                ::LLVMDumpType(::LLVMTypeOf(input_pointer));
                util::optional<spirv::Built_in> built_in;
                static_cast<void>(input_pointer);
                for(auto &decoration : input_member.decorations)
                {
                    switch(decoration.value)
                    {
                    case spirv::Decoration::relaxed_precision:
#warning finish implementing Decoration::relaxed_precision
                        break;
                    case spirv::Decoration::spec_id:
#warning finish implementing Decoration::spec_id
                        break;
                    case spirv::Decoration::block:
#warning finish implementing Decoration::block
                        break;
                    case spirv::Decoration::buffer_block:
#warning finish implementing Decoration::buffer_block
                        break;
                    case spirv::Decoration::row_major:
#warning finish implementing Decoration::row_major
                        break;
                    case spirv::Decoration::col_major:
#warning finish implementing Decoration::col_major
                        break;
                    case spirv::Decoration::array_stride:
#warning finish implementing Decoration::array_stride
                        break;
                    case spirv::Decoration::matrix_stride:
#warning finish implementing Decoration::matrix_stride
                        break;
                    case spirv::Decoration::glsl_shared:
#warning finish implementing Decoration::glsl_shared
                        break;
                    case spirv::Decoration::glsl_packed:
#warning finish implementing Decoration::glsl_packed
                        break;
                    case spirv::Decoration::c_packed:
#warning finish implementing Decoration::c_packed
                        break;
                    case spirv::Decoration::built_in:
                        if(built_in)
                            throw spirv::Parser_error(
                                0, 0, "multiple BuiltIn decorations on the same variable");
                        built_in =
                            util::get<spirv::Decoration_built_in_parameters>(decoration.parameters)
                                .built_in;
                        continue;
                    case spirv::Decoration::no_perspective:
#warning finish implementing Decoration::no_perspective
                        break;
                    case spirv::Decoration::flat:
#warning finish implementing Decoration::flat
                        break;
                    case spirv::Decoration::patch:
#warning finish implementing Decoration::patch
                        break;
                    case spirv::Decoration::centroid:
#warning finish implementing Decoration::centroid
                        break;
                    case spirv::Decoration::sample:
#warning finish implementing Decoration::sample
                        break;
                    case spirv::Decoration::invariant:
#warning finish implementing Decoration::invariant
                        break;
                    case spirv::Decoration::restrict:
#warning finish implementing Decoration::restrict
                        break;
                    case spirv::Decoration::aliased:
#warning finish implementing Decoration::aliased
                        break;
                    case spirv::Decoration::volatile_:
#warning finish implementing Decoration::volatile_
                        break;
                    case spirv::Decoration::constant:
#warning finish implementing Decoration::constant
                        break;
                    case spirv::Decoration::coherent:
#warning finish implementing Decoration::coherent
                        break;
                    case spirv::Decoration::non_writable:
#warning finish implementing Decoration::non_writable
                        break;
                    case spirv::Decoration::non_readable:
#warning finish implementing Decoration::non_readable
                        break;
                    case spirv::Decoration::uniform:
#warning finish implementing Decoration::uniform
                        break;
                    case spirv::Decoration::saturated_conversion:
#warning finish implementing Decoration::saturated_conversion
                        break;
                    case spirv::Decoration::stream:
#warning finish implementing Decoration::stream
                        break;
                    case spirv::Decoration::location:
#warning finish implementing Decoration::location
                        break;
                    case spirv::Decoration::component:
#warning finish implementing Decoration::component
                        break;
                    case spirv::Decoration::index:
#warning finish implementing Decoration::index
                        break;
                    case spirv::Decoration::binding:
#warning finish implementing Decoration::binding
                        break;
                    case spirv::Decoration::descriptor_set:
#warning finish implementing Decoration::descriptor_set
                        break;
                    case spirv::Decoration::offset:
#warning finish implementing Decoration::offset
                        break;
                    case spirv::Decoration::xfb_buffer:
#warning finish implementing Decoration::xfb_buffer
                        break;
                    case spirv::Decoration::xfb_stride:
#warning finish implementing Decoration::xfb_stride
                        break;
                    case spirv::Decoration::func_param_attr:
#warning finish implementing Decoration::func_param_attr
                        break;
                    case spirv::Decoration::fp_rounding_mode:
#warning finish implementing Decoration::fp_rounding_mode
                        break;
                    case spirv::Decoration::fp_fast_math_mode:
#warning finish implementing Decoration::fp_fast_math_mode
                        break;
                    case spirv::Decoration::linkage_attributes:
#warning finish implementing Decoration::linkage_attributes
                        break;
                    case spirv::Decoration::no_contraction:
#warning finish implementing Decoration::no_contraction
                        break;
                    case spirv::Decoration::input_attachment_index:
#warning finish implementing Decoration::input_attachment_index
                        break;
                    case spirv::Decoration::alignment:
#warning finish implementing Decoration::alignment
                        break;
                    case spirv::Decoration::max_byte_offset:
#warning finish implementing Decoration::max_byte_offset
                        break;
                    case spirv::Decoration::alignment_id:
#warning finish implementing Decoration::alignment_id
                        break;
                    case spirv::Decoration::max_byte_offset_id:
#warning finish implementing Decoration::max_byte_offset_id
                        break;
                    case spirv::Decoration::override_coverage_nv:
#warning finish implementing Decoration::override_coverage_nv
                        break;
                    case spirv::Decoration::passthrough_nv:
#warning finish implementing Decoration::passthrough_nv
                        break;
                    case spirv::Decoration::viewport_relative_nv:
#warning finish implementing Decoration::viewport_relative_nv
                        break;
                    case spirv::Decoration::secondary_viewport_relative_nv:
#warning finish implementing Decoration::secondary_viewport_relative_nv
                        break;
                    }
                    throw spirv::Parser_error(
                        0,
                        0,
                        "unimplemented member decoration on shader input variable: "
                            + std::string(get_enumerant_name(decoration.value)));
                }
                if(!built_in)
                    throw spirv::Parser_error(
                        0, 0, "non-built-in shader input variables are not implemented");
                do
                {
                    switch(*built_in)
                    {
                    case spirv::Built_in::position:
#warning finish implementing Built_in::position
                        break;
                    case spirv::Built_in::point_size:
#warning finish implementing Built_in::point_size
                        break;
                    case spirv::Built_in::clip_distance:
#warning finish implementing Built_in::clip_distance
                        break;
                    case spirv::Built_in::cull_distance:
#warning finish implementing Built_in::cull_distance
                        break;
                    case spirv::Built_in::vertex_id:
#warning finish implementing Built_in::vertex_id
                        break;
                    case spirv::Built_in::instance_id:
#warning finish implementing Built_in::instance_id
                        break;
                    case spirv::Built_in::primitive_id:
#warning finish implementing Built_in::primitive_id
                        break;
                    case spirv::Built_in::invocation_id:
#warning finish implementing Built_in::invocation_id
                        break;
                    case spirv::Built_in::layer:
#warning finish implementing Built_in::layer
                        break;
                    case spirv::Built_in::viewport_index:
#warning finish implementing Built_in::viewport_index
                        break;
                    case spirv::Built_in::tess_level_outer:
#warning finish implementing Built_in::tess_level_outer
                        break;
                    case spirv::Built_in::tess_level_inner:
#warning finish implementing Built_in::tess_level_inner
                        break;
                    case spirv::Built_in::tess_coord:
#warning finish implementing Built_in::tess_coord
                        break;
                    case spirv::Built_in::patch_vertices:
#warning finish implementing Built_in::patch_vertices
                        break;
                    case spirv::Built_in::frag_coord:
#warning finish implementing Built_in::frag_coord
                        break;
                    case spirv::Built_in::point_coord:
#warning finish implementing Built_in::point_coord
                        break;
                    case spirv::Built_in::front_facing:
#warning finish implementing Built_in::front_facing
                        break;
                    case spirv::Built_in::sample_id:
#warning finish implementing Built_in::sample_id
                        break;
                    case spirv::Built_in::sample_position:
#warning finish implementing Built_in::sample_position
                        break;
                    case spirv::Built_in::sample_mask:
#warning finish implementing Built_in::sample_mask
                        break;
                    case spirv::Built_in::frag_depth:
#warning finish implementing Built_in::frag_depth
                        break;
                    case spirv::Built_in::helper_invocation:
#warning finish implementing Built_in::helper_invocation
                        break;
                    case spirv::Built_in::num_workgroups:
#warning finish implementing Built_in::num_workgroups
                        break;
                    case spirv::Built_in::workgroup_size:
#warning finish implementing Built_in::workgroup_size
                        break;
                    case spirv::Built_in::workgroup_id:
#warning finish implementing Built_in::workgroup_id
                        break;
                    case spirv::Built_in::local_invocation_id:
#warning finish implementing Built_in::local_invocation_id
                        break;
                    case spirv::Built_in::global_invocation_id:
#warning finish implementing Built_in::global_invocation_id
                        break;
                    case spirv::Built_in::local_invocation_index:
#warning finish implementing Built_in::local_invocation_index
                        break;
                    case spirv::Built_in::work_dim:
#warning finish implementing Built_in::work_dim
                        break;
                    case spirv::Built_in::global_size:
#warning finish implementing Built_in::global_size
                        break;
                    case spirv::Built_in::enqueued_workgroup_size:
#warning finish implementing Built_in::enqueued_workgroup_size
                        break;
                    case spirv::Built_in::global_offset:
#warning finish implementing Built_in::global_offset
                        break;
                    case spirv::Built_in::global_linear_id:
#warning finish implementing Built_in::global_linear_id
                        break;
                    case spirv::Built_in::subgroup_size:
#warning finish implementing Built_in::subgroup_size
                        break;
                    case spirv::Built_in::subgroup_max_size:
#warning finish implementing Built_in::subgroup_max_size
                        break;
                    case spirv::Built_in::num_subgroups:
#warning finish implementing Built_in::num_subgroups
                        break;
                    case spirv::Built_in::num_enqueued_subgroups:
#warning finish implementing Built_in::num_enqueued_subgroups
                        break;
                    case spirv::Built_in::subgroup_id:
#warning finish implementing Built_in::subgroup_id
                        break;
                    case spirv::Built_in::subgroup_local_invocation_id:
#warning finish implementing Built_in::subgroup_local_invocation_id
                        break;
                    case spirv::Built_in::vertex_index:
#warning finish implementing Built_in::vertex_index
                        break;
                    case spirv::Built_in::instance_index:
#warning finish implementing Built_in::instance_index
                        break;
                    case spirv::Built_in::subgroup_eq_mask_khr:
#warning finish implementing Built_in::subgroup_eq_mask_khr
                        break;
                    case spirv::Built_in::subgroup_ge_mask_khr:
#warning finish implementing Built_in::subgroup_ge_mask_khr
                        break;
                    case spirv::Built_in::subgroup_gt_mask_khr:
#warning finish implementing Built_in::subgroup_gt_mask_khr
                        break;
                    case spirv::Built_in::subgroup_le_mask_khr:
#warning finish implementing Built_in::subgroup_le_mask_khr
                        break;
                    case spirv::Built_in::subgroup_lt_mask_khr:
#warning finish implementing Built_in::subgroup_lt_mask_khr
                        break;
                    case spirv::Built_in::base_vertex:
#warning finish implementing Built_in::base_vertex
                        break;
                    case spirv::Built_in::base_instance:
#warning finish implementing Built_in::base_instance
                        break;
                    case spirv::Built_in::draw_index:
#warning finish implementing Built_in::draw_index
                        break;
                    case spirv::Built_in::device_index:
#warning finish implementing Built_in::device_index
                        break;
                    case spirv::Built_in::view_index:
#warning finish implementing Built_in::view_index
                        break;
                    case spirv::Built_in::viewport_mask_nv:
#warning finish implementing Built_in::viewport_mask_nv
                        break;
                    case spirv::Built_in::secondary_position_nv:
#warning finish implementing Built_in::secondary_position_nv
                        break;
                    case spirv::Built_in::secondary_viewport_mask_nv:
#warning finish implementing Built_in::secondary_viewport_mask_nv
                        break;
                    case spirv::Built_in::position_per_view_nv:
#warning finish implementing Built_in::position_per_view_nv
                        break;
                    case spirv::Built_in::viewport_mask_per_view_nv:
#warning finish implementing Built_in::viewport_mask_per_view_nv
                        break;
                    }
                    throw spirv::Parser_error(0,
                                              0,
                                              "unimplemented built in shader input variable: "
                                                  + std::string(get_enumerant_name(*built_in)));
                } while(false);
            }
        }
        else if(member_index == outputs_member)
        {
            for(auto &output_member : outputs_struct->get_members(true))
            {
                auto output_pointer = ::LLVMBuildStructGEP(builder.get(),
                                                           outputs_struct_pointer,
                                                           output_member.llvm_member_index,
                                                           "output");
                static_cast<void>(output_pointer);
                util::optional<spirv::Literal_integer> location;
                for(auto &decoration : output_member.decorations)
                {
                    switch(decoration.value)
                    {
                    case spirv::Decoration::relaxed_precision:
#warning finish implementing Decoration::relaxed_precision
                        break;
                    case spirv::Decoration::spec_id:
#warning finish implementing Decoration::spec_id
                        break;
                    case spirv::Decoration::block:
#warning finish implementing Decoration::block
                        break;
                    case spirv::Decoration::buffer_block:
#warning finish implementing Decoration::buffer_block
                        break;
                    case spirv::Decoration::row_major:
#warning finish implementing Decoration::row_major
                        break;
                    case spirv::Decoration::col_major:
#warning finish implementing Decoration::col_major
                        break;
                    case spirv::Decoration::array_stride:
#warning finish implementing Decoration::array_stride
                        break;
                    case spirv::Decoration::matrix_stride:
#warning finish implementing Decoration::matrix_stride
                        break;
                    case spirv::Decoration::glsl_shared:
#warning finish implementing Decoration::glsl_shared
                        break;
                    case spirv::Decoration::glsl_packed:
#warning finish implementing Decoration::glsl_packed
                        break;
                    case spirv::Decoration::c_packed:
#warning finish implementing Decoration::c_packed
                        break;
                    case spirv::Decoration::built_in:
#warning finish implementing Decoration::built_in
                        break;
                    case spirv::Decoration::no_perspective:
#warning finish implementing Decoration::no_perspective
                        break;
                    case spirv::Decoration::flat:
#warning finish implementing Decoration::flat
                        break;
                    case spirv::Decoration::patch:
#warning finish implementing Decoration::patch
                        break;
                    case spirv::Decoration::centroid:
#warning finish implementing Decoration::centroid
                        break;
                    case spirv::Decoration::sample:
#warning finish implementing Decoration::sample
                        break;
                    case spirv::Decoration::invariant:
#warning finish implementing Decoration::invariant
                        break;
                    case spirv::Decoration::restrict:
#warning finish implementing Decoration::restrict
                        break;
                    case spirv::Decoration::aliased:
#warning finish implementing Decoration::aliased
                        break;
                    case spirv::Decoration::volatile_:
#warning finish implementing Decoration::volatile_
                        break;
                    case spirv::Decoration::constant:
#warning finish implementing Decoration::constant
                        break;
                    case spirv::Decoration::coherent:
#warning finish implementing Decoration::coherent
                        break;
                    case spirv::Decoration::non_writable:
#warning finish implementing Decoration::non_writable
                        break;
                    case spirv::Decoration::non_readable:
#warning finish implementing Decoration::non_readable
                        break;
                    case spirv::Decoration::uniform:
#warning finish implementing Decoration::uniform
                        break;
                    case spirv::Decoration::saturated_conversion:
#warning finish implementing Decoration::saturated_conversion
                        break;
                    case spirv::Decoration::stream:
#warning finish implementing Decoration::stream
                        break;
                    case spirv::Decoration::location:
                        if(location)
                            throw spirv::Parser_error(
                                0, 0, "multiple Location decorations on the same variable");
                        location =
                            util::get<spirv::Decoration_location_parameters>(decoration.parameters)
                                .location;
                        continue;
                    case spirv::Decoration::component:
#warning finish implementing Decoration::component
                        break;
                    case spirv::Decoration::index:
#warning finish implementing Decoration::index
                        break;
                    case spirv::Decoration::binding:
#warning finish implementing Decoration::binding
                        break;
                    case spirv::Decoration::descriptor_set:
#warning finish implementing Decoration::descriptor_set
                        break;
                    case spirv::Decoration::offset:
#warning finish implementing Decoration::offset
                        break;
                    case spirv::Decoration::xfb_buffer:
#warning finish implementing Decoration::xfb_buffer
                        break;
                    case spirv::Decoration::xfb_stride:
#warning finish implementing Decoration::xfb_stride
                        break;
                    case spirv::Decoration::func_param_attr:
#warning finish implementing Decoration::func_param_attr
                        break;
                    case spirv::Decoration::fp_rounding_mode:
#warning finish implementing Decoration::fp_rounding_mode
                        break;
                    case spirv::Decoration::fp_fast_math_mode:
#warning finish implementing Decoration::fp_fast_math_mode
                        break;
                    case spirv::Decoration::linkage_attributes:
#warning finish implementing Decoration::linkage_attributes
                        break;
                    case spirv::Decoration::no_contraction:
#warning finish implementing Decoration::no_contraction
                        break;
                    case spirv::Decoration::input_attachment_index:
#warning finish implementing Decoration::input_attachment_index
                        break;
                    case spirv::Decoration::alignment:
#warning finish implementing Decoration::alignment
                        break;
                    case spirv::Decoration::max_byte_offset:
#warning finish implementing Decoration::max_byte_offset
                        break;
                    case spirv::Decoration::alignment_id:
#warning finish implementing Decoration::alignment_id
                        break;
                    case spirv::Decoration::max_byte_offset_id:
#warning finish implementing Decoration::max_byte_offset_id
                        break;
                    case spirv::Decoration::override_coverage_nv:
#warning finish implementing Decoration::override_coverage_nv
                        break;
                    case spirv::Decoration::passthrough_nv:
#warning finish implementing Decoration::passthrough_nv
                        break;
                    case spirv::Decoration::viewport_relative_nv:
#warning finish implementing Decoration::viewport_relative_nv
                        break;
                    case spirv::Decoration::secondary_viewport_relative_nv:
#warning finish implementing Decoration::secondary_viewport_relative_nv
                        break;
                    }
                    throw spirv::Parser_error(
                        0,
                        0,
                        "unimplemented member decoration on shader output variable: "
                            + std::string(get_enumerant_name(decoration.value)));
                }
                if(!location)
                    throw spirv::Parser_error(
                        0, 0, "fragment shader output variable is missing Location decoration");
                if(*location != 0)
                    throw spirv::Parser_error(
                        0,
                        0,
                        "nonzero Location for fragment shader output variable is unimplemented");
                auto llvm_output_member_type = output_member.type->get_or_make_type();
                if(llvm_output_member_type.type != llvm_vec4_type)
                    throw spirv::Parser_error(
                        0, 0, "fragment shader output variable type is unimplemented");
                auto callback = [llvm_output_member_type, &output_color, this, output_pointer]()
                {
                    if(output_color)
                        throw spirv::Parser_error(
                            0, 0, "duplicate fragment shader output variable");
                    output_color = ::LLVMBuildLoad(builder.get(), output_pointer, "output_color");
                    ::LLVMSetAlignment(output_color, llvm_output_member_type.alignment);
                };
                after_call_callbacks.push_back(std::move(callback));
            }
        }
        else
        {
            throw spirv::Parser_error(0, 0, "internal error: unhandled Io_struct member");
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
    for(auto &fn : after_call_callbacks)
        fn();
    after_call_callbacks.clear();
    if(!output_color)
        throw spirv::Parser_error(0, 0, "no fragment shader color output variables");
    auto constant_one = ::LLVMConstReal(llvm_float_type, 1.0);
    ::LLVMValueRef constant_vec4_of_one;
    {
        constexpr std::size_t vector_length = 4;
        ::LLVMValueRef args[vector_length] = {
            constant_one, constant_one, constant_one, constant_one,
        };
        constant_vec4_of_one = ::LLVMConstVector(args, vector_length);
    }
    auto constant_vec4_of_zero = ::LLVMConstNull(::LLVMTypeOf(constant_vec4_of_one));
    auto output_color_is_too_small = ::LLVMBuildFCmp(builder.get(),
                                                     ::LLVMRealULT,
                                                     output_color,
                                                     constant_vec4_of_zero,
                                                     "output_color_is_too_small");
    auto output_color_is_too_large = ::LLVMBuildFCmp(builder.get(),
                                                     ::LLVMRealOGT,
                                                     output_color,
                                                     constant_vec4_of_one,
                                                     "output_color_is_too_large");
    auto clamped_output_color = ::LLVMBuildSelect(
        builder.get(),
        output_color_is_too_small,
        constant_vec4_of_zero,
        ::LLVMBuildSelect(
            builder.get(), output_color_is_too_large, constant_vec4_of_one, output_color, ""),
        "clamped_output_color");
    float multiplier_value = std::nextafterf(0x100, -1);
    auto llvm_multiplier = ::LLVMConstReal(llvm_float_type, multiplier_value);
    ::LLVMValueRef multiplier_vec4;
    {
        constexpr std::size_t vector_length = 4;
        ::LLVMValueRef args[vector_length] = {
            llvm_multiplier, llvm_multiplier, llvm_multiplier, llvm_multiplier,
        };
        multiplier_vec4 = ::LLVMConstVector(args, vector_length);
    }
    auto scaled_output_color = ::LLVMBuildFMul(
        builder.get(), multiplier_vec4, clamped_output_color, "scaled_output_color");
    auto converted_output_color = ::LLVMBuildFPToUI(
        builder.get(), scaled_output_color, llvm_u8vec4_type, "converted_output_color");
    auto packed_output_color = ::LLVMBuildBitCast(
        builder.get(), converted_output_color, llvm_pixel_type, "packed_output_color");
    ::LLVMBuildStore(builder.get(), packed_output_color, color_attachment_pixel);
    static_assert(
        std::is_same<decltype(std::declval<Fragment_shader_function>()(nullptr)), void>::value, "");
    ::LLVMBuildRetVoid(builder.get());
    return entry_function;
}
}
}
