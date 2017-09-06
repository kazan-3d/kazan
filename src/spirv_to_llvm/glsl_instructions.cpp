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

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_round(Glsl_std_450_op_round instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_round_even(
    Glsl_std_450_op_round_even instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_trunc(Glsl_std_450_op_trunc instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_f_abs(Glsl_std_450_op_f_abs instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_s_abs(Glsl_std_450_op_s_abs instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_f_sign(Glsl_std_450_op_f_sign instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_s_sign(Glsl_std_450_op_s_sign instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_floor(Glsl_std_450_op_floor instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_ceil(Glsl_std_450_op_ceil instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_fract(Glsl_std_450_op_fract instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_radians(Glsl_std_450_op_radians instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_degrees(Glsl_std_450_op_degrees instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_sin(Glsl_std_450_op_sin instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_cos(Glsl_std_450_op_cos instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_tan(Glsl_std_450_op_tan instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_asin(Glsl_std_450_op_asin instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_acos(Glsl_std_450_op_acos instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_atan(Glsl_std_450_op_atan instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_sinh(Glsl_std_450_op_sinh instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_cosh(Glsl_std_450_op_cosh instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_tanh(Glsl_std_450_op_tanh instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_asinh(Glsl_std_450_op_asinh instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_acosh(Glsl_std_450_op_acosh instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_atanh(Glsl_std_450_op_atanh instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_atan2(Glsl_std_450_op_atan2 instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pow(Glsl_std_450_op_pow instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_exp(Glsl_std_450_op_exp instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_log(Glsl_std_450_op_log instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_exp2(Glsl_std_450_op_exp2 instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_log2(Glsl_std_450_op_log2 instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_sqrt(Glsl_std_450_op_sqrt instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_inverse_sqrt(
    Glsl_std_450_op_inverse_sqrt instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_determinant(
    Glsl_std_450_op_determinant instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_matrix_inverse(
    Glsl_std_450_op_matrix_inverse instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_modf(Glsl_std_450_op_modf instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_modf_struct(
    Glsl_std_450_op_modf_struct instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_f_min(Glsl_std_450_op_f_min instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_u_min(Glsl_std_450_op_u_min instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_s_min(Glsl_std_450_op_s_min instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_f_max(Glsl_std_450_op_f_max instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_u_max(Glsl_std_450_op_u_max instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_s_max(Glsl_std_450_op_s_max instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_f_clamp(Glsl_std_450_op_f_clamp instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_u_clamp(Glsl_std_450_op_u_clamp instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_s_clamp(Glsl_std_450_op_s_clamp instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_f_mix(Glsl_std_450_op_f_mix instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_i_mix(Glsl_std_450_op_i_mix instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_step(Glsl_std_450_op_step instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_smooth_step(
    Glsl_std_450_op_smooth_step instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_fma(Glsl_std_450_op_fma instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_frexp(Glsl_std_450_op_frexp instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_frexp_struct(
    Glsl_std_450_op_frexp_struct instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_ldexp(Glsl_std_450_op_ldexp instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pack_snorm4x8(
    Glsl_std_450_op_pack_snorm4x8 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pack_unorm4x8(
    Glsl_std_450_op_pack_unorm4x8 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pack_snorm2x16(
    Glsl_std_450_op_pack_snorm2x16 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pack_unorm2x16(
    Glsl_std_450_op_pack_unorm2x16 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pack_half2x16(
    Glsl_std_450_op_pack_half2x16 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pack_double2x32(
    Glsl_std_450_op_pack_double2x32 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_unpack_snorm2x16(
    Glsl_std_450_op_unpack_snorm2x16 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_unpack_unorm2x16(
    Glsl_std_450_op_unpack_unorm2x16 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_unpack_half2x16(
    Glsl_std_450_op_unpack_half2x16 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_unpack_snorm4x8(
    Glsl_std_450_op_unpack_snorm4x8 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_unpack_unorm4x8(
    Glsl_std_450_op_unpack_unorm4x8 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_unpack_double2x32(
    Glsl_std_450_op_unpack_double2x32 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_length(Glsl_std_450_op_length instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_distance(
    Glsl_std_450_op_distance instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_cross(Glsl_std_450_op_cross instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_normalize(
    Glsl_std_450_op_normalize instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_face_forward(
    Glsl_std_450_op_face_forward instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_reflect(Glsl_std_450_op_reflect instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_refract(Glsl_std_450_op_refract instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_find_i_lsb(
    Glsl_std_450_op_find_i_lsb instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_find_s_msb(
    Glsl_std_450_op_find_s_msb instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_find_u_msb(
    Glsl_std_450_op_find_u_msb instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_interpolate_at_centroid(
    Glsl_std_450_op_interpolate_at_centroid instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_interpolate_at_sample(
    Glsl_std_450_op_interpolate_at_sample instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_interpolate_at_offset(
    Glsl_std_450_op_interpolate_at_offset instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_n_min(Glsl_std_450_op_n_min instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_n_max(Glsl_std_450_op_n_max instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_n_clamp(Glsl_std_450_op_n_clamp instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}
}
}
