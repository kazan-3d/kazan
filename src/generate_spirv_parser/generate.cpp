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
#include "util/optional.h"
#include <limits>
#include <algorithm>
#include <cstdlib>
#include <iostream>
#include <unordered_set>

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
namespace generate
{
Generator::Generator_state::Generator_state(const Generator *generator,
                                            Generator_args &generator_args,
                                            const ast::Top_level &top_level)
    : generator_args(generator_args),
      indent_level(0),
      full_output_file_name(generator_args.output_directory + "/"
                            + generator->output_base_file_name),
      guard_macro_name(get_guard_macro_name_from_file_name(full_output_file_name)),
      os(),
      top_level(top_level),
      operand_kind_map(),
      operand_has_any_parameters_map()
{
    os.exceptions(std::ios::badbit | std::ios::failbit);
    for(auto &operand_kind : top_level.operand_kinds.operand_kinds)
    {
        operand_kind_map.emplace(operand_kind.kind, &operand_kind);
        bool &has_any_parameters = operand_has_any_parameters_map[&operand_kind];
        has_any_parameters = false;
        if(util::holds_alternative<ast::Operand_kinds::Operand_kind::Enumerants>(
               operand_kind.value))
        {
            auto &enumerants =
                util::get<ast::Operand_kinds::Operand_kind::Enumerants>(operand_kind.value);
            for(auto &enumerant : enumerants.enumerants)
            {
                if(!enumerant.parameters.empty())
                {
                    has_any_parameters = true;
                    break;
                }
            }
        }
    }
}

void Generator::Generator_state::open_output_file()
{
    os.open(full_output_file_name);
}

constexpr Generator::Indent_t Generator::indent;
constexpr const char *Generator::vulkan_cpu_namespace_name;
constexpr const char *Generator::spirv_namespace_name;
constexpr const char *Generator::spirv_namespace_names[];
constexpr const char *Generator::extension_enum_name;
constexpr const char *Generator::capability_enum_name;
constexpr const char *Generator::op_enum_name;

std::string Generator::get_guard_macro_name_from_file_name(std::string file_name)
{
    auto retval = std::move(file_name);
    for(char &ch : retval)
    {
        if(ch >= 'a' && ch <= 'z')
        {
            ch = ch - 'a' + 'A'; // convert to uppercase
            continue;
        }
        if(ch >= 'A' && ch <= 'Z')
            continue;
        if(ch >= '0' && ch <= '9')
            continue;
        ch = '_';
    }
    retval += '_';
    if(retval[0] >= '0' && retval[0] <= '9')
        retval.insert(0, 1, '_');
    for(std::size_t double_underline_index = retval.find("__");
        double_underline_index != std::string::npos;
        double_underline_index = retval.find("__", double_underline_index + 1))
    {
        // insert a u in all pairs of underlines to prevent generating a reserved identifier
        retval.insert(++double_underline_index, "u");
    }
    if(retval.size() >= 2 && retval[0] == '_' && retval[1] >= 'A' && retval[1] <= 'Z')
    {
        // insert a u to prevent generating a reserved identifier: starting with an underline and a
        // capital letter
        retval.insert(1, "u");
    }
    return retval;
}

namespace
{
constexpr bool is_uppercase_letter(char ch) noexcept
{
    if(ch >= 'A' && ch <= 'Z')
        return true;
    return false;
}

constexpr bool is_lowercase_letter(char ch) noexcept
{
    if(ch >= 'a' && ch <= 'z')
        return true;
    return false;
}

constexpr bool is_letter(char ch) noexcept
{
    return is_uppercase_letter(ch) || is_lowercase_letter(ch);
}

constexpr bool is_identifier_start(char ch) noexcept
{
    return is_letter(ch) || ch == '_';
}

constexpr bool is_digit(char ch) noexcept
{
    if(ch >= '0' && ch <= '9')
        return true;
    return false;
}

constexpr bool is_identifier_continue(char ch) noexcept
{
    return is_identifier_start(ch) || is_digit(ch);
}
}

std::string Generator::get_enumerant_name(const char *enumeration_name,
                                          std::size_t enumeration_name_size,
                                          std::string enumerant_name,
                                          bool input_name_should_have_prefix)
{
    bool starts_with_enumeration_name =
        enumerant_name.compare(0, enumeration_name_size, enumeration_name, enumeration_name_size)
        == 0;
    bool starts_with_doubled_enumeration_name = false;
    if(starts_with_enumeration_name)
        starts_with_doubled_enumeration_name = enumerant_name.compare(enumeration_name_size,
                                                                      enumeration_name_size,
                                                                      enumeration_name,
                                                                      enumeration_name_size)
                                               == 0;
    std::size_t needed_prefix_count;
    if(input_name_should_have_prefix)
    {
        if(!starts_with_enumeration_name)
            needed_prefix_count = 2;
        else if(starts_with_doubled_enumeration_name)
            needed_prefix_count = 1;
        else
            needed_prefix_count = 0;
    }
    else
    {
        if(starts_with_enumeration_name)
            needed_prefix_count = 1; // ensure that we don't end up with name collisions
        else if(enumerant_name.empty())
            needed_prefix_count = 1; // return something other than the empty string
        else
            needed_prefix_count = is_identifier_start(enumerant_name[0]) ? 0 : 1;
    }
    for(std::size_t i = 0; i < needed_prefix_count; i++)
        enumerant_name.insert(0, enumeration_name, enumeration_name_size);
    return enumerant_name;
}

void Generator::write_indent_absolute(Generator_state &state, std::size_t amount)
{
    static constexpr auto indent_string = "    ";
    for(std::size_t i = 0; i < amount; i++)
        state << indent_string;
}

void Generator::write_indent_interpreted_text(Generator_state &state,
                                              const char *text,
                                              std::ptrdiff_t offset,
                                              bool start_indented)
{
    bool did_indent = start_indented;
    std::size_t indent_amount = offset + state.indent_level;
    for(; *text; text++)
    {
        auto &ch = *text;
        if(ch == '\n')
        {
            state << ch;
            did_indent = false;
            indent_amount = offset + state.indent_level;
        }
        else if(!did_indent && ch == '`')
        {
            indent_amount++;
        }
        else
        {
            if(!did_indent)
            {
                did_indent = true;
                write_indent_absolute(state, indent_amount);
            }
            state << ch;
        }
    }
}

void Generator::write_automatically_generated_file_warning(Generator_state &state)
{
    state
        << "/* This file is automatically generated by generate_spirv_parser. DO NOT MODIFY. */\n";
}

void Generator::write_copyright_comment(Generator_state &state, const ast::Copyright &copyright)
{
    state << "/*\n";
    for(auto &line : copyright.lines)
    {
        if(line.empty())
        {
            state << " *\n";
            continue;
        }
        state << " * ";
        bool was_last_star = false;
        for(char ch : line)
        {
            if(was_last_star && ch == '/')
                state << ' ';
            was_last_star = (ch == '*');
            state << ch;
        }
        state << "\n";
    }
    state << " */\n";
}

void Generator::write_file_guard_start(Generator_state &state)
{
    state << "#ifndef " << state.guard_macro_name << R"(
#define )"
          << state.guard_macro_name << "\n"
                                       "\n";
}

void Generator::write_file_guard_end(Generator_state &state)
{
    state << "#endif /* " << state.guard_macro_name << " */\n";
}

void Generator::write_namespace_start(Generator_state &state, const char *namespace_name)
{
    state << "namespace " << namespace_name << "\n"
                                               "{\n";
}

void Generator::write_namespace_start(Generator_state &state, const std::string &namespace_name)
{
    state << "namespace " << namespace_name << "\n"
                                               "{\n";
}

void Generator::write_namespace_end(Generator_state &state)
{
    state << "}\n";
}

void Generator::write_unsigned_integer_literal(Generator_state &state,
                                               std::uint64_t value,
                                               Integer_literal_base base,
                                               std::size_t minimum_digit_count)
{
    constexpr std::uint64_t max_unsigned_value = std::numeric_limits<std::uint16_t>::max();
    constexpr std::uint64_t max_unsigned_long_value = std::numeric_limits<std::uint32_t>::max();
    auto literal_type =
        value <= max_unsigned_value ? "U" : value <= max_unsigned_long_value ? "UL" : "ULL";
    auto number_prefix = "";
    unsigned base_as_number = 10;
    switch(base)
    {
    case Integer_literal_base::dec:
        minimum_digit_count = 1;
        break;
    case Integer_literal_base::hex:
        base_as_number = 0x10;
        number_prefix = "0x";
        break;
    case Integer_literal_base::oct:
        base_as_number = 010;
        number_prefix = "0";
        break;
    }
    auto number_string = json::ast::Number_value::append_unsigned_integer_to_string(
                             value, number_prefix, base_as_number, minimum_digit_count)
                         + literal_type;
    state << number_string;
}

void Generator::write_signed_integer_literal(Generator_state &state, std::int64_t value)
{
    constexpr std::int64_t max_int_value = std::numeric_limits<std::int16_t>::max();
    constexpr std::int64_t min_int_value = std::numeric_limits<std::int16_t>::min();
    constexpr std::int64_t max_long_value = std::numeric_limits<std::int32_t>::max();
    constexpr std::int64_t min_long_value = std::numeric_limits<std::int32_t>::min();
    auto literal_type = "";
    if(value < min_int_value || value > max_int_value)
        literal_type = "L";
    if(value < min_long_value || value > max_long_value)
        literal_type = "LL";
    state << value << literal_type;
}

struct Generator::Get_extensions_visitor
{
    std::unordered_set<std::string> &retval;
    constexpr Get_extensions_visitor(std::unordered_set<std::string> &retval) noexcept
        : retval(retval)
    {
    }
    template <typename T>
    void operator()(const T &)
    {
    }
    void operator()(const ast::Extensions &extensions)
    {
        for(auto &extension : extensions.extensions)
            retval.insert(extension);
    }
};

std::unordered_set<std::string> Generator::get_extensions(const ast::Top_level &top_level)
{
    std::unordered_set<std::string> retval;
    top_level.visit(Get_extensions_visitor(retval));
    return retval;
}

void Generator::write_capabilities_set(Generator_state &state,
                                       const ast::Capabilities &capabilities)
{
    state << "util::Enum_set<" << capability_enum_name << ">{";
    auto separator = "";
    for(auto &capability : capabilities.capabilities)
    {
        state << separator << capability_enum_name
              << "::" << get_enumerant_name(capability_enum_name, capability, false);
        separator = ", ";
    }
    state << "}";
}

void Generator::write_extensions_set(Generator_state &state, const ast::Extensions &extensions)
{
    state << "util::Enum_set<" << extension_enum_name << ">{";
    auto separator = "";
    for(auto &extension : extensions.extensions)
    {
        state << separator << extension_enum_name
              << "::" << get_enumerant_name(extension_enum_name, extension, false);
        separator = ", ";
    }
    state << "}";
}

std::string Generator::get_name_from_words(const std::string &words)
{
    enum class Char_class
    {
        Uppercase,
        OtherIdentifier,
        WordSeparator,
    };
    auto get_char_class = [](char ch) -> Char_class
    {
        if(is_uppercase_letter(ch))
            return Char_class::Uppercase;
        if(is_letter(ch) || is_digit(ch))
            return Char_class::OtherIdentifier;
        return Char_class::WordSeparator;
    };
    auto find_words = [&](auto found_word_callback) -> void
    {
        util::optional<std::size_t> word_start;
        auto finish_word = [&](std::size_t index)
        {
            found_word_callback(util::string_view(words.data() + *word_start, index - *word_start));
            word_start = {};
        };
        auto start_word = [&](std::size_t index)
        {
            word_start = index;
        };
        auto last_char_class = Char_class::WordSeparator;
        for(std::size_t i = 0; i < words.size(); i++)
        {
            auto current_char_class = get_char_class(words[i]);
            if(word_start)
            {
                switch(current_char_class)
                {
                case Char_class::WordSeparator:
                    finish_word(i);
                    break;
                case Char_class::Uppercase:
                    if(last_char_class != Char_class::Uppercase)
                    {
                        finish_word(i);
                        start_word(i);
                    }
                    else if(i + 1 < words.size()
                            && get_char_class(words[i + 1]) == Char_class::OtherIdentifier)
                    {
                        finish_word(i);
                        start_word(i);
                    }
                    break;
                case Char_class::OtherIdentifier:
                    break;
                }
            }
            else if(current_char_class != Char_class::WordSeparator)
            {
                start_word(i);
            }
            last_char_class = current_char_class;
        }
        if(word_start)
            finish_word(words.size());
    };
    std::size_t retval_size = 0;
    bool first = true;
    find_words([&](util::string_view word)
               {
                   if(!first)
                       retval_size++; // separating '_'
                   first = false;
                   retval_size += word.size();
               });
    std::string retval;
    retval.reserve(retval_size);
    first = true;
    find_words([&](util::string_view word)
               {
                   if(!first)
                       retval += '_';
                   first = false;
                   retval += word;
               });
    for(char &ch : retval)
    {
        if(is_uppercase_letter(ch))
            ch = ch - 'A' + 'a'; // to lowercase
    }
    static constexpr const char *const reserved_words[] = {
        "alignas",
        "alignof",
        "and",
        "and_eq",
        "asm",
        "atomic_cancel",
        "atomic_commit",
        "atomic_noexcept",
        "auto",
        "bitand",
        "bitor",
        "bool",
        "break",
        "case",
        "catch",
        "char",
        "char16_t",
        "char32_t",
        "class",
        "compl",
        "concept",
        "concepts",
        "const",
        "const_cast",
        "constexpr",
        "continue",
        "decltype",
        "default",
        "delete",
        "do",
        "double",
        "dynamic_cast",
        "else",
        "enum",
        "explicit",
        "export",
        "extern",
        "false",
        "float",
        "for",
        "friend",
        "goto",
        "if",
        "import",
        "inline",
        "int",
        "long",
        "module",
        "modules",
        "mutable",
        "namespace",
        "new",
        "noexcept",
        "not",
        "not_eq",
        "nullptr",
        "operator",
        "or",
        "or_eq",
        "private",
        "protected",
        "public",
        "register",
        "reinterpret_cast",
        "requires",
        "return",
        "short",
        "signed",
        "sizeof",
        "static",
        "static_assert",
        "static_cast",
        "struct",
        "switch",
        "synchronized",
        "template",
        "this",
        "thread_local",
        "throw",
        "true",
        "try",
        "typedef",
        "typeid",
        "typename",
        "union",
        "unsigned",
        "using",
        "virtual",
        "void",
        "volatile",
        "wchar_t",
        "while",
        "xor",
        "xor_eq",
    };
    for(const char *reserved_word : reserved_words)
    {
        if(retval == reserved_word)
        {
            retval += '_';
            break;
        }
    }
    return retval;
}

#if 0
#warning testing Generator::get_name_from_words
struct Generator::Tester
{
    struct Test_runner
    {
        Test_runner()
        {
            test();
            std::exit(1);
        }
    };
    static Test_runner test_runner;
    static void test()
    {
        for(auto &input : {
                    "abc  def", "AbcDef", "ABCDef", "'abc, def'",
                })
        {
            std::cout << "\"" << input << "\" -> " << get_name_from_words(input)
                      << std::endl;
        }
    }
};

Generator::Tester::Test_runner Generator::Tester::test_runner;
#endif

std::string Generator::get_member_name_from_operand(
    const ast::Instructions::Instruction::Operands::Operand &operand)
{
    if(!operand.name.empty())
        return get_name_from_words(operand.name);
    util::string_view id_str = "Id";
    if(util::string_view(operand.kind).compare(0, id_str.size(), id_str) == 0
       && id_str.size() < operand.kind.size()
       && is_uppercase_letter(operand.kind[id_str.size()]))
        return get_name_from_words(operand.kind.substr(id_str.size()));
    return get_name_from_words(operand.kind);
}

std::string Generator::get_member_name_from_parameter(
    const ast::Operand_kinds::Operand_kind::Enumerants::Enumerant::Parameters::Parameter &parameter)
{
    if(!parameter.name.empty())
        return get_name_from_words(parameter.name);
    return get_name_from_words(parameter.kind);
}

std::string Generator::get_operand_with_parameters_name(
    Generator_state &state, const ast::Operand_kinds::Operand_kind &operand_kind)
{
    if(get_operand_has_any_parameters(state, operand_kind))
        return operand_kind.kind + "_with_parameters";
    return operand_kind.kind;
}

void Generator::write_struct_nonstatic_members_and_constructors(Generator_state &state,
                                                                const std::string &struct_name,
                                                                const std::string *member_types,
                                                                const std::string *member_names,
                                                                std::size_t member_count)
{
    for(std::size_t i = 0; i < member_count; i++)
        state << indent << member_types[i] << " " << member_names[i] << ";\n";
    state << indent << struct_name << "()\n";
    {
        auto push_indent = state.pushed_indent();
        for(std::size_t i = 0; i < member_count; i++)
        {
            state << indent;
            if(i == 0)
                state << ": ";
            else
                state << "  ";
            state << member_names[i] << "()";
            if(i != member_count - 1)
                state << ",";
            state << "\n";
        }
    }
    state << indent(R"({
}
)");
    if(member_count != 0)
    {
        state << indent;
        if(member_count == 1)
            state << "explicit ";
        state << struct_name << "(";
        for(std::size_t i = 0; i < member_count; i++)
        {
            state << member_types[i] << " " << member_names[i];
            if(i != member_count - 1)
                state << ", ";
        }
        state << ")\n";
        {
            auto push_indent = state.pushed_indent();
            for(std::size_t i = 0; i < member_count; i++)
            {
                state << indent;
                if(i == 0)
                    state << ": ";
                else
                    state << "  ";
                state << member_names[i] << "(std::move(" << member_names[i] << "))";
                if(i != member_count - 1)
                    state << ",";
                state << "\n";
            }
        }
        state << indent(R"({
}
)");
    }
}

std::vector<ast::Operand_kinds::Operand_kind::Enumerants::Enumerant>
    Generator::get_unique_enumerants(
        std::vector<ast::Operand_kinds::Operand_kind::Enumerants::Enumerant> enumerants)
{
    std::unordered_set<std::uint32_t> values;
    std::size_t output_index = 0;
    for(std::size_t input_index = 0; input_index < enumerants.size(); input_index++)
    {
        if(std::get<1>(values.insert(enumerants[input_index].value)))
        {
            if(output_index != input_index)
                enumerants[output_index] = std::move(enumerants[input_index]);
            output_index++;
        }
    }
    enumerants.erase(enumerants.begin() + output_index, enumerants.end());
    return enumerants;
}

struct Spirv_header_generator final : public Generator
{
    Spirv_header_generator() : Generator("spirv.h")
    {
    }
    enum class Enum_priority
    {
        default_priority = 0,
        capability = 1,
    };
    static Enum_priority get_enum_priority(const std::string &enum_name) noexcept
    {
        if(enum_name == capability_enum_name)
            return Enum_priority::capability;
        return Enum_priority::default_priority;
    }
    static bool compare_enum_names(const std::string &l, const std::string &r) noexcept
    {
        auto l_priority = get_enum_priority(l);
        auto r_priority = get_enum_priority(r);
        if(l_priority > r_priority)
            return true; // higher priority sorts first
        if(l_priority < r_priority)
            return false;
        return l < r;
    }
    /** lower priority means that the operand kind is declared first */
    static int get_operand_category_priority(
        ast::Operand_kinds::Operand_kind::Category category) noexcept
    {
        switch(category)
        {
        case ast::Operand_kinds::Operand_kind::Category::bit_enum:
        case ast::Operand_kinds::Operand_kind::Category::value_enum:
            return 1;
        case ast::Operand_kinds::Operand_kind::Category::id:
            return 0;
        case ast::Operand_kinds::Operand_kind::Category::literal:
            return 0;
        case ast::Operand_kinds::Operand_kind::Category::composite:
            return 2;
        }
        return 0;
    }
    static bool compare_operand_kinds(const ast::Operand_kinds::Operand_kind &l,
                                      const ast::Operand_kinds::Operand_kind &r)
    {
        auto l_priority = get_operand_category_priority(l.category);
        auto r_priority = get_operand_category_priority(r.category);
        if(l_priority != r_priority)
            return l_priority < r_priority;
        return compare_enum_names(l.kind, r.kind);
    }
    virtual void run(Generator_args &generator_args, const ast::Top_level &top_level) const override
    {
        Generator_state state(this, generator_args, top_level);
        state.open_output_file();
        write_file_comments(state, top_level.copyright);
        write_file_guard_start(state);
        state << indent(R"(#include <cstdint>
#include "util/enum.h"
#include "util/optional.h"
#include "util/variant.h"
#include <vector>

)");
        write_namespaces_start(state, spirv_namespace_names);
        state << indent(R"(typedef std::uint32_t Word;
typedef Word Id;
enum class Op : Word;
constexpr Word magic_number = )")
              << unsigned_hex_integer_literal(top_level.magic_number, 8)
              << indent(true,
                        ";\n"
                        "constexpr std::uint32_t major_version = ")
              << unsigned_dec_integer_literal(top_level.major_version)
              << indent(true,
                        ";\n"
                        "constexpr std::uint32_t minor_version = ")
              << unsigned_dec_integer_literal(top_level.minor_version)
              << indent(true,
                        ";\n"
                        "constexpr std::uint32_t revision = ")
              << unsigned_dec_integer_literal(top_level.revision) << ";\n";
        auto extensions_set = get_extensions(top_level);
        std::vector<std::string> extensions_list;
        extensions_list.reserve(extensions_set.size());
        for(auto &extension : extensions_set)
            extensions_list.push_back(extension);
        std::sort(extensions_list.begin(), extensions_list.end());
        state << indent(
                     "\n"
                     "enum class ")
              << extension_enum_name << indent(true,
                                               " : std::size_t\n"
                                               "{\n");
        {
            auto push_indent = state.pushed_indent();
            for(auto &extension : extensions_list)
                state << indent << get_enumerant_name(extension_enum_name, extension, false)
                      << ",\n";
        }
        state << indent(
                     "};\n"
                     "\n"
                     "vulkan_cpu_util_generate_enum_traits(")
              << extension_enum_name;
        {
            auto push_indent = state.pushed_indent();
            for(auto &extension : extensions_list)
                state << ",\n" << indent << extension_enum_name
                      << "::" << get_enumerant_name(extension_enum_name, extension, false);
            state << ");\n";
        }
        state << indent(
                     "\n"
                     "constexpr const char *get_enumerant_name(")
              << extension_enum_name << indent(true,
                                               " v) noexcept\n"
                                               "{\n");
        {
            auto push_indent = state.pushed_indent();
            state << indent(
                "switch(v)\n"
                "{\n");
            for(auto &extension : extensions_list)
            {
                state << indent("case ") << extension_enum_name
                      << "::" << get_enumerant_name(extension_enum_name, extension, false)
                      << indent(true,
                                ":\n"
                                "`return \"")
                      << extension << "\";\n";
            }
            state << indent(
                "}\n"
                "return \"\";\n");
        }
        state << "}\n";
        std::vector<const ast::Operand_kinds::Operand_kind *> operand_kinds;
        operand_kinds.reserve(top_level.operand_kinds.operand_kinds.size());
        for(auto &operand_kind : top_level.operand_kinds.operand_kinds)
            operand_kinds.push_back(&operand_kind);
        std::sort(
            operand_kinds.begin(),
            operand_kinds.end(),
            [](const ast::Operand_kinds::Operand_kind *a, const ast::Operand_kinds::Operand_kind *b)
            {
                return compare_operand_kinds(*a, *b);
            });
        for(auto *operand_kind : operand_kinds)
        {
            switch(operand_kind->category)
            {
            case ast::Operand_kinds::Operand_kind::Category::bit_enum:
            case ast::Operand_kinds::Operand_kind::Category::value_enum:
            {
                bool is_bit_enum =
                    operand_kind->category == ast::Operand_kinds::Operand_kind::Category::bit_enum;
                auto &enumerants =
                    util::get<ast::Operand_kinds::Operand_kind::Enumerants>(operand_kind->value);
                auto unique_enumerants = get_unique_enumerants(enumerants.enumerants);
                state << "\n"
                         "enum class "
                      << get_enum_name(*operand_kind) << " : Word\n"
                                                         "{\n";
                {
                    auto push_indent = state.pushed_indent();
                    for(auto &enumerant : enumerants.enumerants)
                    {
                        state << indent
                              << get_enumerant_name(operand_kind->kind, enumerant.enumerant, false)
                              << " = ";
                        if(is_bit_enum)
                            state << unsigned_hex_integer_literal(enumerant.value);
                        else
                            state << unsigned_dec_integer_literal(enumerant.value);
                        state << ",\n";
                    }
                }
                state << "};\n"
                         "\n"
                         "vulkan_cpu_util_generate_enum_traits("
                      << get_enum_name(*operand_kind);
                {
                    auto push_indent = state.pushed_indent();
                    for(auto &enumerant : unique_enumerants)
                        state << ",\n" << indent << get_enum_name(*operand_kind) << "::"
                              << get_enumerant_name(operand_kind->kind, enumerant.enumerant, false);
                    state << ");\n";
                }
                state << "\n"
                         "constexpr const char *get_enumerant_name("
                      << get_enum_name(*operand_kind) << " v) noexcept\n"
                                                         "{\n";
                {
                    auto push_indent = state.pushed_indent();
                    state << indent(
                        "switch(v)\n"
                        "{\n");
                    for(auto &enumerant : unique_enumerants)
                    {
                        state << indent("case ") << get_enum_name(*operand_kind) << "::"
                              << get_enumerant_name(operand_kind->kind, enumerant.enumerant, false)
                              << indent(true,
                                        ":\n"
                                        "`return \"")
                              << enumerant.enumerant << "\";\n";
                    }
                    state << indent(
                        "}\n"
                        "return \"\";\n");
                }
                state << "}\n";
                state << "\n"
                         "constexpr util::Enum_set<"
                      << capability_enum_name << "> get_directly_required_capability_set("
                      << get_enum_name(*operand_kind) << " v) noexcept\n"
                                                         "{\n";
                {
                    auto push_indent = state.pushed_indent();
                    state << indent(
                        "switch(v)\n"
                        "{\n");
                    for(auto &enumerant : unique_enumerants)
                    {
                        state << indent("case ") << get_enum_name(*operand_kind) << "::"
                              << get_enumerant_name(operand_kind->kind, enumerant.enumerant, false)
                              << indent(true,
                                        ":\n"
                                        "`return ")
                              << enumerant.capabilities << ";\n";
                    }
                    state << indent(
                        "}\n"
                        "return {};\n");
                }
                state << "}\n"
                         "\n"
                         "constexpr util::Enum_set<"
                      << extension_enum_name << "> get_directly_required_extension_set("
                      << get_enum_name(*operand_kind) << " v) noexcept\n"
                                                         "{\n";
                {
                    auto push_indent = state.pushed_indent();
                    state << indent(
                        "switch(v)\n"
                        "{\n");
                    for(auto &enumerant : unique_enumerants)
                    {
                        state << indent("case ") << get_enum_name(*operand_kind) << "::"
                              << get_enumerant_name(operand_kind->kind, enumerant.enumerant, false)
                              << indent(true,
                                        ":\n"
                                        "`return ")
                              << enumerant.extensions << ";\n";
                    }
                    state << indent(
                        "}\n"
                        "return {};\n");
                }
                state << "}\n";
                break;
            }
            case ast::Operand_kinds::Operand_kind::Category::composite:
            {
                auto &bases =
                    util::get<ast::Operand_kinds::Operand_kind::Bases>(operand_kind->value);
                state << "\n"
                         "struct "
                      << operand_kind->kind << "\n"
                                               "{\n";
                auto push_indent = state.pushed_indent();
                std::vector<std::string> member_types;
                std::vector<std::string> member_names;
                member_types.reserve(bases.values.size());
                member_names.reserve(bases.values.size());
                for(std::size_t i = 0; i < bases.values.size(); i++)
                {
                    member_types.push_back(
                        get_operand_with_parameters_name(state, bases.values[i]));
                    member_names.push_back(
                        json::ast::Number_value::append_unsigned_integer_to_string(i + 1, "part_"));
                }
                write_struct_nonstatic_members_and_constructors(state,
                                                                operand_kind->kind,
                                                                member_types.data(),
                                                                member_names.data(),
                                                                bases.values.size());
                push_indent.finish();
                state << "};\n";
                break;
            }
            case ast::Operand_kinds::Operand_kind::Category::id:
            {
                auto &doc = util::get<ast::Operand_kinds::Operand_kind::Doc>(operand_kind->value);
                state << "\n"
                         "/** ";
                bool was_last_star = false;
                for(char ch : doc.value)
                {
                    if(was_last_star && ch == '/')
                        state << ' ';
                    was_last_star = (ch == '*');
                    state << ch;
                }
                state << " */\n"
                         "typedef Id "
                      << operand_kind->kind << ";\n";
                break;
            }
            case ast::Operand_kinds::Operand_kind::Category::literal:
            {
                auto &doc = util::get<ast::Operand_kinds::Operand_kind::Doc>(operand_kind->value);
                auto base_type = "std::vector<Word>";
                if(operand_kind->kind == "LiteralInteger")
                    base_type = "std::uint32_t"; // TODO: fix after determining if LiteralInteger
                // can be multiple words
                else if(operand_kind->kind == "LiteralString")
                    base_type = "std::string";
                else if(operand_kind->kind == "LiteralExtInstInteger")
                    base_type = "Word";
                else if(operand_kind->kind == "LiteralSpecConstantOpInteger")
                    base_type = "Op";
#warning finish
                state << "\n"
                         "/** ";
                bool was_last_star = false;
                for(char ch : doc.value)
                {
                    if(was_last_star && ch == '/')
                        state << ' ';
                    was_last_star = (ch == '*');
                    state << ch;
                }
                state << " */\n"
                         "typedef "
                      << base_type << " " << operand_kind->kind << ";\n";
                break;
            }
            }
        }
        for(auto *operand_kind : operand_kinds)
        {
            switch(operand_kind->category)
            {
            case ast::Operand_kinds::Operand_kind::Category::bit_enum:
            case ast::Operand_kinds::Operand_kind::Category::value_enum:
            {
                bool is_bit_enum =
                    operand_kind->category == ast::Operand_kinds::Operand_kind::Category::bit_enum;
                auto &enumerants =
                    util::get<ast::Operand_kinds::Operand_kind::Enumerants>(operand_kind->value);
                auto unique_enumerants = get_unique_enumerants(enumerants.enumerants);
                if(get_operand_has_any_parameters(state, *operand_kind))
                {
                    for(auto &enumerant : unique_enumerants)
                    {
                        if(enumerant.parameters.empty())
                            continue;
                        auto struct_name = get_enumerant_parameters_struct_name(
                            operand_kind->kind, enumerant.enumerant, false);
                        state << "\n"
                                 "struct "
                              << struct_name << indent(true, R"(
{
`static constexpr )") << get_enum_name(*operand_kind)
                              << indent(true, R"( get_enumerant() noexcept
`{
``return )") << get_enum_name(*operand_kind)
                              << "::"
                              << get_enumerant_name(operand_kind->kind, enumerant.enumerant, false)
                              << indent(true, R"(;
`}
)");
                        std::vector<std::string> member_types;
                        std::vector<std::string> member_names;
                        member_types.reserve(enumerant.parameters.parameters.size());
                        member_names.reserve(enumerant.parameters.parameters.size());
                        for(auto &parameter : enumerant.parameters.parameters)
                        {
                            member_types.push_back(
                                get_operand_with_parameters_name(state, parameter.kind));
                            member_names.push_back(get_member_name_from_parameter(parameter));
                        }
                        auto push_indent = state.pushed_indent();
                        write_struct_nonstatic_members_and_constructors(
                            state,
                            struct_name,
                            member_types.data(),
                            member_names.data(),
                            enumerant.parameters.parameters.size());
                        push_indent.finish();
                        state << "};\n";
                    }
                    auto struct_name = get_operand_with_parameters_name(state, *operand_kind);
                    state << indent(R"(
struct )") << struct_name << indent(true, R"(
{
`typedef util::variant<util::monostate)");
                    for(auto &enumerant : unique_enumerants)
                    {
                        if(enumerant.parameters.empty())
                            continue;
                        state << ",\n" << indent(2) << get_enumerant_parameters_struct_name(
                                             operand_kind->kind, enumerant.enumerant, false);
                    }
                    state << indent(true, R"(> Parameters;
)");
                    auto push_indent = state.pushed_indent();
                    constexpr std::size_t member_count = 2;
                    std::string member_types[member_count] = {
                        get_enum_name(*operand_kind), "Parameters",
                    };
                    std::string member_names[member_count] = {
                        "value", "parameters",
                    };
                    write_struct_nonstatic_members_and_constructors(
                        state, struct_name, member_types, member_names, member_count);
                    push_indent.finish();
                    state << "};\n";
                }
                break;
            }
            case ast::Operand_kinds::Operand_kind::Category::composite:
            case ast::Operand_kinds::Operand_kind::Category::id:
            case ast::Operand_kinds::Operand_kind::Category::literal:
                break;
            }
        }
#warning finish converting
        std::vector<const ast::Instructions::Instruction *> instructions;
        instructions.reserve(top_level.instructions.instructions.size());
        for(auto &instruction : top_level.instructions.instructions)
            instructions.push_back(&instruction);
        std::sort(
            instructions.begin(),
            instructions.end(),
            [](const ast::Instructions::Instruction *a, const ast::Instructions::Instruction *b)
            {
                return a->opcode < b->opcode;
            });
        state << "\n"
                 "enum class "
              << op_enum_name << " : Word\n"
                                 "{\n";
        {
            auto push_indent = state.pushed_indent();
            for(auto &instruction : top_level.instructions.instructions)
            {
                state << indent << get_enumerant_name(op_enum_name, instruction.opname, true)
                      << " = " << unsigned_dec_integer_literal(instruction.opcode) << ",\n";
            }
        }
        state << "};\n";
        state << "\n"
                 "vulkan_cpu_util_generate_enum_traits("
              << op_enum_name;
        {
            auto push_indent = state.pushed_indent();
            for(auto &instruction : top_level.instructions.instructions)
                state << ",\n" << indent << op_enum_name
                      << "::" << get_enumerant_name(op_enum_name, instruction.opname, true);
            state << ");\n";
        }
        state << "\n"
                 "constexpr const char *get_enumerant_name("
              << op_enum_name << " v) noexcept\n"
                                 "{\n";
        {
            auto push_indent = state.pushed_indent();
            state << indent(
                "switch(v)\n"
                "{\n");
            for(auto &instruction : top_level.instructions.instructions)
            {
                state << indent("case ") << op_enum_name
                      << "::" << get_enumerant_name(op_enum_name, instruction.opname, true)
                      << indent(true,
                                ":\n"
                                "return \"")
                      << instruction.opname << "\";\n";
            }
            state << indent(
                "}\n"
                "return \"\";\n");
        }
        state << "}\n"
                 "\n"
                 "constexpr util::Enum_set<"
              << capability_enum_name << "> get_directly_required_capability_set(" << op_enum_name
              << " v) noexcept\n"
                 "{\n";
        {
            auto push_indent = state.pushed_indent();
            state << indent(
                "switch(v)\n"
                "{\n");
            for(auto &instruction : top_level.instructions.instructions)
            {
                state << indent("case ") << op_enum_name
                      << "::" << get_enumerant_name(op_enum_name, instruction.opname, true)
                      << indent(true,
                                ":\n"
                                "return ")
                      << instruction.capabilities << ";\n";
            }
            state << indent(
                "}\n"
                "return {};\n");
        }
        state << "}\n"
                 "\n"
                 "constexpr util::Enum_set<"
              << extension_enum_name << "> get_directly_required_extension_set(" << op_enum_name
              << " v) noexcept\n"
                 "{\n";
        {
            auto push_indent = state.pushed_indent();
            state << indent(
                "switch(v)\n"
                "{\n");
            for(auto &instruction : top_level.instructions.instructions)
            {
                state << indent("case ") << op_enum_name
                      << "::" << get_enumerant_name(op_enum_name, instruction.opname, true)
                      << ":\n";
                auto push_indent2 = state.pushed_indent();
                state << indent("return ") << instruction.extensions << ";\n";
            }
            state << indent(
                "}\n"
                "return {};\n");
        }
        state << "}\n";
        for(auto &instruction : top_level.instructions.instructions)
        {
            auto struct_name = get_enumerant_name(op_enum_name, instruction.opname, true);
            state << "\n"
                     "struct "
                  << struct_name << "\n"
                                    "{\n";
            {
                auto push_indent = state.pushed_indent();
                state << indent("static constexpr ") << op_enum_name << " get_opcode() noexcept\n"
                      << indent("{\n");
                {
                    auto push_indent2 = state.pushed_indent();
                    state << indent("return ") << op_enum_name
                          << "::" << get_enumerant_name(op_enum_name, instruction.opname, true)
                          << ";\n";
                }
                state << indent("}\n");
                std::vector<std::string> member_names;
                std::vector<std::string> member_types;
                member_names.reserve(instruction.operands.operands.size());
                member_types.reserve(instruction.operands.operands.size());
                for(auto &operand : instruction.operands.operands)
                {
                    std::string member_type;
                    switch(operand.quantifier)
                    {
                    case ast::Instructions::Instruction::Operands::Operand::Quantifier::none:
                    {
                        member_type = operand.kind;
                        break;
                    }
                    case ast::Instructions::Instruction::Operands::Operand::Quantifier::optional:
                    {
                        member_type = "util::optional<" + operand.kind + ">";
                        break;
                    }
                    case ast::Instructions::Instruction::Operands::Operand::Quantifier::variable:
                    {
                        member_type = "std::vector<" + operand.kind + ">";
                        break;
                    }
                    }
                    member_types.push_back(std::move(member_type));
                    member_names.push_back(get_member_name_from_operand(operand));
                }
                write_struct_nonstatic_members_and_constructors(state,
                                                                struct_name,
                                                                member_types.data(),
                                                                member_names.data(),
                                                                member_types.size());
            }
            state << "};\n";
        }

#warning finish
        write_namespaces_end(state, spirv_namespace_names);
        write_file_guard_end(state);
    }
};

struct Spirv_source_generator final : public Generator
{
    Spirv_source_generator() : Generator("spirv.cpp")
    {
    }
    virtual void run(Generator_args &generator_args, const ast::Top_level &top_level) const override
    {
        Generator_state state(this, generator_args, top_level);
        state.open_output_file();
        write_file_comments(state, top_level.copyright);
        state << "#include \"spirv.h\"\n";
    }
};

struct Parser_header_generator final : public Generator
{
    Parser_header_generator() : Generator("parser.h")
    {
    }
    static std::string get_dump_operand_function_name(std::string kind)
    {
        return "dump_operand_" + std::move(kind);
    }
    static std::string get_parse_operand_function_name(std::string kind)
    {
        return "parse_operand_" + std::move(kind);
    }
    static std::string get_parse_instruction_function_name(std::string opname)
    {
        return "parse_instruction_" + get_enumerant_name(op_enum_name, opname, true);
    }
    virtual void run(Generator_args &generator_args, const ast::Top_level &top_level) const override
    {
        Generator_state state(this, generator_args, top_level);
        state.open_output_file();
        write_file_comments(state, top_level.copyright);
        write_file_guard_start(state);
        state << R"(#include "spirv.h"
#include <memory>
#include <ostream>
#include "util/optional.h"
#include "json/json.h"
#include <vector>

)";
        write_namespaces_start(state, spirv_namespace_names);
        state << indent(R"(struct Parse_error
{
`std::size_t word_index;
`std::size_t instruction_word_index;
`std::string message;
`Parse_error(std::size_t word_index, std::size_t instruction_word_index, std::string message) noexcept
``: word_index(word_index),
``  instruction_word_index(instruction_word_index),
``  message(std::move(message))
`{
`}
`virtual ~Parse_error() = default;
};

)");
        state << "struct Parse_semantics_generic\n"
                 "{\n";
        {
            auto push_indent = state.pushed_indent();
            state << indent(R"(virtual ~Parse_semantics_generic() = default;
virtual std::unique_ptr<Parse_error> handle_error(std::size_t word_index, std::size_t instruction_word_index, std::string message) = 0;
virtual void handle_spirv_version(unsigned major, unsigned minor) = 0;
virtual void handle_generator_magic_number(Word value) = 0;
virtual void handle_id_bound(Word id_bound) = 0;
)");
            for(auto &instruction : top_level.instructions.instructions)
            {
                auto struct_name = get_enumerant_name(op_enum_name, instruction.opname, true);
                state << indent("virtual void handle_instruction(") << struct_name
                      << " instruction) = 0;\n";
            }
#warning finish
        }
        state << indent(R"(};

struct Parse_dump final : public Parse_semantics_generic
{
`std::ostream &os;
`explicit constexpr Parse_dump(std::ostream &os) noexcept : os(os)
`{
`}
)");
        {
            auto push_indent = state.pushed_indent();
            state << indent(
                R"(virtual std::unique_ptr<Parse_error> handle_error(std::size_t word_index, std::size_t instruction_word_index, std::string message) override;
virtual void handle_spirv_version(unsigned major, unsigned minor) override;
virtual void handle_generator_magic_number(Word value) override;
virtual void handle_id_bound(Word id_bound) override;
)");
            for(auto &instruction : top_level.instructions.instructions)
            {
                auto struct_name = get_enumerant_name(op_enum_name, instruction.opname, true);
                state << indent("virtual void handle_instruction(") << struct_name
                      << " instruction) override;\n";
            }
            for(auto &operand_kind : top_level.operand_kinds.operand_kinds)
            {
                auto dump_function_name = get_dump_operand_function_name(operand_kind.kind);
                state << indent("void ") << dump_function_name << "(const " << operand_kind.kind
                      << " &v);\n";
                state << indent("void ") << dump_function_name << "(const util::optional<"
                      << operand_kind.kind << "> &v);\n";
                state << indent("void ") << dump_function_name << "(const std::vector<"
                      << operand_kind.kind << "> &v);\n";
            }
#warning finish
        }
        state << "};\n"
                 "\n"
                 "template <typename Semantics = Parse_semantics_generic>\n"
                 "struct Parser\n"
                 "{\n";
        {
            auto push_indent = state.pushed_indent();
            for(auto &operand_kind : top_level.operand_kinds.operand_kinds)
            {
                auto parse_function_name = get_parse_operand_function_name(operand_kind.kind);
                state
                    << indent(R"(static std::unique_ptr<Parse_error> )") << parse_function_name
                    << indent(
                           true,
                           R"((const Word *words, std::size_t word_count, Semantics &semantics, std::size_t error_instruction_index, std::size_t &word_index, )")
                    << operand_kind.kind << indent(true,
                                                   R"( &value)
{
`if(word_index >= word_count)
``return semantics.handle_error(error_instruction_index + word_index, error_instruction_index, "instruction missing operand");
)");
                auto push_indent = state.pushed_indent();
                switch(operand_kind.category)
                {
                case ast::Operand_kinds::Operand_kind::Category::bit_enum:
                {
                    state << indent(R"(value = static_cast<)") << operand_kind.kind
                          << indent(true, R"(>(words[word_index++]);
)");
                    break;
                }
                case ast::Operand_kinds::Operand_kind::Category::value_enum:
                {
                    state << indent(R"(value = static_cast<)") << operand_kind.kind
                          << indent(true, R"(>(words[word_index]);
if(util::Enum_traits<)") << operand_kind.kind
                          << indent(true, R"(>::find_value(value) == util::Enum_traits<)")
                          << operand_kind.kind << indent(true, R"(>::npos)
`return semantics.handle_error(error_instruction_index + word_index, error_instruction_index, "invalid enum value");
word_index++;
)");
                    break;
                }
                case ast::Operand_kinds::Operand_kind::Category::composite:
                {
                    auto &bases =
                        util::get<ast::Operand_kinds::Operand_kind::Bases>(operand_kind.value);
                    for(std::size_t i = 0; i < bases.values.size(); i++)
                    {
                        state << indent;
                        if(i == 0)
                            state << indent(true, "auto ");
                        state << indent(true, "parse_error = ")
                              << get_parse_operand_function_name(bases.values[i])
                              << "(words, word_count, semantics, error_instruction_index, "
                                 "word_index, value."
                              << json::ast::Number_value::append_unsigned_integer_to_string(i + 1,
                                                                                            "part_")
                              << indent(true, R"();
if(parse_error)
`return parse_error;
)");
                    }
                    break;
                }
                case ast::Operand_kinds::Operand_kind::Category::id:
                {
                    state << indent(R"(value = static_cast<Id>(words[word_index++]);
)");
                    break;
                }
                case ast::Operand_kinds::Operand_kind::Category::literal:
                {
                    if(operand_kind.kind == "LiteralInteger")
                    {
                        // TODO: fix after determining if LiteralInteger can be multiple words
                        state << indent(R"(value = words[word_index++];
)");
                    }
                    else if(operand_kind.kind == "LiteralExtInstInteger")
                    {
                        state << indent(R"(value = words[word_index++];
)");
                    }
                    else if(operand_kind.kind == "LiteralString")
                    {
                        state << indent(
                            R"(value.clear();
bool done = false;
while(!done)
{
`if(word_index >= word_count)
``return semantics.handle_error(error_instruction_index + word_index, error_instruction_index, "string missing terminating null");
`Word word = words[word_index++];
`for(std::size_t i = 0; i < 4; i++)
`{
``unsigned char ch = word & 0xFFU;
``word >>= 8;
``if(ch == '\0')
``{
```done = true;
```if(word != 0)
````return semantics.handle_error(error_instruction_index + word_index, error_instruction_index, "string has non-zero padding");
``}
``else
``{
```value += ch;
``}
`}
}
)");
                    }
                    else if(operand_kind.kind == "LiteralSpecConstantOpInteger")
                    {
#warning finish
                        state << indent(R"(value = static_cast<)") << op_enum_name
                              << indent(true, R"(>(words[word_index++]);
)");
                    }
                    else
                    {
                        state << indent(
                            R"(static_assert(std::is_same<decltype(value), std::vector<Word> &>::value, "missing parse code for operand kind");
value.clear();
value.reserve(word_count - word_index);
while(word_index < word_count)
`value.push_back(words[word_index++]);
)");
                    }
                    break;
                }
                }
                push_indent.finish();
                state << indent(R"(`return nullptr;
}
)");
            }
            for(auto &instruction : top_level.instructions.instructions)
            {
                auto struct_name = get_enumerant_name(op_enum_name, instruction.opname, true);
                auto parse_function_name = get_parse_instruction_function_name(instruction.opname);
                state
                    << indent(R"(static std::unique_ptr<Parse_error> )") << parse_function_name
                    << indent(
                           true,
                           R"((const Word *words, std::size_t word_count, Semantics &semantics, std::size_t error_instruction_index)
{
`std::size_t word_index = 1; // skip opcode
)");
                auto push_indent2 = state.pushed_indent();
                state << indent << struct_name << " instruction;\n";
                if(!instruction.operands.empty())
                    state << indent("std::unique_ptr<Parse_error> parse_error;\n");
                for(auto &operand : instruction.operands.operands)
                {
                    auto parse_operand_function_name =
                        get_parse_operand_function_name(operand.kind);
                    auto member_name = get_member_name_from_operand(operand);
                    switch(operand.quantifier)
                    {
                    case ast::Instructions::Instruction::Operands::Operand::Quantifier::none:
                    {
                        state
                            << indent(R"(parse_error = )") << parse_operand_function_name
                            << indent(
                                   true,
                                   R"((words, word_count, semantics, error_instruction_index, word_index, instruction.)")
                            << member_name << indent(true, R"();
if(parse_error)
`return parse_error;
)");
                        break;
                    }
                    case ast::Instructions::Instruction::Operands::Operand::Quantifier::optional:
                    {
                        state
                            << indent(R"(if(word_index < word_count)
{
`instruction.)") << member_name
                            << indent(true, R"(.emplace();
`parse_error = )") << parse_operand_function_name
                            << indent(
                                   true,
                                   R"((words, word_count, semantics, error_instruction_index, word_index, *instruction.)")
                            << member_name << indent(true, R"();
`if(parse_error)
``return parse_error;
}
)");
                        break;
                    }
                    case ast::Instructions::Instruction::Operands::Operand::Quantifier::variable:
                    {
                        state
                            << indent(R"(while(word_index < word_count)
{
`instruction.)") << member_name
                            << indent(true, R"(.emplace_back();
`parse_error = )") << parse_operand_function_name
                            << indent(
                                   true,
                                   R"((words, word_count, semantics, error_instruction_index, word_index, instruction.)")
                            << member_name << indent(true, R"(.back());
`if(parse_error)
``return parse_error;
}
)");
                    }
                    }
                }
                push_indent2.finish();
                state << indent(R"(`if(word_index < word_count)
``return semantics.handle_error(error_instruction_index + word_index, error_instruction_index, "extra words at end of instruction");
`semantics.handle_instruction(std::move(instruction));
`return nullptr;
}
)");
            }
            state << indent(
                R"(static std::unique_ptr<Parse_error> parse_instruction(const Word *words, std::size_t word_count, Semantics &semantics, std::size_t error_instruction_index)
{
`Op op = static_cast<Op>(words[0] & 0xFFFFU);
`switch(op)
`{
)");
            for(auto &instruction : top_level.instructions.instructions)
            {
                auto push_indent2 = state.pushed_indent(2);
                auto enumerant_name = get_enumerant_name(op_enum_name, instruction.opname, true);
                auto parse_function_name = get_parse_instruction_function_name(instruction.opname);
                state << indent("case ") << op_enum_name << "::" << enumerant_name
                      << indent(true, R"(:
`return )") << parse_function_name
                      << indent(true, R"((words, word_count, semantics, error_instruction_index);
)");
            }
            state << indent(R"(`}
`return semantics.handle_error(error_instruction_index, error_instruction_index, json::ast::Number_value::append_unsigned_integer_to_string(static_cast<Word>(op), "unknown instruction: 0x", 0x10));
}

static std::unique_ptr<Parse_error> parse(const Word *words, std::size_t word_count, Semantics &semantics)
{
`std::size_t word_index = 0;
`if(word_index >= word_count)
``return semantics.handle_error(word_index, 0, "hit EOF when parsing magic number");
`if(words[word_index] != magic_number)
``return semantics.handle_error(word_index, 0, "invalid magic number");
`word_index++;
`if(word_index >= word_count)
``return semantics.handle_error(word_index, 0, "hit EOF when parsing SPIR-V version");
`if(words[word_index] & ~0xFFFF00UL)
``return semantics.handle_error(word_index, 0, "invalid SPIR-V version");
`auto input_major_version = words[word_index] >> 16;
`auto input_minor_version = (words[word_index] >> 8) & 0xFFU;
`semantics.handle_spirv_version(input_major_version, input_minor_version);
`if(input_major_version != major_version || input_minor_version > minor_version)
``return semantics.handle_error(word_index, 0, "SPIR-V version not supported");
`word_index++;
`if(word_index >= word_count)
``return semantics.handle_error(word_index, 0, "hit EOF when parsing generator's magic number");
`semantics.handle_generator_magic_number(words[word_index++]);
`if(word_index >= word_count)
``return semantics.handle_error(word_index, 0, "hit EOF when parsing id bound");
`semantics.handle_id_bound(words[word_index++]);
`if(word_index >= word_count)
``return semantics.handle_error(word_index, 0, "hit EOF when parsing SPIR-V shader header");
`if(words[word_index] != 0)
``return semantics.handle_error(word_index, 0, "nonzero reserved word in SPIR-V shader header");
`word_index++;
`// now we've finished reading the shader header, the rest of the shader is just instructions
`while(word_index < word_count)
`{
``auto instruction_word_count = words[word_index] >> 16;
``if(instruction_word_count == 0)
```return semantics.handle_error(word_index, word_index, "invalid instruction");
``if(word_index + instruction_word_count > word_count)
```return semantics.handle_error(word_index, word_index, "instruction longer than rest of shader");
``auto parse_error = parse_instruction(words + word_index, instruction_word_count, semantics, word_index);
``if(parse_error)
```return parse_error;
``word_index += instruction_word_count;
`}
`return nullptr;
}
)");
#warning finish
        }
        state << "};\n";
#warning finish
        write_namespaces_end(state, spirv_namespace_names);
        write_file_guard_end(state);
    }
};

struct Parser_source_generator final : public Generator
{
    Parser_source_generator() : Generator("parser.cpp")
    {
    }
    virtual void run(Generator_args &generator_args, const ast::Top_level &top_level) const override
    {
        Generator_state state(this, generator_args, top_level);
        state.open_output_file();
        write_file_comments(state, top_level.copyright);
        state << "#include \"parser.h\"\n"
                 "#include <type_traits>\n"
                 "\n";
        write_namespaces_start(state, spirv_namespace_names);
        state << indent(R"(namespace
{
/** instantiate Parser with Parse_semantics_generic to help catch bugs */
[[gnu::unused]] auto parser_test(const Word *words, std::size_t word_count, Parse_semantics_generic &semantics)
{
`return Parser<>::parse(words, word_count, semantics);
}
}

std::unique_ptr<Parse_error> Parse_dump::handle_error(std::size_t word_index, std::size_t instruction_word_index, std::string message)
{
`return std::unique_ptr<Parse_error>(new Parse_error(word_index, instruction_word_index, std::move(message)));
}

void Parse_dump::handle_spirv_version(unsigned major, unsigned minor)
{
`os << "SPIR-V version " << major << "." << minor << "\n";
}

void Parse_dump::handle_generator_magic_number(Word value)
{
`os << "generator magic number: " << json::ast::Number_value::append_unsigned_integer_to_string(value, "0x", 0x10) << "\n";
}

void Parse_dump::handle_id_bound(Word id_bound)
{
`os << "id bound: " << json::ast::Number_value::unsigned_integer_to_string(id_bound) << "\n";
}
)");
        for(auto &operand_kind : top_level.operand_kinds.operand_kinds)
        {
            auto dump_function_name =
                Parser_header_generator::get_dump_operand_function_name(operand_kind.kind);
            {
                state << indent(R"(
void Parse_dump::)") << dump_function_name
                      << "(const " << operand_kind.kind << R"( &v)
{
)";
                auto push_indent = state.pushed_indent();
                switch(operand_kind.category)
                {
                case ast::Operand_kinds::Operand_kind::Category::bit_enum:
                {
                    state << indent(R"(Word bits = static_cast<Word>(v);
util::Enum_set<)") << operand_kind.kind
                          << indent(true, R"(> enum_set{};
for(auto value : util::Enum_traits<)")
                          << operand_kind.kind << indent(true, R"(>::values)
{
`if(static_cast<Word>(value) == 0)
`{
``if(v == value)
```enum_set.insert(value);
``continue;
`}
`if((bits & static_cast<Word>(value)) == static_cast<Word>(value))
`{
``bits &= ~static_cast<Word>(value);
``enum_set.insert(value);
`}
}
bool first = true;
for(auto value : enum_set)
{
`if(first)
``first = false;
`else
``os << " | ";
`os << get_enumerant_name(value);
}
if(bits)
{
`if(!first)
``os << " | ";
`os << json::ast::Number_value::append_unsigned_integer_to_string(bits, "0x", 0x10);
}
else if(first)
{
`os << "0";
}
)");
                    break;
                }
                case ast::Operand_kinds::Operand_kind::Category::value_enum:
                {
                    state << indent(R"(if(util::Enum_traits<)") << operand_kind.kind
                          << indent(true, R"(>::find_value(v) == util::Enum_traits<)")
                          << operand_kind.kind << indent(true, R"(>::npos)
`os << json::ast::Number_value::unsigned_integer_to_string(static_cast<Word>(v));
else
`os << get_enumerant_name(v);
)");
                    break;
                }
                case ast::Operand_kinds::Operand_kind::Category::composite:
                {
                    auto &bases =
                        util::get<ast::Operand_kinds::Operand_kind::Bases>(operand_kind.value);
                    state << indent("os << \"{\";\n");
                    for(std::size_t i = 0; i < bases.values.size(); i++)
                    {
                        if(i != 0)
                        {
                            state << indent("os << \", \";\n");
                        }
                        state << indent << Parser_header_generator::get_dump_operand_function_name(
                                               bases.values[i])
                              << "(v."
                              << json::ast::Number_value::append_unsigned_integer_to_string(i + 1,
                                                                                            "part_")
                              << ");\n";
                    }
                    state << indent("os << \"}\";\n");
                    break;
                }
                case ast::Operand_kinds::Operand_kind::Category::id:
                {
                    state << indent(
                        R"(os << json::ast::Number_value::append_unsigned_integer_to_string(v, "#");
)");
                    break;
                }
                case ast::Operand_kinds::Operand_kind::Category::literal:
                {
                    if(operand_kind.kind == "LiteralInteger")
                    {
                        state << indent(
                            R"(os << json::ast::Number_value::append_unsigned_integer_to_string(v, "0x");
)");
                    }
                    else if(operand_kind.kind == "LiteralExtInstInteger")
                    {
                        state << indent(
                            R"(os << json::ast::Number_value::append_unsigned_integer_to_string(v, "0x");
)");
                    }
                    else if(operand_kind.kind == "LiteralString")
                    {
                        state << indent(
                            R"(json::ast::String_value::write(os, v);
)");
                    }
                    else if(operand_kind.kind == "LiteralSpecConstantOpInteger")
                    {
                        state << indent(R"(if(util::Enum_traits<)") << op_enum_name
                              << indent(true, R"(>::find_value(v) == util::Enum_traits<)")
                              << op_enum_name << indent(true, R"(>::npos)
`os << json::ast::Number_value::unsigned_integer_to_string(static_cast<Word>(v));
else
`os << get_enumerant_name(v);
)");
                    }
                    else
                    {
                        state << indent(
                            R"(static_assert(std::is_same<decltype(v), const std::vector<Word> &>::value, "missing dump code for operand kind");
auto separator = "";
os << "{";
for(Word value : v)
{
`os << separator;
`separator = ", ";
`os << json::ast::Number_value::append_unsigned_integer_to_string(value, "0x", 0x10, 8);
}
os << "}";
)");
                    }
                    break;
                }
                }
                push_indent.finish();
                state << indent("}\n");
            }
            state << indent(R"(
void Parse_dump::)")
                  << dump_function_name << "(const util::optional<" << operand_kind.kind
                  << indent(true, R"(> &v)
{
`if(v)
)") << indent(2) << dump_function_name
                  << indent(true, R"((*v);
`else
``os << "nullopt";
}

void Parse_dump::)")
                  << dump_function_name << "(const std::vector<" << operand_kind.kind
                  << indent(true, R"(> &v)
{
`auto separator = "";
`os << "{";
`for(auto &value : v)
`{
``os << separator;
``separator = ", ";
)") << indent(2) << dump_function_name
                  << indent(true, R"((value);
`}
`os << "}";
}
)");
        }
        for(auto &instruction : top_level.instructions.instructions)
        {
            auto struct_name = get_enumerant_name(op_enum_name, instruction.opname, true);
            state << indent(
                         "\n"
                         "void Parse_dump::handle_instruction(")
                  << struct_name << indent(true, R"( instruction)
{
`os << ")") << instruction.opname
                  << indent(true, R"(\n";
)");
            for(auto &operand : instruction.operands.operands)
            {
                auto push_indent = state.pushed_indent();
                auto member_name = get_member_name_from_operand(operand);
                state << indent("os << \"    ") << member_name << indent(true, R"(:";
)") << indent << Parser_header_generator::get_dump_operand_function_name(operand.kind)
                      << indent(true, R"((instruction.)") << member_name << indent(true, R"();
os << "\n";
)");
            }
            state << indent("}\n");
        }
        write_namespaces_end(state, spirv_namespace_names);
    }
};

std::unique_ptr<Generator> Generators::make_spirv_header_generator()
{
    return std::unique_ptr<Generator>(new Spirv_header_generator);
}

std::unique_ptr<Generator> Generators::make_spirv_source_generator()
{
    return std::unique_ptr<Generator>(new Spirv_source_generator);
}

std::unique_ptr<Generator> Generators::make_parser_header_generator()
{
    return std::unique_ptr<Generator>(new Parser_header_generator);
}

std::unique_ptr<Generator> Generators::make_parser_source_generator()
{
    return std::unique_ptr<Generator>(new Parser_source_generator);
}

std::vector<std::unique_ptr<Generator>> Generators::make_all_generators()
{
    std::unique_ptr<Generator> generators_array[] = {
        make_spirv_header_generator(),
        make_spirv_source_generator(),
        make_parser_header_generator(),
        make_parser_source_generator(),
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
