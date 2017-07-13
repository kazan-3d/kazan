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
#include "llvm_wrapper/llvm_wrapper.h"
#include <llvm-c/Core.h>

namespace vulkan_cpu
{
namespace spirv_to_llvm
{
class Spirv_to_llvm final : public spirv::Parser_callbacks
{
    Spirv_to_llvm(const Spirv_to_llvm &) = delete;
    Spirv_to_llvm &operator=(const Spirv_to_llvm &) = delete;

private:
    struct Implementation;

private:
    std::shared_ptr<Implementation> imp;

public:
    Spirv_to_llvm(Spirv_to_llvm &&) = default;
    Spirv_to_llvm &operator=(Spirv_to_llvm &&) = default;

public:
    explicit Spirv_to_llvm(const llvm_wrapper::Context &context);
    llvm_wrapper::Module finish();
    virtual void handle_header(unsigned version_number_major,
                               unsigned version_number_minor,
                               spirv::Word generator_magic_number,
                               spirv::Word id_bound,
                               spirv::Word instruction_schema) override;
    virtual void handle_instruction_op_nop(spirv::Op_nop instruction,
                                           std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_undef(spirv::Op_undef instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_source_continued(
        spirv::Op_source_continued instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_source(spirv::Op_source instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_source_extension(
        spirv::Op_source_extension instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_name(spirv::Op_name instruction,
                                            std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_member_name(spirv::Op_member_name instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_string(spirv::Op_string instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_line(spirv::Op_line instruction,
                                            std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_extension(spirv::Op_extension instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_ext_inst_import(
        spirv::Op_ext_inst_import instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_ext_inst(spirv::Op_ext_inst instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_memory_model(spirv::Op_memory_model instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_entry_point(spirv::Op_entry_point instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_execution_mode(spirv::Op_execution_mode instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_capability(spirv::Op_capability instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_void(spirv::Op_type_void instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_bool(spirv::Op_type_bool instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_int(spirv::Op_type_int instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_float(spirv::Op_type_float instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_vector(spirv::Op_type_vector instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_matrix(spirv::Op_type_matrix instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_image(spirv::Op_type_image instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_sampler(spirv::Op_type_sampler instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_sampled_image(
        spirv::Op_type_sampled_image instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_array(spirv::Op_type_array instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_runtime_array(
        spirv::Op_type_runtime_array instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_struct(spirv::Op_type_struct instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_opaque(spirv::Op_type_opaque instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_pointer(spirv::Op_type_pointer instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_function(spirv::Op_type_function instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_event(spirv::Op_type_event instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_device_event(
        spirv::Op_type_device_event instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_reserve_id(
        spirv::Op_type_reserve_id instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_queue(spirv::Op_type_queue instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_pipe(spirv::Op_type_pipe instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_forward_pointer(
        spirv::Op_type_forward_pointer instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant_true(spirv::Op_constant_true instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant_false(spirv::Op_constant_false instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant(spirv::Op_constant instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant_composite(
        spirv::Op_constant_composite instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant_sampler(
        spirv::Op_constant_sampler instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant_null(spirv::Op_constant_null instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_spec_constant_true(
        spirv::Op_spec_constant_true instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_spec_constant_false(
        spirv::Op_spec_constant_false instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_spec_constant(spirv::Op_spec_constant instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_spec_constant_composite(
        spirv::Op_spec_constant_composite instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_spec_constant_op(
        spirv::Op_spec_constant_op instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_function(spirv::Op_function instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_function_parameter(
        spirv::Op_function_parameter instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_function_end(spirv::Op_function_end instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_function_call(spirv::Op_function_call instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_variable(spirv::Op_variable instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_texel_pointer(
        spirv::Op_image_texel_pointer instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_load(spirv::Op_load instruction,
                                            std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_store(spirv::Op_store instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_copy_memory(spirv::Op_copy_memory instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_copy_memory_sized(
        spirv::Op_copy_memory_sized instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_access_chain(spirv::Op_access_chain instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_in_bounds_access_chain(
        spirv::Op_in_bounds_access_chain instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_ptr_access_chain(
        spirv::Op_ptr_access_chain instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_array_length(spirv::Op_array_length instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_generic_ptr_mem_semantics(
        spirv::Op_generic_ptr_mem_semantics instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_in_bounds_ptr_access_chain(
        spirv::Op_in_bounds_ptr_access_chain instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_decorate(spirv::Op_decorate instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_member_decorate(
        spirv::Op_member_decorate instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_decoration_group(
        spirv::Op_decoration_group instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_decorate(spirv::Op_group_decorate instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_member_decorate(
        spirv::Op_group_member_decorate instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_vector_extract_dynamic(
        spirv::Op_vector_extract_dynamic instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_vector_insert_dynamic(
        spirv::Op_vector_insert_dynamic instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_vector_shuffle(spirv::Op_vector_shuffle instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_composite_construct(
        spirv::Op_composite_construct instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_composite_extract(
        spirv::Op_composite_extract instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_composite_insert(
        spirv::Op_composite_insert instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_copy_object(spirv::Op_copy_object instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_transpose(spirv::Op_transpose instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_sampled_image(spirv::Op_sampled_image instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_implicit_lod(
        spirv::Op_image_sample_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_explicit_lod(
        spirv::Op_image_sample_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_dref_implicit_lod(
        spirv::Op_image_sample_dref_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_dref_explicit_lod(
        spirv::Op_image_sample_dref_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_proj_implicit_lod(
        spirv::Op_image_sample_proj_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_proj_explicit_lod(
        spirv::Op_image_sample_proj_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_proj_dref_implicit_lod(
        spirv::Op_image_sample_proj_dref_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_proj_dref_explicit_lod(
        spirv::Op_image_sample_proj_dref_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_fetch(spirv::Op_image_fetch instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_gather(spirv::Op_image_gather instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_dref_gather(
        spirv::Op_image_dref_gather instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_read(spirv::Op_image_read instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_write(spirv::Op_image_write instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image(spirv::Op_image instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_format(
        spirv::Op_image_query_format instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_order(
        spirv::Op_image_query_order instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_size_lod(
        spirv::Op_image_query_size_lod instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_size(
        spirv::Op_image_query_size instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_lod(
        spirv::Op_image_query_lod instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_levels(
        spirv::Op_image_query_levels instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_samples(
        spirv::Op_image_query_samples instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_convert_f_to_u(spirv::Op_convert_f_to_u instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_convert_f_to_s(spirv::Op_convert_f_to_s instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_convert_s_to_f(spirv::Op_convert_s_to_f instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_convert_u_to_f(spirv::Op_convert_u_to_f instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_convert(spirv::Op_u_convert instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_convert(spirv::Op_s_convert instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_convert(spirv::Op_f_convert instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_quantize_to_f16(
        spirv::Op_quantize_to_f16 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_convert_ptr_to_u(
        spirv::Op_convert_ptr_to_u instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_sat_convert_s_to_u(
        spirv::Op_sat_convert_s_to_u instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_sat_convert_u_to_s(
        spirv::Op_sat_convert_u_to_s instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_convert_u_to_ptr(
        spirv::Op_convert_u_to_ptr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_ptr_cast_to_generic(
        spirv::Op_ptr_cast_to_generic instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_generic_cast_to_ptr(
        spirv::Op_generic_cast_to_ptr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_generic_cast_to_ptr_explicit(
        spirv::Op_generic_cast_to_ptr_explicit instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bitcast(spirv::Op_bitcast instruction,
                                               std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_negate(spirv::Op_s_negate instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_negate(spirv::Op_f_negate instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_add(spirv::Op_i_add instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_add(spirv::Op_f_add instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_sub(spirv::Op_i_sub instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_sub(spirv::Op_f_sub instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_mul(spirv::Op_i_mul instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_mul(spirv::Op_f_mul instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_div(spirv::Op_u_div instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_div(spirv::Op_s_div instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_div(spirv::Op_f_div instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_mod(spirv::Op_u_mod instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_rem(spirv::Op_s_rem instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_mod(spirv::Op_s_mod instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_rem(spirv::Op_f_rem instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_mod(spirv::Op_f_mod instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_vector_times_scalar(
        spirv::Op_vector_times_scalar instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_matrix_times_scalar(
        spirv::Op_matrix_times_scalar instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_vector_times_matrix(
        spirv::Op_vector_times_matrix instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_matrix_times_vector(
        spirv::Op_matrix_times_vector instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_matrix_times_matrix(
        spirv::Op_matrix_times_matrix instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_outer_product(spirv::Op_outer_product instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_dot(spirv::Op_dot instruction,
                                           std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_add_carry(spirv::Op_i_add_carry instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_sub_borrow(spirv::Op_i_sub_borrow instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_mul_extended(spirv::Op_u_mul_extended instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_mul_extended(spirv::Op_s_mul_extended instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_any(spirv::Op_any instruction,
                                           std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_all(spirv::Op_all instruction,
                                           std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_is_nan(spirv::Op_is_nan instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_is_inf(spirv::Op_is_inf instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_is_finite(spirv::Op_is_finite instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_is_normal(spirv::Op_is_normal instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_sign_bit_set(spirv::Op_sign_bit_set instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_less_or_greater(
        spirv::Op_less_or_greater instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_ordered(spirv::Op_ordered instruction,
                                               std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_unordered(spirv::Op_unordered instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_logical_equal(spirv::Op_logical_equal instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_logical_not_equal(
        spirv::Op_logical_not_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_logical_or(spirv::Op_logical_or instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_logical_and(spirv::Op_logical_and instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_logical_not(spirv::Op_logical_not instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_select(spirv::Op_select instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_equal(spirv::Op_i_equal instruction,
                                               std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_not_equal(spirv::Op_i_not_equal instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_greater_than(spirv::Op_u_greater_than instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_greater_than(spirv::Op_s_greater_than instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_greater_than_equal(
        spirv::Op_u_greater_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_greater_than_equal(
        spirv::Op_s_greater_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_less_than(spirv::Op_u_less_than instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_less_than(spirv::Op_s_less_than instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_less_than_equal(
        spirv::Op_u_less_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_less_than_equal(
        spirv::Op_s_less_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_ord_equal(spirv::Op_f_ord_equal instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_unord_equal(spirv::Op_f_unord_equal instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_ord_not_equal(
        spirv::Op_f_ord_not_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_unord_not_equal(
        spirv::Op_f_unord_not_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_ord_less_than(
        spirv::Op_f_ord_less_than instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_unord_less_than(
        spirv::Op_f_unord_less_than instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_ord_greater_than(
        spirv::Op_f_ord_greater_than instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_unord_greater_than(
        spirv::Op_f_unord_greater_than instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_ord_less_than_equal(
        spirv::Op_f_ord_less_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_unord_less_than_equal(
        spirv::Op_f_unord_less_than_equal instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_ord_greater_than_equal(
        spirv::Op_f_ord_greater_than_equal instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_unord_greater_than_equal(
        spirv::Op_f_unord_greater_than_equal instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_shift_right_logical(
        spirv::Op_shift_right_logical instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_shift_right_arithmetic(
        spirv::Op_shift_right_arithmetic instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_shift_left_logical(
        spirv::Op_shift_left_logical instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bitwise_or(spirv::Op_bitwise_or instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bitwise_xor(spirv::Op_bitwise_xor instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bitwise_and(spirv::Op_bitwise_and instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_not(spirv::Op_not instruction,
                                           std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bit_field_insert(
        spirv::Op_bit_field_insert instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bit_field_s_extract(
        spirv::Op_bit_field_s_extract instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bit_field_u_extract(
        spirv::Op_bit_field_u_extract instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bit_reverse(spirv::Op_bit_reverse instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bit_count(spirv::Op_bit_count instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_d_pdx(spirv::Op_d_pdx instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_d_pdy(spirv::Op_d_pdy instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_fwidth(spirv::Op_fwidth instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_d_pdx_fine(spirv::Op_d_pdx_fine instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_d_pdy_fine(spirv::Op_d_pdy_fine instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_fwidth_fine(spirv::Op_fwidth_fine instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_d_pdx_coarse(spirv::Op_d_pdx_coarse instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_d_pdy_coarse(spirv::Op_d_pdy_coarse instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_fwidth_coarse(spirv::Op_fwidth_coarse instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_emit_vertex(spirv::Op_emit_vertex instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_end_primitive(spirv::Op_end_primitive instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_emit_stream_vertex(
        spirv::Op_emit_stream_vertex instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_end_stream_primitive(
        spirv::Op_end_stream_primitive instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_control_barrier(
        spirv::Op_control_barrier instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_memory_barrier(spirv::Op_memory_barrier instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_load(spirv::Op_atomic_load instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_store(spirv::Op_atomic_store instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_exchange(
        spirv::Op_atomic_exchange instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_compare_exchange(
        spirv::Op_atomic_compare_exchange instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_compare_exchange_weak(
        spirv::Op_atomic_compare_exchange_weak instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_i_increment(
        spirv::Op_atomic_i_increment instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_i_decrement(
        spirv::Op_atomic_i_decrement instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_i_add(spirv::Op_atomic_i_add instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_i_sub(spirv::Op_atomic_i_sub instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_s_min(spirv::Op_atomic_s_min instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_u_min(spirv::Op_atomic_u_min instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_s_max(spirv::Op_atomic_s_max instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_u_max(spirv::Op_atomic_u_max instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_and(spirv::Op_atomic_and instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_or(spirv::Op_atomic_or instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_xor(spirv::Op_atomic_xor instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_phi(spirv::Op_phi instruction,
                                           std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_loop_merge(spirv::Op_loop_merge instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_selection_merge(
        spirv::Op_selection_merge instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_label(spirv::Op_label instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_branch(spirv::Op_branch instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_branch_conditional(
        spirv::Op_branch_conditional instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_switch(spirv::Op_switch instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_kill(spirv::Op_kill instruction,
                                            std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_return(spirv::Op_return instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_return_value(spirv::Op_return_value instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_unreachable(spirv::Op_unreachable instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_lifetime_start(spirv::Op_lifetime_start instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_lifetime_stop(spirv::Op_lifetime_stop instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_async_copy(
        spirv::Op_group_async_copy instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_wait_events(
        spirv::Op_group_wait_events instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_all(spirv::Op_group_all instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_any(spirv::Op_group_any instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_broadcast(
        spirv::Op_group_broadcast instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_i_add(spirv::Op_group_i_add instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_f_add(spirv::Op_group_f_add instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_f_min(spirv::Op_group_f_min instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_u_min(spirv::Op_group_u_min instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_s_min(spirv::Op_group_s_min instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_f_max(spirv::Op_group_f_max instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_u_max(spirv::Op_group_u_max instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_s_max(spirv::Op_group_s_max instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_read_pipe(spirv::Op_read_pipe instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_write_pipe(spirv::Op_write_pipe instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_reserved_read_pipe(
        spirv::Op_reserved_read_pipe instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_reserved_write_pipe(
        spirv::Op_reserved_write_pipe instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_reserve_read_pipe_packets(
        spirv::Op_reserve_read_pipe_packets instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_reserve_write_pipe_packets(
        spirv::Op_reserve_write_pipe_packets instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_commit_read_pipe(
        spirv::Op_commit_read_pipe instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_commit_write_pipe(
        spirv::Op_commit_write_pipe instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_is_valid_reserve_id(
        spirv::Op_is_valid_reserve_id instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_num_pipe_packets(
        spirv::Op_get_num_pipe_packets instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_max_pipe_packets(
        spirv::Op_get_max_pipe_packets instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_reserve_read_pipe_packets(
        spirv::Op_group_reserve_read_pipe_packets instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_reserve_write_pipe_packets(
        spirv::Op_group_reserve_write_pipe_packets instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_commit_read_pipe(
        spirv::Op_group_commit_read_pipe instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_commit_write_pipe(
        spirv::Op_group_commit_write_pipe instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_enqueue_marker(spirv::Op_enqueue_marker instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_enqueue_kernel(spirv::Op_enqueue_kernel instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_kernel_n_drange_sub_group_count(
        spirv::Op_get_kernel_n_drange_sub_group_count instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_kernel_n_drange_max_sub_group_size(
        spirv::Op_get_kernel_n_drange_max_sub_group_size instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_kernel_work_group_size(
        spirv::Op_get_kernel_work_group_size instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_kernel_preferred_work_group_size_multiple(
        spirv::Op_get_kernel_preferred_work_group_size_multiple instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_retain_event(spirv::Op_retain_event instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_release_event(spirv::Op_release_event instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_create_user_event(
        spirv::Op_create_user_event instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_is_valid_event(spirv::Op_is_valid_event instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_set_user_event_status(
        spirv::Op_set_user_event_status instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_capture_event_profiling_info(
        spirv::Op_capture_event_profiling_info instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_default_queue(
        spirv::Op_get_default_queue instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_build_nd_range(spirv::Op_build_nd_range instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_implicit_lod(
        spirv::Op_image_sparse_sample_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_explicit_lod(
        spirv::Op_image_sparse_sample_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_dref_implicit_lod(
        spirv::Op_image_sparse_sample_dref_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_dref_explicit_lod(
        spirv::Op_image_sparse_sample_dref_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_proj_implicit_lod(
        spirv::Op_image_sparse_sample_proj_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_proj_explicit_lod(
        spirv::Op_image_sparse_sample_proj_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_proj_dref_implicit_lod(
        spirv::Op_image_sparse_sample_proj_dref_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_proj_dref_explicit_lod(
        spirv::Op_image_sparse_sample_proj_dref_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_fetch(
        spirv::Op_image_sparse_fetch instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_gather(
        spirv::Op_image_sparse_gather instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_dref_gather(
        spirv::Op_image_sparse_dref_gather instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_texels_resident(
        spirv::Op_image_sparse_texels_resident instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_no_line(spirv::Op_no_line instruction,
                                               std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_flag_test_and_set(
        spirv::Op_atomic_flag_test_and_set instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_flag_clear(
        spirv::Op_atomic_flag_clear instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_read(
        spirv::Op_image_sparse_read instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_size_of(spirv::Op_size_of instruction,
                                               std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_pipe_storage(
        spirv::Op_type_pipe_storage instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant_pipe_storage(
        spirv::Op_constant_pipe_storage instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_create_pipe_from_pipe_storage(
        spirv::Op_create_pipe_from_pipe_storage instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_kernel_local_size_for_subgroup_count(
        spirv::Op_get_kernel_local_size_for_subgroup_count instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_kernel_max_num_subgroups(
        spirv::Op_get_kernel_max_num_subgroups instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_named_barrier(
        spirv::Op_type_named_barrier instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_named_barrier_initialize(
        spirv::Op_named_barrier_initialize instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_memory_named_barrier(
        spirv::Op_memory_named_barrier instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_module_processed(
        spirv::Op_module_processed instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_execution_mode_id(
        spirv::Op_execution_mode_id instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_decorate_id(spirv::Op_decorate_id instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_subgroup_ballot_khr(
        spirv::Op_subgroup_ballot_khr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_subgroup_first_invocation_khr(
        spirv::Op_subgroup_first_invocation_khr instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_subgroup_all_khr(
        spirv::Op_subgroup_all_khr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_subgroup_any_khr(
        spirv::Op_subgroup_any_khr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_subgroup_all_equal_khr(
        spirv::Op_subgroup_all_equal_khr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_subgroup_read_invocation_khr(
        spirv::Op_subgroup_read_invocation_khr instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_acos(
        spirv::Open_cl_std_op_acos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_acosh(
        spirv::Open_cl_std_op_acosh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_acospi(
        spirv::Open_cl_std_op_acospi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_asin(
        spirv::Open_cl_std_op_asin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_asinh(
        spirv::Open_cl_std_op_asinh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_asinpi(
        spirv::Open_cl_std_op_asinpi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_atan(
        spirv::Open_cl_std_op_atan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_atan2(
        spirv::Open_cl_std_op_atan2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_atanh(
        spirv::Open_cl_std_op_atanh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_atanpi(
        spirv::Open_cl_std_op_atanpi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_atan2pi(
        spirv::Open_cl_std_op_atan2pi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_cbrt(
        spirv::Open_cl_std_op_cbrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_ceil(
        spirv::Open_cl_std_op_ceil instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_copysign(
        spirv::Open_cl_std_op_copysign instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_cos(
        spirv::Open_cl_std_op_cos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_cosh(
        spirv::Open_cl_std_op_cosh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_cospi(
        spirv::Open_cl_std_op_cospi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_erfc(
        spirv::Open_cl_std_op_erfc instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_erf(
        spirv::Open_cl_std_op_erf instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_exp(
        spirv::Open_cl_std_op_exp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_exp2(
        spirv::Open_cl_std_op_exp2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_exp10(
        spirv::Open_cl_std_op_exp10 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_expm1(
        spirv::Open_cl_std_op_expm1 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fabs(
        spirv::Open_cl_std_op_fabs instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fdim(
        spirv::Open_cl_std_op_fdim instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_floor(
        spirv::Open_cl_std_op_floor instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fma(
        spirv::Open_cl_std_op_fma instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fmax(
        spirv::Open_cl_std_op_fmax instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fmin(
        spirv::Open_cl_std_op_fmin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fmod(
        spirv::Open_cl_std_op_fmod instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fract(
        spirv::Open_cl_std_op_fract instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_frexp(
        spirv::Open_cl_std_op_frexp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_hypot(
        spirv::Open_cl_std_op_hypot instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_ilogb(
        spirv::Open_cl_std_op_ilogb instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_ldexp(
        spirv::Open_cl_std_op_ldexp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_lgamma(
        spirv::Open_cl_std_op_lgamma instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_lgamma_r(
        spirv::Open_cl_std_op_lgamma_r instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_log(
        spirv::Open_cl_std_op_log instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_log2(
        spirv::Open_cl_std_op_log2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_log10(
        spirv::Open_cl_std_op_log10 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_log1p(
        spirv::Open_cl_std_op_log1p instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_logb(
        spirv::Open_cl_std_op_logb instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_mad(
        spirv::Open_cl_std_op_mad instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_maxmag(
        spirv::Open_cl_std_op_maxmag instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_minmag(
        spirv::Open_cl_std_op_minmag instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_modf(
        spirv::Open_cl_std_op_modf instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_nan(
        spirv::Open_cl_std_op_nan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_nextafter(
        spirv::Open_cl_std_op_nextafter instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_pow(
        spirv::Open_cl_std_op_pow instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_pown(
        spirv::Open_cl_std_op_pown instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_powr(
        spirv::Open_cl_std_op_powr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_remainder(
        spirv::Open_cl_std_op_remainder instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_remquo(
        spirv::Open_cl_std_op_remquo instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_rint(
        spirv::Open_cl_std_op_rint instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_rootn(
        spirv::Open_cl_std_op_rootn instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_round(
        spirv::Open_cl_std_op_round instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_rsqrt(
        spirv::Open_cl_std_op_rsqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_sin(
        spirv::Open_cl_std_op_sin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_sincos(
        spirv::Open_cl_std_op_sincos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_sinh(
        spirv::Open_cl_std_op_sinh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_sinpi(
        spirv::Open_cl_std_op_sinpi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_sqrt(
        spirv::Open_cl_std_op_sqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_tan(
        spirv::Open_cl_std_op_tan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_tanh(
        spirv::Open_cl_std_op_tanh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_tanpi(
        spirv::Open_cl_std_op_tanpi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_tgamma(
        spirv::Open_cl_std_op_tgamma instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_trunc(
        spirv::Open_cl_std_op_trunc instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_cos(
        spirv::Open_cl_std_op_half_cos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_divide(
        spirv::Open_cl_std_op_half_divide instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_exp(
        spirv::Open_cl_std_op_half_exp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_exp2(
        spirv::Open_cl_std_op_half_exp2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_exp10(
        spirv::Open_cl_std_op_half_exp10 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_log(
        spirv::Open_cl_std_op_half_log instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_log2(
        spirv::Open_cl_std_op_half_log2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_log10(
        spirv::Open_cl_std_op_half_log10 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_powr(
        spirv::Open_cl_std_op_half_powr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_recip(
        spirv::Open_cl_std_op_half_recip instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_rsqrt(
        spirv::Open_cl_std_op_half_rsqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_sin(
        spirv::Open_cl_std_op_half_sin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_sqrt(
        spirv::Open_cl_std_op_half_sqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_tan(
        spirv::Open_cl_std_op_half_tan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_cos(
        spirv::Open_cl_std_op_native_cos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_divide(
        spirv::Open_cl_std_op_native_divide instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_exp(
        spirv::Open_cl_std_op_native_exp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_exp2(
        spirv::Open_cl_std_op_native_exp2 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_exp10(
        spirv::Open_cl_std_op_native_exp10 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_log(
        spirv::Open_cl_std_op_native_log instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_log2(
        spirv::Open_cl_std_op_native_log2 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_log10(
        spirv::Open_cl_std_op_native_log10 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_powr(
        spirv::Open_cl_std_op_native_powr instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_recip(
        spirv::Open_cl_std_op_native_recip instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_rsqrt(
        spirv::Open_cl_std_op_native_rsqrt instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_sin(
        spirv::Open_cl_std_op_native_sin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_sqrt(
        spirv::Open_cl_std_op_native_sqrt instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_tan(
        spirv::Open_cl_std_op_native_tan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_abs(
        spirv::Open_cl_std_op_s_abs instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_abs_diff(
        spirv::Open_cl_std_op_s_abs_diff instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_add_sat(
        spirv::Open_cl_std_op_s_add_sat instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_add_sat(
        spirv::Open_cl_std_op_u_add_sat instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_hadd(
        spirv::Open_cl_std_op_s_hadd instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_hadd(
        spirv::Open_cl_std_op_u_hadd instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_rhadd(
        spirv::Open_cl_std_op_s_rhadd instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_rhadd(
        spirv::Open_cl_std_op_u_rhadd instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_clamp(
        spirv::Open_cl_std_op_s_clamp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_clamp(
        spirv::Open_cl_std_op_u_clamp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_clz(
        spirv::Open_cl_std_op_clz instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_ctz(
        spirv::Open_cl_std_op_ctz instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_mad_hi(
        spirv::Open_cl_std_op_s_mad_hi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_mad_sat(
        spirv::Open_cl_std_op_u_mad_sat instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_mad_sat(
        spirv::Open_cl_std_op_s_mad_sat instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_max(
        spirv::Open_cl_std_op_s_max instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_max(
        spirv::Open_cl_std_op_u_max instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_min(
        spirv::Open_cl_std_op_s_min instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_min(
        spirv::Open_cl_std_op_u_min instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_mul_hi(
        spirv::Open_cl_std_op_s_mul_hi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_rotate(
        spirv::Open_cl_std_op_rotate instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_sub_sat(
        spirv::Open_cl_std_op_s_sub_sat instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_sub_sat(
        spirv::Open_cl_std_op_u_sub_sat instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_upsample(
        spirv::Open_cl_std_op_u_upsample instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_upsample(
        spirv::Open_cl_std_op_s_upsample instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_popcount(
        spirv::Open_cl_std_op_popcount instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_mad24(
        spirv::Open_cl_std_op_s_mad24 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_mad24(
        spirv::Open_cl_std_op_u_mad24 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_mul24(
        spirv::Open_cl_std_op_s_mul24 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_mul24(
        spirv::Open_cl_std_op_u_mul24 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_abs(
        spirv::Open_cl_std_op_u_abs instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_abs_diff(
        spirv::Open_cl_std_op_u_abs_diff instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_mul_hi(
        spirv::Open_cl_std_op_u_mul_hi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_mad_hi(
        spirv::Open_cl_std_op_u_mad_hi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fclamp(
        spirv::Open_cl_std_op_fclamp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_degrees(
        spirv::Open_cl_std_op_degrees instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fmax_common(
        spirv::Open_cl_std_op_fmax_common instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fmin_common(
        spirv::Open_cl_std_op_fmin_common instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_mix(
        spirv::Open_cl_std_op_mix instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_radians(
        spirv::Open_cl_std_op_radians instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_step(
        spirv::Open_cl_std_op_step instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_smoothstep(
        spirv::Open_cl_std_op_smoothstep instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_sign(
        spirv::Open_cl_std_op_sign instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_cross(
        spirv::Open_cl_std_op_cross instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_distance(
        spirv::Open_cl_std_op_distance instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_length(
        spirv::Open_cl_std_op_length instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_normalize(
        spirv::Open_cl_std_op_normalize instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fast_distance(
        spirv::Open_cl_std_op_fast_distance instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fast_length(
        spirv::Open_cl_std_op_fast_length instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fast_normalize(
        spirv::Open_cl_std_op_fast_normalize instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_bitselect(
        spirv::Open_cl_std_op_bitselect instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_select(
        spirv::Open_cl_std_op_select instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vloadn(
        spirv::Open_cl_std_op_vloadn instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstoren(
        spirv::Open_cl_std_op_vstoren instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vload_half(
        spirv::Open_cl_std_op_vload_half instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vload_halfn(
        spirv::Open_cl_std_op_vload_halfn instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstore_half(
        spirv::Open_cl_std_op_vstore_half instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstore_half_r(
        spirv::Open_cl_std_op_vstore_half_r instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstore_halfn(
        spirv::Open_cl_std_op_vstore_halfn instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstore_halfn_r(
        spirv::Open_cl_std_op_vstore_halfn_r instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vloada_halfn(
        spirv::Open_cl_std_op_vloada_halfn instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstorea_halfn(
        spirv::Open_cl_std_op_vstorea_halfn instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstorea_halfn_r(
        spirv::Open_cl_std_op_vstorea_halfn_r instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_shuffle(
        spirv::Open_cl_std_op_shuffle instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_shuffle2(
        spirv::Open_cl_std_op_shuffle2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_printf(
        spirv::Open_cl_std_op_printf instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_prefetch(
        spirv::Open_cl_std_op_prefetch instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_round(
        spirv::Glsl_std_450_op_round instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_round_even(
        spirv::Glsl_std_450_op_round_even instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_trunc(
        spirv::Glsl_std_450_op_trunc instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_f_abs(
        spirv::Glsl_std_450_op_f_abs instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_s_abs(
        spirv::Glsl_std_450_op_s_abs instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_f_sign(
        spirv::Glsl_std_450_op_f_sign instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_s_sign(
        spirv::Glsl_std_450_op_s_sign instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_floor(
        spirv::Glsl_std_450_op_floor instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_ceil(
        spirv::Glsl_std_450_op_ceil instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_fract(
        spirv::Glsl_std_450_op_fract instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_radians(
        spirv::Glsl_std_450_op_radians instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_degrees(
        spirv::Glsl_std_450_op_degrees instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_sin(
        spirv::Glsl_std_450_op_sin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_cos(
        spirv::Glsl_std_450_op_cos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_tan(
        spirv::Glsl_std_450_op_tan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_asin(
        spirv::Glsl_std_450_op_asin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_acos(
        spirv::Glsl_std_450_op_acos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_atan(
        spirv::Glsl_std_450_op_atan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_sinh(
        spirv::Glsl_std_450_op_sinh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_cosh(
        spirv::Glsl_std_450_op_cosh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_tanh(
        spirv::Glsl_std_450_op_tanh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_asinh(
        spirv::Glsl_std_450_op_asinh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_acosh(
        spirv::Glsl_std_450_op_acosh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_atanh(
        spirv::Glsl_std_450_op_atanh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_atan2(
        spirv::Glsl_std_450_op_atan2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pow(
        spirv::Glsl_std_450_op_pow instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_exp(
        spirv::Glsl_std_450_op_exp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_log(
        spirv::Glsl_std_450_op_log instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_exp2(
        spirv::Glsl_std_450_op_exp2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_log2(
        spirv::Glsl_std_450_op_log2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_sqrt(
        spirv::Glsl_std_450_op_sqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_inverse_sqrt(
        spirv::Glsl_std_450_op_inverse_sqrt instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_determinant(
        spirv::Glsl_std_450_op_determinant instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_matrix_inverse(
        spirv::Glsl_std_450_op_matrix_inverse instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_modf(
        spirv::Glsl_std_450_op_modf instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_modf_struct(
        spirv::Glsl_std_450_op_modf_struct instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_f_min(
        spirv::Glsl_std_450_op_f_min instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_u_min(
        spirv::Glsl_std_450_op_u_min instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_s_min(
        spirv::Glsl_std_450_op_s_min instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_f_max(
        spirv::Glsl_std_450_op_f_max instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_u_max(
        spirv::Glsl_std_450_op_u_max instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_s_max(
        spirv::Glsl_std_450_op_s_max instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_f_clamp(
        spirv::Glsl_std_450_op_f_clamp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_u_clamp(
        spirv::Glsl_std_450_op_u_clamp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_s_clamp(
        spirv::Glsl_std_450_op_s_clamp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_f_mix(
        spirv::Glsl_std_450_op_f_mix instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_i_mix(
        spirv::Glsl_std_450_op_i_mix instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_step(
        spirv::Glsl_std_450_op_step instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_smooth_step(
        spirv::Glsl_std_450_op_smooth_step instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_fma(
        spirv::Glsl_std_450_op_fma instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_frexp(
        spirv::Glsl_std_450_op_frexp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_frexp_struct(
        spirv::Glsl_std_450_op_frexp_struct instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_ldexp(
        spirv::Glsl_std_450_op_ldexp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pack_snorm4x8(
        spirv::Glsl_std_450_op_pack_snorm4x8 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pack_unorm4x8(
        spirv::Glsl_std_450_op_pack_unorm4x8 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pack_snorm2x16(
        spirv::Glsl_std_450_op_pack_snorm2x16 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pack_unorm2x16(
        spirv::Glsl_std_450_op_pack_unorm2x16 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pack_half2x16(
        spirv::Glsl_std_450_op_pack_half2x16 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pack_double2x32(
        spirv::Glsl_std_450_op_pack_double2x32 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_unpack_snorm2x16(
        spirv::Glsl_std_450_op_unpack_snorm2x16 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_unpack_unorm2x16(
        spirv::Glsl_std_450_op_unpack_unorm2x16 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_unpack_half2x16(
        spirv::Glsl_std_450_op_unpack_half2x16 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_unpack_snorm4x8(
        spirv::Glsl_std_450_op_unpack_snorm4x8 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_unpack_unorm4x8(
        spirv::Glsl_std_450_op_unpack_unorm4x8 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_unpack_double2x32(
        spirv::Glsl_std_450_op_unpack_double2x32 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_length(
        spirv::Glsl_std_450_op_length instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_distance(
        spirv::Glsl_std_450_op_distance instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_cross(
        spirv::Glsl_std_450_op_cross instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_normalize(
        spirv::Glsl_std_450_op_normalize instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_face_forward(
        spirv::Glsl_std_450_op_face_forward instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_reflect(
        spirv::Glsl_std_450_op_reflect instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_refract(
        spirv::Glsl_std_450_op_refract instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_find_i_lsb(
        spirv::Glsl_std_450_op_find_i_lsb instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_find_s_msb(
        spirv::Glsl_std_450_op_find_s_msb instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_find_u_msb(
        spirv::Glsl_std_450_op_find_u_msb instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_interpolate_at_centroid(
        spirv::Glsl_std_450_op_interpolate_at_centroid instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_interpolate_at_sample(
        spirv::Glsl_std_450_op_interpolate_at_sample instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_interpolate_at_offset(
        spirv::Glsl_std_450_op_interpolate_at_offset instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_n_min(
        spirv::Glsl_std_450_op_n_min instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_n_max(
        spirv::Glsl_std_450_op_n_max instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_n_clamp(
        spirv::Glsl_std_450_op_n_clamp instruction, std::size_t instruction_start_index) override;
};
}
}

#endif /* SPIRV_TO_LLVM_SPIRV_TO_LLVM_H_ */
