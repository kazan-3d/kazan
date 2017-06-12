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
#include <fstream>
#include <memory>
#include <string>
#include <cassert>
#include <type_traits>
#include <cstdint>
#include <unordered_set>

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
namespace generate
{
class Generator
{
public:
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

protected:
    class Push_indent;
    struct Generator_state
    {
        Generator_args &generator_args;
        std::size_t indent_level;
        std::string full_output_file_name;
        std::string guard_macro_name;
        std::ofstream os;
        explicit Generator_state(const Generator *generator, Generator_args &generator_args);
        void open_output_file();
        template <typename T, typename = decltype(os << std::declval<T>())>
        Generator_state &operator<<(T &&v)
        {
            os << std::forward<T>(v);
            return *this;
        }
        Push_indent pushed_indent() noexcept;
    };
    class Push_indent final
    {
        Push_indent(const Push_indent &) = delete;
        Push_indent &operator=(const Push_indent &) = delete;

    private:
        Generator_state *state;

    public:
        explicit Push_indent(Generator_state &state) noexcept : state(&state)
        {
            state.indent_level++;
        }
        Push_indent(Push_indent &&rt) noexcept : state(rt.state)
        {
            rt.state = nullptr;
        }
        void finish() noexcept
        {
            assert(state);
            state->indent_level--;
            state = nullptr;
        }
        ~Push_indent()
        {
            if(state)
                state->indent_level--;
        }
    };
    struct Indent_t
    {
        explicit Indent_t() = default;
        friend Generator_state &operator<<(Generator_state &state, Indent_t)
        {
            write_indent(state);
            return state;
        }
    };
    static constexpr Indent_t indent{};
    enum class Integer_literal_base
    {
        dec = 0,
        hex,
        oct
    };
    struct Unsigned_integer_literal
    {
        std::uint64_t value;
        Integer_literal_base base;
        std::size_t minimum_digit_count;
        constexpr Unsigned_integer_literal(std::uint64_t value,
                                           Integer_literal_base base,
                                           std::size_t minimum_digit_count = 1) noexcept
            : value(value),
              base(base),
              minimum_digit_count(minimum_digit_count)
        {
        }
        friend Generator_state &operator<<(Generator_state &state, Unsigned_integer_literal v)
        {
            write_unsigned_integer_literal(state, v.value, v.base, v.minimum_digit_count);
            return state;
        }
    };
    static constexpr Unsigned_integer_literal unsigned_dec_integer_literal(
        std::uint64_t value) noexcept
    {
        return Unsigned_integer_literal(value, Integer_literal_base::dec);
    }
    static constexpr Unsigned_integer_literal unsigned_hex_integer_literal(
        std::uint64_t value, std::size_t minimum_digit_count = 1) noexcept
    {
        return Unsigned_integer_literal(value, Integer_literal_base::hex, minimum_digit_count);
    }
    static constexpr Unsigned_integer_literal unsigned_oct_integer_literal(
        std::uint64_t value, std::size_t minimum_digit_count = 1) noexcept
    {
        return Unsigned_integer_literal(value, Integer_literal_base::oct, minimum_digit_count);
    }
    struct Signed_integer_literal
    {
        std::int64_t value;
        constexpr explicit Signed_integer_literal(std::int64_t value) noexcept : value(value)
        {
        }
        friend Generator_state &operator<<(Generator_state &state, Signed_integer_literal v)
        {
            write_signed_integer_literal(state, v.value);
            return state;
        }
    };
    static constexpr Signed_integer_literal signed_integer_literal(std::int64_t value) noexcept
    {
        return Signed_integer_literal(value);
    }

protected:
    const char *const output_base_file_name;

protected:
    static std::string get_guard_macro_name_from_file_name(std::string file_name);
    static void write_indent(Generator_state &state);
    static void write_automatically_generated_file_warning(Generator_state &state);
    static void write_copyright_comment(Generator_state &state, const ast::Copyright &copyright);
    static void write_file_comments(Generator_state &state, const ast::Copyright &copyright)
    {
        write_automatically_generated_file_warning(state);
        write_copyright_comment(state, copyright);
    }
    static void write_file_guard_start(Generator_state &state);
    static void write_file_guard_end(Generator_state &state);
    static void write_namespace_start(Generator_state &state, const char *namespace_name);
    static void write_namespace_start(Generator_state &state, const std::string &namespace_name);

private:
    static void write_namespace_end(Generator_state &state);

protected:
    static void write_namespace_end(Generator_state &state, const char *namespace_name)
    {
        write_namespace_end(state);
    }
    static void write_namespace_end(Generator_state &state, const std::string &namespace_name)
    {
        write_namespace_end(state);
    }
    static void write_namespaces_start(Generator_state &state,
                                       const char *const *namespace_names,
                                       std::size_t namespace_name_count)
    {
        for(std::size_t i = 0; i < namespace_name_count; i++)
            write_namespace_start(state, namespace_names[i]);
    }
    static void write_namespaces_start(Generator_state &state,
                                       const std::string *namespace_names,
                                       std::size_t namespace_name_count)
    {
        for(std::size_t i = 0; i < namespace_name_count; i++)
            write_namespace_start(state, namespace_names[i]);
    }
    static void write_namespaces_end(Generator_state &state,
                                     const char *const *namespace_names,
                                     std::size_t namespace_name_count)
    {
        for(std::size_t i = 0; i < namespace_name_count; i++)
            write_namespace_end(state, namespace_names[namespace_name_count - i - 1]);
        state << '\n';
    }
    static void write_namespaces_end(Generator_state &state,
                                     const std::string *namespace_names,
                                     std::size_t namespace_name_count)
    {
        for(std::size_t i = 0; i < namespace_name_count; i++)
            write_namespace_end(state, namespace_names[namespace_name_count - i - 1]);
        state << '\n';
    }
    template <typename T, std::size_t N>
    static void write_namespaces_start(Generator_state &state, const T(&namespace_names)[N])
    {
        write_namespaces_start(state, namespace_names, N);
    }
    template <typename T, std::size_t N>
    static void write_namespaces_end(Generator_state &state, const T(&namespace_names)[N])
    {
        write_namespaces_end(state, namespace_names, N);
    }
    static void write_namespaces_start(Generator_state &state,
                                       std::initializer_list<std::string> namespace_names)
    {
        write_namespaces_start(state, namespace_names.begin(), namespace_names.size());
    }
    static void write_namespaces_start(Generator_state &state,
                                       std::initializer_list<const char *> namespace_names)
    {
        write_namespaces_start(state, namespace_names.begin(), namespace_names.size());
    }
    static void write_namespaces_end(Generator_state &state,
                                     std::initializer_list<std::string> namespace_names)
    {
        write_namespaces_end(state, namespace_names.begin(), namespace_names.size());
    }
    static void write_namespaces_end(Generator_state &state,
                                     std::initializer_list<const char *> namespace_names)
    {
        write_namespaces_end(state, namespace_names.begin(), namespace_names.size());
    }
    static void write_unsigned_integer_literal(Generator_state &state,
                                               std::uint64_t value,
                                               Integer_literal_base base,
                                               std::size_t minimum_digit_count);
    static void write_signed_integer_literal(Generator_state &state, std::int64_t value);

private:
    struct Get_extensions_visitor;

protected:
    static std::unordered_set<std::string> get_extensions(const ast::Top_level &top_level);

protected:
    static constexpr const char *vulkan_cpu_namespace_name = "vulkan_cpu";
    static constexpr const char *spirv_namespace_name = "spirv";
    static constexpr const char *spirv_namespace_names[] = {
        vulkan_cpu_namespace_name, spirv_namespace_name,
    };

public:
    explicit Generator(const char *output_base_file_name) noexcept
        : output_base_file_name(output_base_file_name)
    {
    }
    virtual void run(Generator_args &generator_args, const ast::Top_level &top_level) const = 0;
    void run(Generator_args &&generator_args, const ast::Top_level &top_level) const
    {
        run(generator_args, top_level);
    }

public:
    virtual ~Generator() = default;
};

inline Generator::Push_indent Generator::Generator_state::pushed_indent() noexcept
{
    return Push_indent(*this);
}

struct Spirv_header_generator;

struct Generators
{
    static std::unique_ptr<Generator> make_spirv_header_generator();
};
}
}
}

#endif /* GENERATE_SPIRV_PARSER_GENERATE_H_ */