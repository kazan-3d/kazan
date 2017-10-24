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
#include "parser_callbacks_extensions.h"
#include "json/json.h"

namespace kazan
{
namespace spirv_to_llvm
{
namespace parser_callbacks
{
void Extensions_callbacks::handle_instruction_op_extension(spirv::Op_extension instruction,
                                                           std::size_t instruction_start_index)
{
    throw spirv::Parser_error(instruction_start_index,
                              instruction_start_index,
                              "unimplemented SPIR-V extension: " + std::string(instruction.name));
}

void Extensions_callbacks::handle_instruction_op_ext_inst_import(
    spirv::Op_ext_inst_import instruction, std::size_t instruction_start_index)
{
    for(auto instruction_set : util::Enum_traits<spirv::Extension_instruction_set>::values)
    {
        if(instruction_set == spirv::Extension_instruction_set::unknown)
            continue;
        if(instruction.name == get_enumerant_name(instruction_set))
        {
            if(!is_id_defined_at(instruction.result, instruction_start_index))
                set_id(instruction.result,
                       std::make_unique<Spirv_extended_instruction_set>(instruction_start_index,
                                                                        instruction_set));
            return;
        }
    }
    throw spirv::Parser_error(
        instruction_start_index,
        instruction_start_index,
        "unknown SPIR-V extension instruction set: \"" + std::string(instruction.name) + "\"");
}

void Extensions_callbacks::handle_instruction_op_ext_inst(spirv::Op_ext_inst instruction,
                                                          std::size_t instruction_start_index)
{
    // handles unknown extension instructions;
    // the correct handle_instruction_* callback is called instead for known instructions
    auto &instruction_set = get_id<Spirv_extended_instruction_set>(instruction.set);
    throw spirv::Parser_error(instruction_start_index,
                              instruction_start_index,
                              json::ast::Number_value::append_unsigned_integer_to_string(
                                  instruction.instruction,
                                  "unknown SPIR-V extension instruction: "
                                      + std::string(get_enumerant_name(instruction_set.value))
                                      + ": 0x",
                                  0x10));
}
}
}
}
