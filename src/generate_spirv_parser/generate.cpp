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
        std::list<Output_struct> enum_structs;
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
        }
        void write_enum_structs(detail::Generated_output_stream &output_stream, Output_part) const
        {
            for(auto &enum_struct : enum_structs)
            {
                output_stream << "\n";
                enum_struct.write_whole_output(output_stream);
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
                auto enumeration_iter = state.get_enumeration(operand_kind.json_name);
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
                                             + operand_kind.json_name);
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
#warning finish
            include_guard_start << R"(#error generator not finished being implemented

)";
            write_literal_kinds(state);
            write_basic_constants(state);
            write_enums(state);
            write_id_types(state);
            write_enum_parameters(state);
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
        explicit Parser_h(const util::filesystem::path &file_path) : Header_file_base(file_path)
        {
        }
        virtual void fill_output(State &state) override
        {
            Header_file_base::fill_output(state);
            write_local_include_path(state.spirv_h.file_path);
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
    std::list<Enumeration_descriptor>::const_iterator get_enumeration(const std::string &json_name)
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
    std::list<Id_type_descriptor>::const_iterator get_id_type(const std::string &json_name)
    {
        auto iter = id_type_map.find(json_name);
        if(iter == id_type_map.end())
            throw Generate_error("unknown id type: " + json_name);
        return std::get<1>(*iter);
    }

private:
    struct Enum_parameter
    {
        std::list<Enumerant_descriptor>::const_iterator enumerant;
        std::string cpp_name;
        std::string json_kind;
        std::string json_name;
        explicit Enum_parameter(std::list<Enumerant_descriptor>::const_iterator enumerant,
                                std::string json_kind,
                                std::string json_name)
            : enumerant(enumerant),
              cpp_name(json_name.empty() ?
                           detail::name_from_words_all_lowercase(json_kind).to_string() :
                           detail::name_from_words_all_lowercase(json_name).to_string()),
              json_kind(std::move(json_kind)),
              json_name(std::move(json_name))
        {
        }
    };
    struct Operand_kind_descriptor
    {
        std::string json_name;
        util::variant<util::monostate,
                      std::list<Enumeration_descriptor>::const_iterator,
                      std::list<Id_type_descriptor>::const_iterator,
                      ast::Operand_kinds::Operand_kind::Literal_kind> value;
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
            };
            util::visit(Visitor{retval, json_name}, value);
            return retval;
        }
        std::string make_cpp_name_with_parameters() const
        {
            auto *iter = util::get_if<std::list<Enumeration_descriptor>::const_iterator>(&value);
            if(iter && !enum_parameters.empty())
                return detail::name_from_words_initial_capital(json_name, "with parameters"_sv)
                    .to_string();
            return cpp_name;
        }
        std::string cpp_name;
        std::string cpp_name_with_parameters;
        explicit Operand_kind_descriptor(std::string json_name)
            : Operand_kind_descriptor(std::move(json_name), util::monostate{})
        {
        }
        template <typename T>
        Operand_kind_descriptor(std::string json_name,
                                T arg,
                                std::list<Enum_parameter> enum_parameters = {})
            : json_name(std::move(json_name)),
              value(std::move(arg)),
              enum_parameters(std::move(enum_parameters)),
              cpp_name(make_cpp_name()),
              cpp_name_with_parameters(make_cpp_name_with_parameters())
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
        auto name = operand_kind_descriptor.json_name;
        auto iter =
            operand_kind_list.insert(operand_kind_list.end(), std::move(operand_kind_descriptor));
        if(!std::get<1>(operand_kind_map.emplace(name, iter)))
            throw Generate_error("duplicate operand kind: " + name);
        return iter;
    }
    std::list<Operand_kind_descriptor>::const_iterator get_operand_kind(
        const std::string &json_name)
    {
        auto iter = operand_kind_map.find(json_name);
        if(iter == operand_kind_map.end())
            throw Generate_error("unknown operand kind: " + json_name);
        return std::get<1>(*iter);
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
                add_operand_kind(Operand_kind_descriptor(operand_kind.kind,
                                                         enumerations_map.at(operand_kind.kind),
                                                         std::move(enum_parameters)));
                continue;
            }
            case ast::Operand_kinds::Operand_kind::Category::id:
                add_operand_kind(
                    Operand_kind_descriptor(operand_kind.kind, id_type_map.at(operand_kind.kind)));
                continue;
            case ast::Operand_kinds::Operand_kind::Category::literal:
                add_operand_kind(Operand_kind_descriptor(operand_kind.kind,
                                                         get_literal_kind(operand_kind.kind)));
                continue;
            case ast::Operand_kinds::Operand_kind::Category::composite:
#warning finish
                add_operand_kind(Operand_kind_descriptor(operand_kind.kind));
                continue;
            }
            assert(false);
        }
    }

public:
    void run()
    {
        fill_literal_type_descriptors();
        fill_enumerations();
        fill_id_type_descriptors();
        fill_operand_kinds();
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
