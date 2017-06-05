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

#ifndef GENERATE_SPIRV_PARSER_AST_H_
#define GENERATE_SPIRV_PARSER_AST_H_

#include "../json/json.h"
#include <cstdint>

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
namespace ast
{
struct Copyright
{
    json::ast::Array value;
    Copyright() : value()
    {
    }
    explicit Copyright(json::ast::Array value) noexcept : value(std::move(value))
    {
    }
};

struct Instructions
{
#warning finish
};

struct Operand_kinds
{
#warning finish
};

struct Top_level
{
    Copyright copyright;
    std::uint32_t magic_number;
    std::size_t major_version;
    std::size_t minor_version;
    std::size_t revision;
    Instructions instructions;
    Operand_kinds operand_kinds;
    Top_level(Copyright copyright,
              std::uint32_t magic_number,
              std::size_t major_version,
              std::size_t minor_version,
              std::size_t revision,
              Instructions instructions,
              Operand_kinds operand_kinds)
        : copyright(std::move(copyright)),
          magic_number(magic_number),
          major_version(major_version),
          minor_version(minor_version),
          revision(revision),
          instructions(std::move(instructions)),
          operand_kinds(std::move(operand_kinds))
    {
    }
};
}
}
}

#endif /* GENERATE_SPIRV_PARSER_AST_H_ */
