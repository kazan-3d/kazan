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
#include <vector>
#include <string>

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
namespace ast
{
struct Copyright
{
    std::vector<std::string> lines;
    Copyright() : lines()
    {
    }
    explicit Copyright(std::vector<std::string> lines) noexcept : lines(std::move(lines))
    {
    }
    json::ast::Value to_json() const;
};

struct Instructions
{
#warning finish
    json::ast::Value to_json() const;
};

struct Operand_kinds
{
    struct Operand_kind
    {
        enum class Category
        {
            bit_enum,
            value_enum,
            id,
            literal,
            composite,
        };
        Category category;
        static constexpr const char *get_json_name_from_category(Category category) noexcept
        {
            switch(category)
            {
            case Category::bit_enum:
                return "BitEnum";
            case Category::value_enum:
                return "ValueEnum";
            case Category::id:
                return "Id";
            case Category::literal:
                return "Literal";
            case Category::composite:
                return "Composite";
            }
            return "";
        }
        std::string kind;
        struct Enumerants
        {
            static constexpr const char *get_json_key_name() noexcept
            {
                return "enumerants";
            }
            struct Enumerant
            {
#warning finish
                json::ast::Value to_json() const;
            };
            std::vector<Enumerant> enumerants;
            explicit Enumerants(std::vector<Enumerant> enumerants) noexcept : enumerants(enumerants)
            {
            }
            json::ast::Value to_json() const;
        };
        struct Doc
        {
            static constexpr const char *get_json_key_name() noexcept
            {
                return "doc";
            }
            std::string value;
            json::ast::Value to_json() const;
        };
        struct Bases
        {
            static constexpr const char *get_json_key_name() noexcept
            {
                return "bases";
            }
            std::vector<std::string> values;
            json::ast::Value to_json() const;
        };
        typedef util::variant<Enumerants, Doc, Bases> Value;
        Value value;
        static bool does_category_match_value(Category category, const Value &value) noexcept
        {
            switch(category)
            {
            case Category::bit_enum:
            case Category::value_enum:
                return util::holds_alternative<Enumerants>(value);
            case Category::id:
            case Category::literal:
                return util::holds_alternative<Doc>(value);
            case Category::composite:
                return util::holds_alternative<Bases>(value);
            }
            return false;
        }
        static constexpr const char *get_value_json_key_name_from_category(
            Category category) noexcept
        {
            switch(category)
            {
            case Category::bit_enum:
            case Category::value_enum:
                return Enumerants::get_json_key_name();
            case Category::id:
            case Category::literal:
                return Doc::get_json_key_name();
            case Category::composite:
                return Bases::get_json_key_name();
            }
            return "";
        }
#warning finish
        Operand_kind(Category category, std::string kind, Value value) noexcept
            : category(category),
              kind(kind),
              value(std::move(value))
        {
        }
        json::ast::Value to_json() const;
    };
    std::vector<Operand_kind> operand_kinds;
    explicit Operand_kinds(std::vector<Operand_kind> operand_kinds) noexcept
        : operand_kinds(std::move(operand_kinds))
    {
    }
    json::ast::Value to_json() const;
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
    json::ast::Value to_json() const;
};
}
}
}

#endif /* GENERATE_SPIRV_PARSER_AST_H_ */
