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
#include "parser_callbacks_debug.h"

namespace kazan
{
namespace spirv_to_llvm
{
namespace parser_callbacks
{
void Debug_callbacks::clear_line_info_because_end_of_block()
{
    handle_instruction_op_no_line({}, 0);
}

Spirv_location Debug_callbacks::get_location(std::size_t instruction_start_index) const noexcept
{
    return Spirv_location(current_location, instruction_start_index);
}

void Debug_callbacks::handle_instruction_op_source_continued(spirv::Op_source_continued instruction,
                                                             std::size_t instruction_start_index)
{
    static_cast<void>(instruction);
    static_cast<void>(instruction_start_index);
}

void Debug_callbacks::handle_instruction_op_source(spirv::Op_source instruction,
                                                   std::size_t instruction_start_index)
{
    if(instruction.file)
    {
        auto &filename = get_id<Spirv_string>(*instruction.file);
        current_location.filename = &filename;
        source_filename = &filename;
    }
    static_cast<void>(instruction_start_index);
}

void Debug_callbacks::handle_instruction_op_source_extension(spirv::Op_source_extension instruction,
                                                             std::size_t instruction_start_index)
{
    static_cast<void>(instruction);
    static_cast<void>(instruction_start_index);
}

void Debug_callbacks::handle_instruction_op_name(spirv::Op_name instruction,
                                                 std::size_t instruction_start_index)
{
    auto &map = per_shader_state->names;
    if(map.count(instruction.target) == 0)
        map[instruction.target] = std::string(instruction.name);
    static_cast<void>(instruction_start_index);
}

void Debug_callbacks::handle_instruction_op_member_name(spirv::Op_member_name instruction,
                                                        std::size_t instruction_start_index)
{
    auto &map = per_shader_state->member_names[instruction.type];
    if(map.count(instruction.member) == 0)
        map[instruction.member] = std::string(instruction.name);
    static_cast<void>(instruction_start_index);
}

void Debug_callbacks::handle_instruction_op_string(spirv::Op_string instruction,
                                                   std::size_t instruction_start_index)
{
    if(!is_id_defined_at(instruction.result, instruction_start_index))
        set_id(instruction.result,
               std::make_unique<Spirv_string>(instruction_start_index, static_cast<std::string>(instruction.string)));
}

void Debug_callbacks::handle_instruction_op_line(spirv::Op_line instruction,
                                                 std::size_t instruction_start_index)
{
    current_location.filename = &get_id<Spirv_string>(instruction.file);
    current_location.line_info = Spirv_location::Line_info(instruction.line, instruction.column);
    static_cast<void>(instruction_start_index);
}

void Debug_callbacks::handle_instruction_op_no_line(spirv::Op_no_line instruction,
                                                    std::size_t instruction_start_index)
{
    current_location.filename = source_filename;
    current_location.line_info.reset();
    static_cast<void>(instruction);
    static_cast<void>(instruction_start_index);
}

void Debug_callbacks::handle_instruction_op_module_processed(spirv::Op_module_processed instruction,
                                                             std::size_t instruction_start_index)
{
    static_cast<void>(instruction);
    static_cast<void>(instruction_start_index);
}
}
}
}
