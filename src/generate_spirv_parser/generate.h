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
#include "util/string_view.h"
#include <fstream>
#include <memory>
#include <string>
#include <cassert>
#include <type_traits>
#include <cstdint>
#include <unordered_set>
#include <unordered_map>
#include <vector>
#include <stdexcept>

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
namespace generate
{
struct Generate_error : public std::runtime_error
{
    using runtime_error::runtime_error;
};

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
