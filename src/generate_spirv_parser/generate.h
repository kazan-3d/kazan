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

#ifndef GENERATE_SPIRV_PARSER_GENERATE_H_
#define GENERATE_SPIRV_PARSER_GENERATE_H_

#include "ast.h"
#include "util/filesystem.h"
#include "util/string_view.h"
#include "word_iterator.h"
#include "util/enum.h"
#include <stdexcept>
#include <deque>
#include <cstdint>
#include <string>
#include <utility>
#include <cassert>

namespace kazan
{
namespace generate_spirv_parser
{
namespace generate
{
struct Generate_error : public std::runtime_error
{
    using runtime_error::runtime_error;
};

namespace detail
{
using namespace util::string_view_literals;

constexpr util::string_view keywords[] = {
    "alignas"_sv,
    "alignof"_sv,
    "and"_sv,
    "and_eq"_sv,
    "asm"_sv,
    "atomic_cancel"_sv,
    "atomic_commit"_sv,
    "atomic_noexcept"_sv,
    "auto"_sv,
    "bitand"_sv,
    "bitor"_sv,
    "bool"_sv,
    "break"_sv,
    "case"_sv,
    "catch"_sv,
    "char"_sv,
    "char16_t"_sv,
    "char32_t"_sv,
    "class"_sv,
    "compl"_sv,
    "concept"_sv,
    "concepts"_sv,
    "const"_sv,
    "const_cast"_sv,
    "constexpr"_sv,
    "continue"_sv,
    "decltype"_sv,
    "default"_sv,
    "delete"_sv,
    "do"_sv,
    "double"_sv,
    "dynamic_cast"_sv,
    "else"_sv,
    "enum"_sv,
    "explicit"_sv,
    "export"_sv,
    "extern"_sv,
    "false"_sv,
    "float"_sv,
    "for"_sv,
    "friend"_sv,
    "goto"_sv,
    "if"_sv,
    "import"_sv,
    "inline"_sv,
    "int"_sv,
    "long"_sv,
    "module"_sv,
    "modules"_sv,
    "mutable"_sv,
    "namespace"_sv,
    "new"_sv,
    "noexcept"_sv,
    "not"_sv,
    "not_eq"_sv,
    "nullptr"_sv,
    "operator"_sv,
    "or"_sv,
    "or_eq"_sv,
    "private"_sv,
    "protected"_sv,
    "public"_sv,
    "register"_sv,
    "reinterpret_cast"_sv,
    "requires"_sv,
    "return"_sv,
    "short"_sv,
    "signed"_sv,
    "sizeof"_sv,
    "static"_sv,
    "static_assert"_sv,
    "static_cast"_sv,
    "struct"_sv,
    "switch"_sv,
    "synchronized"_sv,
    "template"_sv,
    "this"_sv,
    "thread_local"_sv,
    "throw"_sv,
    "true"_sv,
    "try"_sv,
    "typedef"_sv,
    "typeid"_sv,
    "typename"_sv,
    "union"_sv,
    "unsigned"_sv,
    "using"_sv,
    "virtual"_sv,
    "void"_sv,
    "volatile"_sv,
    "wchar_t"_sv,
    "while"_sv,
    "xor"_sv,
    "xor_eq"_sv,
};

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

private:
    static constexpr std::size_t output_tab_width_no_tabs_allowed = 0;
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
    static constexpr util::string_view literal_command = "literal:"_sv;
    static constexpr util::string_view push_start_command = "push_start"_sv;
    static constexpr util::string_view pop_start_command = "pop_start"_sv;
    static constexpr util::string_view add_start_offset_command = "add_start_offset:"_sv;
    static constexpr util::string_view restart_indent_command = "restart_indent"_sv;

public:
    static constexpr char indent_indicator_char = ' ';
    static constexpr char literal_indent_indicator_char = '`';
    static constexpr std::size_t indent_indicators_per_indent = 4;
    static constexpr char escape_char = '@';
    static constexpr bool indent_blank_lines = false;

public:
    void write_to_file(bool do_reindent = true) const;

private:
    void write_unsigned_integer(std::uint64_t value,
                                unsigned base = 10,
                                std::size_t min_length = 1);
    void write_signed_integer(std::int64_t value, unsigned base = 10);
    void write_literal(util::string_view value)
    {
        *this << escape_char << literal_command << static_cast<std::uint64_t>(value.size())
              << escape_char << value << escape_char;
    }

public:
    template <typename T>
    Generated_output_stream &operator<<(T) = delete;
    Generated_output_stream &operator<<(char ch)
    {
        value.push_back(ch);
        return *this;
    }
    Generated_output_stream &operator<<(const Generated_output_stream &s)
    {
        assert(this != &s);
        value.insert(value.end(), s.value.begin(), s.value.end());
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
    class Literal_holder
    {
        friend class Generated_output_stream;

    private:
        util::string_view value;
        constexpr explicit Literal_holder(util::string_view value) noexcept : value(value)
        {
        }
    };
    Generated_output_stream &operator<<(const Literal_holder &v)
    {
        write_literal(v.value);
        return *this;
    }
    static constexpr Literal_holder literal(util::string_view value) noexcept
    {
        return Literal_holder{value};
    }
    struct Push_start
    {
    };
    Generated_output_stream &operator<<(Push_start)
    {
        *this << escape_char << push_start_command << escape_char;
        return *this;
    }
    struct Pop_start
    {
    };
    Generated_output_stream &operator<<(Pop_start)
    {
        *this << escape_char << pop_start_command << escape_char;
        return *this;
    }
    class Add_start_offset_holder
    {
        friend class Generated_output_stream;

    private:
        std::int64_t offset;
        constexpr explicit Add_start_offset_holder(std::int64_t offset) noexcept : offset(offset)
        {
        }
    };
    Generated_output_stream &operator<<(const Add_start_offset_holder &v)
    {
        *this << escape_char << add_start_offset_command << v.offset << escape_char;
        return *this;
    }
    static constexpr Add_start_offset_holder add_start_offset(std::int64_t offset) noexcept
    {
        return Add_start_offset_holder{offset};
    }
    struct Restart_indent
    {
    };
    Generated_output_stream &operator<<(Restart_indent)
    {
        *this << escape_char << restart_indent_command << escape_char;
        return *this;
    }
    class Unsigned_integer_holder
    {
        friend class Generated_output_stream;

    private:
        std::uint64_t value;
        unsigned base;
        std::size_t min_length;
        constexpr explicit Unsigned_integer_holder(std::uint64_t value,
                                                   unsigned base,
                                                   std::size_t min_length) noexcept
            : value(value),
              base(base),
              min_length(min_length)
        {
        }
    };
    Generated_output_stream &operator<<(const Unsigned_integer_holder &v)
    {
        write_unsigned_integer(v.value, v.base, v.min_length);
        return *this;
    }
    static constexpr Unsigned_integer_holder unsigned_integer(std::uint64_t value,
                                                              unsigned base = 10,
                                                              std::size_t min_length = 1) noexcept
    {
        return Unsigned_integer_holder{value, base, min_length};
    }
    class Signed_integer_holder
    {
        friend class Generated_output_stream;

    private:
        std::int64_t value;
        unsigned base;
        constexpr explicit Signed_integer_holder(std::int64_t value, unsigned base) noexcept
            : value(value),
              base(base)
        {
        }
    };
    Generated_output_stream &operator<<(const Signed_integer_holder &v)
    {
        write_signed_integer(v.value, v.base);
        return *this;
    }
    static constexpr Signed_integer_holder signed_integer(std::int64_t value,
                                                          unsigned base = 10) noexcept
    {
        return Signed_integer_holder{value, base};
    }
    struct Guard_macro
    {
    };
    Generated_output_stream &operator<<(Guard_macro);
    enum Name_format
    {
        initial_capital,
        all_lowercase,
        all_uppercase,
        all_uppercase_with_trailing_underline,
    };

private:
    static std::string name_from_words_helper(Name_format name_format, std::string name);

public:
    template <std::size_t N>
    class Name_from_words_holder
    {
        friend class Generated_output_stream;

    private:
        Name_format name_format;
        Chained_word_iterator<N> iter;
        template <typename... Args>
        constexpr explicit Name_from_words_holder(
            Name_format name_format,
            Args &&... args) noexcept(noexcept(Chained_word_iterator<N>(std::
                                                                            forward<Args>(
                                                                                args)...)))
            : name_format(name_format), iter(std::forward<Args>(args)...)
        {
            static_assert(sizeof...(Args) == N, "");
        }

    public:
        std::string to_string() const
        {
            std::size_t name_size = 0;
            for(const util::string_view &word : iter)
            {
                // don't skip first '_' to allow for trailing '_' to prevent generating keywords
                name_size += 1 + word.size();
            }
            std::string name;
            name.reserve(name_size);
            bool first = true;
            for(const util::string_view &word : iter)
            {
                if(first)
                    first = false;
                else
                    name += '_';
                name += word;
            }
            return name_from_words_helper(name_format, std::move(name));
        }
    };
    template <std::size_t N>
    Generated_output_stream &operator<<(const Name_from_words_holder<N> &v)
    {
        *this << v.to_string();
        return *this;
    }
    template <typename... Args>
    static constexpr Name_from_words_holder<sizeof...(Args)>
        name_from_words(Name_format name_format, Args &&... args) noexcept(noexcept(
            Name_from_words_holder<sizeof...(Args)>(name_format, std::forward<Args>(args)...)))
    {
        return Name_from_words_holder<sizeof...(Args)>(name_format, std::forward<Args>(args)...);
    }
};

constexpr auto literal(util::string_view value) noexcept
{
    return Generated_output_stream::literal(value);
}

constexpr auto add_start_offset(std::int64_t offset) noexcept
{
    return Generated_output_stream::add_start_offset(offset);
}

constexpr auto unsigned_integer(std::uint64_t value) noexcept
{
    return Generated_output_stream::unsigned_integer(value);
}

constexpr auto unsigned_integer(std::uint64_t value, unsigned base) noexcept
{
    return Generated_output_stream::unsigned_integer(value, base);
}

constexpr auto unsigned_integer(std::uint64_t value, unsigned base, std::size_t min_length) noexcept
{
    return Generated_output_stream::unsigned_integer(value, base, min_length);
}

constexpr auto signed_integer(std::int64_t value) noexcept
{
    return Generated_output_stream::signed_integer(value);
}

constexpr auto signed_integer(std::int64_t value, unsigned base) noexcept
{
    return Generated_output_stream::signed_integer(value, base);
}

constexpr Generated_output_stream::Guard_macro guard_macro{};
constexpr Generated_output_stream::Push_start push_start{};
constexpr Generated_output_stream::Pop_start pop_start{};
constexpr Generated_output_stream::Restart_indent restart_indent{};

template <typename... Args>
constexpr auto name_from_words(
    Generated_output_stream::Name_format name_format,
    Args &&... args) noexcept(noexcept(Generated_output_stream::name_from_words(name_format,
                                                                                std::forward<Args>(
                                                                                    args)...)))
{
    return Generated_output_stream::name_from_words(name_format, std::forward<Args>(args)...);
}

template <typename... Args>
constexpr auto name_from_words_all_lowercase(Args &&... args) noexcept(
    noexcept(Generated_output_stream::name_from_words(Generated_output_stream::all_lowercase,
                                                      std::forward<Args>(args)...)))
{
    return Generated_output_stream::name_from_words(Generated_output_stream::all_lowercase,
                                                    std::forward<Args>(args)...);
}

template <typename... Args>
constexpr auto name_from_words_all_uppercase(Args &&... args) noexcept(
    noexcept(Generated_output_stream::name_from_words(Generated_output_stream::all_uppercase,
                                                      std::forward<Args>(args)...)))
{
    return Generated_output_stream::name_from_words(Generated_output_stream::all_uppercase,
                                                    std::forward<Args>(args)...);
}

template <typename... Args>
constexpr auto name_from_words_initial_capital(Args &&... args) noexcept(
    noexcept(Generated_output_stream::name_from_words(Generated_output_stream::initial_capital,
                                                      std::forward<Args>(args)...)))
{
    return Generated_output_stream::name_from_words(Generated_output_stream::initial_capital,
                                                    std::forward<Args>(args)...);
}

template <typename... Args>
constexpr auto name_from_words_all_uppercase_with_trailing_underline(Args &&... args) noexcept(
    noexcept(Generated_output_stream::name_from_words(
        Generated_output_stream::all_uppercase_with_trailing_underline,
        std::forward<Args>(args)...)))
{
    return Generated_output_stream::name_from_words(
        Generated_output_stream::all_uppercase_with_trailing_underline,
        std::forward<Args>(args)...);
}
}

struct Generator
{
    struct Generator_args
    {
        std::string output_directory;
        explicit Generator_args(std::string output_directory) noexcept
            : output_directory(std::move(output_directory))
        {
        }
        Generator_args(Generator_args &&) = default;
        Generator_args &operator=(Generator_args &&) = default;
        Generator_args(const Generator_args &) = delete;
        Generator_args &operator=(const Generator_args &) = delete;
    };
    virtual ~Generator() = default;
    virtual void run(Generator_args &generator_args, const ast::Top_level &top_level) const = 0;
    void run(Generator_args &&generator_args, const ast::Top_level &top_level) const
    {
        run(generator_args, top_level);
    }
};

struct Spirv_and_parser_generator;

struct Generators
{
    static std::unique_ptr<Generator> make_spirv_and_parser_generator();
    static std::vector<std::unique_ptr<Generator>> make_all_generators();
};
}
}
}

#endif /* GENERATE_SPIRV_PARSER_GENERATE_H_ */
