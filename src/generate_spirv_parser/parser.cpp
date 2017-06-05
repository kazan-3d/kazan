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
std::string path::to_string() const
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
            json::ast::string_value::write(ss, util::get<std::string>(e));
        }
        ss << ']';
    }
    return ss.str();
}

namespace
{
template <typename Value, std::size_t N>
Value get_value_or_throw_parse_error(util::optional<Value> value,
                                     path_builder_base *path_builder,
                                     const char(&message)[N])
{
    if(value)
        return std::move(*value);
    throw parse_error(path_builder ? path_builder->path() : path{}, message);
}

ast::copyright parse_copyright(json::ast::value value, const path_builder_base *parent_path_builder)
{
    if(json::ast::get_value_kind(value) != json::ast::value_kind::array)
        throw parse_error(parent_path_builder->path(), "copyright is not an array");
    auto &copyright_array =
        static_cast<json::ast::array &>(*util::get<json::ast::composite_value_pointer>(value));
    for(std::size_t index = 0; index < copyright_array.values.size(); index++)
    {
        path_builder<std::size_t> path_builder(&index, parent_path_builder);
        auto &element = copyright_array.values[index];
        if(json::ast::get_value_kind(element) != json::ast::value_kind::string)
            throw parse_error(parent_path_builder->path(),
                              "copyright array's element is not a string");
    }
    return ast::copyright(std::move(copyright_array));
}

ast::operand_kinds parse_operand_kinds(json::ast::value value,
                                       const path_builder_base *parent_path_builder)
{
    if(json::ast::get_value_kind(value) != json::ast::value_kind::array)
        throw parse_error(parent_path_builder->path(), "operand_kinds is not an array");
    auto &operand_kinds_array =
        static_cast<json::ast::array &>(*util::get<json::ast::composite_value_pointer>(value));
    static_cast<void>(operand_kinds_array);
#warning finish
    return ast::operand_kinds();
}

ast::instructions parse_instructions(json::ast::value value,
                                     const path_builder_base *parent_path_builder)
{
    if(json::ast::get_value_kind(value) != json::ast::value_kind::array)
        throw parse_error(parent_path_builder->path(), "instructions is not an array");
    auto &instructions_array =
        static_cast<json::ast::array &>(*util::get<json::ast::composite_value_pointer>(value));
    static_cast<void>(instructions_array);
#warning finish
    return ast::instructions();
}

template <typename T>
T parse_integer(const json::ast::value &value,
                const path_builder_base *parent_path_builder,
                const char *name)
{
    if(json::ast::get_value_kind(value) != json::ast::value_kind::number)
        throw parse_error(parent_path_builder->path(), std::string(name) + " is not a number");
    auto number_value = util::get<json::ast::number_value>(value);
    T retval = number_value.value;
    if(retval != number_value.value) // not an exact value
        throw parse_error(parent_path_builder->path(), std::string(name) + " is not an integer");
    return retval;
}

constexpr int get_digit_value(int ch, unsigned base) noexcept
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

template <typename T>
T parse_hex_integer_string(const json::ast::value &value,
                           const path_builder_base *parent_path_builder,
                           const char *name,
                           std::size_t min_length,
                           std::size_t max_length)
{
    if(json::ast::get_value_kind(value) != json::ast::value_kind::string)
        throw parse_error(parent_path_builder->path(), std::string(name) + " is not a string");
    auto &string_value = util::get<json::ast::string_value>(value);
    constexpr std::size_t hex_number_prefix_length = 2; // std::strlen("0x")
    if(string_value.value.size() < hex_number_prefix_length || string_value.value[0] != '0'
       || (string_value.value[1] != 'x' && string_value.value[1] != 'X'))
        throw parse_error(parent_path_builder->path(),
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
            throw parse_error(parent_path_builder->path(),
                              std::string(name) + ": not a valid hex digit");
        if(digit_count > max_length)
            throw parse_error(parent_path_builder->path(),
                              std::string(name) + " has too many digits");
        if(retval > max_value / base
           || (retval = max_value / base && static_cast<unsigned>(digit) > max_value % base))
            throw parse_error(parent_path_builder->path(), std::string(name) + ": value too big");
        retval *= base;
        retval += digit;
    }
    if(digit_count < min_length)
        throw parse_error(parent_path_builder->path(),
                          std::string(name) + " doesn't have enough digits");
    return retval;
}
}

ast::top_level parse(json::ast::value &&top_level_value)
{
    if(json::ast::get_value_kind(top_level_value) != json::ast::value_kind::object)
        throw parse_error({}, "top level value is not an object");
    auto &top_level_object = static_cast<const json::ast::object &>(
        *util::get<json::ast::composite_value_pointer>(top_level_value));
    util::optional<ast::copyright> copyright;
    util::optional<std::uint32_t> magic_number;
    util::optional<std::size_t> major_version;
    util::optional<std::size_t> minor_version;
    util::optional<std::size_t> revision;
    util::optional<ast::instructions> instructions;
    util::optional<ast::operand_kinds> operand_kinds;
    for(auto &entry : top_level_object.values)
    {
        const auto &key = std::get<0>(entry);
        auto &entry_value = std::get<1>(entry);
        path_builder<std::string> path_builder(&key, nullptr);
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
            throw parse_error(path_builder.path(), "unknown key");
        }
    }
    auto retval = ast::top_level(
        get_value_or_throw_parse_error(std::move(copyright), nullptr, "missing copyright"),
        get_value_or_throw_parse_error(magic_number, nullptr, "missing magic_number"),
        get_value_or_throw_parse_error(major_version, nullptr, "missing major_version"),
        get_value_or_throw_parse_error(minor_version, nullptr, "missing minor_version"),
        get_value_or_throw_parse_error(revision, nullptr, "missing revision"),
        get_value_or_throw_parse_error(instructions, nullptr, "missing instructions"),
        get_value_or_throw_parse_error(operand_kinds, nullptr, "missing operand_kinds"));
    throw parse_error({}, "not finished implementing");
}
}
}
}