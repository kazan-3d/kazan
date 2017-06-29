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
#include "ast.h"

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
namespace ast
{
constexpr const char *Extension_instruction_set::json_file_name_prefix;
constexpr const char *Extension_instruction_set::json_file_name_suffix;

constexpr const char *Top_level::core_grammar_json_file_name;

namespace
{
std::string to_hex_string(std::uint32_t v, std::size_t min_digit_count)
{
    return json::ast::Number_value::append_unsigned_integer_to_string(
        v, "0x", 0x10, min_digit_count);
}

constexpr json::Location make_empty_location() noexcept
{
    // use function to make empty location so it will be easy to find all occurrences if location
    // info is added to ast
    return {};
}
}

json::ast::Value Copyright::to_json() const
{
    json::ast::Array retval;
    retval.values.reserve(lines.size());
    for(auto &line : lines)
        retval.values.push_back(json::ast::Value(make_empty_location(), line));
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value Instructions::Instruction::Operands::Operand::to_json() const
{
    json::ast::Object retval;
    retval.values["kind"] = json::ast::Value(make_empty_location(), kind);
    if(!name.empty())
        retval.values["name"] = json::ast::Value(make_empty_location(), name);
    if(quantifier != Quantifier::none)
        retval.values["quantifier"] =
            json::ast::Value(make_empty_location(), get_quantifier_string(quantifier));
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value Instructions::Instruction::Operands::to_json() const
{
    json::ast::Array retval;
    retval.values.reserve(operands.size());
    for(auto &operand : operands)
        retval.values.push_back(operand.to_json());
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value Instructions::Instruction::to_json() const
{
    json::ast::Object retval;
    retval.values["opname"] = json::ast::Value(make_empty_location(), opname);
    retval.values["opcode"] = json::ast::Value(make_empty_location(), opcode);
    if(!operands.empty())
        retval.values["operands"] = operands.to_json();
    if(!capabilities.empty())
        retval.values["capabilities"] = capabilities.to_json();
    if(!extensions.empty())
        retval.values["extensions"] = extensions.to_json();
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value Instructions::to_json() const
{
    json::ast::Array retval;
    retval.values.reserve(instructions.size());
    for(auto &instruction : instructions)
        retval.values.push_back(instruction.to_json());
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value Capabilities::to_json() const
{
    json::ast::Array retval;
    retval.values.reserve(capabilities.size());
    for(auto &capability : capabilities)
        retval.values.push_back(json::ast::Value(make_empty_location(), capability));
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value Extensions::to_json() const
{
    json::ast::Array retval;
    retval.values.reserve(extensions.size());
    for(auto &extension : extensions)
        retval.values.push_back(json::ast::Value(make_empty_location(), extension));
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value
    Operand_kinds::Operand_kind::Enumerants::Enumerant::Parameters::Parameter::to_json() const
{
    json::ast::Object retval;
    retval.values["kind"] = json::ast::Value(make_empty_location(), kind);
    if(!name.empty())
        retval.values["name"] = json::ast::Value(make_empty_location(), name);
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value Operand_kinds::Operand_kind::Enumerants::Enumerant::Parameters::to_json() const
{
    json::ast::Array retval;
    retval.values.reserve(parameters.size());
    for(auto &parameter : parameters)
        retval.values.push_back(parameter.to_json());
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value Operand_kinds::Operand_kind::Enumerants::Enumerant::to_json(
    bool is_bit_enumerant) const
{
    json::ast::Object retval;
    retval.values["enumerant"] = json::ast::Value(make_empty_location(), enumerant);
    json::ast::Value value_out;
    if(is_bit_enumerant)
        value_out = json::ast::Value(make_empty_location(), to_hex_string(value, 4));
    else
        value_out = json::ast::Value(make_empty_location(), value);
    retval.values["value"] = value_out;
    if(!capabilities.empty())
        retval.values["capabilities"] = capabilities.to_json();
    if(!parameters.empty())
        retval.values["parameters"] = parameters.to_json();
    if(!extensions.empty())
        retval.values["extensions"] = extensions.to_json();
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value Operand_kinds::Operand_kind::Enumerants::to_json(bool is_bit_enumerant) const
{
    json::ast::Array retval;
    retval.values.reserve(enumerants.size());
    for(auto &enumerant : enumerants)
        retval.values.push_back(enumerant.to_json(is_bit_enumerant));
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value Operand_kinds::Operand_kind::Doc::to_json() const
{
    return json::ast::Value(make_empty_location(), value);
}

json::ast::Value Operand_kinds::Operand_kind::Bases::to_json() const
{
    json::ast::Array retval;
    retval.values.reserve(values.size());
    for(auto &value : values)
        retval.values.push_back(json::ast::Value(make_empty_location(), value));
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value Operand_kinds::Operand_kind::to_json() const
{
    json::ast::Object retval;
    retval.values["category"] =
        json::ast::Value(make_empty_location(), get_json_name_from_category(category));
    retval.values["kind"] = json::ast::Value(make_empty_location(), kind);
    retval.values[get_value_json_key_name_from_category(category)] = util::visit(
        [&](auto &v) -> json::ast::Value
        {
            return v.to_json(category);
        },
        value);
    return json::ast::Value(make_empty_location(), std::move(retval));
}

json::ast::Value Operand_kinds::to_json() const
{
    json::ast::Array retval;
    retval.values.reserve(operand_kinds.size());
    for(auto &operand_kind : operand_kinds)
        retval.values.push_back(operand_kind.to_json());
    return json::ast::Value(make_empty_location(), std::move(retval));
}

util::optional<std::string> Extension_instruction_set::get_import_name_from_instruction_set_name(
    util::string_view instruction_set_name)
{
    if(instruction_set_name == "glsl.std.450")
        return "GLSL.std.450";
    if(instruction_set_name == "opencl.std.100")
        return "OpenCL.std";
    return {};
}

Json_file Extension_instruction_set::to_json() const
{
    json::ast::Object grammar;
    grammar.values["copyright"] = copyright.to_json();
    grammar.values["version"] = json::ast::Value(make_empty_location(), version);
    grammar.values["revision"] = json::ast::Value(make_empty_location(), revision);
    grammar.values["instructions"] = instructions.to_json();
    auto file_name = json_file_name_prefix + instruction_set_name + json_file_name_suffix;
    for(char &ch : file_name)
    {
        if(ch >= 'A' && ch <= 'Z')
            ch = ch - 'A' + 'a'; // to lower
    }
    return Json_file(std::move(file_name),
                     json::ast::Value(make_empty_location(), std::move(grammar)),
                     import_name);
}

std::vector<Json_file> Top_level::to_json() const
{
    json::ast::Object core_grammar;
    core_grammar.values["copyright"] = copyright.to_json();
    core_grammar.values["magic_number"] =
        json::ast::Value(make_empty_location(), to_hex_string(magic_number, 8));
    core_grammar.values["major_version"] = json::ast::Value(make_empty_location(), major_version);
    core_grammar.values["minor_version"] = json::ast::Value(make_empty_location(), minor_version);
    core_grammar.values["revision"] = json::ast::Value(make_empty_location(), revision);
    core_grammar.values["instructions"] = instructions.to_json();
    core_grammar.values["operand_kinds"] = operand_kinds.to_json();
    std::vector<Json_file> retval;
    retval.reserve(extension_instruction_sets.size() + 1);
    retval.push_back(Json_file("spirv.core.grammar.json",
                               json::ast::Value(make_empty_location(), std::move(core_grammar)),
                               util::nullopt));
    for(auto &extension_instruction_set : extension_instruction_sets)
        retval.push_back(extension_instruction_set.to_json());
    return retval;
}
}
}
}
