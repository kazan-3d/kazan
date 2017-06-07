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
    struct Generator_state
    {
        Generator_args &generator_args;
        std::size_t indent_level;
        std::string full_output_file_name;
        std::ofstream os;
        explicit Generator_state(const Generator *generator, Generator_args &generator_args);
        void open_output_file();
        template <typename T, typename = decltype(os << std::declval<T>())>
        Generator_state &operator<<(T &&v)
        {
            os << std::forward<T>(v);
            return *this;
        }
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

protected:
    const char *const output_base_file_name;

protected:
    static void write_indent(Generator_state &state);
    static void write_automatically_generated_file_warning(Generator_state &state);
    static void write_copyright_comment(Generator_state &state, const ast::Copyright &copyright);
    static void write_file_comments(Generator_state &state, const ast::Copyright &copyright)
    {
        write_automatically_generated_file_warning(state);
        write_copyright_comment(state, copyright);
    }

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

struct Spirv_header_generator;

struct Generators
{
    static std::unique_ptr<Generator> make_spirv_header_generator();
};
}
}
}

#endif /* GENERATE_SPIRV_PARSER_GENERATE_H_ */