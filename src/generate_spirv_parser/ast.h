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

struct Capabilities
{
    std::vector<std::string> capabilities;
    Capabilities() : capabilities()
    {
    }
    explicit Capabilities(std::vector<std::string> capabilities) noexcept
        : capabilities(std::move(capabilities))
    {
    }
    bool empty() const noexcept
    {
        return capabilities.empty();
    }
    json::ast::Value to_json() const;
};

struct Extensions
{
    std::vector<std::string> extensions;
    Extensions() : extensions()
    {
    }
    explicit Extensions(std::vector<std::string> extensions) noexcept
        : extensions(std::move(extensions))
    {
    }
    bool empty() const noexcept
    {
        return extensions.empty();
    }
    json::ast::Value to_json() const;
};

struct Instructions
{
    struct Instruction
    {
        struct Operands
        {
            struct Operand
            {
                enum class Quantifier
                {
                    none,
                    optional,
                    variable,
                };
                static constexpr const char *get_quantifier_string(Quantifier quantifier) noexcept
                {
                    switch(quantifier)
                    {
                    case Quantifier::none:
                        return "";
                    case Quantifier::optional:
                        return "?";
                    case Quantifier::variable:
                        return "*";
                    }
                    return "";
                }
                std::string kind;
                std::string name;
                Quantifier quantifier;
                Operand(std::string kind, std::string name, Quantifier quantifier) noexcept
                    : kind(std::move(kind)),
                      name(std::move(name)),
                      quantifier(quantifier)
                {
                }
                json::ast::Value to_json() const;
            };
            std::vector<Operand> operands;
            Operands() : operands()
            {
            }
            explicit Operands(std::vector<Operand> operands) noexcept
                : operands(std::move(operands))
            {
            }
            bool empty() const noexcept
            {
                return operands.empty();
            }
            json::ast::Value to_json() const;
        };
        std::string opname;
        std::uint32_t opcode;
        Operands operands;
        Capabilities capabilities;
        Instruction(std::string opname,
                    std::uint32_t opcode,
                    Operands operands,
                    Capabilities capabilities) noexcept : opname(std::move(opname)),
                                                          opcode(opcode),
                                                          operands(std::move(operands)),
                                                          capabilities(std::move(capabilities))
        {
        }
        json::ast::Value to_json() const;
    };
    std::vector<Instruction> instructions;
    explicit Instructions(std::vector<Instruction> instructions) noexcept
        : instructions(std::move(instructions))
    {
    }
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
                std::string enumerant;
                std::uint32_t value;
                Capabilities capabilities;
                struct Parameters
                {
                    struct Parameter
                    {
                        std::string kind;
                        std::string name;
                        explicit Parameter(std::string kind, std::string name) noexcept
                            : kind(std::move(kind)),
                              name(std::move(name))
                        {
                        }
                        json::ast::Value to_json() const;
                    };
                    std::vector<Parameter> parameters;
                    Parameters() : parameters()
                    {
                    }
                    explicit Parameters(std::vector<Parameter> parameters) noexcept
                        : parameters(std::move(parameters))
                    {
                    }
                    json::ast::Value to_json() const;
                    bool empty() const noexcept
                    {
                        return parameters.empty();
                    }
                };
                Parameters parameters;
                Extensions extensions;
                Enumerant(std::string enumerant,
                          std::uint32_t value,
                          Capabilities capabilities,
                          Parameters parameters,
                          Extensions extensions) noexcept : enumerant(std::move(enumerant)),
                                                            value(value),
                                                            capabilities(std::move(capabilities)),
                                                            parameters(std::move(parameters)),
                                                            extensions(std::move(extensions))
                {
                }
                json::ast::Value to_json(bool is_bit_enumerant) const;
            };
            std::vector<Enumerant> enumerants;
            explicit Enumerants(std::vector<Enumerant> enumerants) noexcept : enumerants(enumerants)
            {
            }
            json::ast::Value to_json(bool is_bit_enumerant) const;
            json::ast::Value to_json(Category category) const
            {
                return to_json(category == Category::bit_enum);
            }
        };
        struct Doc
        {
            static constexpr const char *get_json_key_name() noexcept
            {
                return "doc";
            }
            std::string value;
            Doc() = default;
            explicit Doc(std::string value) noexcept : value(std::move(value))
            {
            }
            json::ast::Value to_json() const;
            json::ast::Value to_json(Category category) const
            {
                return to_json();
            }
        };
        struct Bases
        {
            static constexpr const char *get_json_key_name() noexcept
            {
                return "bases";
            }
            std::vector<std::string> values;
            Bases() = default;
            explicit Bases(std::vector<std::string> values) noexcept : values(std::move(values))
            {
            }
            json::ast::Value to_json() const;
            json::ast::Value to_json(Category category) const
            {
                return to_json();
            }
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
