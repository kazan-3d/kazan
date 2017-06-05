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
#include "../util/optional.h"
#include <sstream>
#include <limits>

namespace vulkan_cpu
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
           || (retval = max_value / base && static_cast<unsigned>(digit) > max_value % base))
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
                                    const char *name)
{
    if(value.get_value_kind() != json::ast::Value_kind::string)
        throw Parse_error(
            value.location, parent_path_builder->path(), std::string(name) + " is not a string");
    auto &string_value = value.get_string();
    if(string_value.value.empty())
        throw Parse_error(value.location,
                          parent_path_builder->path(),
                          std::string(name) + " must not be an empty string");
    if(!is_identifier_start(string_value.value[0]))
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

ast::Operand_kinds::Operand_kind::Enumerants::Enumerant
    parse_operand_kinds_operand_kind_enumerants_enumerant(
        json::ast::Value value, const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::object)
        throw Parse_error(
            value.location, parent_path_builder->path(), "enumerant is not an object");
    auto &enumerant_object = value.get_object();
    static_cast<void>(enumerant_object);
#warning finish
    return ast::Operand_kinds::Operand_kind::Enumerants::Enumerant();
}

ast::Operand_kinds::Operand_kind::Enumerants parse_operand_kinds_operand_kind_enumerants(
    json::ast::Value value, const Path_builder_base *parent_path_builder)
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
            std::move(enumerants_array.values[index]), &path_builder));
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
            return parse_identifier_string(std::move(entry_value), path_builder, kind_name);
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
                    std::move(entry_value), &path_builder);
                break;
            case ast::Operand_kinds::Operand_kind::Category::id:
            case ast::Operand_kinds::Operand_kind::Category::literal:
                if(entry_value.get_value_kind() != json::ast::Value_kind::string)
                    throw Parse_error(
                        entry_value.location, path_builder.path(), "doc is not a string");
                operand_kind_value = ast::Operand_kinds::Operand_kind::Doc{
                    std::move(entry_value.get_string().value)};
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
                operand_kind_value = ast::Operand_kinds::Operand_kind::Bases{std::move(bases)};
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

ast::Instructions parse_instructions(json::ast::Value value,
                                     const Path_builder_base *parent_path_builder)
{
    if(value.get_value_kind() != json::ast::Value_kind::array)
        throw Parse_error(
            value.location, parent_path_builder->path(), "instructions is not an array");
    auto &instructions_array = value.get_array();
    static_cast<void>(instructions_array);
#warning finish
    return ast::Instructions();
}
}

ast::Top_level parse(json::ast::Value &&top_level_value)
{
    if(top_level_value.get_value_kind() != json::ast::Value_kind::object)
        throw Parse_error(top_level_value.location, {}, "top level value is not an object");
    auto &top_level_object = top_level_value.get_object();
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
            std::move(copyright), top_level_value.location, nullptr, "missing copyright"),
        get_value_or_throw_parse_error(
            magic_number, top_level_value.location, nullptr, "missing magic_number"),
        get_value_or_throw_parse_error(
            major_version, top_level_value.location, nullptr, "missing major_version"),
        get_value_or_throw_parse_error(
            minor_version, top_level_value.location, nullptr, "missing minor_version"),
        get_value_or_throw_parse_error(
            revision, top_level_value.location, nullptr, "missing revision"),
        get_value_or_throw_parse_error(
            std::move(instructions), top_level_value.location, nullptr, "missing instructions"),
        get_value_or_throw_parse_error(
            std::move(operand_kinds), top_level_value.location, nullptr, "missing operand_kinds"));
}
}
}
}