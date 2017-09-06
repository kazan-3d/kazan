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
#include "parser.h"
#include "util/optional.h"
#include "util/string_view.h"
#include <sstream>
#include <limits>
#include <iostream>
#include <cstdlib>
#include <list>

namespace kazan
{
namespace generate_spirv_parser
{
namespace parser
{
std::string Path::to_string() const
{
    std::ostringstream ss;
    ss << "root";
    for(auto &e : elements)
    {
        ss << '[';
        if(util::holds_alternative<std::size_t>(e))
        {
            ss << util::get<std::size_t>(e);
        }
        else
        {
            json::ast::String_value::write(ss, util::get<std::string>(e));
        }
        ss << ']';
    }
    return ss.str();
}

namespace
{
template <typename Value>
Value get_value_or_throw_parse_error(util::optional<Value> value,
                                     const json::Location &location,
                                     const Path_builder_base *path_builder,
                                     const char *message)
{
    if(value)
        return std::move(*value);
    throw Parse_error(location, path_builder ? path_builder->path() : Path{}, message);
}

template <typename Value>
Value get_value_or_throw_parse_error(util::optional<Value> value,
                                     const json::Location &location,
                                     const Path_builder_base *path_builder,
                                     const std::string &message)
{
    return get_value_or_throw_parse_error(
        std::move(value), location, path_builder, message.c_str());
}

template <typename Callback>
decltype(auto) get_object_member_or_throw_parse_error(const json::Location &object_location,
                                                      json::ast::Object &object,
                                                      const Path_builder_base *parent_path_builder,
                                                      const std::string &name,
                                                      Callback callback)
{
    auto iter = object.values.find(name);
    if(iter == object.values.end())
        throw Parse_error(object_location,
                          parent_path_builder ? parent_path_builder->path() : Path{},
                          "missing " + name);
    auto &entry = *iter;
    auto &key = std::get<0>(entry);
    auto &entry_value = std::get<1>(entry);
    const Path_builder<std::string> path_builder(&key, parent_path_builder);
    return callback(entry_value, &path_builder);
}


template <typename T>
T parse_integer(const json::ast::Value &value,
                const Path_builder_base *parent_path_builder,
                const char *name)
{
    if(value.get_value_kind() != json::ast::Value_kind::number)
        throw Parse_error(
            value.location, parent_path_builder->path(), std::string(name) + " is not a number");
    auto number_value = value.get_number();
    T retval = number_value.value;
    if(retval != number_value.value) // not an exact value
        throw Parse_error(
            value.location, parent_path_builder->path(), std::string(name) + " is not an integer");
    return retval;
}

constexpr int get_digit_value(unsigned char ch, unsigned base) noexcept
{
    unsigned retval{};
    if(ch >= '0' && ch <= '9')
        retval = ch - '0';
    else if(ch >= 'a' && ch <= 'z')
        retval = ch - 'a' + 0xA;
    else if(ch >= 'A' && ch <= 'Z')
        retval = ch - 'A' + 0xA;
    else
        return -1;
    if(retval >= base)
        return -1;
    return retval;
}

constexpr bool is_identifier_start(unsigned char ch) noexcept
{
    if(ch >= 'a' && ch <= 'z')
        return true;
    if(ch >= 'A' && ch <= 'Z')
        return true;
    return ch == '_';
}

constexpr bool is_identifier_continue(unsigned char ch) noexcept
{
    if(ch >= '0' && ch <= '9')
        return true;
    return is_identifier_start(ch);
}

template <typename T>
T parse_hex_integer_string(const json::ast::Value &value,
                           const Path_builder_base *parent_path_builder,
                           const char *name,
                           std::size_t min_length,
                           std::size_t max_length)
{
    if(value.get_value_kind() != json::ast::Value_kind::string)
        throw Parse_error(
            value.location, parent_path_builder->path(), std::string(name) + " is not a string");
    auto &string_value = value.get_string();
    constexpr std::size_t hex_number_prefix_length = 2; // std::strlen("0x")
    if(string_value.value.size() < hex_number_prefix_length || string_value.value[0] != '0'
       || (string_value.value[1] != 'x' && string_value.value[1] != 'X'))
        throw Parse_error(value.location,
                          parent_path_builder->path(),
                          std::string(name) + " is not a valid hex number in a string");
    constexpr T max_value = std::numeric_limits<T>::max();
    constexpr unsigned base = 0x10;
    T retval = 0;
    std::size_t digit_count = 0;
    for(std::size_t i = hex_number_prefix_length; i < string_value.value.size(); i++)
    {
        digit_count++;
        char ch = string_value.value[i];
        int digit = get_digit_value(ch, base);
        if(digit < 0)
            throw Parse_error(value.location,
                              parent_path_builder->path(),
                              std::string(name) + ": not a valid hex digit");
        if(digit_count > max_length)
            throw Parse_error(value.location,
                              parent_path_builder->path(),
                              std::string(name) + " has too many digits");
        if(retval > max_value / base
           || (retval == max_value / base && static_cast<unsigned>(digit) > max_value % base))
            throw Parse_error(
                value.location, parent_path_builder->path(), std::string(name) + ": value too big");
        retval *= base;
        retval += digit;
    }
    if(digit_count < min_length)
        throw Parse_error(value.location,
                          parent_path_builder->path(),
                          std::string(name) + " doesn't have enough digits");
    return retval;
}

template <typename Enum>
struct Enum_value_descriptor
{
    const char *name;
    Enum value;
    constexpr Enum_value_descriptor(const char *name, Enum value) noexcept : name(name),
                                                                             value(value)
    {
    }
};

template <typename Enum, const char *(*Get_Name)(Enum value), Enum... Values>
constexpr std::initializer_list<Enum_value_descriptor<Enum>> make_enum_value_descriptors = {
    {Get_Name(Values), Values}...};

template <typename Enum>
Enum parse_enum_string(const json::ast::Value &value,
                       const Path_builder_base *parent_path_builder,
                       const char *name,
                       std::initializer_list<Enum_value_descriptor<Enum>> enum_value_descriptors)
{
    if(value.get_value_kind() != json::ast::Value_kind::string)
        throw Parse_error(
            value.location, parent_path_builder->path(), std::string(name) + " is not a string");
    auto &string_value = value.get_string();
    for(auto &descriptor : enum_value_descriptors)
    {
        if(string_value.value == descriptor.name)
            return descriptor.value;
    }
    throw Parse_error(
        value.location, parent_path_builder->path(), std::string(name) + ": unknown value");
}

std::string parse_identifier_string(json::ast::Value value,
                                    const Path_builder_base *parent_path_builder,
                                    const char *name,
                                    bool can_start_with_digit = false)
{
    if(value.get_value_kind() != json::ast::Value_kind::string)
        throw Parse_error(
            value.location, parent_path_builder->path(), std::string(name) + " is not a string");
    auto &string_value = value.get_string();
    if(string_value.value.empty())
        throw Parse_error(value.location,
                          parent_path_builder->path(),
                          std::string(name) + " must not be an empty string");
    if(!can_start_with_digit && !is_identifier_start(string_value.value[0]))
        throw Parse_error(
            value.location,
            parent_path_builder->path(),
            std::string(name)
                + ": invalid identifier in string: must start with letter or underline");
    for(char ch : string_value.value)
        if(!is_identifier_continue(ch))
            throw Parse_error(
                value.location,
                parent_path_builder->path(),
                std::string(name)
                + ": invalid identifier in string: character is not a letter, digit, or underline");
    return std::move(string_value.value);
}

ast::Copyright parse_copyright(json::ast::Value value, const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::array)
        throw Parse_error(value.location, parent_path_builder->path(), "copyright is not an array");
    auto &copyright_array = value.get_array();
    std::vector<std::string> lines;
    lines.reserve(copyright_array.values.size());
    for(std::size_t index = 0; index < copyright_array.values.size(); index++)
    {
        Path_builder<std::size_t> path_builder(&index, parent_path_builder);
        auto &element = copyright_array.values[index];
        if(element.get_value_kind() != json::ast::Value_kind::string)
            throw Parse_error(element.location,
                              parent_path_builder->path(),
                              "copyright array's element is not a string");
        lines.push_back(std::move(element.get_string().value));
    }
    return ast::Copyright(std::move(lines));
}

ast::Capabilities parse_capabilities(json::ast::Value value,
                                     const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::array)
        throw Parse_error(
            value.location, parent_path_builder->path(), "capabilities is not an array");
    auto &capabilities_array = value.get_array();
    std::vector<std::string> capabilities;
    capabilities.reserve(capabilities_array.values.size());
    for(std::size_t index = 0; index < capabilities_array.values.size(); index++)
    {
        Path_builder<std::size_t> path_builder(&index, parent_path_builder);
        auto &element = capabilities_array.values[index];
        capabilities.push_back(
            parse_identifier_string(std::move(element), &path_builder, "capabilities"));
    }
    return ast::Capabilities(std::move(capabilities));
}

ast::Extensions parse_extensions(json::ast::Value value,
                                 const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::array)
        throw Parse_error(
            value.location, parent_path_builder->path(), "extensions is not an array");
    auto &extensions_array = value.get_array();
    std::vector<std::string> extensions;
    extensions.reserve(extensions_array.values.size());
    for(std::size_t index = 0; index < extensions_array.values.size(); index++)
    {
        Path_builder<std::size_t> path_builder(&index, parent_path_builder);
        auto &element = extensions_array.values[index];
        extensions.push_back(
            parse_identifier_string(std::move(element), &path_builder, "extensions"));
    }
    return ast::Extensions(std::move(extensions));
}

ast::Operand_kinds::Operand_kind::Enumerants::Enumerant::Parameters::Parameter
    parse_operand_kinds_operand_kind_enumerants_enumerant_parameters_parameter(
        json::ast::Value value, const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::object)
        throw Parse_error(
            value.location, parent_path_builder->path(), "parameter is not an object");
    auto &parameter_object = value.get_object();
    constexpr auto kind_name = "kind";
    std::string kind = get_object_member_or_throw_parse_error(
        value.location,
        parameter_object,
        parent_path_builder,
        kind_name,
        [&](json::ast::Value &entry_value, const Path_builder_base *path_builder)
        {
            return parse_identifier_string(std::move(entry_value), path_builder, kind_name);
        });
    std::string name = "";
    for(auto &entry : parameter_object.values)
    {
        const auto &key = std::get<0>(entry);
        auto &entry_value = std::get<1>(entry);
        Path_builder<std::string> path_builder(&key, parent_path_builder);
        if(key == "name")
        {
            if(entry_value.get_value_kind() != json::ast::Value_kind::string)
                throw Parse_error(
                    entry_value.location, path_builder.path(), "name is not a string");
            name = std::move(entry_value.get_string().value);
        }
        else if(key != kind_name)
        {
            throw Parse_error(entry_value.location, path_builder.path(), "unknown key");
        }
    }
    return ast::Operand_kinds::Operand_kind::Enumerants::Enumerant::Parameters::Parameter(
        std::move(kind), std::move(name));
}

ast::Operand_kinds::Operand_kind::Enumerants::Enumerant::Parameters
    parse_operand_kinds_operand_kind_enumerants_enumerant_parameters(
        json::ast::Value value, const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::array)
        throw Parse_error(
            value.location, parent_path_builder->path(), "parameters is not an array");
    auto &parameters_array = value.get_array();
    std::vector<ast::Operand_kinds::Operand_kind::Enumerants::Enumerant::Parameters::Parameter>
        parameters;
    parameters.reserve(parameters_array.values.size());
    for(std::size_t index = 0; index < parameters_array.values.size(); index++)
    {
        Path_builder<std::size_t> path_builder(&index, parent_path_builder);
        auto &element = parameters_array.values[index];
        parameters.push_back(
            parse_operand_kinds_operand_kind_enumerants_enumerant_parameters_parameter(
                std::move(element), &path_builder));
    }
    return ast::Operand_kinds::Operand_kind::Enumerants::Enumerant::Parameters(
        std::move(parameters));
}

ast::Operand_kinds::Operand_kind::Enumerants::Enumerant
    parse_operand_kinds_operand_kind_enumerants_enumerant(
        json::ast::Value value, const Path_builder_base *parent_path_builder, bool is_bit_enumerant)
{
    if(value.get_value_kind() != json::ast::Value_kind::object)
        throw Parse_error(
            value.location, parent_path_builder->path(), "enumerant is not an object");
    auto &enumerant_object = value.get_object();
    constexpr auto enumerant_name = "enumerant";
    std::string enumerant = get_object_member_or_throw_parse_error(
        value.location,
        enumerant_object,
        parent_path_builder,
        enumerant_name,
        [&](json::ast::Value &entry_value, const Path_builder_base *path_builder)
        {
            return parse_identifier_string(
                std::move(entry_value), path_builder, enumerant_name, true);
        });
    constexpr auto value_name = "value";
    std::uint32_t enumerant_value = get_object_member_or_throw_parse_error(
        value.location,
        enumerant_object,
        parent_path_builder,
        value_name,
        [&](json::ast::Value &entry_value, const Path_builder_base *path_builder) -> std::uint32_t
        {
            if(is_bit_enumerant)
                return parse_hex_integer_string<std::uint32_t>(
                    entry_value, path_builder, value_name, 1, 8);
            return parse_integer<std::uint32_t>(entry_value, path_builder, value_name);
        });
    ast::Capabilities capabilities;
    ast::Operand_kinds::Operand_kind::Enumerants::Enumerant::Parameters parameters;
    ast::Extensions extensions;
    for(auto &entry : enumerant_object.values)
    {
        const auto &key = std::get<0>(entry);
        auto &entry_value = std::get<1>(entry);
        Path_builder<std::string> path_builder(&key, parent_path_builder);
        if(key == "capabilities")
        {
            capabilities = parse_capabilities(std::move(entry_value), &path_builder);
        }
        else if(key == "parameters")
        {
            parameters = parse_operand_kinds_operand_kind_enumerants_enumerant_parameters(
                std::move(entry_value), &path_builder);
        }
        else if(key == "extensions")
        {
            extensions = parse_extensions(std::move(entry_value), &path_builder);
        }
        else if(key != enumerant_name && key != value_name)
        {
            throw Parse_error(entry_value.location, path_builder.path(), "unknown key");
        }
    }
    return ast::Operand_kinds::Operand_kind::Enumerants::Enumerant(std::move(enumerant),
                                                                   enumerant_value,
                                                                   std::move(capabilities),
                                                                   std::move(parameters),
                                                                   std::move(extensions));
}

ast::Operand_kinds::Operand_kind::Enumerants parse_operand_kinds_operand_kind_enumerants(
    json::ast::Value value, const Path_builder_base *parent_path_builder, bool is_bit_enumerant)
{
    if(value.get_value_kind() != json::ast::Value_kind::array)
        throw Parse_error(
            value.location, parent_path_builder->path(), "enumerants is not an array");
    auto &enumerants_array = value.get_array();
    std::vector<ast::Operand_kinds::Operand_kind::Enumerants::Enumerant> enumerants;
    enumerants.reserve(enumerants_array.values.size());
    for(std::size_t index = 0; index < enumerants_array.values.size(); index++)
    {
        Path_builder<std::size_t> path_builder(&index, parent_path_builder);
        enumerants.push_back(parse_operand_kinds_operand_kind_enumerants_enumerant(
            std::move(enumerants_array.values[index]), &path_builder, is_bit_enumerant));
    }
    return ast::Operand_kinds::Operand_kind::Enumerants(std::move(enumerants));
}

ast::Operand_kinds::Operand_kind parse_operand_kinds_operand_kind(
    json::ast::Value value, const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::object)
        throw Parse_error(
            value.location, parent_path_builder->path(), "operand kind is not an object");
    auto &operand_kind_object = value.get_object();
    constexpr auto category_name = "category";
    constexpr auto kind_name = "kind";
    ast::Operand_kinds::Operand_kind::Category category = get_object_member_or_throw_parse_error(
        value.location,
        operand_kind_object,
        parent_path_builder,
        category_name,
        [&](json::ast::Value &entry_value, const Path_builder_base *path_builder)
        {
            return parse_enum_string<ast::Operand_kinds::Operand_kind::Category>(
                std::move(entry_value),
                path_builder,
                "category",
                make_enum_value_descriptors<ast::Operand_kinds::Operand_kind::Category,
                                            ast::Operand_kinds::Operand_kind::
                                                get_json_name_from_category,
                                            ast::Operand_kinds::Operand_kind::Category::bit_enum,
                                            ast::Operand_kinds::Operand_kind::Category::value_enum,
                                            ast::Operand_kinds::Operand_kind::Category::id,
                                            ast::Operand_kinds::Operand_kind::Category::literal,
                                            ast::Operand_kinds::Operand_kind::Category::composite>);
        });
    std::string kind = get_object_member_or_throw_parse_error(
        value.location,
        operand_kind_object,
        parent_path_builder,
        kind_name,
        [&](json::ast::Value &entry_value, const Path_builder_base *path_builder)
        {
            auto retval = parse_identifier_string(std::move(entry_value), path_builder, kind_name);
            if(category == ast::Operand_kinds::Operand_kind::Category::literal
               && !ast::Operand_kinds::Operand_kind::get_literal_kind_from_json_name(retval))
                throw Parse_error(
                    entry_value.location, path_builder->path(), "unknown literal kind");
            return retval;
        });
    util::optional<ast::Operand_kinds::Operand_kind::Value> operand_kind_value;
    for(auto &entry : operand_kind_object.values)
    {
        const auto &key = std::get<0>(entry);
        auto &entry_value = std::get<1>(entry);
        Path_builder<std::string> path_builder(&key, parent_path_builder);
        if(key == ast::Operand_kinds::Operand_kind::get_value_json_key_name_from_category(category))
        {
            switch(category)
            {
            case ast::Operand_kinds::Operand_kind::Category::bit_enum:
            case ast::Operand_kinds::Operand_kind::Category::value_enum:
                operand_kind_value = parse_operand_kinds_operand_kind_enumerants(
                    std::move(entry_value),
                    &path_builder,
                    category == ast::Operand_kinds::Operand_kind::Category::bit_enum);
                break;
            case ast::Operand_kinds::Operand_kind::Category::id:
            case ast::Operand_kinds::Operand_kind::Category::literal:
                if(entry_value.get_value_kind() != json::ast::Value_kind::string)
                    throw Parse_error(
                        entry_value.location, path_builder.path(), "doc is not a string");
                operand_kind_value = ast::Operand_kinds::Operand_kind::Doc(
                    std::move(entry_value.get_string().value));
                break;
            case ast::Operand_kinds::Operand_kind::Category::composite:
            {
                if(entry_value.get_value_kind() != json::ast::Value_kind::array)
                    throw Parse_error(
                        entry_value.location, path_builder.path(), "bases is not an array");
                auto &bases_array = entry_value.get_array();
                std::vector<std::string> bases;
                bases.reserve(bases_array.values.size());
                for(std::size_t i = 0; i < bases_array.values.size(); i++)
                {
                    Path_builder<std::size_t> path_builder2(&i, &path_builder);
                    auto &entry = bases_array.values[i];
                    if(entry.get_value_kind() != json::ast::Value_kind::string)
                        throw Parse_error(entry_value.location,
                                          path_builder.path(),
                                          "bases element is not a string");
                    bases.push_back(std::move(entry.get_string().value));
                }
                operand_kind_value = ast::Operand_kinds::Operand_kind::Bases(std::move(bases));
                break;
            }
            }
        }
        else if(key != category_name && key != kind_name)
        {
            throw Parse_error(entry_value.location, path_builder.path(), "unknown key");
        }
    }
    return ast::Operand_kinds::Operand_kind(
        category,
        std::move(kind),
        get_value_or_throw_parse_error(
            std::move(operand_kind_value),
            value.location,
            parent_path_builder,
            std::string("missing ")
                + ast::Operand_kinds::Operand_kind::get_value_json_key_name_from_category(
                      category)));
}

ast::Operand_kinds parse_operand_kinds(json::ast::Value value,
                                       const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::array)
        throw Parse_error(
            value.location, parent_path_builder->path(), "operand_kinds is not an array");
    auto &operand_kinds_array = value.get_array();
    std::vector<ast::Operand_kinds::Operand_kind> operand_kinds;
    operand_kinds.reserve(operand_kinds_array.values.size());
    for(std::size_t index = 0; index < operand_kinds_array.values.size(); index++)
    {
        Path_builder<std::size_t> path_builder(&index, parent_path_builder);
        operand_kinds.push_back(parse_operand_kinds_operand_kind(
            std::move(operand_kinds_array.values[index]), &path_builder));
    }
    return ast::Operand_kinds(std::move(operand_kinds));
}

ast::Instructions::Instruction::Operands::Operand parse_instructions_instruction_operands_operand(
    json::ast::Value value, const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::object)
        throw Parse_error(value.location, parent_path_builder->path(), "operand is not an object");
    auto &operand_object = value.get_object();
    constexpr auto kind_name = "kind";
    std::string kind = get_object_member_or_throw_parse_error(
        value.location,
        operand_object,
        parent_path_builder,
        kind_name,
        [&](json::ast::Value &entry_value, const Path_builder_base *path_builder)
        {
            return parse_identifier_string(std::move(entry_value), path_builder, kind_name);
        });
    std::string name;
    auto quantifier = ast::Instructions::Instruction::Operands::Operand::Quantifier::none;
    for(auto &entry : operand_object.values)
    {
        const auto &key = std::get<0>(entry);
        auto &entry_value = std::get<1>(entry);
        Path_builder<std::string> path_builder(&key, parent_path_builder);
        if(key == "name")
        {
            if(entry_value.get_value_kind() != json::ast::Value_kind::string)
                throw Parse_error(
                    entry_value.location, path_builder.path(), "name is not a string");
            name = std::move(entry_value.get_string().value);
        }
        else if(key == "quantifier")
        {
            quantifier = parse_enum_string(
                std::move(entry_value),
                &path_builder,
                "quantifier",
                make_enum_value_descriptors<ast::Instructions::Instruction::Operands::Operand::
                                                Quantifier,
                                            ast::Instructions::Instruction::Operands::Operand::
                                                get_quantifier_string,
                                            ast::Instructions::Instruction::Operands::Operand::
                                                Quantifier::none,
                                            ast::Instructions::Instruction::Operands::Operand::
                                                Quantifier::optional,
                                            ast::Instructions::Instruction::Operands::Operand::
                                                Quantifier::variable>);
        }
        else if(key != kind_name)
        {
            throw Parse_error(entry_value.location, path_builder.path(), "unknown key");
        }
    }
    return ast::Instructions::Instruction::Operands::Operand(
        std::move(kind), std::move(name), quantifier);
}

ast::Instructions::Instruction::Operands parse_instructions_instruction_operands(
    json::ast::Value value, const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::array)
        throw Parse_error(value.location, parent_path_builder->path(), "operands is not an array");
    auto &operands_array = value.get_array();
    std::vector<ast::Instructions::Instruction::Operands::Operand> operands;
    operands.reserve(operands_array.values.size());
    for(std::size_t index = 0; index < operands_array.values.size(); index++)
    {
        Path_builder<std::size_t> path_builder(&index, parent_path_builder);
        operands.push_back(parse_instructions_instruction_operands_operand(
            std::move(operands_array.values[index]), &path_builder));
    }
    return ast::Instructions::Instruction::Operands(std::move(operands));
}

ast::Instructions::Instruction parse_instructions_instruction(
    json::ast::Value value, const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::object)
        throw Parse_error(
            value.location, parent_path_builder->path(), "instruction is not an object");
    auto &instruction_object = value.get_object();
    constexpr auto opname_name = "opname";
    std::string opname = get_object_member_or_throw_parse_error(
        value.location,
        instruction_object,
        parent_path_builder,
        opname_name,
        [&](json::ast::Value &entry_value, const Path_builder_base *path_builder)
        {
            return parse_identifier_string(std::move(entry_value), path_builder, opname_name);
        });
    constexpr auto opcode_name = "opcode";
    auto opcode = get_object_member_or_throw_parse_error(
        value.location,
        instruction_object,
        parent_path_builder,
        opcode_name,
        [&](json::ast::Value &entry_value, const Path_builder_base *path_builder)
        {
            return parse_integer<std::uint32_t>(std::move(entry_value), path_builder, opcode_name);
        });
    ast::Instructions::Instruction::Operands operands;
    ast::Capabilities capabilities;
    ast::Extensions extensions;
    for(auto &entry : instruction_object.values)
    {
        const auto &key = std::get<0>(entry);
        auto &entry_value = std::get<1>(entry);
        Path_builder<std::string> path_builder(&key, parent_path_builder);
        if(key == "operands")
        {
            operands =
                parse_instructions_instruction_operands(std::move(entry_value), &path_builder);
        }
        else if(key == "capabilities")
        {
            capabilities = parse_capabilities(std::move(entry_value), &path_builder);
        }
        else if(key == "extensions")
        {
            extensions = parse_extensions(std::move(entry_value), &path_builder);
        }
        else if(key != opname_name && key != opcode_name)
        {
            throw Parse_error(entry_value.location, path_builder.path(), "unknown key");
        }
    }
    return ast::Instructions::Instruction(std::move(opname),
                                          opcode,
                                          std::move(operands),
                                          std::move(capabilities),
                                          std::move(extensions));
}

ast::Instructions parse_instructions(json::ast::Value value,
                                     const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::array)
        throw Parse_error(
            value.location, parent_path_builder->path(), "instructions is not an array");
    auto &instructions_array = value.get_array();
    std::vector<ast::Instructions::Instruction> instructions;
    instructions.reserve(instructions_array.values.size());
    for(std::size_t index = 0; index < instructions_array.values.size(); index++)
    {
        Path_builder<std::size_t> path_builder(&index, parent_path_builder);
        instructions.push_back(parse_instructions_instruction(
            std::move(instructions_array.values[index]), &path_builder));
    }
    return ast::Instructions(std::move(instructions));
}

ast::Extension_instruction_set parse_extension_instruction_set(json::ast::Value top_level_value,
                                                               std::string file_name,
                                                               std::string import_name)
{
    util::string_view file_name_prefix = ast::Extension_instruction_set::json_file_name_prefix;
    util::string_view file_name_suffix = ast::Extension_instruction_set::json_file_name_suffix;
    if(file_name.size() <= file_name_prefix.size() + file_name_suffix.size()
       || util::string_view(file_name).compare(0, file_name_prefix.size(), file_name_prefix) != 0
       || util::string_view(file_name).compare(
              file_name.size() - file_name_suffix.size(), file_name_suffix.size(), file_name_suffix)
              != 0)
        throw Parse_error(top_level_value.location, {}, "file name is unrecognizable");
    auto instruction_set_name = file_name;
    instruction_set_name.erase(instruction_set_name.size() - file_name_suffix.size(),
                               file_name_suffix.size());
    instruction_set_name.erase(0, file_name_prefix.size());
    if(top_level_value.get_value_kind() != json::ast::Value_kind::object)
        throw Parse_error(top_level_value.location, {}, "top level value is not an object");
    auto &top_level_object = top_level_value.get_object();
    util::optional<ast::Copyright> copyright;
    util::optional<std::size_t> version;
    util::optional<std::size_t> revision;
    util::optional<ast::Instructions> instructions;
    for(auto &entry : top_level_object.values)
    {
        const auto &key = std::get<0>(entry);
        auto &entry_value = std::get<1>(entry);
        Path_builder<std::string> path_builder(&key, nullptr);
        if(key == "copyright")
        {
            copyright = parse_copyright(std::move(entry_value), &path_builder);
        }
        else if(key == "version")
        {
            version = parse_integer<std::size_t>(entry_value, &path_builder, "version");
        }
        else if(key == "revision")
        {
            revision = parse_integer<std::size_t>(entry_value, &path_builder, "revision");
        }
        else if(key == "instructions")
        {
            instructions = parse_instructions(std::move(entry_value), &path_builder);
        }
        else
        {
            throw Parse_error(entry_value.location, path_builder.path(), "unknown key");
        }
    }
    auto retval = ast::Extension_instruction_set(
        std::move(instruction_set_name),
        import_name,
        get_value_or_throw_parse_error(
            std::move(copyright), top_level_value.location, nullptr, "missing copyright"),
        get_value_or_throw_parse_error(
            version, top_level_value.location, nullptr, "missing version"),
        get_value_or_throw_parse_error(
            revision, top_level_value.location, nullptr, "missing revision"),
        get_value_or_throw_parse_error(
            std::move(instructions), top_level_value.location, nullptr, "missing instructions"));
    std::cerr << "Parsed extension instruction set: " << import_name << " from " << file_name
              << std::endl;
    return retval;
}
}

std::shared_ptr<std::vector<ast::Json_file>> read_required_files(
    const util::filesystem::path &dir_path)
{
    struct Result_holder
    {
        std::vector<ast::Json_file> retval;
        std::list<json::Source> sources;
    };
    auto result_holder = std::make_shared<Result_holder>();
    auto retval =
        std::shared_ptr<std::vector<ast::Json_file>>(result_holder, &result_holder->retval);
    auto &sources = result_holder->sources;
    retval->push_back(ast::Json_file(
        ast::Top_level::core_grammar_json_file_name, json::ast::Value({}, nullptr), {}));
    util::string_view extension_grammar_prefix =
        ast::Extension_instruction_set::json_file_name_prefix;
    util::string_view extension_grammar_suffix =
        ast::Extension_instruction_set::json_file_name_suffix;
    for(auto &entry : util::filesystem::directory_iterator(dir_path))
    {
        auto filename = entry.path().filename().string();
        if(filename == ast::Top_level::core_grammar_json_file_name)
        {
            // already added; just check file type
        }
        else if(filename.size() > extension_grammar_prefix.size() + extension_grammar_suffix.size()
                && util::string_view(filename)
                           .compare(0, extension_grammar_prefix.size(), extension_grammar_prefix)
                       == 0
                && util::string_view(filename)
                           .compare(filename.size() - extension_grammar_suffix.size(),
                                    extension_grammar_suffix.size(),
                                    extension_grammar_suffix)
                       == 0)
        {
            util::string_view instruction_set_name = filename;
            instruction_set_name.remove_prefix(extension_grammar_prefix.size());
            instruction_set_name.remove_suffix(extension_grammar_suffix.size());
            auto import_name =
                ast::Extension_instruction_set::get_import_name_from_instruction_set_name(
                    instruction_set_name);
            if(!import_name)
            {
                std::cerr << "Warning: unknown extended instruction set grammar file -- ignored: "
                          << entry.path() << std::endl;
                continue;
            }
            retval->push_back(ast::Json_file(
                std::move(filename), json::ast::Value({}, nullptr), std::move(*import_name)));
        }
        else
            continue;
        if(!entry.is_regular_file())
            throw Parse_error({}, {}, "file is not a regular file: " + entry.path().string());
    }
    for(auto &file : *retval)
    {
        sources.push_back(json::Source::load_file(dir_path / file.file_name));
        auto &source = sources.back();
        file.json = json::parse(&source);
    }
    return retval;
}

ast::Top_level parse(std::vector<ast::Json_file> &&json_files)
{
    util::optional<json::ast::Value> top_level_value;
    std::vector<ast::Extension_instruction_set> extension_instruction_sets;
    if(!json_files.empty())
        extension_instruction_sets.reserve(json_files.size() - 1);
    for(auto &file : json_files)
    {
        if(file.extension_instruction_set_import_name)
            extension_instruction_sets.push_back(parse_extension_instruction_set(
                std::move(file.json),
                std::move(file.file_name),
                std::move(*file.extension_instruction_set_import_name)));
        else if(top_level_value)
            throw Parse_error(top_level_value->location, {}, "multiple core grammar files");
        else
            top_level_value = std::move(file.json);
    }
    if(!top_level_value)
        throw Parse_error(top_level_value->location, {}, "no core grammar file");
    if(top_level_value->get_value_kind() != json::ast::Value_kind::object)
        throw Parse_error(top_level_value->location, {}, "top level value is not an object");
    auto &top_level_object = top_level_value->get_object();
    util::optional<ast::Copyright> copyright;
    util::optional<std::uint32_t> magic_number;
    util::optional<std::size_t> major_version;
    util::optional<std::size_t> minor_version;
    util::optional<std::size_t> revision;
    util::optional<ast::Instructions> instructions;
    util::optional<ast::Operand_kinds> operand_kinds;
    for(auto &entry : top_level_object.values)
    {
        const auto &key = std::get<0>(entry);
        auto &entry_value = std::get<1>(entry);
        Path_builder<std::string> path_builder(&key, nullptr);
        if(key == "copyright")
        {
            copyright = parse_copyright(std::move(entry_value), &path_builder);
        }
        else if(key == "magic_number")
        {
            magic_number = parse_hex_integer_string<std::uint32_t>(
                entry_value, &path_builder, "magic_number", 1, 8);
        }
        else if(key == "major_version")
        {
            major_version = parse_integer<std::size_t>(entry_value, &path_builder, "major_version");
        }
        else if(key == "minor_version")
        {
            minor_version = parse_integer<std::size_t>(entry_value, &path_builder, "minor_version");
        }
        else if(key == "revision")
        {
            revision = parse_integer<std::size_t>(entry_value, &path_builder, "revision");
        }
        else if(key == "instructions")
        {
            instructions = parse_instructions(std::move(entry_value), &path_builder);
        }
        else if(key == "operand_kinds")
        {
            operand_kinds = parse_operand_kinds(std::move(entry_value), &path_builder);
        }
        else
        {
            throw Parse_error(entry_value.location, path_builder.path(), "unknown key");
        }
    }
    return ast::Top_level(
        get_value_or_throw_parse_error(
            std::move(copyright), top_level_value->location, nullptr, "missing copyright"),
        get_value_or_throw_parse_error(
            magic_number, top_level_value->location, nullptr, "missing magic_number"),
        get_value_or_throw_parse_error(
            major_version, top_level_value->location, nullptr, "missing major_version"),
        get_value_or_throw_parse_error(
            minor_version, top_level_value->location, nullptr, "missing minor_version"),
        get_value_or_throw_parse_error(
            revision, top_level_value->location, nullptr, "missing revision"),
        get_value_or_throw_parse_error(
            std::move(instructions), top_level_value->location, nullptr, "missing instructions"),
        get_value_or_throw_parse_error(
            std::move(operand_kinds), top_level_value->location, nullptr, "missing operand_kinds"),
        std::move(extension_instruction_sets));
}

#if 0
namespace
{
void test_fn()
{
    try
    {
        std::size_t path_index = 0;
        Path_builder<std::size_t> path_builder(&path_index, nullptr);
        std::cout << parse_hex_integer_string<std::uint32_t>(
                      json::ast::Value({}, "0x1234"), &path_builder, "test", 1, 8)
                  << std::endl;
    }
    catch(std::exception &e)
    {
        std::cout << e.what() << std::endl;
    }
}

struct Test
{
    Test()
    {
        test_fn();
        std::exit(0);
    }
} test;
}
#endif
}
}
}
