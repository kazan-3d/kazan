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
#include <cmath>
#include <list>
#include "util/variant.h"
#include "util/optional.h"
#include "location.h"

namespace kazan
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
    friend constexpr bool operator==(const Null_value &, const Null_value &) noexcept
    {
        return true;
    }
    friend constexpr bool operator!=(const Null_value &, const Null_value &) noexcept
    {
        return false;
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
    friend constexpr bool operator==(const Boolean_value &a, const Boolean_value &b) noexcept
    {
        return a.value == b.value;
    }
    friend constexpr bool operator!=(const Boolean_value &a, const Boolean_value &b) noexcept
    {
        return !operator==(a, b);
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
    friend bool operator==(const String_value &a, const String_value &b) noexcept
    {
        return a.value == b.value;
    }
    friend bool operator!=(const String_value &a, const String_value &b) noexcept
    {
        return !operator==(a, b);
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
                                                         unsigned base = default_base,
                                                         std::size_t min_length = 1);
    static std::string unsigned_integer_to_string(std::uint64_t value,
                                                  std::string buffer = {},
                                                  unsigned base = default_base,
                                                  std::size_t min_length = 1)
    {
        buffer.clear();
        return append_unsigned_integer_to_string(value, std::move(buffer), base, min_length);
    }
    static std::size_t unsigned_integer_to_buffer(std::uint64_t value,
                                                  char *output_buffer,
                                                  std::size_t output_buffer_size,
                                                  bool require_null_terminator = true,
                                                  unsigned base = default_base,
                                                  std::size_t min_length = 1) noexcept;
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
    friend bool operator==(const Number_value &a, const Number_value &b) noexcept
    {
        return a.value == b.value || (std::isnan(a.value) && std::isnan(b.value));
    }
    friend bool operator!=(const Number_value &a, const Number_value &b) noexcept
    {
        return !operator==(a, b);
    }
};

struct Composite_value;

class Composite_value_reference
{
private:
    std::shared_ptr<Composite_value> value;

public:
    constexpr Composite_value_reference() noexcept = default;
    template <typename T,
              typename = typename std::enable_if<std::is_base_of<Composite_value, T>::value>::type>
    Composite_value_reference(std::shared_ptr<T> value) noexcept : value(std::move(value))
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
    friend bool operator==(const Composite_value_reference &a,
                           const Composite_value_reference &b) noexcept;
    friend bool operator!=(const Composite_value_reference &a,
                           const Composite_value_reference &b) noexcept
    {
        return !operator==(a, b);
    }
};

struct Object;
struct Array;

struct Value
{
    Location location;
    util::variant<Null_value, Boolean_value, String_value, Number_value, Composite_value_reference>
        value;
    constexpr Value()
    {
    }
    constexpr explicit Value(Location location, Null_value value)
        : location(std::move(location)), value(std::move(value))
    {
    }
    constexpr explicit Value(Location location, Boolean_value value)
        : location(std::move(location)), value(std::move(value))
    {
    }
    explicit Value(Location location, String_value value)
        : location(std::move(location)), value(std::move(value))
    {
    }
    explicit Value(Location location, Number_value value)
        : location(std::move(location)), value(std::move(value))
    {
    }
    explicit Value(Location location, Composite_value_reference value)
        : location(std::move(location)), value(std::move(value))
    {
    }
    explicit Value(Location location, Array &&value);
    explicit Value(Location location, Object &&value);
    Value duplicate() const;
    Null_value &get_null()
    {
        return util::get<Null_value>(value);
    }
    const Null_value &get_null() const
    {
        return util::get<Null_value>(value);
    }
    Boolean_value &get_boolean()
    {
        return util::get<Boolean_value>(value);
    }
    const Boolean_value &get_boolean() const
    {
        return util::get<Boolean_value>(value);
    }
    String_value &get_string()
    {
        return util::get<String_value>(value);
    }
    const String_value &get_string() const
    {
        return util::get<String_value>(value);
    }
    Number_value &get_number()
    {
        return util::get<Number_value>(value);
    }
    const Number_value &get_number() const
    {
        return util::get<Number_value>(value);
    }
    Object &get_object();
    const Object &get_object() const;
    Array &get_array();
    const Array &get_array() const;
    Value_kind get_value_kind() const noexcept;
    friend bool operator==(const Value &a, const Value &b) noexcept
    {
        return a.value == b.value;
    }
    friend bool operator!=(const Value &a, const Value &b) noexcept
    {
        return !operator==(a, b);
    }
};

struct Composite_value
{
    Composite_value() = default;
    virtual ~Composite_value() = default;
    virtual void write(std::ostream &os, Write_state &state) const = 0;
    virtual Composite_value_reference duplicate() const = 0;
    virtual Value_kind get_value_kind() const noexcept = 0;
    virtual bool operator==(const Composite_value &rt) const noexcept = 0;
    bool operator!=(const Composite_value &rt) const noexcept
    {
        return !operator==(rt);
    }
};

/** returns true if a and b are structurally equal */
inline bool operator==(const Composite_value_reference &a,
                       const Composite_value_reference &b) noexcept
{
    if(a.value == b.value)
        return true;
    if(!a.value || !b.value)
        return false;
    return *a.value == *b.value;
}


inline Value Value::duplicate() const
{
    return util::visit(
        [this](const auto &v) -> Value
        {
            return Value(location, v->duplicate());
        },
        value);
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
    virtual Composite_value_reference duplicate() const override
    {
        std::unordered_map<std::string, Value> new_values;
        for(auto &entry : values)
        {
            new_values.emplace(std::get<0>(entry), std::get<1>(entry).duplicate());
        }
        return std::make_shared<Object>(std::move(new_values));
    }
    Value_kind get_value_kind() const noexcept override
    {
        return Value_kind::object;
    }
    bool operator==(const Object &rt) const noexcept
    {
        return values == rt.values;
    }
    virtual bool operator==(const Composite_value &rt) const noexcept override
    {
        if(dynamic_cast<const Object *>(&rt))
            return operator==(static_cast<const Object &>(rt));
        return false;
    }
};

inline Object &Value::get_object()
{
    return dynamic_cast<Object &>(*util::get<Composite_value_reference>(value));
}

inline const Object &Value::get_object() const
{
    return dynamic_cast<const Object &>(*util::get<Composite_value_reference>(value));
}

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
    virtual Composite_value_reference duplicate() const override
    {
        std::vector<Value> new_values;
        new_values.reserve(values.size());
        for(auto &value : values)
            new_values.emplace_back(value.duplicate());
        return std::make_shared<Array>(std::move(new_values));
    }
    Value_kind get_value_kind() const noexcept override
    {
        return Value_kind::array;
    }
    bool operator==(const Array &rt) const noexcept
    {
        return values == rt.values;
    }
    virtual bool operator==(const Composite_value &rt) const noexcept override
    {
        if(dynamic_cast<const Array *>(&rt))
            return operator==(static_cast<const Array &>(rt));
        return false;
    }
};

inline Array &Value::get_array()
{
    return dynamic_cast<Array &>(*util::get<Composite_value_reference>(value));
}

inline const Array &Value::get_array() const
{
    return dynamic_cast<const Array &>(*util::get<Composite_value_reference>(value));
}

inline Value_kind Value::get_value_kind() const noexcept
{
    return util::visit(
        [](const auto &v) -> Value_kind
        {
            return v->get_value_kind();
        },
        value);
}

inline Value::Value(Location location, Array &&value)
    : location(std::move(location)), value(std::make_shared<Array>(std::move(value)))
{
}

inline Value::Value(Location location, Object &&value)
    : location(std::move(location)), value(std::make_shared<Object>(std::move(value)))
{
}
}

inline void write(std::ostream &os, const ast::Value &v, Write_state &state)
{
    util::visit(
        [&](const auto &v) -> void
        {
            v->write(os, state);
        },
        v.value);
}

inline void write(std::ostream &os, const ast::Value &v, Write_options options = {})
{
    Write_state state(std::move(options));
    write(os, v, state);
}

struct Difference
{
    std::list<util::variant<std::size_t, std::string>> element_selectors;
    Difference() noexcept = default;
    explicit Difference(
        std::list<util::variant<std::size_t, std::string>> element_selectors) noexcept
        : element_selectors(std::move(element_selectors))
    {
    }
    std::string append_to_string(std::string buffer) const
    {
        for(auto &element_selector : element_selectors)
        {
            if(util::holds_alternative<std::size_t>(element_selector))
            {
                buffer = ast::Number_value::append_unsigned_integer_to_string(
                             util::get<std::size_t>(element_selector), std::move(buffer) + '[')
                         + ']';
            }
            else
            {
                buffer += "[\"";
                buffer += util::get<std::string>(element_selector);
                buffer += "\"]";
            }
        }
        return buffer;
    }
    std::string to_string() const
    {
        return append_to_string({});
    }
    static util::optional<Difference> find_difference(const ast::Value &a, const ast::Value &b);
};
}
}

#endif /* JSON_JSON_H_ */
