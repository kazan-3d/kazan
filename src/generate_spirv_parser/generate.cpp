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
#include "generate.h"
#include "json/json.h"
#include <fstream>
#include <cassert>
#include <limits>
#include <unordered_map>
#include <unordered_set>
#include <set>
#include <list>
#include <deque>

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
namespace generate
{
constexpr std::size_t detail::Generated_output_stream::output_tab_width_no_tabs_allowed;
constexpr util::string_view detail::Generated_output_stream::literal_command;
constexpr char detail::Generated_output_stream::indent_indicator_char;
constexpr char detail::Generated_output_stream::literal_indent_indicator_char;
constexpr std::size_t detail::Generated_output_stream::indent_indicators_per_indent;
constexpr char detail::Generated_output_stream::escape_char;
constexpr bool detail::Generated_output_stream::indent_blank_lines;

void detail::Generated_output_stream::write_to_file(bool do_reindent) const
{
    std::ofstream os;
    os.exceptions(std::ios::badbit);
    os.open(file_path.c_str());
    if(!os)
        throw util::filesystem::filesystem_error(
            "open failed", file_path, std::make_error_code(std::io_errc::stream));
    os.exceptions(std::ios::badbit | std::ios::failbit);
    if(do_reindent)
    {
        auto iter = value.begin();
        bool is_at_start_of_line = true;
        std::size_t start_indent_depth = 0;
        std::size_t indent_depth = 0;
        constexpr std::size_t output_indent_width = 4;
        constexpr std::size_t output_tab_width = output_tab_width_no_tabs_allowed;
        while(iter != value.end())
        {
            if(*iter == '\n')
            {
                if(indent_blank_lines && is_at_start_of_line)
                    write_indent(
                        [&](char ch)
                        {
                            os << ch;
                        },
                        indent_depth,
                        output_tab_width);
                is_at_start_of_line = true;
                indent_depth = start_indent_depth;
                os << *iter++;
            }
            else if(is_at_start_of_line)
            {
                switch(*iter)
                {
                case '\r':
                case '\t':
                case '\f':
                case '\0':
                    assert(false);
                    continue;
                case literal_indent_indicator_char:
                    ++iter;
                    indent_depth++;
                    continue;
                case indent_indicator_char:
                    for(std::size_t i = 0; i < indent_indicators_per_indent; i++)
                    {
                        assert(iter != value.end());
                        assert(*iter == indent_indicator_char);
                        ++iter;
                    }
                    indent_depth += output_indent_width;
                    continue;
                case escape_char:
                {
                    ++iter;
                    assert(iter != value.end());
                    if(*iter != escape_char)
                    {
                        if(*iter >= 'a' && *iter <= 'z')
                        {
                            std::string command;
                            while(true)
                            {
                                assert(iter != value.end());
                                if(*iter == escape_char)
                                    break;
                                command += *iter++;
                            }
                            assert(iter != value.end());
                            assert(*iter == escape_char);
                            ++iter;
                            auto command_sv = util::string_view(command);
                            if(command_sv.compare(0, literal_command.size(), literal_command) == 0)
                            {
                                auto arg = command_sv.substr(literal_command.size());
                                std::size_t count = 0;
                                do
                                {
                                    count *= 10;
                                    assert(!arg.empty() && arg.front() >= '0'
                                           && arg.front() <= '9');
                                    count += arg.front() - '0';
                                    arg.remove_prefix(1);
                                } while(!arg.empty());
                                write_indent(
                                    [&](char ch)
                                    {
                                        os << ch;
                                    },
                                    indent_depth,
                                    output_tab_width);
                                indent_depth = 0;
                                for(std::size_t i = 0; i < count; i++)
                                {
                                    assert(iter != value.end());
                                    os << *iter++;
                                }
                                assert(iter != value.end() && *iter == escape_char);
                                ++iter;
                                continue;
                            }
                            else
                            {
                                assert(false);
                            }
                        }
                        switch(*iter)
                        {
                        case '-':
                            ++iter;
                            assert(start_indent_depth >= output_indent_width);
                            assert(indent_depth >= output_indent_width);
                            start_indent_depth -= output_indent_width;
                            indent_depth -= output_indent_width;
                            continue;
                        case '_':
                            ++iter;
                            assert(start_indent_depth >= output_indent_width);
                            start_indent_depth -= output_indent_width;
                            continue;
                        case '+':
                            ++iter;
                            start_indent_depth += output_indent_width;
                            indent_depth += output_indent_width;
                            continue;
                        }
                        assert(false);
                        continue;
                    }
                    break;
                }
                }
                write_indent(
                    [&](char ch)
                    {
                        os << ch;
                    },
                    indent_depth,
                    output_tab_width);
                is_at_start_of_line = false;
                os << *iter++;
            }
            else
            {
                os << *iter++;
            }
        }
    }
    else
    {
        for(char ch : value)
            os << ch;
    }
    os.close(); // manually close to not hide error exceptions
}

void detail::Generated_output_stream::write_unsigned_integer(std::uint64_t value,
                                                             unsigned base,
                                                             std::size_t min_length)
{
    static_assert(std::numeric_limits<decltype(value)>::radix == 2, "");
    constexpr std::size_t buffer_size = std::numeric_limits<decltype(value)>::digits;
    char buffer[buffer_size];
    while(min_length > buffer_size)
    {
        *this << '0';
        min_length--;
    }
    std::size_t length = json::ast::Number_value::unsigned_integer_to_buffer(
        value, buffer, buffer_size, false, base, min_length);
    *this << util::string_view(buffer, length);
}

void detail::Generated_output_stream::write_signed_integer(std::int64_t value, unsigned base)
{
    static_assert(std::numeric_limits<decltype(value)>::radix == 2, "");
    constexpr std::size_t buffer_size =
        std::numeric_limits<decltype(value)>::digits + 1; // one extra for sign
    char buffer[buffer_size];
    std::size_t length =
        json::ast::Number_value::signed_integer_to_buffer(value, buffer, buffer_size, false, base);
    *this << util::string_view(buffer, length);
}

detail::Generated_output_stream &detail::Generated_output_stream::operator<<(Guard_macro)
{
    *this << name_from_words_all_uppercase_with_trailing_underline(get_file_path().string());
    return *this;
}

std::string detail::Generated_output_stream::name_from_words_helper(Name_format name_format,
                                                                    std::string name)
{
    for(char &ch : name)
        if(ch >= 'A' && ch <= 'Z')
            ch = ch - 'A' + 'a'; // to lowercase
    if(name.empty() || (name[0] >= '0' && name[0] <= '9'))
        name.insert(0, 1, '_');
    bool has_trailing_underline = false;
    switch(name_format)
    {
    case initial_capital:
        // can't be empty because of previous insert
        if(name[0] >= 'a' && name[0] <= 'z')
            name[0] = name[0] - 'a' + 'A'; // to uppercase
        break;
    case all_uppercase_with_trailing_underline:
    case all_uppercase:
        if(name_format == all_uppercase_with_trailing_underline)
            has_trailing_underline = true;
        for(char &ch : name)
            if(ch >= 'a' && ch <= 'z')
                ch = ch - 'a' + 'A'; // to uppercase
        break;
    case all_lowercase:
        break;
    }
    if(!has_trailing_underline)
    {
        for(auto &keyword : keywords)
        {
            if(name == keyword)
            {
                has_trailing_underline = true;
                break;
            }
        }
    }
    if(has_trailing_underline)
        name += '_';
    return name;
}

struct Spirv_and_parser_generator : public Generator
{
    struct State;
    virtual void run(Generator_args &generator_args,
                     const ast::Top_level &top_level) const override;
};

using namespace util::string_view_literals;

struct Spirv_and_parser_generator::State
{
private:
    const ast::Top_level &top_level;
    detail::Generated_output_stream spirv_h;
    detail::Generated_output_stream spirv_cpp;
    detail::Generated_output_stream parser_h;
    detail::Generated_output_stream parser_cpp;

public:
    State(const util::filesystem::path &output_directory, const ast::Top_level &top_level)
        : top_level(top_level),
          spirv_h(output_directory / "spirv.h"),
          spirv_cpp(output_directory / "spirv.cpp"),
          parser_h(output_directory / "parser.h"),
          parser_cpp(output_directory / "parser.cpp")
    {
    }

private:
    void write_file_comments()
    {
        constexpr auto automatically_generated_file_warning_comment =
            R"(/* This file is automatically generated by generate_spirv_parser. DO NOT MODIFY. */
)"_sv;
        spirv_h << automatically_generated_file_warning_comment << top_level.copyright;
        spirv_cpp << automatically_generated_file_warning_comment << top_level.copyright;
        parser_h << automatically_generated_file_warning_comment << top_level.copyright;
        parser_cpp << automatically_generated_file_warning_comment << top_level.copyright;
    }
    static void write_opening_inclusion_guard(detail::Generated_output_stream &os)
    {
        using detail::guard_macro;
        os << R"(#ifndef )" << guard_macro << R"(
#define )" << guard_macro
           << R"(

)";
    }
    static void write_closing_inclusion_guard(detail::Generated_output_stream &os)
    {
        using detail::guard_macro;
        os << R"(
#endif /* )"
           << guard_macro << R"( */
)";
    }
    void write_opening_inclusion_guards()
    {
        write_opening_inclusion_guard(spirv_h);
        write_opening_inclusion_guard(parser_h);
    }
    void write_closing_inclusion_guards()
    {
        write_closing_inclusion_guard(spirv_h);
        write_closing_inclusion_guard(parser_h);
    }
    static void write_local_include(detail::Generated_output_stream &os, util::string_view file)
    {
        os << R"(#include ")" << file << R"("
)";
    }
    static void write_system_include(detail::Generated_output_stream &os, util::string_view file)
    {
        os << R"(#include <)" << file << R"(>
)";
    }
    void write_includes()
    {
        write_local_include(spirv_cpp, spirv_h.get_file_path().filename().string());
        write_local_include(parser_h, spirv_h.get_file_path().filename().string());
        write_local_include(parser_cpp, parser_h.get_file_path().filename().string());
        write_system_include(spirv_h, "cstdint");
        write_system_include(spirv_h, "vector");
        write_system_include(spirv_h, "string");
        write_system_include(spirv_h, "iterator");
        write_local_include(spirv_h, "util/string_view.h");
        write_local_include(spirv_h, "util/enum.h");
        write_local_include(spirv_h, "spirv/word.h");
        write_local_include(spirv_h, "spirv/literal_string.h");
    }
    static void write_opening_namespaces(detail::Generated_output_stream &os)
    {
        os << R"(
namespace vulkan_cpu
{
namespace spirv
{
)";
    }
    void write_opening_namespaces()
    {
        write_opening_namespaces(spirv_h);
        write_opening_namespaces(spirv_cpp);
        write_opening_namespaces(parser_h);
        write_opening_namespaces(parser_cpp);
    }
    static void write_closing_namespaces(detail::Generated_output_stream &os)
    {
        os << R"(}
}
)";
    }
    void write_closing_namespaces()
    {
        write_closing_namespaces(spirv_h);
        write_closing_namespaces(spirv_cpp);
        write_closing_namespaces(parser_h);
        write_closing_namespaces(parser_cpp);
    }

private:
    static constexpr util::string_view op_enum_json_name = "Op"_sv;
    static constexpr util::string_view extension_enum_json_name = "Extension"_sv;
    static constexpr util::string_view capability_enum_json_name = "Capability"_sv;
    static constexpr util::string_view extension_instruction_set_enum_json_name =
        "Extension_instruction_set"_sv;
    static constexpr util::string_view unknown_extension_instruction_set_enumerant_json_name =
        "Unknown"_sv;
    struct Enumerant_descriptor
    {
        std::uint32_t value;
        std::string cpp_name;
        std::string json_name;
        ast::Capabilities capabilities;
        ast::Extensions extensions;
        static std::string make_cpp_name(util::string_view json_enumeration_name,
                                         util::string_view json_enumerant_name)
        {
            using detail::name_from_words_all_lowercase;
            bool starts_with_enumeration_name = false;
            if(json_enumerant_name.substr(0, json_enumeration_name.size()) == json_enumeration_name)
                starts_with_enumeration_name = true;
            bool json_name_should_have_prefix = json_enumeration_name == op_enum_json_name;
            if(json_name_should_have_prefix)
            {
                if(json_enumerant_name.substr(0, json_enumeration_name.size())
                   != json_enumeration_name)
                    return name_from_words_all_lowercase(
                               json_enumeration_name, json_enumeration_name, json_enumerant_name)
                        .to_string();
                if(json_enumerant_name.substr(json_enumeration_name.size(),
                                              json_enumeration_name.size())
                   == json_enumeration_name)
                    return name_from_words_all_lowercase(json_enumeration_name, json_enumerant_name)
                        .to_string();
                return name_from_words_all_lowercase(json_enumerant_name).to_string();
            }
            if(json_enumerant_name.empty())
                throw Generate_error("json enumerant name can't be empty");
            return name_from_words_all_lowercase(json_enumerant_name).to_string();
        }
        Enumerant_descriptor(std::uint32_t value,
                             util::string_view json_enumeration_name,
                             std::string json_name,
                             ast::Capabilities capabilities,
                             ast::Extensions extensions)
            : value(value),
              cpp_name(make_cpp_name(json_enumeration_name, json_name)),
              json_name(std::move(json_name)),
              capabilities(std::move(capabilities)),
              extensions(std::move(extensions))
        {
        }
    };
    struct Enumeration_descriptor
    {
        bool is_bitwise;
        std::string cpp_name;
        std::string json_name;
        std::list<Enumerant_descriptor> enumerants;
        typedef std::unordered_map<std::string, std::list<Enumerant_descriptor>::const_iterator>
            Json_name_to_enumerant_map;
        Json_name_to_enumerant_map json_name_to_enumerant_map;
        static Json_name_to_enumerant_map make_json_name_to_enumerant_map(
            const std::list<Enumerant_descriptor> *enumerants)
        {
            Json_name_to_enumerant_map retval;
            for(auto i = enumerants->begin(); i != enumerants->end(); ++i)
                retval[i->json_name] = i;
            return retval;
        }
        Enumeration_descriptor(bool is_bitwise,
                               std::string json_name,
                               std::list<Enumerant_descriptor> enumerants)
            : is_bitwise(is_bitwise),
              cpp_name(detail::name_from_words_initial_capital(json_name).to_string()),
              json_name(std::move(json_name)),
              enumerants(std::move(enumerants)),
              json_name_to_enumerant_map(make_json_name_to_enumerant_map(&this->enumerants))
        {
        }
    };

private:
    std::list<Enumeration_descriptor> enumerations_list;
    std::unordered_map<std::string, std::list<Enumeration_descriptor>::const_iterator>
        enumerations_map;
    util::optional<std::list<Enumeration_descriptor>::const_iterator> capability_enumeration;
    util::optional<std::list<Enumeration_descriptor>::const_iterator> extension_enumeration;
    util::optional<std::list<Enumeration_descriptor>::const_iterator> op_enumeration;
    util::optional<std::list<Enumeration_descriptor>::const_iterator>
        extension_instruction_set_enumeration;
    std::unordered_map<std::string, std::list<Enumeration_descriptor>::const_iterator>
        instruction_set_extension_op_enumeration_map;

private:
    std::list<Enumeration_descriptor>::const_iterator add_enumeration(
        Enumeration_descriptor &&enumeration_descriptor)
    {
        auto name = enumeration_descriptor.json_name;
        auto iter =
            enumerations_list.insert(enumerations_list.end(), std::move(enumeration_descriptor));
        if(!std::get<1>(enumerations_map.emplace(name, iter)))
            throw Generate_error("duplicate enumeration: " + name);
        return iter;
    }
    void fill_enumerations_helper(std::set<std::string> &extensions_set,
                                  const ast::Operand_kinds::Operand_kind &ast_operand_kind)
    {
        auto *ast_enumerants =
            util::get_if<ast::Operand_kinds::Operand_kind::Enumerants>(&ast_operand_kind.value);
        if(ast_enumerants)
        {
            std::list<Enumerant_descriptor> enumerants;
            for(auto &ast_enumerant : ast_enumerants->enumerants)
            {
                enumerants.push_back(Enumerant_descriptor(ast_enumerant.value,
                                                          ast_operand_kind.kind,
                                                          ast_enumerant.enumerant,
                                                          ast_enumerant.capabilities,
                                                          ast_enumerant.extensions));
                for(auto &extension : ast_enumerant.extensions.extensions)
                    extensions_set.insert(extension);
            }
            auto iter = add_enumeration(Enumeration_descriptor(
                ast_operand_kind.category == ast::Operand_kinds::Operand_kind::Category::bit_enum,
                ast_operand_kind.kind,
                std::move(enumerants)));
            if(ast_operand_kind.kind == capability_enum_json_name)
            {
                if(capability_enumeration)
                    throw Generate_error("Too many " + std::string(capability_enum_json_name)
                                         + " enums");
                capability_enumeration = iter;
            }
        }
    }
    void fill_enumerations()
    {
        std::set<std::string> extensions_set;
        for(auto &operand_kind : top_level.operand_kinds.operand_kinds)
            fill_enumerations_helper(extensions_set, operand_kind);
        std::list<Enumerant_descriptor> op_enumerants;
        for(auto &instruction : top_level.instructions.instructions)
        {
            op_enumerants.push_back(Enumerant_descriptor(instruction.opcode,
                                                         op_enum_json_name,
                                                         instruction.opname,
                                                         instruction.capabilities,
                                                         instruction.extensions));
            for(auto &extension : instruction.extensions.extensions)
                extensions_set.insert(extension);
        }
        op_enumeration = add_enumeration(Enumeration_descriptor(
            false, static_cast<std::string>(op_enum_json_name), std::move(op_enumerants)));
        std::list<Enumerant_descriptor> extension_instruction_set_enumerants;
        std::size_t extension_instruction_set_index = 0;
        extension_instruction_set_enumerants.push_back(Enumerant_descriptor(
            extension_instruction_set_index++,
            extension_instruction_set_enum_json_name,
            static_cast<std::string>(unknown_extension_instruction_set_enumerant_json_name),
            {},
            {}));
        for(auto &instruction_set : top_level.extension_instruction_sets)
        {
            std::string json_enumeration_name =
                std::string(op_enum_json_name) + " " + instruction_set.import_name;
            std::list<Enumerant_descriptor> enumerants;
            for(auto &instruction : instruction_set.instructions.instructions)
            {
                enumerants.push_back(Enumerant_descriptor(instruction.opcode,
                                                          json_enumeration_name,
                                                          instruction.opname,
                                                          instruction.capabilities,
                                                          instruction.extensions));
                for(auto &extension : instruction.extensions.extensions)
                    extensions_set.insert(extension);
            }
            auto iter = add_enumeration(
                Enumeration_descriptor(false, json_enumeration_name, std::move(enumerants)));
            instruction_set_extension_op_enumeration_map.emplace(instruction_set.import_name, iter);
            extension_instruction_set_enumerants.push_back(
                Enumerant_descriptor(extension_instruction_set_index++,
                                     extension_instruction_set_enum_json_name,
                                     instruction_set.import_name,
                                     {},
                                     {}));
        }
        std::list<Enumerant_descriptor> extension_enumerants;
        std::uint32_t extension_index = 0;
        for(auto &extension : extensions_set)
            extension_enumerants.push_back(Enumerant_descriptor(
                extension_index++, extension_enum_json_name, extension, {}, {}));
        extension_enumeration = add_enumeration(
            Enumeration_descriptor(false,
                                   static_cast<std::string>(extension_enum_json_name),
                                   std::move(extension_enumerants)));
        extension_instruction_set_enumeration = add_enumeration(Enumeration_descriptor(
            false,
            static_cast<std::string>(extension_instruction_set_enum_json_name),
            std::move(extension_instruction_set_enumerants)));
        if(!capability_enumeration)
            throw Generate_error("missing " + std::string(capability_enum_json_name) + " enum");
    }
    void write_basic_types()
    {
        spirv_h << R"(typedef Word Id;
)";
    }
    static std::string instruction_set_version_name(const ast::Extension_instruction_set &v)
    {
        using detail::name_from_words_all_lowercase;
        return name_from_words_all_lowercase("version"_sv, v.import_name).to_string();
    }
    static std::string instruction_set_revision_name(const ast::Extension_instruction_set &v)
    {
        using detail::name_from_words_all_lowercase;
        return name_from_words_all_lowercase("revision"_sv, v.import_name).to_string();
    }
    void write_basic_constants()
    {
        using detail::unsigned_integer;
        spirv_h << R"(
constexpr Word magic_number = 0x)"
                << unsigned_integer(top_level.magic_number, 0x10, 8) << R"(UL;
constexpr std::uint32_t major_version = )"
                << unsigned_integer(top_level.major_version) << R"(UL;
constexpr std::uint32_t minor_version = )"
                << unsigned_integer(top_level.minor_version) << R"(UL;
constexpr std::uint32_t revision = )"
                << unsigned_integer(top_level.revision) << R"(UL;
)";
        for(auto &instruction_set : top_level.extension_instruction_sets)
        {
            spirv_h << R"(
constexpr std::uint32_t )"
                    << instruction_set_version_name(instruction_set) << R"( = )"
                    << unsigned_integer(instruction_set.version) << R"(UL;
constexpr std::uint32_t )"
                    << instruction_set_revision_name(instruction_set) << R"( = )"
                    << unsigned_integer(instruction_set.revision) << R"(UL;
)";
        }
    }
    void write_enum_declarations()
    {
        spirv_h << "\n";
        for(auto &enumeration : enumerations_list)
            spirv_h << "enum class " << enumeration.cpp_name << " : Word;\n";
    }
    void write_enum_definitions()
    {
        using detail::unsigned_integer;
        for(auto &enumeration : enumerations_list)
        {
            spirv_h << R"(
enum class )" << enumeration.cpp_name
                    << R"( : Word
{
@+)";
            for(auto &enumerant : enumeration.enumerants)
            {
                spirv_h << enumerant.cpp_name << " = ";
                if(enumeration.is_bitwise)
                    spirv_h << "0x" << unsigned_integer(enumerant.value, 0x10) << "UL";
                else
                    spirv_h << unsigned_integer(enumerant.value, 10) << "UL";
                spirv_h << ",\n";
            }
            spirv_h << R"(@-};

vulkan_cpu_util_generate_enum_traits()"
                    << enumeration.cpp_name;
            for(auto &enumerant : enumeration.enumerants)
                spirv_h << R"(,
`````````````````````````````````````)"
                        << enumeration.cpp_name << "::" << enumerant.cpp_name;
            spirv_h << R"();
)";
        }
    }
    const Enumerant_descriptor &get_capability(const std::string &capability)
    {
        auto &enumerant_map = capability_enumeration.value()->json_name_to_enumerant_map;
        auto iter = enumerant_map.find(capability);
        if(iter == enumerant_map.end())
            throw Generate_error("unknown capability: " + capability);
        return *std::get<1>(*iter);
    }
    const Enumerant_descriptor &get_extension(const std::string &extension)
    {
        auto &enumerant_map = extension_enumeration.value()->json_name_to_enumerant_map;
        auto iter = enumerant_map.find(extension);
        if(iter == enumerant_map.end())
            throw Generate_error("unknown extension: " + extension);
        return *std::get<1>(*iter);
    }
    void write_enum_properties_definitions()
    {
        for(auto &enumeration : enumerations_list)
        {
            spirv_h << R"(
constexpr util::string_view get_enumerant_name()"
                    << enumeration.cpp_name << R"( v) noexcept
{
    using namespace util::string_view_literals;
    switch(v)
    {
@+@+)";
            std::unordered_set<std::uint32_t> values;
            for(auto &enumerant : enumeration.enumerants)
            {
                if(std::get<1>(values.insert(enumerant.value)))
                {
                    spirv_h << "case " << enumeration.cpp_name << "::" << enumerant.cpp_name << R"(:
    return ")" << enumerant.json_name
                            << R"("_sv;
)";
                }
            }
            spirv_h << R"(@-@_}
    return ""_sv;
}

constexpr util::Enum_set<)"
                    << capability_enumeration.value()->cpp_name
                    << R"(> get_directly_required_capabilities()" << enumeration.cpp_name
                    << R"( v) noexcept
{
    switch(v)
    {
@+@+)";
            values.clear();
            for(auto &enumerant : enumeration.enumerants)
            {
                if(std::get<1>(values.insert(enumerant.value)))
                {
                    spirv_h << "case " << enumeration.cpp_name << "::" << enumerant.cpp_name << R"(:
    return {)";
                    auto separator = ""_sv;
                    for(auto &capability : enumerant.capabilities.capabilities)
                    {
                        spirv_h << separator;
                        separator = ", "_sv;
                        spirv_h << capability_enumeration.value()->cpp_name
                                << "::" << get_capability(capability).cpp_name;
                    }
                    spirv_h << R"(};
)";
                }
            }
            spirv_h << R"(@-@_}
    return {};
}

constexpr util::Enum_set<)"
                    << extension_enumeration.value()->cpp_name
                    << R"(> get_directly_required_extensions()" << enumeration.cpp_name
                    << R"( v) noexcept
{
    switch(v)
    {
@+@+)";
            values.clear();
            for(auto &enumerant : enumeration.enumerants)
            {
                if(std::get<1>(values.insert(enumerant.value)))
                {
                    spirv_h << "case " << enumeration.cpp_name << "::" << enumerant.cpp_name << R"(:
    return {)";
                    auto separator = ""_sv;
                    for(auto &extension : enumerant.extensions.extensions)
                    {
                        spirv_h << separator;
                        separator = ", "_sv;
                        spirv_h << extension_enumeration.value()->cpp_name
                                << "::" << get_extension(extension).cpp_name;
                    }
                    spirv_h << R"(};
)";
                }
            }
            spirv_h << R"(@-@_}
    return {};
}
)";
        }
    }

private:
    struct Literal_type_descriptor
    {
        ast::Operand_kinds::Operand_kind::Literal_kind literal_kind;
        std::string cpp_name;
        static std::string get_cpp_name(ast::Operand_kinds::Operand_kind::Literal_kind literal_kind)
        {
            using detail::name_from_words_initial_capital;
            return name_from_words_initial_capital(
                       ast::Operand_kinds::Operand_kind::get_json_name_from_literal_kind(
                           literal_kind))
                .to_string();
        }
        explicit Literal_type_descriptor(
            ast::Operand_kinds::Operand_kind::Literal_kind literal_kind)
            : literal_kind(literal_kind), cpp_name(get_cpp_name(literal_kind))
        {
        }
    };

private:
    util::Enum_map<ast::Operand_kinds::Operand_kind::Literal_kind,
                       Literal_type_descriptor> literal_type_descriptors;

private:
    void fill_literal_type_descriptors()
    {
        for(auto literal_kind :
            util::Enum_traits<ast::Operand_kinds::Operand_kind::Literal_kind>::values)
        {
            literal_type_descriptors.emplace(literal_kind, Literal_type_descriptor(literal_kind));
        }
    }
    void write_literal_kinds()
    {
        for(auto &operand_kind : top_level.operand_kinds.operand_kinds)
        {
            if(operand_kind.category != ast::Operand_kinds::Operand_kind::Category::literal)
                continue;
            auto literal_kind = ast::Operand_kinds::Operand_kind::get_literal_kind_from_json_name(
                operand_kind.kind);
            if(!literal_kind)
                throw Generate_error("unknown literal kind: " + operand_kind.kind);
            auto underlying_type = "<<<<<Unknown>>>>>"_sv;
            switch(*literal_kind)
            {
            case ast::Operand_kinds::Operand_kind::Literal_kind::literal_integer:
                underlying_type = "std::uint64_t"_sv;
                break;
            case ast::Operand_kinds::Operand_kind::Literal_kind::literal_string:
                // Literal_string is defined in write_basic_types
                continue;
            case ast::Operand_kinds::Operand_kind::Literal_kind::literal_context_dependent_number:
                underlying_type = "std::vector<Word>"_sv;
                break;
            case ast::Operand_kinds::Operand_kind::Literal_kind::literal_ext_inst_integer:
                underlying_type = "Word"_sv;
                break;
            case ast::Operand_kinds::Operand_kind::Literal_kind::literal_spec_constant_op_integer:
                underlying_type = op_enumeration.value()->cpp_name;
                break;
            }
            auto &descriptor = literal_type_descriptors.at(*literal_kind);
            spirv_h << R"(
typedef )" << underlying_type
                    << " " << descriptor.cpp_name << R"(;
)";
        }
    }

public:
    void run()
    {
        fill_literal_type_descriptors();
        fill_enumerations();
        write_file_comments();
        write_opening_inclusion_guards();
#warning finish
        spirv_h << R"(#error generator not finished being implemented

)";
        write_includes();
        write_opening_namespaces();
        write_basic_types();
        write_basic_constants();
        write_enum_declarations();
        write_enum_definitions();
        write_enum_properties_definitions();
        write_literal_kinds();
        write_closing_namespaces();
        write_closing_inclusion_guards();
        spirv_h.write_to_file();
        spirv_cpp.write_to_file();
        parser_h.write_to_file();
        parser_cpp.write_to_file();
    }
};

constexpr util::string_view Spirv_and_parser_generator::State::op_enum_json_name;
constexpr util::string_view Spirv_and_parser_generator::State::extension_enum_json_name;
constexpr util::string_view Spirv_and_parser_generator::State::capability_enum_json_name;
constexpr util::string_view
    Spirv_and_parser_generator::State::extension_instruction_set_enum_json_name;
constexpr util::string_view
    Spirv_and_parser_generator::State::unknown_extension_instruction_set_enumerant_json_name;

void Spirv_and_parser_generator::run(Generator_args &generator_args,
                                     const ast::Top_level &top_level) const
{
    State(generator_args.output_directory, top_level).run();
}

std::unique_ptr<Generator> Generators::make_spirv_and_parser_generator()
{
    return std::unique_ptr<Generator>(new Spirv_and_parser_generator);
}

std::vector<std::unique_ptr<Generator>> Generators::make_all_generators()
{
    std::unique_ptr<Generator> generators_array[] = {
        make_spirv_and_parser_generator(),
    };
    // use array then move because you can't move out of an std::initializer_list
    std::vector<std::unique_ptr<Generator>> retval;
    retval.reserve(sizeof(generators_array) / sizeof(generators_array[0]));
    for(auto &generator : generators_array)
        retval.push_back(std::move(generator));
    return retval;
}
}
}
}
