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
#include "util/string_view.h"
#include "util/optional.h"
#include <limits>
#include <algorithm>
#include <cstdlib>
#include <iostream>

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
namespace generate
{
Generator::Generator_state::Generator_state(const Generator *generator,
                                            Generator_args &generator_args)
    : generator_args(generator_args),
      indent_level(0),
      full_output_file_name(generator_args.output_directory + "/"
                            + generator->output_base_file_name),
      guard_macro_name(get_guard_macro_name_from_file_name(full_output_file_name)),
      os()
{
    os.exceptions(std::ios::badbit | std::ios::failbit);
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

void Generator::write_indent(Generator_state &state)
{
    static constexpr auto indent_string = "    ";
    for(std::size_t i = state.indent_level; i > 0; i--)
        state << indent_string;
}

void Generator::write_automatically_generated_file_warning(Generator_state &state)
{
    state << "/* This file is automatically generated by "
             "generate_spirv_parser. DO NOT MODIFY. */\n";
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
    state << "#ifdef " << state.guard_macro_name << "\n#define " << state.guard_macro_name
          << "\n\n";
}

void Generator::write_file_guard_end(Generator_state &state)
{
    state << "#endif /* " << state.guard_macro_name << " */\n";
}

void Generator::write_namespace_start(Generator_state &state, const char *namespace_name)
{
    state << "namespace " << namespace_name << "\n{\n";
}

void Generator::write_namespace_start(Generator_state &state, const std::string &namespace_name)
{
    state << "namespace " << namespace_name << "\n{\n";
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

std::string Generator::get_member_name_from_words(const std::string &words)
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
    return retval;
}

#if 0
#warning testing Generator::get_member_name_from_words
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
            std::cout << "\"" << input << "\" -> " << get_member_name_from_words(input)
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
        return get_member_name_from_words(operand.name);
    util::string_view id_str = "Id";
    if(util::string_view(operand.kind).compare(0, id_str.size(), id_str) == 0
       && id_str.size() < operand.kind.size()
       && is_uppercase_letter(operand.kind[id_str.size()]))
        return get_member_name_from_words(operand.kind.substr(id_str.size()));
    return get_member_name_from_words(operand.kind);
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
    state << indent << "{\n";
    state << indent << "}\n";
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
        state << indent << "{\n";
        state << indent << "}\n";
    }
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
    static bool compare_operand_kinds(const ast::Operand_kinds::Operand_kind &l,
                                      const ast::Operand_kinds::Operand_kind &r)
    {
        // treat both enum categories as equivalent
        auto l_category = l.category == ast::Operand_kinds::Operand_kind::Category::bit_enum ?
                              ast::Operand_kinds::Operand_kind::Category::value_enum :
                              l.category;
        auto r_category = r.category == ast::Operand_kinds::Operand_kind::Category::bit_enum ?
                              ast::Operand_kinds::Operand_kind::Category::value_enum :
                              r.category;
        if(l_category != r_category)
            return l_category < r_category;
        return compare_enum_names(l.kind, r.kind);
    }
    virtual void run(Generator_args &generator_args, const ast::Top_level &top_level) const override
    {
        Generator_state state(this, generator_args);
        state.open_output_file();
        write_file_comments(state, top_level.copyright);
        write_file_guard_start(state);
        state << "#include <cstdint>\n"
                 "#include \"util/enum.h\"\n"
                 "#include \"util/optional.h\"\n"
                 "#include <vector>\n";
        state << "\n";
        write_namespaces_start(state, spirv_namespace_names);
        state << "typedef std::uint32_t Word;\n";
        state << "typedef Word Id;\n";
        state << "constexpr Word magic_number = "
              << unsigned_hex_integer_literal(top_level.magic_number, 8) << ";\n";
        state << "constexpr std::uint32_t major_version = "
              << unsigned_dec_integer_literal(top_level.major_version) << ";\n";
        state << "constexpr std::uint32_t minor_version = "
              << unsigned_dec_integer_literal(top_level.minor_version) << ";\n";
        state << "constexpr std::uint32_t revision = "
              << unsigned_dec_integer_literal(top_level.revision) << ";\n";
        auto extensions_set = get_extensions(top_level);
        std::vector<std::string> extensions_list;
        extensions_list.reserve(extensions_set.size());
        for(auto &extension : extensions_set)
            extensions_list.push_back(extension);
        std::sort(extensions_list.begin(), extensions_list.end());
        state << "\n"
                 "enum class "
              << extension_enum_name << " : std::size_t\n"
                                        "{\n";
        {
            auto push_indent = state.pushed_indent();
            for(auto &extension : extensions_list)
                state << indent << get_enumerant_name(extension_enum_name, extension, false)
                      << ",\n";
        }
        state << "};\n"
                 "\n"
                 "vulkan_cpu_util_generate_enum_traits("
              << extension_enum_name;
        {
            auto push_indent = state.pushed_indent();
            for(auto &extension : extensions_list)
                state << ",\n" << indent << extension_enum_name
                      << "::" << get_enumerant_name(extension_enum_name, extension, false);
            state << ");\n";
        }
        state << "\n"
                 "constexpr const char *get_enumerant_name("
              << extension_enum_name << " v) noexcept\n"
                                        "{\n";
        {
            auto push_indent = state.pushed_indent();
            state << indent << "switch(v)\n" << indent << "{\n";
            for(auto &extension : extensions_list)
            {
                state << indent << "case " << extension_enum_name
                      << "::" << get_enumerant_name(extension_enum_name, extension, false) << ":\n";
                auto push_indent2 = state.pushed_indent();
                state << indent << "return \"" << extension << "\";\n";
            }
            state << indent << "}\n" << indent << "return \"\";\n";
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
                state << "\n"
                         "enum class "
                      << operand_kind->kind << " : Word\n"
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
                state << "};\n";
                if(!is_bit_enum)
                {
                    state << "\n"
                             "vulkan_cpu_util_generate_enum_traits("
                          << operand_kind->kind;
                    {
                        auto push_indent = state.pushed_indent();
                        for(auto &enumerant : enumerants.enumerants)
                            state << ",\n" << indent << operand_kind->kind
                                  << "::" << get_enumerant_name(
                                                 operand_kind->kind, enumerant.enumerant, false);
                        state << ");\n";
                    }
                }
                state << "\n"
                         "constexpr const char *get_enumerant_name("
                      << operand_kind->kind << " v) noexcept\n"
                                               "{\n";
                {
                    auto push_indent = state.pushed_indent();
                    state << indent << "switch(v)\n" << indent << "{\n";
                    for(auto &enumerant : enumerants.enumerants)
                    {
                        state << indent << "case " << operand_kind->kind << "::"
                              << get_enumerant_name(operand_kind->kind, enumerant.enumerant, false)
                              << ":\n";
                        auto push_indent2 = state.pushed_indent();
                        state << indent << "return \"" << enumerant.enumerant << "\";\n";
                    }
                    state << indent << "}\n" << indent << "return \"\";\n";
                }
                state << "}\n"
                         "\n"
                         "constexpr util::Enum_set<"
                      << capability_enum_name << "> get_directly_required_capability_set("
                      << operand_kind->kind << " v) noexcept\n"
                                               "{\n";
                {
                    auto push_indent = state.pushed_indent();
                    state << indent << "switch(v)\n" << indent << "{\n";
                    for(auto &enumerant : enumerants.enumerants)
                    {
                        state << indent << "case " << operand_kind->kind << "::"
                              << get_enumerant_name(operand_kind->kind, enumerant.enumerant, false)
                              << ":\n";
                        auto push_indent2 = state.pushed_indent();
                        state << indent << "return " << enumerant.capabilities << ";\n";
                    }
                    state << indent << "}\n" << indent << "return {};\n";
                }
                state << "}\n"
                         "\n"
                         "constexpr util::Enum_set<"
                      << extension_enum_name << "> get_directly_required_extension_set("
                      << operand_kind->kind << " v) noexcept\n"
                                               "{\n";
                {
                    auto push_indent = state.pushed_indent();
                    state << indent << "switch(v)\n" << indent << "{\n";
                    for(auto &enumerant : enumerants.enumerants)
                    {
                        state << indent << "case " << operand_kind->kind << "::"
                              << get_enumerant_name(operand_kind->kind, enumerant.enumerant, false)
                              << ":\n";
                        auto push_indent2 = state.pushed_indent();
                        state << indent << "return " << enumerant.extensions << ";\n";
                    }
                    state << indent << "}\n" << indent << "return {};\n";
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
                std::vector<std::string> member_names;
                member_names.reserve(bases.values.size());
                for(std::size_t i = 0; i < bases.values.size(); i++)
                    member_names.push_back(
                        json::ast::Number_value::append_unsigned_integer_to_string(i + 1, "part_"));
                write_struct_nonstatic_members_and_constructors(state,
                                                                operand_kind->kind,
                                                                bases.values.data(),
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
                    base_type = "std::uint64_t";
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
            state << indent << "switch(v)\n" << indent << "{\n";
            for(auto &instruction : top_level.instructions.instructions)
            {
                state << indent << "case " << op_enum_name
                      << "::" << get_enumerant_name(op_enum_name, instruction.opname, true)
                      << ":\n";
                auto push_indent2 = state.pushed_indent();
                state << indent << "return \"" << instruction.opname << "\";\n";
            }
            state << indent << "}\n" << indent << "return \"\";\n";
        }
        state << "}\n"
                 "\n"
                 "constexpr util::Enum_set<"
              << capability_enum_name << "> get_directly_required_capability_set(" << op_enum_name
              << " v) noexcept\n"
                 "{\n";
        {
            auto push_indent = state.pushed_indent();
            state << indent << "switch(v)\n" << indent << "{\n";
            for(auto &instruction : top_level.instructions.instructions)
            {
                state << indent << "case " << op_enum_name
                      << "::" << get_enumerant_name(op_enum_name, instruction.opname, true)
                      << ":\n";
                auto push_indent2 = state.pushed_indent();
                state << indent << "return " << instruction.capabilities << ";\n";
            }
            state << indent << "}\n" << indent << "return {};\n";
        }
        state << "}\n"
                 "\n"
                 "constexpr util::Enum_set<"
              << extension_enum_name << "> get_directly_required_extension_set(" << op_enum_name
              << " v) noexcept\n"
                 "{\n";
        {
            auto push_indent = state.pushed_indent();
            state << indent << "switch(v)\n" << indent << "{\n";
            for(auto &instruction : top_level.instructions.instructions)
            {
                state << indent << "case " << op_enum_name
                      << "::" << get_enumerant_name(op_enum_name, instruction.opname, true)
                      << ":\n";
                auto push_indent2 = state.pushed_indent();
                state << indent << "return " << instruction.extensions << ";\n";
            }
            state << indent << "}\n" << indent << "return {};\n";
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
                state << indent << "static constexpr " << op_enum_name << " get_opcode() noexcept\n"
                      << indent << "{\n";
                {
                    auto push_indent2 = state.pushed_indent();
                    state << indent << "return " << op_enum_name
                          << "::" << get_enumerant_name(op_enum_name, instruction.opname, true)
                          << ";\n";
                }
                state << indent << "}\n";
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

std::unique_ptr<Generator> Generators::make_spirv_header_generator()
{
    return std::unique_ptr<Generator>(new Spirv_header_generator);
}

std::vector<std::unique_ptr<Generator>> Generators::make_all_generators()
{
    std::unique_ptr<Generator> generators_array[] = {
        make_spirv_header_generator(),
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
