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

#ifndef JSON_JSON_H_
#define JSON_JSON_H_

#include <string>
#include <unordered_map>
#include <vector>
#include <iosfwd>
#include <memory>
#include <utility>
#include <limits>
#include "../util/variant.h"

namespace vulkan_cpu
{
namespace json
{
namespace ast
{
struct composite_value
{
    composite_value() = default;
    virtual ~composite_value() = default;
    virtual void write(std::ostream &os) const = 0;
    virtual std::unique_ptr<composite_value> duplicate() const = 0;
};

struct null_value final
{
    void write(std::ostream &os) const;
    null_value duplicate() const noexcept
    {
        return {};
    }
    const null_value *operator->() const noexcept
    {
        return this;
    }
    const null_value &operator*() const noexcept
    {
        return *this;
    }
};

struct boolean_value final
{
    bool value;
    constexpr boolean_value(bool value) noexcept : value(value)
    {
    }
    void write(std::ostream &os) const;
    boolean_value duplicate() const noexcept
    {
        return *this;
    }
    const boolean_value *operator->() const noexcept
    {
        return this;
    }
    const boolean_value &operator*() const noexcept
    {
        return *this;
    }
};

struct string_value final
{
    std::string value;
    string_value(std::string value) noexcept : value(std::move(value))
    {
    }
    string_value(const char *value) : value(std::move(value))
    {
    }
    static void write(std::ostream &os, const std::string &value);
    void write(std::ostream &os) const
    {
        write(os, value);
    }
    string_value duplicate() const noexcept
    {
        return *this;
    }
    const string_value *operator->() const noexcept
    {
        return this;
    }
    const string_value &operator*() const noexcept
    {
        return *this;
    }
};

struct number_value final
{
    double value;
    static_assert(std::numeric_limits<double>::is_iec559 && std::numeric_limits<double>::radix == 2,
                  "double is not a ieee754 float64");
    number_value(double value) noexcept : value(value)
    {
    }
    explicit operator std::string() const
    {
        return to_string();
    }
    static constexpr unsigned max_base = 36;
    static constexpr unsigned min_base = 2;
    static constexpr unsigned default_base = 10; // the json spec only supports base 10
    std::string to_string(std::string buffer_in = {}, unsigned base = default_base) const;
    std::size_t to_string(char *output_buffer,
                          std::size_t output_buffer_size,
                          bool require_null_terminator = true,
                          unsigned base = default_base) const noexcept;
    void write(std::ostream &os, unsigned base = default_base) const;
    number_value duplicate() const noexcept
    {
        return *this;
    }
    const number_value *operator->() const noexcept
    {
        return this;
    }
    const number_value &operator*() const noexcept
    {
        return *this;
    }
};

typedef util::
    variant<null_value, boolean_value, string_value, number_value, std::unique_ptr<composite_value>>
        value;

inline value duplicate(const value &v)
{
    return util::visit(
        [](const auto &v) -> value
        {
            return v->duplicate();
        },
        v);
}

inline void write(std::ostream &os, const value &v)
{
    util::visit(
        [&](const auto &v) -> void
        {
            return v->write(os);
        },
        v);
}

struct object final : public composite_value
{
    std::unordered_map<std::string, value> values;
    object() : values()
    {
    }
    object(std::unordered_map<std::string, value> values) noexcept : values(std::move(values))
    {
    }
    virtual void write(std::ostream &os) const override;
    virtual std::unique_ptr<composite_value> duplicate() const override
    {
        std::unordered_map<std::string, value> new_values;
        for(auto &entry : values)
        {
        	new_values.emplace(std::get<0>(entry), ast::duplicate(std::get<1>(entry)));
        }
        return std::unique_ptr<composite_value>(new object(std::move(new_values)));
    }
};

struct array final : public composite_value
{
    std::vector<value> values;
    array() : values()
    {
    }
    array(std::vector<value> values) noexcept : values(std::move(values))
    {
    }
    virtual void write(std::ostream &os) const override;
    virtual std::unique_ptr<composite_value> duplicate() const override
    {
    	std::vector<value> new_values;
    	new_values.reserve(values.size());
    	for(auto &value : values)
    		new_values.emplace_back(ast::duplicate(value));
        return std::unique_ptr<composite_value>(new array(std::move(new_values)));
    }
};
}
}
}

#endif /* JSON_JSON_H_ */
