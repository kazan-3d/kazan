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
#include "instruction_properties.h"
#include <fstream>
#include <iostream>
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
constexpr util::string_view detail::Generated_output_stream::push_start_command;
constexpr util::string_view detail::Generated_output_stream::pop_start_command;
constexpr util::string_view detail::Generated_output_stream::add_start_offset_command;
constexpr util::string_view detail::Generated_output_stream::restart_indent_command;
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
        std::vector<std::size_t> start_indent_depth_stack;
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
                            else if(command_sv.compare(0,
                                                       add_start_offset_command.size(),
                                                       add_start_offset_command)
                                    == 0)
                            {
                                auto arg = command_sv.substr(add_start_offset_command.size());
                                std::int64_t offset = 0;
                                bool is_negative_offset = false;
                                assert(!arg.empty());
                                if(arg.front() == '-')
                                {
                                    arg.remove_prefix(1);
                                    is_negative_offset = true;
                                }
                                do
                                {
                                    offset *= 10;
                                    assert(!arg.empty() && arg.front() >= '0'
                                           && arg.front() <= '9');
                                    offset += arg.front() - '0';
                                    arg.remove_prefix(1);
                                } while(!arg.empty());
                                if(is_negative_offset)
                                    offset = -offset;
                                assert(offset > 0
                                       || start_indent_depth >= static_cast<std::size_t>(-offset));
                                start_indent_depth += offset;
                                continue;
                            }
                            else if(command_sv == push_start_command)
                            {
                                start_indent_depth_stack.push_back(start_indent_depth);
                                continue;
                            }
                            else if(command_sv == pop_start_command)
                            {
                                assert(!start_indent_depth_stack.empty());
                                start_indent_depth = start_indent_depth_stack.back();
                                start_indent_depth_stack.pop_back();
                                continue;
                            }
                            else if(command_sv == restart_indent_command)
                            {
                                indent_depth = start_indent_depth;
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

namespace
{
enum class Output_part
{
    file_comments,
    include_guard_start,
    includes,
    namespaces_start,
    basic_types,
    basic_constants,
    id_types,
    enum_definitions,
    enum_properties_definitions,
    literal_types,
    enum_structs,
    composite_types,
    instruction_structs,
    parse_error_class,
    parser_callbacks_class,
    dump_callbacks_class,
    parser_class,
    namespaces_end,
    include_guard_end,

    struct_opening,
    struct_members,
    struct_default_constructor,
    struct_default_constructor_initializers,
    struct_default_constructor_body,
    struct_fill_constructor_start,
    struct_fill_constructor_args,
    struct_fill_constructor_args_end,
    struct_fill_constructor_initializers,
    struct_fill_constructor_body,
    struct_closing,
};

vulkan_cpu_util_generate_enum_traits(Output_part,
                                     Output_part::file_comments,
                                     Output_part::include_guard_start,
                                     Output_part::includes,
                                     Output_part::namespaces_start,
                                     Output_part::basic_types,
                                     Output_part::basic_constants,
                                     Output_part::id_types,
                                     Output_part::enum_definitions,
                                     Output_part::enum_properties_definitions,
                                     Output_part::literal_types,
                                     Output_part::enum_structs,
                                     Output_part::composite_types,
                                     Output_part::instruction_structs,
                                     Output_part::parse_error_class,
                                     Output_part::parser_callbacks_class,
                                     Output_part::dump_callbacks_class,
                                     Output_part::parser_class,
                                     Output_part::namespaces_end,
                                     Output_part::include_guard_end,
                                     Output_part::struct_opening,
                                     Output_part::struct_members,
                                     Output_part::struct_default_constructor,
                                     Output_part::struct_default_constructor_initializers,
                                     Output_part::struct_default_constructor_body,
                                     Output_part::struct_fill_constructor_start,
                                     Output_part::struct_fill_constructor_args,
                                     Output_part::struct_fill_constructor_args_end,
                                     Output_part::struct_fill_constructor_initializers,
                                     Output_part::struct_fill_constructor_body,
                                     Output_part::struct_closing);

static_assert(util::Enum_traits<Output_part>::is_compact,
              "mismatch between declaration and generate enum traits");
}

struct Spirv_and_parser_generator::State
{
private:
    struct Operand_descriptor;
    struct Output_base
    {
        Output_base(const Output_base &) = delete;
        Output_base &operator=(const Output_base &) = delete;
        typedef void (Output_base::*Write_function)(detail::Generated_output_stream &output_stream,
                                                    Output_part part) const;
        typedef util::variant<const detail::Generated_output_stream *,
                              const Output_base *,
                              Write_function> Output_part_value;
        util::Enum_map<Output_part, Output_part_value> output_parts;
        const util::filesystem::path file_path;
        template <typename T>
        void register_output_part(Output_part part,
                                  const detail::Generated_output_stream T::*variable)
        {
            static_assert(std::is_base_of<Output_base, T>::value, "");
            assert(dynamic_cast<T *>(this));
            auto *derived_class = static_cast<T *>(this);
            output_parts.insert_or_assign(part, &(derived_class->*variable));
        }
        template <typename T>
        void register_output_part(Output_part part,
                                  void (T::*write_function)(detail::Generated_output_stream &,
                                                            Output_part) const)
        {
            static_assert(std::is_base_of<Output_base, T>::value, "");
            assert(dynamic_cast<T *>(this));
            output_parts.insert_or_assign(part, static_cast<Write_function>(write_function));
        }
        template <typename T, typename T2>
        void register_output_part(
            typename std::enable_if<std::is_base_of<Output_base, T>::value, Output_part>::type part,
            const T T2::*variable)
        {
            static_assert(std::is_base_of<Output_base, T2>::value, "");
            assert(dynamic_cast<T2 *>(this));
            auto *derived_class = static_cast<T2 *>(this);
            output_parts.insert_or_assign(part, &(derived_class->*variable));
        }
        explicit Output_base(const util::filesystem::path &file_path)
            : output_parts(), file_path(file_path)
        {
        }
        void write_whole_output(detail::Generated_output_stream &output_stream) const
        {
            for(auto &part : output_parts)
            {
                struct Visitor
                {
                    detail::Generated_output_stream &output_stream;
                    const Output_base *this_;
                    Output_part output_part;
                    void operator()(const Output_base *v)
                    {
                        v->write_whole_output(output_stream);
                    }
                    void operator()(const detail::Generated_output_stream *v)
                    {
                        output_stream << *v;
                    }
                    void operator()(Write_function write_function)
                    {
                        (this_->*write_function)(output_stream, output_part);
                    }
                };
                util::visit(Visitor{output_stream, this, std::get<0>(part)}, std::get<1>(part));
            }
        }
        detail::Generated_output_stream get_whole_output() const
        {
            detail::Generated_output_stream retval(file_path);
            write_whole_output(retval);
            return retval;
        }
        virtual void write_to_file() const
        {
            get_whole_output().write_to_file();
        }
    };
    struct Output_struct final : public Output_base
    {
        const std::string struct_name;
        detail::Generated_output_stream struct_members;
        detail::Generated_output_stream struct_default_constructor_initializers;
        detail::Generated_output_stream struct_fill_constructor_args;
        detail::Generated_output_stream struct_fill_constructor_initializers;
        std::size_t nonstatic_member_count;
        Output_struct(const util::filesystem::path &file_path, util::string_view struct_name)
            : Output_base(file_path),
              struct_name(struct_name),
              struct_members(file_path),
              struct_default_constructor_initializers(file_path),
              struct_fill_constructor_args(file_path),
              struct_fill_constructor_initializers(file_path),
              nonstatic_member_count(0)
        {
            register_output_part(Output_part::struct_opening, &Output_struct::write_output_part);
            register_output_part(Output_part::struct_members, &Output_struct::struct_members);
            register_output_part(Output_part::struct_default_constructor,
                                 &Output_struct::write_output_part);
            register_output_part(Output_part::struct_default_constructor_initializers,
                                 &Output_struct::struct_default_constructor_initializers);
            register_output_part(Output_part::struct_default_constructor_body,
                                 &Output_struct::write_output_part);
            register_output_part(Output_part::struct_fill_constructor_start,
                                 &Output_struct::write_output_part);
            register_output_part(Output_part::struct_fill_constructor_args,
                                 &Output_struct::struct_fill_constructor_args);
            register_output_part(Output_part::struct_fill_constructor_args_end,
                                 &Output_struct::write_output_part);
            register_output_part(Output_part::struct_fill_constructor_initializers,
                                 &Output_struct::struct_fill_constructor_initializers);
            register_output_part(Output_part::struct_fill_constructor_body,
                                 &Output_struct::write_output_part);
            register_output_part(Output_part::struct_closing, &Output_struct::write_output_part);
        }
        std::string get_struct_fill_constructor_start() const
        {
            std::string retval;
            if(nonstatic_member_count == 1)
                retval += "explicit ";
            retval += struct_name;
            retval += '(';
            return retval;
        }
        void write_output_part(detail::Generated_output_stream &output_stream,
                               Output_part part) const
        {
            switch(part)
            {
            case Output_part::struct_opening:
                output_stream << "struct " << struct_name << R"(
{
@+)";
                return;
            case Output_part::struct_default_constructor:
                output_stream << struct_name << R"(()
@+)";
                if(nonstatic_member_count > 0)
                    output_stream << detail::add_start_offset(2) << ": ";
                return;
            case Output_part::struct_default_constructor_body:
                if(nonstatic_member_count > 0)
                    output_stream << "\n" << detail::add_start_offset(-2) << detail::restart_indent;
                output_stream << R"(@-{
}
)";
                return;
            case Output_part::struct_fill_constructor_start:
                if(nonstatic_member_count > 0)
                {
                    auto struct_fill_constructor_start = get_struct_fill_constructor_start();
                    output_stream << detail::push_start
                                  << detail::add_start_offset(struct_fill_constructor_start.size())
                                  << struct_fill_constructor_start;
                }
                return;
            case Output_part::struct_fill_constructor_args_end:
                if(nonstatic_member_count > 0)
                {
                    output_stream << R"()
)" << detail::pop_start << detail::restart_indent
                                  << detail::add_start_offset(2) << R"(@+: )";
                }
                return;
            case Output_part::struct_fill_constructor_body:
                if(nonstatic_member_count > 0)
                {
                    output_stream << "\n" << detail::add_start_offset(-2) << detail::restart_indent
                                  << R"(@-{
}
)";
                }
                return;
            case Output_part::struct_closing:
                output_stream << R"(@-};
)";
                return;
            default:
                break;
            }
            assert(false);
        }
        static util::string_view get_variable_declaration_type_name_separator(
            util::string_view type)
        {
            assert(!type.empty());
            if(type.back() == '&' || type.back() == '*')
                return ""_sv;
            return " "_sv;
        }
        void add_nonstatic_member(util::string_view member_type,
                                  util::string_view member_name,
                                  bool needs_move)
        {
            if(nonstatic_member_count != 0)
            {
                struct_default_constructor_initializers << ",\n";
                struct_fill_constructor_initializers << ",\n";
                struct_fill_constructor_args << ",\n";
            }
            nonstatic_member_count++;
            struct_members << member_type
                           << get_variable_declaration_type_name_separator(member_type)
                           << member_name << ";\n";
            struct_default_constructor_initializers << member_name << "()";
            auto move_start = ""_sv;
            auto move_end = ""_sv;
            if(needs_move)
            {
                move_start = "std::move("_sv;
                move_end = ")"_sv;
            }
            struct_fill_constructor_initializers << member_name << "(" << move_start << member_name
                                                 << move_end << ")";
            struct_fill_constructor_args
                << member_type << get_variable_declaration_type_name_separator(member_type)
                << member_name;
        }
    };
    struct Output_file_base : public Output_base
    {
        detail::Generated_output_stream file_comments;
        detail::Generated_output_stream includes;
        detail::Generated_output_stream namespaces_start;
        detail::Generated_output_stream namespaces_end;
        explicit Output_file_base(const util::filesystem::path &file_path)
            : Output_base(file_path),
              file_comments(file_path),
              includes(file_path),
              namespaces_start(file_path),
              namespaces_end(file_path)
        {
            register_output_part(Output_part::file_comments, &Output_file_base::file_comments);
            register_output_part(Output_part::includes, &Output_file_base::includes);
            register_output_part(Output_part::namespaces_start,
                                 &Output_file_base::namespaces_start);
            register_output_part(Output_part::namespaces_end, &Output_file_base::namespaces_end);
        }
        virtual void fill_output(State &state)
        {
            constexpr auto automatically_generated_file_warning_comment =
                R"(/* This file is automatically generated by generate_spirv_parser. DO NOT MODIFY. */
)"_sv;
            file_comments << automatically_generated_file_warning_comment
                          << state.top_level.copyright;
            namespaces_start << R"(
namespace vulkan_cpu
{
namespace spirv
{
)";
            namespaces_end << R"(}
}
)";
        }
        void write_local_include_string(util::string_view header_file)
        {
            includes << R"(#include ")" << header_file << R"("
)";
        }
        void write_local_include_path(util::filesystem::path header_file)
        {
            auto dir_path = file_path;
            dir_path.remove_filename();
            write_local_include_string(header_file.lexically_proximate(dir_path).generic_string());
        }
        void write_system_include(util::string_view header_file)
        {
            includes << R"(#include <)" << header_file << R"(>
)";
        }
    };
    struct Header_file_base : public Output_file_base
    {
        detail::Generated_output_stream include_guard_start;
        detail::Generated_output_stream include_guard_end;
        explicit Header_file_base(const util::filesystem::path &file_path)
            : Output_file_base(file_path),
              include_guard_start(file_path),
              include_guard_end(file_path)
        {
            register_output_part(Output_part::include_guard_start,
                                 &Header_file_base::include_guard_start);
            register_output_part(Output_part::include_guard_end,
                                 &Header_file_base::include_guard_end);
        }
        virtual void fill_output(State &state) override
        {
            using detail::guard_macro;
            Output_file_base::fill_output(state);
            include_guard_start << R"(#ifndef )" << guard_macro << R"(
#define )" << guard_macro << R"(

)";
            include_guard_end << R"(
#endif /* )" << guard_macro << R"( */
)";
        }
    };
    struct Source_file_base : public Output_file_base
    {
        const Header_file_base *const header;
        explicit Source_file_base(const util::filesystem::path &file_path,
                                  const Header_file_base *header)
            : Output_file_base(file_path), header(header)
        {
        }
        virtual void fill_output(State &state) override
        {
            using detail::guard_macro;
            Output_file_base::fill_output(state);
            write_local_include_path(header->file_path);
        }
    };
    struct Spirv_h : public Header_file_base
    {
        detail::Generated_output_stream basic_types;
        detail::Generated_output_stream basic_constants;
        detail::Generated_output_stream id_types;
        detail::Generated_output_stream enum_definitions;
        detail::Generated_output_stream enum_properties_definitions;
        detail::Generated_output_stream literal_types;
        std::list<Output_struct> composite_types;
        std::list<Output_struct> enum_structs;
        std::list<Output_struct> instruction_structs;
        explicit Spirv_h(const util::filesystem::path &file_path)
            : Header_file_base(file_path),
              basic_types(file_path),
              basic_constants(file_path),
              id_types(file_path),
              enum_definitions(file_path),
              enum_properties_definitions(file_path),
              literal_types(file_path)
        {
            register_output_part(Output_part::basic_types, &Spirv_h::basic_types);
            register_output_part(Output_part::basic_constants, &Spirv_h::basic_constants);
            register_output_part(Output_part::id_types, &Spirv_h::id_types);
            register_output_part(Output_part::enum_definitions, &Spirv_h::enum_definitions);
            register_output_part(Output_part::enum_properties_definitions,
                                 &Spirv_h::enum_properties_definitions);
            register_output_part(Output_part::literal_types, &Spirv_h::literal_types);
            register_output_part(Output_part::enum_structs, &Spirv_h::write_enum_structs);
            register_output_part(Output_part::composite_types, &Spirv_h::write_composite_types);
            register_output_part(Output_part::instruction_structs,
                                 &Spirv_h::write_instruction_structs);
        }
        void write_enum_structs(detail::Generated_output_stream &output_stream, Output_part) const
        {
            for(auto &enum_struct : enum_structs)
            {
                output_stream << "\n";
                enum_struct.write_whole_output(output_stream);
            }
        }
        void write_composite_types(detail::Generated_output_stream &output_stream,
                                   Output_part) const
        {
            for(auto &composite_type : composite_types)
            {
                output_stream << "\n";
                composite_type.write_whole_output(output_stream);
            }
        }
        void write_instruction_structs(detail::Generated_output_stream &output_stream,
                                       Output_part) const
        {
            for(auto &instruction_struct : instruction_structs)
            {
                output_stream << "\n";
                instruction_struct.write_whole_output(output_stream);
            }
        }
        void write_literal_kinds(State &state)
        {
            for(auto &operand_kind : state.top_level.operand_kinds.operand_kinds)
            {
                if(operand_kind.category != ast::Operand_kinds::Operand_kind::Category::literal)
                    continue;
                auto literal_kind =
                    ast::Operand_kinds::Operand_kind::get_literal_kind_from_json_name(
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
                case ast::Operand_kinds::Operand_kind::Literal_kind::
                    literal_context_dependent_number:
                    underlying_type = "std::vector<Word>"_sv;
                    break;
                case ast::Operand_kinds::Operand_kind::Literal_kind::literal_ext_inst_integer:
                    underlying_type = "Word"_sv;
                    break;
                case ast::Operand_kinds::Operand_kind::Literal_kind::
                    literal_spec_constant_op_integer:
                    underlying_type = state.op_enumeration.value()->cpp_name;
                    break;
                }
                auto &descriptor = state.literal_type_descriptors.at(*literal_kind);
                literal_types << R"(
typedef )" << underlying_type << " "
                              << descriptor.cpp_name << R"(;
)";
            }
        }
        void write_basic_constants(State &state)
        {
            using detail::unsigned_integer;
            basic_constants << R"(
constexpr Word magic_number = 0x)"
                            << unsigned_integer(state.top_level.magic_number, 0x10, 8) << R"(UL;
constexpr std::uint32_t major_version = )"
                            << unsigned_integer(state.top_level.major_version) << R"(UL;
constexpr std::uint32_t minor_version = )"
                            << unsigned_integer(state.top_level.minor_version) << R"(UL;
constexpr std::uint32_t revision = )"
                            << unsigned_integer(state.top_level.revision) << R"(UL;
)";
            for(auto &instruction_set : state.top_level.extension_instruction_sets)
            {
                basic_constants << R"(
constexpr std::uint32_t )" << instruction_set_version_name(instruction_set)
                                << R"( = )" << unsigned_integer(instruction_set.version) << R"(UL;
constexpr std::uint32_t )" << instruction_set_revision_name(instruction_set)
                                << R"( = )" << unsigned_integer(instruction_set.revision) << R"(UL;
)";
            }
        }
        void write_enums(State &state)
        {
            using detail::unsigned_integer;
            for(auto &enumeration : state.enumerations_list)
            {
                enum_definitions << R"(
enum class )" << enumeration.cpp_name
                                 << R"( : Word
{
@+)";
                enum_properties_definitions << R"(
constexpr util::string_view get_enumerant_name()"
                                            << enumeration.cpp_name << R"( v) noexcept
{
    using namespace util::string_view_literals;
    switch(v)
    {
@+@+)";
                for(auto &enumerant : enumeration.enumerants)
                {
                    enum_definitions << enumerant.cpp_name << " = ";
                    if(enumeration.is_bitwise)
                        enum_definitions << "0x" << unsigned_integer(enumerant.value, 0x10) << "UL";
                    else
                        enum_definitions << unsigned_integer(enumerant.value, 10) << "UL";
                    enum_definitions << ",\n";
                }
                enum_definitions << R"(@-};

vulkan_cpu_util_generate_enum_traits()"
                                 << enumeration.cpp_name;
                std::unordered_set<std::uint32_t> values;
                for(auto &enumerant : enumeration.enumerants)
                {
                    enum_definitions << R"(,
`````````````````````````````````````)"
                                     << enumeration.cpp_name << "::" << enumerant.cpp_name;
                    if(std::get<1>(values.insert(enumerant.value)))
                    {
                        enum_properties_definitions << "case " << enumeration.cpp_name
                                                    << "::" << enumerant.cpp_name << R"(:
    return ")" << enumerant.json_name << R"("_sv;
)";
                    }
                }
                enum_definitions << R"();
)";
                if(enumeration.is_bitwise)
                {
                    enum_definitions << R"(
constexpr )" << enumeration.cpp_name << R"( operator~()"
                                     << enumeration.cpp_name << R"( v) noexcept
{
    return static_cast<)" << enumeration.cpp_name
                                     << R"(>(~static_cast<Word>(v));
}

constexpr )" << enumeration.cpp_name << R"( operator&()"
                                     << enumeration.cpp_name << R"( a, )" << enumeration.cpp_name
                                     << R"( b) noexcept
{
    return static_cast<)" << enumeration.cpp_name
                                     << R"(>(static_cast<Word>(a) & static_cast<Word>(b));
}

constexpr )" << enumeration.cpp_name << R"( operator|()"
                                     << enumeration.cpp_name << R"( a, )" << enumeration.cpp_name
                                     << R"( b) noexcept
{
    return static_cast<)" << enumeration.cpp_name
                                     << R"(>(static_cast<Word>(a) | static_cast<Word>(b));
}

constexpr )" << enumeration.cpp_name << R"( operator^()"
                                     << enumeration.cpp_name << R"( a, )" << enumeration.cpp_name
                                     << R"( b) noexcept
{
    return static_cast<)" << enumeration.cpp_name
                                     << R"(>(static_cast<Word>(a) ^ static_cast<Word>(b));
}

constexpr )" << enumeration.cpp_name << R"( &operator&=()"
                                     << enumeration.cpp_name << R"( &a, )" << enumeration.cpp_name
                                     << R"( b) noexcept
{
    a = a & b;
    return a;
}

constexpr )" << enumeration.cpp_name << R"( &operator|=()"
                                     << enumeration.cpp_name << R"( &a, )" << enumeration.cpp_name
                                     << R"( b) noexcept
{
    a = a | b;
    return a;
}

constexpr )" << enumeration.cpp_name << R"( &operator^=()"
                                     << enumeration.cpp_name << R"( &a, )" << enumeration.cpp_name
                                     << R"( b) noexcept
{
    a = a ^ b;
    return a;
}
)";
                }
                enum_properties_definitions << R"(@-@_}
    return ""_sv;
}

constexpr util::Enum_set<)" << state.capability_enumeration.value()->cpp_name
                                            << R"(> get_directly_required_capabilities()"
                                            << enumeration.cpp_name << R"( v) noexcept
{
    switch(v)
    {
@+@+)";
                values.clear();
                for(auto &enumerant : enumeration.enumerants)
                {
                    if(std::get<1>(values.insert(enumerant.value)))
                    {
                        enum_properties_definitions << "case " << enumeration.cpp_name
                                                    << "::" << enumerant.cpp_name << R"(:
    return {)";
                        auto separator = ""_sv;
                        for(auto &capability : enumerant.capabilities.capabilities)
                        {
                            enum_properties_definitions << separator;
                            separator = ", "_sv;
                            enum_properties_definitions
                                << state.capability_enumeration.value()->cpp_name
                                << "::" << state.get_capability(capability).cpp_name;
                        }
                        enum_properties_definitions << R"(};
)";
                    }
                }
                enum_properties_definitions << R"(@-@_}
    return {};
}

constexpr util::Enum_set<)" << state.extension_enumeration.value()->cpp_name
                                            << R"(> get_directly_required_extensions()"
                                            << enumeration.cpp_name << R"( v) noexcept
{
    switch(v)
    {
@+@+)";
                values.clear();
                for(auto &enumerant : enumeration.enumerants)
                {
                    if(std::get<1>(values.insert(enumerant.value)))
                    {
                        enum_properties_definitions << "case " << enumeration.cpp_name
                                                    << "::" << enumerant.cpp_name << R"(:
    return {)";
                        auto separator = ""_sv;
                        for(auto &extension : enumerant.extensions.extensions)
                        {
                            enum_properties_definitions << separator;
                            separator = ", "_sv;
                            enum_properties_definitions
                                << state.extension_enumeration.value()->cpp_name
                                << "::" << state.get_extension(extension).cpp_name;
                        }
                        enum_properties_definitions << R"(};
)";
                    }
                }
                enum_properties_definitions << R"(@-@_}
    return {};
}
)";
            }
        }
        void write_id_types(State &state)
        {
            id_types << "\n";
            for(auto &id_type : state.id_type_list)
            {
                id_types << "typedef Id " << id_type.cpp_name << R"(;
)";
            }
        }
        void write_enum_parameters(State &state)
        {
            std::unordered_map<std::string, std::list<Output_struct>::iterator>
                enumerant_parameter_structs;
            for(auto &operand_kind : state.operand_kind_list)
            {
                if(operand_kind.enum_parameters.empty())
                    continue;
                auto enumeration_iter = state.get_enumeration(operand_kind.operand_kind->kind);
                enumerant_parameter_structs.clear();
                for(auto &parameter : operand_kind.enum_parameters)
                {
                    auto enumerant_parameter_struct_iter =
                        enumerant_parameter_structs.find(parameter.enumerant->json_name);
                    if(enumerant_parameter_struct_iter == enumerant_parameter_structs.end())
                    {
                        auto output_struct_iter =
                            enum_structs.emplace(enum_structs.end(),
                                                 file_path,
                                                 parameter.enumerant->parameters_struct_cpp_name);
                        enumerant_parameter_struct_iter =
                            std::get<0>(enumerant_parameter_structs.emplace(
                                parameter.enumerant->json_name, output_struct_iter));
                    }
                    auto &output_struct = *std::get<1>(*enumerant_parameter_struct_iter);
                    auto parameter_type = state.get_operand_kind(parameter.json_kind);
                    if(!parameter_type->enum_parameters.empty())
                        throw Generate_error("enum parameter can't contain enum with parameters: "
                                             + operand_kind.operand_kind->kind);
                    output_struct.add_nonstatic_member(
                        parameter_type->cpp_name_with_parameters, parameter.cpp_name, true);
                }
                auto &enum_with_parameters_struct = *enum_structs.emplace(
                    enum_structs.end(), file_path, operand_kind.cpp_name_with_parameters);
                enum_with_parameters_struct.add_nonstatic_member(
                    enumeration_iter->cpp_name, "value", false);
                if(enumeration_iter->is_bitwise)
                {
                    for(auto &enumerant : enumeration_iter->enumerants)
                    {
                        if(enumerant.parameters.empty())
                            continue;
                        enum_with_parameters_struct.add_nonstatic_member(
                            "util::optional<" + enumerant.parameters_struct_cpp_name + ">",
                            enumerant.parameters_variable_cpp_name,
                            true);
                    }
                }
                else
                {
                    auto parameters_name = "Parameters"_sv;
                    auto variant_start = "typedef util::variant<"_sv;
                    enum_with_parameters_struct.struct_members
                        << detail::push_start << detail::add_start_offset(variant_start.size())
                        << variant_start << "util::monostate";
                    for(auto &enumerant : enumeration_iter->enumerants)
                    {
                        if(enumerant.parameters.empty())
                            continue;
                        enum_with_parameters_struct.struct_members
                            << ",\n" << enumerant.parameters_struct_cpp_name;
                    }
                    enum_with_parameters_struct.struct_members << "> " << parameters_name << ";\n"
                                                               << detail::pop_start
                                                               << detail::restart_indent;
                    enum_with_parameters_struct.add_nonstatic_member(
                        parameters_name, "parameters", true);
                }
            }
        }
        void write_composite_types(State &state)
        {
            for(auto &composite_type : state.composite_type_list)
            {
                auto &composite_type_struct = *composite_types.emplace(
                    composite_types.end(), file_path, composite_type.cpp_name);
                for(auto &base : composite_type.bases)
                    composite_type_struct.add_nonstatic_member(
                        state.get_operand_kind(base.json_type)->cpp_name_with_parameters,
                        base.cpp_name,
                        true);
            }
        }
        void write_instruction_operand(State &state,
                                       const Operand_descriptor &operand,
                                       Output_struct &instruction_struct)
        {
            typedef Operand_descriptor::Quantifier Quantifier;
            std::string member_type =
                state.get_operand_kind(operand.json_kind)->cpp_name_with_parameters;
            switch(operand.quantifier)
            {
            case Quantifier::none:
                break;
            case Quantifier::optional:
                member_type = "util::optional<" + std::move(member_type) + ">";
                break;
            case Quantifier::variable:
                member_type = "std::vector<" + std::move(member_type) + ">";
                break;
            }
            instruction_struct.add_nonstatic_member(member_type, operand.cpp_name, true);
        }
        void write_instructions(State &state)
        {
            for(auto &instruction_descriptor : state.instruction_descriptor_list)
            {
                auto &instruction_struct = *instruction_structs.emplace(
                    instruction_structs.end(), file_path, instruction_descriptor.cpp_struct_name);
                instruction_struct.struct_members
                    << "static constexpr " << instruction_descriptor.enumeration->cpp_name
                    << R"( get_operation() noexcept
{
    return )" << instruction_descriptor.enumeration->cpp_name
                    << "::" << instruction_descriptor.enumerant->cpp_name << R"(;
}
)";
                for(auto &operand : instruction_descriptor.implied_operands)
                    write_instruction_operand(state, operand, instruction_struct);
                for(auto &operand : instruction_descriptor.explicit_operands)
                    write_instruction_operand(state, operand, instruction_struct);
            }
        }
        virtual void fill_output(State &state) override
        {
            Header_file_base::fill_output(state);
            write_system_include("cstdint");
            write_system_include("vector");
            write_system_include("string");
            write_system_include("iterator");
            write_local_include_string("util/string_view.h");
            write_local_include_string("util/enum.h");
            write_local_include_string("util/optional.h");
            write_local_include_string("util/variant.h");
            write_local_include_string("spirv/word.h");
            write_local_include_string("spirv/literal_string.h");
            basic_types << R"(typedef Word Id;
)";
            write_literal_kinds(state);
            write_basic_constants(state);
            write_enums(state);
            write_id_types(state);
            write_enum_parameters(state);
            write_composite_types(state);
            write_instructions(state);
        }
    };
    struct Spirv_cpp : public Source_file_base
    {
        explicit Spirv_cpp(const util::filesystem::path &file_path, const Spirv_h *header)
            : Source_file_base(file_path, header)
        {
        }
    };
    struct Parser_h : public Header_file_base
    {
        detail::Generated_output_stream parse_error_class;
        detail::Generated_output_stream parser_callbacks_class;
        detail::Generated_output_stream dump_callbacks_class;
        detail::Generated_output_stream parser_class;
        explicit Parser_h(const util::filesystem::path &file_path)
            : Header_file_base(file_path),
              parse_error_class(file_path),
              parser_callbacks_class(file_path),
              dump_callbacks_class(file_path),
              parser_class(file_path)
        {
            register_output_part(Output_part::parse_error_class, &Parser_h::parse_error_class);
            register_output_part(Output_part::parser_callbacks_class,
                                 &Parser_h::parser_callbacks_class);
            register_output_part(Output_part::dump_callbacks_class,
                                 &Parser_h::dump_callbacks_class);
            register_output_part(Output_part::parser_class, &Parser_h::parser_class);
        }
        void write_instruction_operand(State &state, Operand_descriptor &operand)
        {
            typedef Operand_descriptor::Quantifier Quantifier;
            auto operand_kind = state.get_operand_kind(operand.json_kind);
            switch(operand.quantifier)
            {
            case Quantifier::none:
                dump_callbacks_class << operand_kind->cpp_dump_function_name << "(instruction."
                                     << operand.cpp_name << ", indent_depth + 1);\n";
#warning finish
                break;
            case Quantifier::optional:
                dump_callbacks_class << "if(instruction." << operand.cpp_name << R"()
    )" << operand_kind->cpp_dump_function_name
                                     << "(*instruction." << operand.cpp_name
                                     << R"(, indent_depth + 1);
)";
#warning finish
                break;
            case Quantifier::variable:
                dump_callbacks_class << "for(auto &operand : instruction." << operand.cpp_name
                                     << R"()
    )" << operand_kind->cpp_dump_function_name
                                     << R"((operand, indent_depth + 1);
)";
#warning finish
                break;
            }
        }
        virtual void fill_output(State &state) override
        {
            Header_file_base::fill_output(state);
            write_local_include_path(state.spirv_h.file_path);
            write_local_include_string("util/optional.h"_sv);
            write_local_include_string("util/string_view.h"_sv);
            write_local_include_string("json/json.h"_sv);
            write_system_include("sstream"_sv);
            write_system_include("vector"_sv);
            write_system_include("cassert"_sv);
            write_system_include("type_traits"_sv);
#warning finish
            include_guard_start << R"(#error generator not finished being implemented

)";
            parse_error_class << R"a(struct Parser_error : public std::runtime_error
{
    std::size_t error_index;
    std::size_t instruction_start_index;
    static std::string make_error_message(std::size_t error_index,
    ``````````````````````````````````````std::size_t instruction_start_index,
    ``````````````````````````````````````util::string_view message)
    {
        std::ostringstream ss;
        ss << "parse error at 0x" << std::hex << std::uppercase << error_index;
        if(instruction_start_index != 0)
            ss << " (instruction starts at 0x" << instruction_start_index << ")";
        ss << ": " << message;
        return ss.str();
    }
    Parser_error(std::size_t error_index, std::size_t instruction_start_index, util::string_view message)
        : runtime_error(make_error_message(error_index, instruction_start_index, message)),
        ``error_index(error_index),
        ``instruction_start_index(instruction_start_index)
    {
    }
};
)a";
            parser_callbacks_class << R"(
struct Parser_callbacks
{
    virtual ~Parser_callbacks() = default;
    virtual void handle_header(unsigned version_number_major,
    ```````````````````````````unsigned version_number_minor,
    ```````````````````````````Word generator_magic_number,
    ```````````````````````````Word id_bound,
    ```````````````````````````Word instruction_schema) = 0;
@+)";
            dump_callbacks_class << R"(
struct Dump_callbacks final : public Parser_callbacks
{
    std::ostringstream ss;
    Dump_callbacks() : ss()
    {
        ss << std::uppercase;
    }
    void write_indent(std::size_t indent_count)
    {
        for(std::size_t i = 0; i < indent_count; i++)
            ss << "    ";
    }
    virtual void handle_header(unsigned version_number_major,
    ```````````````````````````unsigned version_number_minor,
    ```````````````````````````Word generator_magic_number,
    ```````````````````````````Word id_bound,
    ```````````````````````````Word instruction_schema) override
    {
        ss << "SPIR-V Version: " << std::dec << version_number_major << '.' << version_number_minor << '\n';
        ss << "Generator Magic Number: 0x" << std::hex << generator_magic_number << '\n';
        ss << "Id Bound: " << std::dec << id_bound << '\n';
        ss << "Instruction Schema (reserved): " << std::dec << instruction_schema << '\n';
    }
@+)";
            parser_class << R"(
class Parser final
{
    Parser(const Parser &) = delete;
    Parser &operator =(const Parser &) = delete;

private:
    struct Id_state
    {
        util::optional<Extension_instruction_set> instruction_set;
        util::optional<std::size_t> type_word_count;
    };

private:
    Parser_callbacks &parser_callbacks;
    std::vector<Id_state> id_states;
    const Word *shader_words;
    std::size_t shader_size;

private:
    Parser(Parser_callbacks &parser_callbacks,
    ```````const Word *shader_words,
    ```````std::size_t shader_size) noexcept
        : parser_callbacks(parser_callbacks),
        ``id_states(),
        ``shader_words(shader_words),
        ``shader_size(shader_size)
    {
    }
    Id_state &get_id_state(Id id) noexcept
    {
        assert(id > 0 && id <= id_states.size());
        return id_states[id - 1];
    }
@+)";
            for(auto &operand_kind : state.operand_kind_list)
            {
                dump_callbacks_class << "void " << operand_kind.cpp_dump_function_name << "(const "
                                     << operand_kind.cpp_name_with_parameters
                                     << R"( &operand, std::size_t indent_depth)
{
@+)";
                typedef ast::Operand_kinds::Operand_kind::Category Category;
                switch(operand_kind.operand_kind->category)
                {
                case Category::bit_enum:
                {
                    dump_callbacks_class << R"(write_indent(indent_depth);
ss << ")" << operand_kind.operand_kind->kind
                                         << R"(:\n";
)";
                    util::string_view zero_enumerant_name = "0"_sv;
                    auto enumeration = util::get<std::list<Enumeration_descriptor>::const_iterator>(
                        operand_kind.value);
                    for(auto &enumerant : enumeration->enumerants)
                    {
                        if(enumerant.value == 0)
                        {
                            zero_enumerant_name = enumerant.json_name;
                            break;
                        }
                    }
                    dump_callbacks_class << R"(Word bits = static_cast<Word>(operand)"
                                         << (operand_kind.enum_parameters.empty() ? "" : ".value")
                                         << R"();
if(bits == 0)
{
    write_indent(indent_depth + 1);
    ss << ")" << zero_enumerant_name << R"(\n";
    return;
}
)";
                    for(auto &enumerant : enumeration->enumerants)
                    {
                        if(enumerant.value == 0)
                        {
                            if(!enumerant.parameters.empty())
                                throw Generate_error(
                                    "in bitwise enum, zero enumerant can't have parameters: "
                                    + enumeration->json_name
                                    + "."
                                    + enumerant.json_name);
                            continue;
                        }
                        else if(enumerant.value & (enumerant.value - 1))
                        {
                            throw Generate_error(
                                "in bitwise enum, enumerant is not a power of 2 or zero: "
                                + enumeration->json_name
                                + "."
                                + enumerant.json_name);
                        }
                        dump_callbacks_class << R"(if(bits & static_cast<Word>()"
                                             << enumeration->cpp_name << "::" << enumerant.cpp_name
                                             << R"())
{
    write_indent(indent_depth + 1);
    ss << ")" << enumerant.json_name << (enumerant.parameters.empty() ? "" : ":")
                                             << R"(\n";
    bits &= ~static_cast<Word>()" << enumeration->cpp_name
                                             << "::" << enumerant.cpp_name << R"();
@+)";
                        if(!enumerant.parameters.empty())
                        {
                            dump_callbacks_class << "auto &parameters = *operand."
                                                 << enumerant.parameters_variable_cpp_name << ";\n";
                        }
                        for(auto *parameter : enumerant.parameters)
                        {
                            auto parameter_operand_kind =
                                state.get_operand_kind(parameter->json_kind);
                            dump_callbacks_class << parameter_operand_kind->cpp_dump_function_name
                                                 << R"((parameters.)" << parameter->cpp_name
                                                 << R"(, indent_depth + 2);
)";
                        }
                        dump_callbacks_class << R"(@-}
)";
                    }
                    break;
                }
                case Category::value_enum:
                {
                    dump_callbacks_class << R"(write_indent(indent_depth);
ss << ")" << operand_kind.operand_kind->kind
                                         << R"(: ";
switch(operand)" << (operand_kind.enum_parameters.empty() ? "" : ".value")
                                         << R"()
{
)";
                    auto enumeration = util::get<std::list<Enumeration_descriptor>::const_iterator>(
                        operand_kind.value);
                    std::unordered_set<std::uint32_t> values;
                    for(auto &enumerant : enumeration->enumerants)
                    {
                        if(!std::get<1>(values.insert(enumerant.value)))
                            continue; // skip duplicate values
                        dump_callbacks_class << "case " << enumeration->cpp_name
                                             << "::" << enumerant.cpp_name << R"(:
)" << (enumerant.parameters.empty() ? "" : "{\n")
                                             << R"(    ss << ")" << enumerant.json_name
                                             << (enumerant.parameters.empty() ? "" : ":") << R"(\n";
@+)";
                        if(!enumerant.parameters.empty())
                        {
                            dump_callbacks_class << "auto &parameters = util::get<"
                                                 << enumerant.parameters_struct_cpp_name
                                                 << ">(operand.parameters);\n";
                            for(auto *parameter : enumerant.parameters)
                            {
                                auto parameter_operand_kind =
                                    state.get_operand_kind(parameter->json_kind);
                                dump_callbacks_class
                                    << parameter_operand_kind->cpp_dump_function_name
                                    << R"((parameters.)" << parameter->cpp_name
                                    << R"(, indent_depth + 1);
)";
                            }
                            dump_callbacks_class << R"(return;
@-}
)";
                        }
                        else
                        {
                            dump_callbacks_class << R"(return;
@-)";
                        }
                    }
                    dump_callbacks_class << R"(}
ss << "<Unknown> (" << static_cast<Word>(operand)"
                                         << (operand_kind.enum_parameters.empty() ? "" : ".value")
                                         << R"() << ")\n";
)";
                    break;
                }
                case Category::id:
                    dump_callbacks_class << R"(write_indent(indent_depth);
ss << ")" << operand_kind.operand_kind->kind
                                         << R"(: " << std::dec << operand << '\n';
)";
                    break;
                case Category::literal:
                {
                    typedef ast::Operand_kinds::Operand_kind::Literal_kind Literal_kind;
                    auto literal = util::get<Literal_kind>(operand_kind.value);
                    dump_callbacks_class << R"(write_indent(indent_depth);
ss << ")" << operand_kind.operand_kind->kind
                                         << R"(: ";
)";
                    switch(literal)
                    {
                    case Literal_kind::literal_integer:
                        dump_callbacks_class << R"(ss << "0x" << std::hex << operand << std::dec;
ss << " u64=" << static_cast<std::uint64_t>(operand);
ss << " s64=" << static_cast<std::int64_t>(operand);
ss << " u32=" << static_cast<std::uint32_t>(operand);
ss << " s32=" << static_cast<std::int32_t>(operand) << '\n';
)";
                        break;
                    case Literal_kind::literal_string:
                        dump_callbacks_class
                            << R"(json::ast::String_value::write(ss, static_cast<std::string>(operand));
ss << '\n';
)";
                        break;
                    case Literal_kind::literal_context_dependent_number:
                        dump_callbacks_class << R"(ss << "{";
auto separator = "";
static_assert(std::is_same<decltype(operand), const std::vector<Word> &>::value, "");
for(auto word : operand)
{
    ss << separator;
    separator = ", ";
    ss << "0x" << std::hex << word;
}
ss << "}\n";
)";
                        break;
                    case Literal_kind::literal_ext_inst_integer:
                        dump_callbacks_class << R"(ss << std::dec << operand << '\n';
)";
                        break;
                    case Literal_kind::literal_spec_constant_op_integer:
                        dump_callbacks_class << R"(ss << get_enumerant_name(operand) << '\n';
)";
                        break;
                    }
                    break;
                }
                case Category::composite:
                {
                    dump_callbacks_class << R"(write_indent(indent_depth);
ss << ")" << operand_kind.operand_kind->kind
                                         << R"(:\n";
)";
                    auto &composite =
                        util::get<std::list<Composite_type_descriptor>::const_iterator>(
                            operand_kind.value);
                    for(auto &base : composite->bases)
                    {
                        auto base_operand_kind = state.get_operand_kind(base.json_type);
                        dump_callbacks_class << base_operand_kind->cpp_dump_function_name
                                             << "(operand." << base.cpp_name
                                             << ", indent_depth + 1);\n";
                    }
                    break;
                }
                }
                dump_callbacks_class << R"(@-}
)";
            }
            for(auto &instruction : state.instruction_descriptor_list)
            {
                parser_callbacks_class << "virtual void " << instruction.cpp_parse_callback_name
                                       << "(" << instruction.cpp_struct_name
                                       << " instruction) = 0;\n";
                dump_callbacks_class << "virtual void " << instruction.cpp_parse_callback_name
                                     << "(" << instruction.cpp_struct_name
                                     << R"( instruction) override
{
    ss << ")";
                if(instruction.extension_instruction_set)
                    dump_callbacks_class << op_ext_inst_json_name;
                else
                    dump_callbacks_class << instruction.json_name;
                dump_callbacks_class << R"(:\n";
@+)";
                if(!instruction.implied_operands.empty() || !instruction.explicit_operands.empty()
                   || instruction.extension_instruction_set)
                    dump_callbacks_class << "constexpr std::size_t indent_depth = 0;\n";
                else
                    dump_callbacks_class << "static_cast<void>(instruction);\n";
                for(auto &operand : instruction.implied_operands)
                    write_instruction_operand(state, operand);
                if(instruction.extension_instruction_set)
                {
                    dump_callbacks_class << R"(write_indent(indent_depth + 1);
ss << ")" << instruction.json_name << R"(\n";
)";
                }
                for(auto &operand : instruction.explicit_operands)
                    write_instruction_operand(state, operand);
#warning finish
                dump_callbacks_class << "@-}\n";
            }
            dump_callbacks_class << R"(@-};
)";
            parser_callbacks_class << R"(@-};
)";
            parser_class << R"(@-};
)";
        }
    };
    struct Parser_cpp : public Source_file_base
    {
        explicit Parser_cpp(const util::filesystem::path &file_path, const Parser_h *header)
            : Source_file_base(file_path, header)
        {
        }
    };

private:
    const ast::Top_level &top_level;
    Spirv_h spirv_h;
    Spirv_cpp spirv_cpp;
    Parser_h parser_h;
    Parser_cpp parser_cpp;

public:
    State(const util::filesystem::path &output_directory, const ast::Top_level &top_level)
        : top_level(top_level),
          spirv_h(output_directory / "spirv.h"),
          spirv_cpp(output_directory / "spirv.cpp", &spirv_h),
          parser_h(output_directory / "parser.h"),
          parser_cpp(output_directory / "parser.cpp", &parser_h)
    {
    }

private:
    static constexpr util::string_view op_enum_json_name = "Op"_sv;
    static constexpr util::string_view extension_enum_json_name = "Extension"_sv;
    static constexpr util::string_view capability_enum_json_name = "Capability"_sv;
    static constexpr util::string_view extension_instruction_set_enum_json_name =
        "Extension_instruction_set"_sv;
    static constexpr util::string_view unknown_extension_instruction_set_enumerant_json_name =
        "Unknown"_sv;
    static constexpr util::string_view op_ext_inst_import_json_name = "OpExtInstImport"_sv;
    static constexpr util::string_view op_ext_inst_json_name = "OpExtInst"_sv;
    static constexpr util::string_view id_result_json_name = "IdResult"_sv;
    static constexpr util::string_view id_result_type_json_name = "IdResultType"_sv;
    static constexpr util::string_view id_ref_json_name = "IdRef"_sv;
    struct Enum_parameter;
    struct Enumerant_descriptor
    {
        std::uint32_t value;
        std::string cpp_name;
        std::string parameters_struct_cpp_name;
        std::string parameters_variable_cpp_name;
        std::string json_name;
        ast::Capabilities capabilities;
        ast::Extensions extensions;
        std::vector<const Enum_parameter *> parameters;
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
        static std::string make_parameters_struct_cpp_name(util::string_view json_enumeration_name,
                                                           util::string_view cpp_name)
        {
            using detail::name_from_words_initial_capital;
            return name_from_words_initial_capital(json_enumeration_name, cpp_name, "parameters"_sv)
                .to_string();
        }
        static std::string make_parameters_variable_cpp_name(util::string_view cpp_name)
        {
            using detail::name_from_words_all_lowercase;
            return name_from_words_all_lowercase(cpp_name).to_string();
        }
        Enumerant_descriptor(std::uint32_t value,
                             util::string_view json_enumeration_name,
                             std::string json_name,
                             ast::Capabilities capabilities,
                             ast::Extensions extensions)
            : value(value),
              cpp_name(make_cpp_name(json_enumeration_name, json_name)),
              parameters_struct_cpp_name(
                  make_parameters_struct_cpp_name(json_enumeration_name, cpp_name)),
              parameters_variable_cpp_name(make_parameters_variable_cpp_name(cpp_name)),
              json_name(std::move(json_name)),
              capabilities(std::move(capabilities)),
              extensions(std::move(extensions)),
              parameters()
        {
        }
    };
    struct Enumeration_descriptor
    {
        bool is_bitwise;
        std::string cpp_name;
        std::string json_name;
        std::list<Enumerant_descriptor> enumerants;
        typedef std::unordered_map<std::string, std::list<Enumerant_descriptor>::iterator>
            Json_name_to_enumerant_map;
        Json_name_to_enumerant_map json_name_to_enumerant_map;
        static Json_name_to_enumerant_map make_json_name_to_enumerant_map(
            std::list<Enumerant_descriptor> *enumerants)
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
    std::unordered_map<const ast::Extension_instruction_set *,
                       std::list<Enumeration_descriptor>::const_iterator>
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
            instruction_set_extension_op_enumeration_map.emplace(&instruction_set, iter);
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
    const Enumerant_descriptor &get_capability(const std::string &capability) const
    {
        auto &enumerant_map = capability_enumeration.value()->json_name_to_enumerant_map;
        auto iter = enumerant_map.find(capability);
        if(iter == enumerant_map.end())
            throw Generate_error("unknown capability: " + capability);
        return *std::get<1>(*iter);
    }
    const Enumerant_descriptor &get_extension(const std::string &extension) const
    {
        auto &enumerant_map = extension_enumeration.value()->json_name_to_enumerant_map;
        auto iter = enumerant_map.find(extension);
        if(iter == enumerant_map.end())
            throw Generate_error("unknown extension: " + extension);
        return *std::get<1>(*iter);
    }
    std::list<Enumeration_descriptor>::const_iterator get_enumeration(
        const std::string &json_name) const
    {
        auto iter = enumerations_map.find(json_name);
        if(iter == enumerations_map.end())
            throw Generate_error("unknown enum: " + json_name);
        return std::get<1>(*iter);
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
    util::Enum_map<ast::Operand_kinds::Operand_kind::Literal_kind, Literal_type_descriptor>
        literal_type_descriptors;

private:
    void fill_literal_type_descriptors()
    {
        for(auto literal_kind :
            util::Enum_traits<ast::Operand_kinds::Operand_kind::Literal_kind>::values)
        {
            literal_type_descriptors.emplace(literal_kind, Literal_type_descriptor(literal_kind));
        }
    }
    static ast::Operand_kinds::Operand_kind::Literal_kind get_literal_kind(
        util::string_view json_name)
    {
        auto retval = ast::Operand_kinds::Operand_kind::get_literal_kind_from_json_name(json_name);
        if(!retval)
            throw Generate_error("unknown literal kind: " + std::string(json_name));
        return *retval;
    }

private:
    struct Id_type_descriptor
    {
        std::string cpp_name;
        std::string json_name;
        static std::string get_cpp_name(util::string_view json_name)
        {
            using detail::name_from_words_initial_capital;
            return name_from_words_initial_capital(json_name).to_string();
        }
        explicit Id_type_descriptor(std::string json_name)
            : cpp_name(get_cpp_name(json_name)), json_name(std::move(json_name))
        {
        }
    };

private:
    std::list<Id_type_descriptor> id_type_list;
    std::unordered_map<std::string, std::list<Id_type_descriptor>::const_iterator> id_type_map;

private:
    std::list<Id_type_descriptor>::const_iterator add_id_type_descriptor(
        Id_type_descriptor &&id_type_descriptor)
    {
        auto name = id_type_descriptor.json_name;
        auto iter = id_type_list.insert(id_type_list.end(), std::move(id_type_descriptor));
        if(!std::get<1>(id_type_map.emplace(name, iter)))
            throw Generate_error("duplicate id type: " + name);
        return iter;
    }
    void fill_id_type_descriptors()
    {
        for(auto &operand_kind : top_level.operand_kinds.operand_kinds)
        {
            if(operand_kind.category != ast::Operand_kinds::Operand_kind::Category::id)
                continue;
            add_id_type_descriptor(Id_type_descriptor(operand_kind.kind));
        }
    }
    std::list<Id_type_descriptor>::const_iterator get_id_type(const std::string &json_name) const
    {
        auto iter = id_type_map.find(json_name);
        if(iter == id_type_map.end())
            throw Generate_error("unknown id type: " + json_name);
        return std::get<1>(*iter);
    }

private:
    struct Composite_type_descriptor
    {
        struct Base
        {
            std::string cpp_name;
            std::string json_type;
            explicit Base(std::string json_type, std::size_t index)
                : cpp_name(
                      detail::name_from_words_all_lowercase(
                          "part"_sv, json::ast::Number_value::unsigned_integer_to_string(index + 1))
                          .to_string()),
                  json_type(std::move(json_type))
            {
            }
        };
        std::string cpp_name;
        std::string json_name;
        std::list<Base> bases;
        explicit Composite_type_descriptor(std::string json_name, std::list<Base> bases)
            : cpp_name(detail::name_from_words_initial_capital(json_name).to_string()),
              json_name(std::move(json_name)),
              bases(std::move(bases))
        {
        }
    };

private:
    std::list<Composite_type_descriptor> composite_type_list;
    std::unordered_map<std::string, std::list<Composite_type_descriptor>::const_iterator>
        composite_type_map;

private:
    std::list<Composite_type_descriptor>::const_iterator add_composite_type_descriptor(
        Composite_type_descriptor &&composite_type_descriptor)
    {
        auto name = composite_type_descriptor.json_name;
        auto iter = composite_type_list.insert(composite_type_list.end(),
                                               std::move(composite_type_descriptor));
        if(!std::get<1>(composite_type_map.emplace(name, iter)))
            throw Generate_error("duplicate composite type: " + name);
        return iter;
    }
    void fill_composite_type_descriptors()
    {
        for(auto &operand_kind : top_level.operand_kinds.operand_kinds)
        {
            if(operand_kind.category != ast::Operand_kinds::Operand_kind::Category::composite)
                continue;
            std::list<Composite_type_descriptor::Base> bases;
            std::size_t index = 0;
            for(auto &base :
                util::get<ast::Operand_kinds::Operand_kind::Bases>(operand_kind.value).values)
                bases.push_back(Composite_type_descriptor::Base(base, index++));
            add_composite_type_descriptor(
                Composite_type_descriptor(operand_kind.kind, std::move(bases)));
        }
    }
    std::list<Composite_type_descriptor>::const_iterator get_composite_type(
        const std::string &json_name) const
    {
        auto iter = composite_type_map.find(json_name);
        if(iter == composite_type_map.end())
            throw Generate_error("unknown composite type: " + json_name);
        return std::get<1>(*iter);
    }

private:
    struct Enum_parameter
    {
        std::list<Enumerant_descriptor>::const_iterator enumerant;
        std::string cpp_name;
        std::string json_kind;
        std::string json_name;
        static std::string make_cpp_name(util::string_view json_kind, util::string_view json_name)
        {
            using detail::name_from_words_all_lowercase;
            if(!json_name.empty())
                return name_from_words_all_lowercase(json_name).to_string();
            constexpr auto id_str = "Id"_sv;
            if(json_kind.compare(0, id_str.size(), id_str) == 0 && id_str.size() < json_kind.size()
               && json_kind[id_str.size()] >= 'A'
               && json_kind[id_str.size()] <= 'Z')
                return name_from_words_all_lowercase(json_kind.substr(id_str.size())).to_string();
            return name_from_words_all_lowercase(json_kind).to_string();
        }
        explicit Enum_parameter(std::list<Enumerant_descriptor>::const_iterator enumerant,
                                std::string json_kind,
                                std::string json_name)
            : enumerant(enumerant),
              cpp_name(make_cpp_name(json_kind, json_name)),
              json_kind(std::move(json_kind)),
              json_name(std::move(json_name))
        {
        }
    };
    struct Operand_kind_descriptor
    {
        const ast::Operand_kinds::Operand_kind *operand_kind;
        util::variant<util::monostate,
                      std::list<Enumeration_descriptor>::const_iterator,
                      std::list<Id_type_descriptor>::const_iterator,
                      ast::Operand_kinds::Operand_kind::Literal_kind,
                      std::list<Composite_type_descriptor>::const_iterator> value;
        std::list<Enum_parameter> enum_parameters;
        std::string make_cpp_name() const
        {
            std::string retval;
            struct Visitor
            {
                std::string &retval;
                const std::string &json_name;
                void operator()(util::monostate)
                {
                    retval = detail::name_from_words_initial_capital(json_name).to_string();
                }
                void operator()(std::list<Enumeration_descriptor>::const_iterator iter)
                {
                    retval = iter->cpp_name;
                }
                void operator()(std::list<Id_type_descriptor>::const_iterator iter)
                {
                    retval = iter->cpp_name;
                }
                void operator()(ast::Operand_kinds::Operand_kind::Literal_kind literal_kind)
                {
                    retval = Literal_type_descriptor::get_cpp_name(literal_kind);
                }
                void operator()(std::list<Composite_type_descriptor>::const_iterator iter)
                {
                    retval = iter->cpp_name;
                }
            };
            util::visit(Visitor{retval, operand_kind->kind}, value);
            return retval;
        }
        std::string make_cpp_name_with_parameters() const
        {
            auto *iter = util::get_if<std::list<Enumeration_descriptor>::const_iterator>(&value);
            if(iter && !enum_parameters.empty())
                return detail::name_from_words_initial_capital(operand_kind->kind,
                                                               "with parameters"_sv)
                    .to_string();
            return cpp_name;
        }
        std::string make_cpp_dump_function_name() const
        {
            return detail::name_from_words_all_lowercase("dump_operand"_sv, cpp_name).to_string();
        }
        std::string cpp_name;
        std::string cpp_name_with_parameters;
        std::string cpp_dump_function_name;
        bool needs_integer_literal_size = false;
        explicit Operand_kind_descriptor(const ast::Operand_kinds::Operand_kind *operand_kind)
            : Operand_kind_descriptor(operand_kind, util::monostate{})
        {
        }
        template <typename T>
        Operand_kind_descriptor(const ast::Operand_kinds::Operand_kind *operand_kind,
                                T arg,
                                std::list<Enum_parameter> enum_parameters = {})
            : operand_kind(operand_kind),
              value(std::move(arg)),
              enum_parameters(std::move(enum_parameters)),
              cpp_name(make_cpp_name()),
              cpp_name_with_parameters(make_cpp_name_with_parameters()),
              cpp_dump_function_name(make_cpp_dump_function_name())
        {
        }
    };

private:
    std::list<Operand_kind_descriptor> operand_kind_list;
    std::unordered_map<std::string, std::list<Operand_kind_descriptor>::const_iterator>
        operand_kind_map;

private:
    std::list<Operand_kind_descriptor>::const_iterator add_operand_kind(
        Operand_kind_descriptor &&operand_kind_descriptor)
    {
        auto name = operand_kind_descriptor.operand_kind->kind;
        auto iter =
            operand_kind_list.insert(operand_kind_list.end(), std::move(operand_kind_descriptor));
        if(!std::get<1>(operand_kind_map.emplace(name, iter)))
            throw Generate_error("duplicate operand kind: " + name);
        return iter;
    }
    std::list<Operand_kind_descriptor>::const_iterator get_operand_kind(
        const std::string &json_name) const
    {
        auto iter = operand_kind_map.find(json_name);
        if(iter == operand_kind_map.end())
            throw Generate_error("unknown operand kind: " + json_name);
        return std::get<1>(*iter);
    }
    /** returns true if operand_kind changed */
    static bool update_operand_kind(
        Operand_kind_descriptor &operand_kind,
        const std::unordered_map<std::string, std::list<Operand_kind_descriptor>::const_iterator>
            &operand_kind_map)
    {
        if(operand_kind.needs_integer_literal_size)
            return false;
        bool retval = false;
        struct Visitor
        {
            Operand_kind_descriptor &operand_kind;
            const std::unordered_map<std::string,
                                     std::list<Operand_kind_descriptor>::const_iterator>
                &operand_kind_map;
            bool &retval;
            void operator()(util::monostate)
            {
            }
            void operator()(const std::list<Enumeration_descriptor>::const_iterator &)
            {
                // FIXME: verify that all LiteralIntegers in enum parameters are 32-bits
            }
            void operator()(const std::list<Id_type_descriptor>::const_iterator &)
            {
            }
            void operator()(ast::Operand_kinds::Operand_kind::Literal_kind literal_kind)
            {
                if(literal_kind == ast::Operand_kinds::Operand_kind::Literal_kind::literal_integer)
                {
                    if(!operand_kind.needs_integer_literal_size)
                        retval = true;
                    operand_kind.needs_integer_literal_size = true;
                }
            }
            void operator()(
                const std::list<Composite_type_descriptor>::const_iterator &composite_iter)
            {
                for(auto &base : composite_iter->bases)
                {
                    auto iter = operand_kind_map.find(base.json_type);
                    if(iter == operand_kind_map.end())
                        throw Generate_error("unknown operand kind: " + base.json_type);
                    if(std::get<1>(*iter)->needs_integer_literal_size)
                    {
                        if(!operand_kind.needs_integer_literal_size)
                            retval = true;
                        operand_kind.needs_integer_literal_size = true;
                    }
                }
            }
        };
        util::visit(Visitor{operand_kind, operand_kind_map, retval}, operand_kind.value);
        return retval;
    }
    void fill_operand_kinds()
    {
        for(auto &operand_kind : top_level.operand_kinds.operand_kinds)
        {
            switch(operand_kind.category)
            {
            case ast::Operand_kinds::Operand_kind::Category::bit_enum:
            case ast::Operand_kinds::Operand_kind::Category::value_enum:
            {
                auto enumeration_iter = get_enumeration(operand_kind.kind);
                auto &enumerants =
                    util::get<ast::Operand_kinds::Operand_kind::Enumerants>(operand_kind.value);
                std::list<Enum_parameter> enum_parameters;
                for(auto &enumerant : enumerants.enumerants)
                {
                    if(enumerant.parameters.empty())
                        continue;
                    auto enumerant_iter_iter =
                        enumeration_iter->json_name_to_enumerant_map.find(enumerant.enumerant);
                    if(enumerant_iter_iter == enumeration_iter->json_name_to_enumerant_map.end())
                        throw Generate_error("unknown enumerant: " + enumerant.enumerant);
                    auto enumerant_iter = std::get<1>(*enumerant_iter_iter);
                    for(auto &parameter : enumerant.parameters.parameters)
                    {
                        enumerant_iter->parameters.push_back(&*enum_parameters.emplace(
                            enum_parameters.end(), enumerant_iter, parameter.kind, parameter.name));
                    }
                }
                add_operand_kind(Operand_kind_descriptor(&operand_kind,
                                                         enumerations_map.at(operand_kind.kind),
                                                         std::move(enum_parameters)));
                continue;
            }
            case ast::Operand_kinds::Operand_kind::Category::id:
                add_operand_kind(
                    Operand_kind_descriptor(&operand_kind, get_id_type(operand_kind.kind)));
                continue;
            case ast::Operand_kinds::Operand_kind::Category::literal:
                add_operand_kind(
                    Operand_kind_descriptor(&operand_kind, get_literal_kind(operand_kind.kind)));
                continue;
            case ast::Operand_kinds::Operand_kind::Category::composite:
                add_operand_kind(
                    Operand_kind_descriptor(&operand_kind, get_composite_type(operand_kind.kind)));
                continue;
            }
            assert(false);
        }
        for(bool any_changes = true; any_changes;)
        {
            any_changes = false;
            for(auto &operand_kind : operand_kind_list)
            {
                if(update_operand_kind(operand_kind, operand_kind_map))
                    any_changes = true;
            }
        }
    }

private:
    struct Operand_descriptor
    {
        typedef ast::Instructions::Instruction::Operands::Operand::Quantifier Quantifier;
        std::string cpp_name;
        std::string json_name;
        std::string json_kind;
        Quantifier quantifier;
        static std::string make_cpp_name(util::string_view json_name, util::string_view json_kind)
        {
            using detail::name_from_words_all_lowercase;
            if(!json_name.empty())
                return name_from_words_all_lowercase(json_name).to_string();
            constexpr auto id_str = "Id"_sv;
            if(json_kind.compare(0, id_str.size(), id_str) == 0 && id_str.size() < json_kind.size()
               && json_kind[id_str.size()] >= 'A'
               && json_kind[id_str.size()] <= 'Z')
                return name_from_words_all_lowercase(json_kind.substr(id_str.size())).to_string();
            return name_from_words_all_lowercase(json_kind).to_string();
        }
        Operand_descriptor(std::string json_name, std::string json_kind, Quantifier quantifier)
            : cpp_name(make_cpp_name(json_name, json_kind)),
              json_name(std::move(json_name)),
              json_kind(std::move(json_kind)),
              quantifier(quantifier)
        {
        }
    };
    struct Instruction_descriptor
    {
        std::string cpp_struct_name;
        std::string cpp_parse_callback_name;
        std::list<Enumeration_descriptor>::const_iterator enumeration;
        std::list<Enumerant_descriptor>::const_iterator enumerant;
        const ast::Extension_instruction_set *extension_instruction_set;
        std::string json_name;
        std::list<Operand_descriptor> implied_operands;
        std::list<Operand_descriptor> explicit_operands;
        const Instruction_properties_descriptor *properties_descriptor;
        static std::string make_cpp_struct_name(
            const ast::Extension_instruction_set *extension_instruction_set,
            util::string_view json_name)
        {
            if(extension_instruction_set)
                return detail::name_from_words_initial_capital(
                           extension_instruction_set->import_name, "op"_sv, json_name)
                    .to_string();
            return detail::name_from_words_initial_capital(json_name).to_string();
        }
        static std::string make_cpp_parse_callback_name(util::string_view cpp_struct_name)
        {
            return detail::name_from_words_all_lowercase("handle_instruction"_sv, cpp_struct_name)
                .to_string();
        }
        explicit Instruction_descriptor(
            std::list<Enumeration_descriptor>::const_iterator enumeration,
            std::list<Enumerant_descriptor>::const_iterator enumerant,
            const ast::Extension_instruction_set *extension_instruction_set,
            std::string json_name,
            std::list<Operand_descriptor> implied_operands,
            std::list<Operand_descriptor> explicit_operands,
            const Instruction_properties_descriptor *properties_descriptor)
            : cpp_struct_name(make_cpp_struct_name(extension_instruction_set, json_name)),
              cpp_parse_callback_name(make_cpp_parse_callback_name(cpp_struct_name)),
              enumeration(enumeration),
              enumerant(enumerant),
              extension_instruction_set(extension_instruction_set),
              json_name(std::move(json_name)),
              implied_operands(std::move(implied_operands)),
              explicit_operands(std::move(explicit_operands)),
              properties_descriptor(properties_descriptor)
        {
        }
    };

private:
    std::list<Instruction_descriptor> instruction_descriptor_list;
    std::unordered_map<const ast::Extension_instruction_set *,
                       std::unordered_map<std::string,
                                          std::list<Instruction_descriptor>::const_iterator>>
        instruction_descriptor_map;
    std::unordered_map<std::string,
                       std::unordered_map<std::string, const Instruction_properties_descriptor *>>
        instruction_properties_descriptors_map = make_instruction_properties_descriptors_map();

private:
    static std::unordered_map<std::string,
                              std::unordered_map<std::string,
                                                 const Instruction_properties_descriptor *>>
        make_instruction_properties_descriptors_map()
    {
        std::unordered_map<std::string,
                           std::unordered_map<std::string,
                                              const Instruction_properties_descriptor *>> retval;
        for(auto &i : Instruction_properties_descriptors::get())
        {
            retval[std::string(i.extension_instruction_set_import_name)][std::string(
                i.instruction_name)] = &i;
        }
        return retval;
    }
    const Instruction_properties_descriptor *get_instruction_properties_descriptor(
        const std::string &extension_instruction_set_import_name,
        const std::string &instruction_name)
    {
        auto iter1 =
            instruction_properties_descriptors_map.find(extension_instruction_set_import_name);
        if(iter1 == instruction_properties_descriptors_map.end())
            return nullptr;
        auto iter2 = std::get<1>(*iter1).find(instruction_name);
        if(iter2 == std::get<1>(*iter1).end())
            return nullptr;
        return std::get<1>(*iter2);
    }
    static std::string get_instruction_name_for_diagnostics(
        const ast::Extension_instruction_set *extension_instruction_set,
        util::string_view json_name)
    {
        std::string retval;
        if(extension_instruction_set)
            retval = extension_instruction_set->import_name;
        else
            retval = "core";
        retval += ":";
        retval += json_name;
        return retval;
    }
    std::list<Instruction_descriptor>::const_iterator add_instruction_descriptor(
        Instruction_descriptor &&instruction_descriptor)
    {
        auto name = instruction_descriptor.json_name;
        auto extension_instruction_set = instruction_descriptor.extension_instruction_set;
        auto iter = instruction_descriptor_list.insert(instruction_descriptor_list.end(),
                                                       std::move(instruction_descriptor));
        if(!std::get<1>(instruction_descriptor_map[extension_instruction_set].emplace(name, iter)))
            throw Generate_error("duplicate instruction: " + get_instruction_name_for_diagnostics(
                                                                 extension_instruction_set, name));
        return iter;
    }
    std::list<Instruction_descriptor>::const_iterator get_instruction_descriptor(
        const ast::Extension_instruction_set *extension_instruction_set,
        const std::string &json_name)
    {
        auto iter1 = instruction_descriptor_map.find(extension_instruction_set);
        if(iter1 == instruction_descriptor_map.end())
            throw Generate_error(
                "unknown instruction: "
                + get_instruction_name_for_diagnostics(extension_instruction_set, json_name));
        auto iter2 = std::get<1>(*iter1).find(json_name);
        if(iter2 == std::get<1>(*iter1).end())
            throw Generate_error(
                "unknown instruction: "
                + get_instruction_name_for_diagnostics(extension_instruction_set, json_name));
        return std::get<1>(*iter2);
    }
    std::string generate_guessed_instruction_properties_descriptor_string(
        const ast::Extension_instruction_set *extension_instruction_set,
        const ast::Instructions::Instruction &instruction) const
    {
        std::string retval;
        retval += "{\""_sv;
        if(extension_instruction_set)
            retval += extension_instruction_set->import_name;
        retval += "\"_sv, \"";
        retval += instruction.opname;
        retval += "\"_sv, {"_sv;
        auto separator = ""_sv;
        for(auto &operand : instruction.operands.operands)
        {
            retval += separator;
            separator = ", "_sv;
            retval += '{';
            auto operand_kind = get_operand_kind(operand.kind);
            if(operand_kind->needs_integer_literal_size)
                retval += "Integer_literal_size::always_32bits"_sv;
            retval += '}';
        }
        retval += "}},";
        return retval;
    }
    std::list<Operand_descriptor> make_instruction_implied_operands(bool is_extension)
    {
        if(!is_extension)
            return {};
        std::list<Operand_descriptor> retval;
        retval.push_back(Operand_descriptor(
            "",
            std::string(id_result_type_json_name),
            ast::Instructions::Instruction::Operands::Operand::Quantifier::none));
        retval.push_back(Operand_descriptor(
            "",
            std::string(id_result_json_name),
            ast::Instructions::Instruction::Operands::Operand::Quantifier::none));
        retval.push_back(Operand_descriptor(
            "'Set'",
            std::string(id_ref_json_name),
            ast::Instructions::Instruction::Operands::Operand::Quantifier::none));
        return retval;
    }
    void fill_instruction_descriptors(
        const ast::Extension_instruction_set *extension_instruction_set,
        const ast::Instructions &instructions)
    {
        auto instruction_enumeration = this->op_enumeration.value();
        if(extension_instruction_set)
        {
            auto iter =
                instruction_set_extension_op_enumeration_map.find(extension_instruction_set);
            if(iter == instruction_set_extension_op_enumeration_map.end())
                throw Generate_error("unknown extension instruction set: "
                                     + extension_instruction_set->import_name);
            instruction_enumeration = std::get<1>(*iter);
        }
        for(auto &instruction : instructions.instructions)
        {
            auto enumerant_iter =
                instruction_enumeration->json_name_to_enumerant_map.find(instruction.opname);
            if(enumerant_iter == instruction_enumeration->json_name_to_enumerant_map.end())
            {
                std::string error_message = "unknown instruction: ";
                if(extension_instruction_set)
                {
                    error_message += extension_instruction_set->import_name;
                    error_message += " ";
                }
                error_message += instruction.opname;
                throw Generate_error(error_message);
            }
            auto enumerant = std::get<1>(*enumerant_iter);
            auto *instruction_properties_descriptor =
                extension_instruction_set ?
                    get_instruction_properties_descriptor(extension_instruction_set->import_name,
                                                          instruction.opname) :
                    get_instruction_properties_descriptor({}, instruction.opname);
            auto implied_operands =
                make_instruction_implied_operands(extension_instruction_set != nullptr);
            std::list<Operand_descriptor> explicit_operands;
            util::optional<Instruction_properties_descriptor::Operand_descriptors::const_iterator>
                operand_properties_iter;
            if(instruction_properties_descriptor)
                operand_properties_iter =
                    instruction_properties_descriptor->operand_descriptors.begin();
            for(auto &operand : instruction.operands.operands)
            {
                explicit_operands.push_back(
                    Operand_descriptor(operand.name, operand.kind, operand.quantifier));
                bool has_integer_literal_size = false;
                if(operand_properties_iter)
                {
                    if(*operand_properties_iter
                       == instruction_properties_descriptor->operand_descriptors.end())
                        throw Generate_error("instruction properties operand count mismatch: "
                                             + get_instruction_name_for_diagnostics(
                                                   extension_instruction_set, instruction.opname));
                    if((*operand_properties_iter)->integer_literal_size
                       != Instruction_properties_descriptor::Operand_descriptor::
                              Integer_literal_size::not_implemented)
                        has_integer_literal_size = true;
                    ++*operand_properties_iter;
                }
                auto operand_kind = get_operand_kind(operand.kind);
                if(operand_kind->needs_integer_literal_size && !has_integer_literal_size)
                {
                    if(!instruction_properties_descriptor)
                        throw Generate_error(
                            "instruction has no Instruction_properties_descriptor: "
                            + get_instruction_name_for_diagnostics(extension_instruction_set,
                                                                   instruction.opname)
                            + "\nNeeded because operand needs IntegerLiteral size\n"
                            "instruction properties descriptor guess:\n"
                            + generate_guessed_instruction_properties_descriptor_string(extension_instruction_set, instruction));
                    throw Generate_error(
                        "instruction operand properties has no Integer_literal_size: "
                        + get_instruction_name_for_diagnostics(extension_instruction_set,
                                                               instruction.opname));
                }
            }
            if(operand_properties_iter
               && *operand_properties_iter
                      != instruction_properties_descriptor->operand_descriptors.end())
                throw Generate_error("instruction properties operand count mismatch: "
                                     + get_instruction_name_for_diagnostics(
                                           extension_instruction_set, instruction.opname));
            add_instruction_descriptor(Instruction_descriptor(instruction_enumeration,
                                                              enumerant,
                                                              extension_instruction_set,
                                                              instruction.opname,
                                                              std::move(implied_operands),
                                                              std::move(explicit_operands),
                                                              instruction_properties_descriptor));
        }
    }
    void fill_instruction_descriptors()
    {
        fill_instruction_descriptors(nullptr, top_level.instructions);
        for(auto &extension_instruction_set : top_level.extension_instruction_sets)
            fill_instruction_descriptors(&extension_instruction_set,
                                         extension_instruction_set.instructions);
    }

public:
    void run()
    {
        fill_literal_type_descriptors();
        fill_enumerations();
        fill_id_type_descriptors();
        fill_composite_type_descriptors();
        fill_operand_kinds();
        fill_instruction_descriptors();
        for(auto *file :
            std::initializer_list<Output_file_base *>{&spirv_h, &spirv_cpp, &parser_h, &parser_cpp})
            file->fill_output(*this);
        for(auto *file :
            std::initializer_list<Output_file_base *>{&spirv_h, &spirv_cpp, &parser_h, &parser_cpp})
            file->write_to_file();
    }
};

constexpr util::string_view Spirv_and_parser_generator::State::op_enum_json_name;
constexpr util::string_view Spirv_and_parser_generator::State::extension_enum_json_name;
constexpr util::string_view Spirv_and_parser_generator::State::capability_enum_json_name;
constexpr util::string_view
    Spirv_and_parser_generator::State::extension_instruction_set_enum_json_name;
constexpr util::string_view
    Spirv_and_parser_generator::State::unknown_extension_instruction_set_enumerant_json_name;
constexpr util::string_view Spirv_and_parser_generator::State::op_ext_inst_import_json_name;
constexpr util::string_view Spirv_and_parser_generator::State::op_ext_inst_json_name;
constexpr util::string_view Spirv_and_parser_generator::State::id_result_json_name;
constexpr util::string_view Spirv_and_parser_generator::State::id_result_type_json_name;
constexpr util::string_view Spirv_and_parser_generator::State::id_ref_json_name;

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
