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
#include <type_traits>
#include <cassert>
#include "../util/variant.h"

namespace vulkan_cpu
{
namespace json
{
struct write_options
{
    bool composite_value_elements_on_seperate_lines = false;
    bool sort_object_values = false;
    std::string indent_text = "";
    write_options()
    {
    }
    write_options(bool composite_value_elements_on_seperate_lines,
                  bool sort_object_values,
                  std::string indent_text) noexcept
        : composite_value_elements_on_seperate_lines(composite_value_elements_on_seperate_lines),
          sort_object_values(sort_object_values),
          indent_text(std::move(indent_text))
    {
    }
    static write_options defaults()
    {
        return {};
    }
    static write_options pretty(std::string indent_text = "    ")
    {
        return write_options(true, true, std::move(indent_text));
    }
};

struct write_state
{
    write_options options;
    std::size_t indent_level = 0;
    class push_indent final
    {
        push_indent(const push_indent &) = delete;
        push_indent &operator=(const push_indent &) = delete;

    private:
        write_state &state;
        bool finished = false;

    public:
        push_indent(write_state &state) : state(state)
        {
            state.indent_level++;
        }
        void finish()
        {
            assert(!finished);
            state.indent_level--;
            finished = true;
        }
        ~push_indent()
        {
            if(!finished)
                state.indent_level--;
        }
    };
    write_state(write_options options) : options(std::move(options))
    {
    }
    void write_indent(std::ostream &os) const;
};

namespace ast
{
struct null_value final
{
    constexpr null_value() noexcept = default;
    constexpr null_value(std::nullptr_t) noexcept
    {
    }
    void write(std::ostream &os, write_state &state) const;
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
    template <typename T, typename = typename std::enable_if<std::is_same<T, bool>::value>::type>
    constexpr boolean_value(T value) noexcept : value(value)
    {
    }
    void write(std::ostream &os, write_state &state) const;
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
    template <
        typename T,
        typename = typename std::enable_if<!std::is_same<T, std::nullptr_t>::value
                                           && std::is_convertible<T, std::string>::value>::type>
    string_value(T value) noexcept : value(std::move(value))
    {
    }
    static void write(std::ostream &os, const std::string &value, write_state &state);
    void write(std::ostream &os, write_state &state) const
    {
        write(os, value, state);
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
    template <typename T,
              typename = typename std::enable_if<std::is_arithmetic<T>::value
                                                 && !std::is_same<T, bool>::value>::type>
    number_value(T value) noexcept : value(value)
    {
    }
    explicit operator std::string() const
    {
        return to_string();
    }
    static constexpr unsigned max_base = 36;
    static constexpr unsigned min_base = 2;
    static constexpr unsigned default_base = 10; // the json spec only supports base 10
    std::string append_to_string(std::string buffer, unsigned base = default_base) const
    {
        return append_double_to_string(value, std::move(buffer), base);
    }
    std::string to_string(std::string buffer = {}, unsigned base = default_base) const
    {
        return double_to_string(value, std::move(buffer), base);
    }
    std::size_t to_buffer(char *output_buffer,
                          std::size_t output_buffer_size,
                          bool require_null_terminator = true,
                          unsigned base = default_base) const noexcept
    {
        return double_to_buffer(
            value, output_buffer, output_buffer_size, require_null_terminator, base);
    }
    static std::string append_unsigned_integer_to_string(std::uint64_t value,
                                                         std::string buffer,
                                                         unsigned base = default_base);
    static std::string unsigned_integer_to_string(std::uint64_t value,
                                                  std::string buffer = {},
                                                  unsigned base = default_base)
    {
        buffer.clear();
        return append_unsigned_integer_to_string(value, std::move(buffer), base);
    }
    static std::size_t unsigned_integer_to_buffer(std::uint64_t value,
                                                  char *output_buffer,
                                                  std::size_t output_buffer_size,
                                                  bool require_null_terminator = true,
                                                  unsigned base = default_base) noexcept;
    static std::string append_signed_integer_to_string(std::int64_t value,
                                                       std::string buffer,
                                                       unsigned base = default_base);
    static std::string signed_integer_to_string(std::int64_t value,
                                                std::string buffer = {},
                                                unsigned base = default_base)
    {
        buffer.clear();
        return append_signed_integer_to_string(value, std::move(buffer), base);
    }
    static std::size_t signed_integer_to_buffer(std::int64_t value,
                                                char *output_buffer,
                                                std::size_t output_buffer_size,
                                                bool require_null_terminator = true,
                                                unsigned base = default_base) noexcept;
    static std::string append_double_to_string(double value,
                                               std::string buffer,
                                               unsigned base = default_base);
    static std::string double_to_string(double value,
                                        std::string buffer = {},
                                        unsigned base = default_base)
    {
        buffer.clear();
        return append_double_to_string(value, std::move(buffer), base);
    }
    static std::size_t double_to_buffer(double value,
                                        char *output_buffer,
                                        std::size_t output_buffer_size,
                                        bool require_null_terminator = true,
                                        unsigned base = default_base) noexcept;
    void write(std::ostream &os, write_state &state, unsigned base = default_base) const;
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

struct composite_value;

class composite_value_pointer
{
private:
    std::shared_ptr<composite_value> value;

public:
    constexpr composite_value_pointer() noexcept = default;
    template <typename T,
              typename = typename std::enable_if<std::is_base_of<composite_value, T>::value>::type>
    composite_value_pointer(std::shared_ptr<T> value) noexcept : value(std::move(value))
    {
    }
    composite_value *operator->() const noexcept
    {
        return value.operator->();
    }
    composite_value &operator*() const noexcept
    {
        return *value;
    }
};

typedef util::
    variant<null_value, boolean_value, string_value, number_value, composite_value_pointer> value;

struct composite_value
{
    composite_value() = default;
    virtual ~composite_value() = default;
    virtual void write(std::ostream &os, write_state &state) const = 0;
    virtual composite_value_pointer duplicate() const = 0;
    operator value() const
    {
        return duplicate();
    }
};

inline value duplicate(const value &v)
{
    return util::visit(
        [](const auto &v) -> value
        {
            return v->duplicate();
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
    virtual void write(std::ostream &os, write_state &state) const override;
    virtual composite_value_pointer duplicate() const override
    {
        std::unordered_map<std::string, value> new_values;
        for(auto &entry : values)
        {
            new_values.emplace(std::get<0>(entry), ast::duplicate(std::get<1>(entry)));
        }
        return std::make_shared<object>(std::move(new_values));
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
    virtual void write(std::ostream &os, write_state &state) const override;
    virtual composite_value_pointer duplicate() const override
    {
        std::vector<value> new_values;
        new_values.reserve(values.size());
        for(auto &value : values)
            new_values.emplace_back(ast::duplicate(value));
        return std::make_shared<array>(std::move(new_values));
    }
};
}

inline void write(std::ostream &os, const ast::value &v, write_state &state)
{
    util::visit(
        [&](const auto &v) -> void
        {
            return v->write(os, state);
        },
        v);
}

inline void write(std::ostream &os, const ast::value &v, write_options options = {})
{
    write_state state(std::move(options));
    write(os, v, state);
}
}
}

#endif /* JSON_JSON_H_ */
