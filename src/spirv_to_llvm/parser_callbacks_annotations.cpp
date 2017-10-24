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
#include "parser_callbacks_annotations.h"

namespace kazan
{
namespace spirv_to_llvm
{
namespace parser_callbacks
{
void Annotations_callbacks::handle_instruction_op_decorate(spirv::Op_decorate instruction,
                                                           std::size_t instruction_start_index)
{
    auto &decorations = per_shader_state->decorations[instruction.target];
    decorations.emplace(instruction_start_index, std::move(instruction.decoration));
}

void Annotations_callbacks::handle_instruction_op_member_decorate(
    spirv::Op_member_decorate instruction, std::size_t instruction_start_index)
{
    auto &decorations =
        per_shader_state->member_decorations[instruction.structure_type][instruction.member];
    decorations.emplace(instruction_start_index, std::move(instruction.decoration));
}

void Annotations_callbacks::handle_instruction_op_decoration_group(
    spirv::Op_decoration_group instruction, std::size_t instruction_start_index)
{
    if(!is_id_defined_at(instruction.result, instruction_start_index))
    {
        auto decoration_range = get_decoration_range(instruction.result);
        set_id(instruction.result,
               std::make_unique<Spirv_decoration_group>(
                   instruction_start_index,
                   Spirv_decoration_set(std::get<0>(decoration_range),
                                        std::get<1>(decoration_range))));
    }
}

void Annotations_callbacks::handle_instruction_op_group_decorate(
    spirv::Op_group_decorate instruction, std::size_t instruction_start_index)
{
    auto &decoration_group = get_id<Spirv_decoration_group>(instruction.decoration_group);
    for(auto &target : instruction.targets)
        per_shader_state->decorations[target].insert(decoration_group.value.begin(),
                                                     decoration_group.value.end());
    static_cast<void>(instruction_start_index);
}

void Annotations_callbacks::handle_instruction_op_group_member_decorate(
    spirv::Op_group_member_decorate instruction, std::size_t instruction_start_index)
{
    auto &decoration_group = get_id<Spirv_decoration_group>(instruction.decoration_group);
    for(auto &target : instruction.targets)
        per_shader_state->member_decorations[target.part_1][target.part_2].insert(
            decoration_group.value.begin(), decoration_group.value.end());
    static_cast<void>(instruction_start_index);
}

void Annotations_callbacks::handle_instruction_op_decorate_id(spirv::Op_decorate_id instruction,
                                                              std::size_t instruction_start_index)
{
    auto &decorations = per_shader_state->decorations[instruction.target];
    decorations.emplace(instruction_start_index, std::move(instruction.decoration));
}
}
}
}
