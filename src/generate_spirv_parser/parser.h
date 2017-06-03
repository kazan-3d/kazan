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

#ifndef GENERATE_SPIRV_PARSER_PARSER_H_
#define GENERATE_SPIRV_PARSER_PARSER_H_

#include "ast.h"
#include <stdexcept>
#include <cassert>
#include <string>
#include <vector>
#include "../util/variant.h"
#include "../json/json.h"

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
namespace parser
{
struct path
{
    typedef util::variant<std::size_t, std::string> element;
    std::vector<element> elements;
    path() : elements()
    {
    }
    path(std::vector<element> elements) : elements(std::move(elements))
    {
    }
    path(std::initializer_list<element> elements) : elements(elements)
    {
    }
    std::string to_string() const;
};

struct path_builder_base
{
    path_builder_base(const path_builder_base &) = delete;
    path_builder_base &operator=(const path_builder_base &) = delete;
    virtual ~path_builder_base() = default;
    const path_builder_base *const parent;
    const std::size_t element_count;
    explicit path_builder_base(const path_builder_base *parent) noexcept
        : parent(parent),
          element_count(parent ? parent->element_count + 1 : 1)
    {
    }
    virtual path::element get_element() const = 0;
    path path() const
    {
        std::vector<path::element> elements;
        elements.resize(element_count);
        const path_builder_base *node = this;
        for(std::size_t i = 0, j = element_count - 1; i < element_count;
            i++, j--, node = node->parent)
        {
            assert(node);
            elements[j] = node->get_element();
        }
        assert(!node);
        return std::move(elements);
    }
};

template <typename T>
struct path_builder final : public path_builder_base
{
    const T *value;
    path_builder(const T *value, const path_builder_base *parent) noexcept
        : path_builder_base(parent),
          value(value)
    {
    }
    virtual path::element get_element() const override
    {
        return *value;
    }
};

class parse_error : public std::runtime_error
{
public:
    path path;
    parse_error(parser::path path, const std::string &message)
        : runtime_error("at " + path.to_string() + ": " + message), path(std::move(path))
    {
    }
};

ast::top_level parse(json::ast::value &&top_level_value);
}
}
}

#endif /* GENERATE_SPIRV_PARSER_PARSER_H_ */
