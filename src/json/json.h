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
struct Write_options
{
    bool composite_value_elements_on_seperate_lines = false;
    bool sort_object_values = false;
    std::string indent_text = "";
    Write_options()
    {
    }
    Write_options(bool composite_value_elements_on_seperate_lines,
                  bool sort_object_values,
                  std::string indent_text) noexcept
        : composite_value_elements_on_seperate_lines(composite_value_elements_on_seperate_lines),
          sort_object_values(sort_object_values),
          indent_text(std::move(indent_text))
    {
    }
    static Write_options defaults()
    {
        return {};
    }
    static Write_options pretty(std::string indent_text = "    ")
    {
        return Write_options(true, true, std::move(indent_text));
    }
};

struct Write_state
{
    Write_options options;
    std::size_t indent_level = 0;
    class Push_indent final
    {
        Push_indent(const Push_indent &) = delete;
        Push_indent &operator=(const Push_indent &) = delete;

    private:
        Write_state &state;
        bool finished = false;

    public:
        Push_indent(Write_state &state) : state(state)
        {
            state.indent_level++;
        }
        void finish()
        {
            assert(!finished);
            state.indent_level--;
            finished = true;
        }
        ~Push_indent()
        {
            if(!finished)
                state.indent_level--;
        }
    };
    Write_state(Write_options options) : options(std::move(options))
    {
    }
    void write_indent(std::ostream &os) const;
};

namespace ast
{
enum class Value_kind
{
    null,
    boolean,
    string,
    number,
    object,
    array
};

struct Null_value final
{
    constexpr Null_value() noexcept = default;
    constexpr Null_value(std::nullptr_t) noexcept
    {
    }
    void write(std::ostream &os, Write_state &state) const;
    Null_value duplicate() const noexcept
    {
        return {};
    }
    const Null_value *operator->() const noexcept
    {
        return this;
    }
    const Null_value &operator*() const noexcept
    {
        return *this;
    }
    constexpr Value_kind get_value_kind() const noexcept
    {
        return Value_kind::null;
    }
};

struct Boolean_value final
{
    bool value;
    template <typename T, typename = typename std::enable_if<std::is_same<T, bool>::value>::type>
    constexpr Boolean_value(T value) noexcept : value(value)
    {
    }
    void write(std::ostream &os, Write_state &state) const;
    Boolean_value duplicate() const noexcept
    {
        return *this;
    }
    const Boolean_value *operator->() const noexcept
    {
        return this;
    }
    const Boolean_value &operator*() const noexcept
    {
        return *this;
    }
    constexpr Value_kind get_value_kind() const noexcept
    {
        return Value_kind::boolean;
    }
};

struct String_value final
{
    std::string value;
    template <
        typename T,
        typename = typename std::enable_if<!std::is_same<T, std::nullptr_t>::value
                                           && std::is_convertible<T, std::string>::value>::type>
    String_value(T value) noexcept : value(std::move(value))
    {
    }
    static void write(std::ostream &os, const std::string &value, Write_state &state);
    static void write(std::ostream &os, const std::string &value)
    {
        Write_state state(Write_options::defaults());
        write(os, value, state);
    }
    void write(std::ostream &os, Write_state &state) const
    {
        write(os, value, state);
    }
    String_value duplicate() const noexcept
    {
        return *this;
    }
    const String_value *operator->() const noexcept
    {
        return this;
    }
    const String_value &operator*() const noexcept
    {
        return *this;
    }
    constexpr Value_kind get_value_kind() const noexcept
    {
        return Value_kind::string;
    }
};

struct Number_value final
{
    double value;
    static_assert(std::numeric_limits<double>::is_iec559 && std::numeric_limits<double>::radix == 2,
                  "double is not a ieee754 float64");
    template <typename T,
              typename = typename std::enable_if<std::is_arithmetic<T>::value
                                                 && !std::is_same<T, bool>::value>::type>
    Number_value(T value) noexcept : value(value)
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
    void write(std::ostream &os, Write_state &state, unsigned base = default_base) const;
    Number_value duplicate() const noexcept
    {
        return *this;
    }
    const Number_value *operator->() const noexcept
    {
        return this;
    }
    const Number_value &operator*() const noexcept
    {
        return *this;
    }
    constexpr Value_kind get_value_kind() const noexcept
    {
        return Value_kind::number;
    }
};

struct Composite_value;

class Composite_value_pointer
{
private:
    std::shared_ptr<Composite_value> value;

public:
    constexpr Composite_value_pointer() noexcept = default;
    template <typename T,
              typename = typename std::enable_if<std::is_base_of<Composite_value, T>::value>::type>
    Composite_value_pointer(std::shared_ptr<T> value) noexcept : value(std::move(value))
    {
    }
    Composite_value *operator->() const noexcept
    {
        return value.operator->();
    }
    Composite_value &operator*() const noexcept
    {
        return *value;
    }
    const std::shared_ptr<Composite_value> &get() const &noexcept
    {
        return value;
    }
    std::shared_ptr<Composite_value> get() && noexcept
    {
        std::shared_ptr<Composite_value> retval = nullptr;
        retval.swap(value);
        return retval;
    }
};

typedef util::
    variant<Null_value, Boolean_value, String_value, Number_value, Composite_value_pointer> Value;

struct Composite_value
{
    Composite_value() = default;
    virtual ~Composite_value() = default;
    virtual void write(std::ostream &os, Write_state &state) const = 0;
    virtual Composite_value_pointer duplicate() const = 0;
    operator Value() const
    {
        return duplicate();
    }
    virtual Value_kind get_value_kind() const noexcept = 0;
};

inline Value duplicate(const Value &v)
{
    return util::visit(
        [](const auto &v) -> Value
        {
            return v->duplicate();
        },
        v);
}

struct Object final : public Composite_value
{
    std::unordered_map<std::string, Value> values;
    Object() : values()
    {
    }
    Object(std::unordered_map<std::string, Value> values) noexcept : values(std::move(values))
    {
    }
    virtual void write(std::ostream &os, Write_state &state) const override;
    virtual Composite_value_pointer duplicate() const override
    {
        std::unordered_map<std::string, Value> new_values;
        for(auto &entry : values)
        {
            new_values.emplace(std::get<0>(entry), ast::duplicate(std::get<1>(entry)));
        }
        return std::make_shared<Object>(std::move(new_values));
    }
    Value_kind get_value_kind() const noexcept override
    {
        return Value_kind::object;
    }
};

struct Array final : public Composite_value
{
    std::vector<Value> values;
    Array() : values()
    {
    }
    Array(std::vector<Value> values) noexcept : values(std::move(values))
    {
    }
    virtual void write(std::ostream &os, Write_state &state) const override;
    virtual Composite_value_pointer duplicate() const override
    {
        std::vector<Value> new_values;
        new_values.reserve(values.size());
        for(auto &value : values)
            new_values.emplace_back(ast::duplicate(value));
        return std::make_shared<Array>(std::move(new_values));
    }
    Value_kind get_value_kind() const noexcept override
    {
        return Value_kind::array;
    }
};

inline Value_kind get_value_kind(const Value &v) noexcept
{
    return util::visit(
        [&](const auto &v) -> Value_kind
        {
            return v->get_value_kind();
        },
        v);
}
}

inline void write(std::ostream &os, const ast::Value &v, Write_state &state)
{
    util::visit(
        [&](const auto &v) -> void
        {
            v->write(os, state);
        },
        v);
}

inline void write(std::ostream &os, const ast::Value &v, Write_options options = {})
{
    Write_state state(std::move(options));
    write(os, v, state);
}
}
}

#endif /* JSON_JSON_H_ */
