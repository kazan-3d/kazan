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
#include "util/filesystem.h"
#include <limits>
#include <algorithm>
#include <cstdlib>
#include <iostream>
#include <sstream>
#include <deque>
#include <unordered_set>

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
namespace generate
{
using namespace util::string_view_literals;

struct Spirv_and_parser_generator : public Generator
{
    struct State;
    virtual void run(Generator_args &generator_args,
                     const ast::Top_level &top_level) const override;
};

namespace
{
}

struct Spirv_and_parser_generator::State
{
    class Generated_output_stream
    {
    private:
        std::deque<char> value;
        util::filesystem::path file_path;

    public:
        explicit Generated_output_stream(util::filesystem::path file_path) noexcept
            : value(),
              file_path(std::move(file_path))
        {
        }
        const util::filesystem::path &get_file_path() const noexcept
        {
            return file_path;
        }
        static constexpr std::size_t output_tab_width_no_tabs_allowed = 0;
        static constexpr util::string_view literal_command = "literal:"_sv;
        template <typename Fn>
        static void write_indent(Fn write_char,
                                 std::size_t indent_depth,
                                 std::size_t output_tab_width = output_tab_width_no_tabs_allowed)
        {
            if(output_tab_width != output_tab_width_no_tabs_allowed)
            {
                while(indent_depth >= output_tab_width)
                {
                    indent_depth -= output_tab_width;
                    write_char('\t');
                }
            }
            while(indent_depth--)
                write_char(' ');
        }
        static constexpr char indent_indicator_char = ' ';
        static constexpr char literal_indent_indicator_char = '`';
        static constexpr std::size_t indent_indicators_per_indent = 4;
        static constexpr char escape_char = '@';
        static constexpr bool indent_blank_lines = false;
        void write_to_file(bool do_reindent = true) const
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
                                    if(command_sv.compare(
                                           0, literal_command.size(), literal_command)
                                       == 0)
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
        void write_unsigned_integer(std::uint64_t value,
                                    unsigned base = json::ast::Number_value::default_base,
                                    std::size_t min_length = 1)
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
        void write_signed_integer(std::int64_t value,
                                  unsigned base = json::ast::Number_value::default_base)
        {
            static_assert(std::numeric_limits<decltype(value)>::radix == 2, "");
            constexpr std::size_t buffer_size =
                std::numeric_limits<decltype(value)>::digits + 1; // one extra for sign
            char buffer[buffer_size];
            std::size_t length = json::ast::Number_value::signed_integer_to_buffer(
                value, buffer, buffer_size, false, base);
            *this << util::string_view(buffer, length);
        }
        void write_literal(util::string_view value)
        {
            *this << escape_char;
            *this << literal_command;
            write_unsigned_integer(value.size());
            *this << escape_char;
            *this << value;
            *this << escape_char;
        }
        template <typename T>
        Generated_output_stream &operator<<(T) = delete;
        Generated_output_stream &operator<<(char ch)
        {
            value.push_back(ch);
            return *this;
        }
        Generated_output_stream &operator<<(util::string_view sv)
        {
            for(char ch : sv)
                *this << ch;
            return *this;
        }
        Generated_output_stream &operator<<(const char *s)
        {
            return operator<<(util::string_view(s));
        }
        Generated_output_stream &operator<<(const std::string &s)
        {
            return operator<<(util::string_view(s));
        }
        Generated_output_stream &operator<<(std::uint64_t v)
        {
            write_unsigned_integer(v);
            return *this;
        }
        Generated_output_stream &operator<<(std::int64_t v)
        {
            write_signed_integer(v);
            return *this;
        }
        Generated_output_stream &operator<<(const ast::Copyright &v)
        {
            *this << "/*\n";
            for(auto &line : v.lines)
            {
                if(line.empty())
                {
                    *this << "`*\n";
                    continue;
                }
                *this << "`* ";
                bool was_last_star = false;
                for(char ch : line)
                {
                    if(was_last_star && ch == '/')
                        *this << ' ';
                    was_last_star = (ch == '*');
                    *this << ch;
                }
                *this << "\n";
            }
            *this << "`*/\n";
            return *this;
        }
    };
    struct Literal_holder
    {
        util::string_view value;
        friend Generated_output_stream &operator<<(Generated_output_stream &os,
                                                   const Literal_holder &v)
        {
            os.write_literal(v.value);
            return os;
        }
    };
    static Literal_holder literal(util::string_view value)
    {
        return Literal_holder{value};
    }
    struct Unsigned_integer_holder
    {
        std::uint64_t value;
        unsigned base;
        std::size_t min_length;
        friend Generated_output_stream &operator<<(Generated_output_stream &os,
                                                   const Unsigned_integer_holder &v)
        {
            os.write_unsigned_integer(v.value, v.base, v.min_length);
            return os;
        }
    };
    static Unsigned_integer_holder unsigned_integer(
        std::uint64_t value,
        unsigned base = json::ast::Number_value::default_base,
        std::size_t min_length = 1)
    {
        return Unsigned_integer_holder{value, base, min_length};
    }
    struct Signed_integer_holder
    {
        std::int64_t value;
        unsigned base;
        friend Generated_output_stream &operator<<(Generated_output_stream &os,
                                                   const Signed_integer_holder &v)
        {
            os.write_unsigned_integer(v.value, v.base);
            return os;
        }
    };
    static Signed_integer_holder signed_integer(
        std::int64_t value, unsigned base = json::ast::Number_value::default_base)
    {
        return Signed_integer_holder{value, base};
    }
    class Word_iterator
    {
    public:
        typedef std::ptrdiff_t difference_type;
        typedef util::string_view value_type;
        typedef const util::string_view &reference;
        typedef const util::string_view *pointer;
        typedef std::input_iterator_tag iterator_category;

    private:
        enum class Char_class
        {
            uppercase,
            other_identifier,
            word_separator
        };
        static constexpr Char_class get_char_class(char ch) noexcept
        {
            if(ch >= 'A' && ch <= 'Z')
                return Char_class::uppercase;
            if(ch >= 'a' && ch <= 'z')
                return Char_class::other_identifier;
            if(ch >= '0' && ch <= '9')
                return Char_class::other_identifier;
            return Char_class::word_separator;
        }

    private:
        util::string_view word;
        util::string_view words;

    private:
        constexpr void next() noexcept
        {
            util::optional<std::size_t> word_start;
            Char_class last_char_class = Char_class::word_separator;
            for(std::size_t i = 0; i < words.size(); i++)
            {
                auto current_char_class = get_char_class(words[i]);
                if(word_start)
                {
                    switch(current_char_class)
                    {
                    case Char_class::word_separator:
                        word = util::string_view(words.data() + *word_start, i - *word_start);
                        words.remove_prefix(i);
                        last_char_class = current_char_class;
                        return;
                    case Char_class::uppercase:
                        if(last_char_class != Char_class::uppercase)
                        {
                            word = util::string_view(words.data() + *word_start, i - *word_start);
                            words.remove_prefix(i);
                            last_char_class = current_char_class;
                            return;
                        }
                        if(i + 1 < words.size()
                           && get_char_class(words[i + 1]) == Char_class::other_identifier)
                        {
                            word = util::string_view(words.data() + *word_start, i - *word_start);
                            words.remove_prefix(i);
                            last_char_class = current_char_class;
                            return;
                        }
                        break;
                    case Char_class::other_identifier:
                        break;
                    }
                }
                else if(current_char_class != Char_class::word_separator)
                {
                    word_start = i;
                }
                last_char_class = current_char_class;
            }
            if(word_start)
                word = util::string_view(words.data() + *word_start, words.size() - *word_start);
            else
                word = {};
            words = {};
        }
        constexpr bool at_end() const noexcept
        {
            return word.empty();
        }

    public:
        constexpr Word_iterator() noexcept : word(), words()
        {
        }
        constexpr explicit Word_iterator(util::string_view words) noexcept : word(), words(words)
        {
            next();
        }
        constexpr const util::string_view &operator*() const noexcept
        {
            return word;
        }
        constexpr const util::string_view *operator->() const noexcept
        {
            return &word;
        }
        constexpr Word_iterator &operator++() noexcept
        {
            next();
            return *this;
        }
        constexpr Word_iterator operator++(int) noexcept
        {
            auto retval = *this;
            next();
            return retval;
        }
        constexpr bool operator==(const Word_iterator &rt) const noexcept
        {
            return word.empty() == rt.word.empty();
        }
        constexpr bool operator!=(const Word_iterator &rt) const noexcept
        {
            return word.empty() != rt.word.empty();
        }
        constexpr Word_iterator begin() const noexcept
        {
            return *this;
        }
        constexpr Word_iterator end() const noexcept
        {
            return {};
        }
    };
    static void write_guard_macro(Generated_output_stream &os)
    {
        auto path_string = os.get_file_path().string();
        for(auto &word : Word_iterator(path_string))
        {
            for(char ch : word)
            {
                if(ch >= 'a' && ch <= 'z')
                    ch = ch - 'a' + 'A'; // to uppercase
                os << ch;
            }
            os << '_';
        }
    }
    struct Guard_macro
    {
        friend Generated_output_stream &operator<<(Generated_output_stream &os, Guard_macro)
        {
            write_guard_macro(os);
            return os;
        }
    };
    static constexpr Guard_macro guard_macro{};
    const ast::Top_level &top_level;
    Generated_output_stream spirv_h;
    Generated_output_stream spirv_cpp;
    Generated_output_stream parser_h;
    Generated_output_stream parser_cpp;
    State(const util::filesystem::path &output_directory, const ast::Top_level &top_level)
        : top_level(top_level),
          spirv_h(output_directory / "spirv.h"),
          spirv_cpp(output_directory / "spirv.cpp"),
          parser_h(output_directory / "parser.h"),
          parser_cpp(output_directory / "parser.cpp")
    {
    }
    void run()
    {
        constexpr auto automatically_generated_file_warning_comment =
            R"(/* This file is automatically generated by generate_spirv_parser. DO NOT MODIFY. */
)"_sv;
        spirv_h << automatically_generated_file_warning_comment << top_level.copyright;
        spirv_cpp << automatically_generated_file_warning_comment << top_level.copyright;
        parser_h << automatically_generated_file_warning_comment << top_level.copyright;
        parser_cpp << automatically_generated_file_warning_comment << top_level.copyright;
        spirv_h << R"(#ifndef )" << guard_macro << R"(
#define )" << guard_macro
                << R"(
)";
        parser_h << R"(#ifndef )" << guard_macro << R"(
#define )" << guard_macro
                 << R"(
)";
        spirv_cpp << R"(#include ")" << spirv_h.get_file_path().filename().string() << R"("
)";
        parser_h << R"(
#include ")" << spirv_h.get_file_path().filename().string()
                 << R"("
)";
        parser_cpp << R"(#include ")" << parser_h.get_file_path().filename().string() << R"("
)";
        spirv_h << R"(
#endif /* )" << guard_macro
                << R"( */
)";
        parser_h << R"(
#endif /* )" << guard_macro
                 << R"( */
)";
#warning finish
        spirv_h << R"(
#error generator not finished being implemented
)";
        spirv_h.write_to_file();
        spirv_cpp.write_to_file();
        parser_h.write_to_file();
        parser_cpp.write_to_file();
    }
};

constexpr util::string_view
    Spirv_and_parser_generator::State::Generated_output_stream::literal_command;

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
