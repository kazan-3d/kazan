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
#include "json.h"
#include <istream>
#include <ostream>
#include <cmath>
#include <cstdint>
#include <cassert>
#include <tuple>
#include <algorithm>
#include "../util/soft_float.h"

namespace vulkan_cpu
{
namespace json
{
void Write_state::write_indent(std::ostream &os) const
{
    for(std::size_t i = indent_level; i > 0; i--)
        os << options.indent_text;
}

namespace ast
{
namespace soft_float = util::soft_float;

void Null_value::write(std::ostream &os, Write_state &state) const
{
    os << "null";
}

void Boolean_value::write(std::ostream &os, Write_state &state) const
{
    os << (value ? "true" : "false");
}

namespace
{
constexpr char get_digit_char(unsigned digit, bool uppercase) noexcept
{
    if(digit < 10)
        return '0' + digit;
    if(uppercase)
        return digit - 10 + 'A';
    return digit - 10 + 'a';
}
}

void String_value::write(std::ostream &os, const std::string &value, Write_state &state)
{
    os << '\"';
    for(unsigned char ch : value)
    {
        switch(ch)
        {
        case '\\':
        case '\"':
            os << '\\' << ch;
            break;
        case '\b':
            os << "\\b";
            break;
        case '\f':
            os << "\\f";
            break;
        case '\n':
            os << "\\n";
            break;
        case '\r':
            os << "\\r";
            break;
        case '\t':
            os << "\\t";
            break;
        default:
            if(ch < 0x20U)
                os << "\\u00" << get_digit_char(ch >> 4, true) << get_digit_char(ch & 0xFU, true);
            else
                os << ch;
        }
    }
    os << '\"';
}

namespace
{
template <typename Write_Char>
void write_string(Write_Char write_char, const char *str) noexcept(noexcept(write_char('0')))
{
    while(*str)
        write_char(*str++);
}

template <typename Write_Char>
void write_array(Write_Char write_char,
                 const char *array,
                 std::size_t size) noexcept(noexcept(write_char('0')))
{
    for(std::size_t i = 0; i < size; i++)
        write_char(array[i]);
}

constexpr std::array<soft_float::ExtendedFloat, 37> make_base_2_logs() noexcept
{
    return std::array<soft_float::ExtendedFloat, 37>{{
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(0))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(1))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(2))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(3))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(4))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(5))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(6))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(7))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(8))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(9))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(10))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(11))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(12))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(13))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(14))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(15))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(16))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(17))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(18))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(19))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(20))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(21))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(22))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(23))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(24))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(25))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(26))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(27))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(28))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(29))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(30))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(31))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(32))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(33))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(34))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(35))),
        log2(soft_float::ExtendedFloat(static_cast<std::uint64_t>(36))),
    }};
}

constexpr std::size_t max_integer_buffer_size = 64; // max number of digits is base 2 with 64 digits

template <typename Write_Char>
void write_unsigned_integer(Write_Char write_char,
                            std::uint64_t value,
                            unsigned base) noexcept(noexcept(write_char('0')))
{
    assert(base >= Number_value::min_base && base <= Number_value::max_base);
    char buffer[max_integer_buffer_size]{};
    std::size_t buffer_used = 0;
    do
    {
        assert(buffer_used < max_integer_buffer_size);
        buffer[buffer_used++] = get_digit_char(value % base, false);
        value /= base;
    } while(value != 0);
    for(std::size_t i = 0, j = buffer_used - 1; i < buffer_used; i++, j--)
        write_char(buffer[j]);
}

template <typename Write_Char>
void write_signed_integer(Write_Char write_char,
                          std::int64_t value,
                          unsigned base) noexcept(noexcept(write_char('0')))
{
    if(value < 0)
    {
        write_char('-');
        write_unsigned_integer(write_char,
                               -static_cast<std::uint64_t>(value),
                               base); // cast to unsigned first to handle minimum value
    }
    else
    {
        write_unsigned_integer(write_char, static_cast<std::uint64_t>(value), base);
    }
}

template <typename Write_Char>
void write_number(Write_Char write_char,
                  double valueIn,
                  unsigned base) noexcept(noexcept(write_char('0')))
{
    // code modified from
    // https://github.com/programmerjake/javascript-tasklets/blob/master/javascript_tasklets/value.cpp
    // based on the ECMAScript ToString algorithm for numbers
    assert(base >= Number_value::min_base && base <= Number_value::max_base);
    const char exponent_char = base == 10 ? 'e' : base == 16 ? 'h' : base == 8 ? 'o' : 'E';
    soft_float::ExtendedFloat value(valueIn), base_f(static_cast<std::uint64_t>(base));
    auto inv_base_f = soft_float::ExtendedFloat::One() / base_f;
    static constexpr auto base_2_logs = make_base_2_logs();
    auto limit_21 =
        static_cast<std::int64_t>(round(soft_float::ExtendedFloat(static_cast<std::uint64_t>(21))
                                        * (base_2_logs[10] / base_2_logs[base])));
    assert(limit_21 > 0);
    auto limit_6 =
        static_cast<std::int64_t>(round(soft_float::ExtendedFloat(static_cast<std::uint64_t>(6))
                                        * (base_2_logs[10] / base_2_logs[base])));
    assert(limit_6 > 0);
    if(value.isNaN())
    {
        write_string(write_char, "NaN");
        return;
    }
    if(value.isZero())
    {
        write_char('0');
        return;
    }
    if(value.isInfinite())
    {
        if(value.signBit())
            write_string(write_char, "-Infinity");
        else
            write_string(write_char, "Infinity");
        return;
    }
    if(value.signBit())
    {
        write_char('-');
        value = -value;
        valueIn = -valueIn;
    }
    auto n_f = log2(value) / base_2_logs[base] + soft_float::ExtendedFloat::One();
    auto n = static_cast<std::int64_t>(floor(n_f));
    soft_float::ExtendedFloat base_to_the_power_of_n = pow(base_f, n);
    soft_float::ExtendedFloat base_to_the_power_of_minus_n =
        soft_float::ExtendedFloat::One() / base_to_the_power_of_n;
    auto scaled_value = value * base_to_the_power_of_minus_n;
    if(scaled_value + scalbn(soft_float::ExtendedFloat::One(), -62)
       < inv_base_f) // extra is to handle round-off error
    {
        n--;
        base_to_the_power_of_n *= inv_base_f;
        base_to_the_power_of_minus_n *= base_f;
        scaled_value = value * base_to_the_power_of_minus_n;
    }
    else if(scaled_value >= soft_float::ExtendedFloat::One())
    {
        n++;
        base_to_the_power_of_n *= base_f;
        base_to_the_power_of_minus_n *= inv_base_f;
        scaled_value = value * base_to_the_power_of_minus_n;
    }
    std::int64_t k = 0;
    soft_float::ExtendedFloat s_f = soft_float::ExtendedFloat::One();
    auto base_to_the_power_of_k = soft_float::ExtendedFloat::One();
    auto base_to_the_power_of_minus_k = soft_float::ExtendedFloat::One();
    while(s_f < soft_float::ExtendedFloat::TwoToThe64())
    {
        k++;
        base_to_the_power_of_k *= base_f;
        base_to_the_power_of_minus_k *= inv_base_f;
        s_f = round(scaled_value * base_to_the_power_of_k);
        if(valueIn
           == static_cast<double>(s_f * base_to_the_power_of_minus_k * base_to_the_power_of_n))
            break;
    }
    std::uint64_t s = static_cast<std::uint64_t>(s_f);
    char s_digits[max_integer_buffer_size]{};
    std::size_t s_digits_size = 0;
    write_unsigned_integer(
        [&](char ch)
        {
            assert(s_digits_size < max_integer_buffer_size);
            s_digits[s_digits_size++] = ch;
        },
        s,
        base);
    assert(s_digits_size == static_cast<std::uint64_t>(k));
    if(k <= n && n <= limit_21)
    {
        write_array(write_char, s_digits, s_digits_size);
        for(std::size_t i = n - k; i > 0; i--)
            write_char('0');
    }
    else if(0 < n && n <= limit_21)
    {
        for(std::int64_t i = 0; i < n; i++)
            write_char(s_digits[i]);
        write_char('.');
        for(std::int64_t i = n; i < k; i++)
            write_char(s_digits[i]);
    }
    else if(-limit_6 < n && n <= 0)
    {
        write_string(write_char, "0.");
        for(std::size_t i = -n; i > 0; i--)
            write_char('0');
        write_array(write_char, s_digits, s_digits_size);
    }
    else if(k == 1)
    {
        write_array(write_char, s_digits, s_digits_size);
        write_char(exponent_char);
        if(n - 1 >= 0)
        {
            write_char('+');
            write_signed_integer(write_char, n - 1, 10);
        }
        else
            write_signed_integer(write_char, n - 1, 10);
    }
    else
    {
        write_char(s_digits[0]);
        write_char('.');
        for(std::int64_t i = 1; i < k; i++)
            write_char(s_digits[i]);
        write_char(exponent_char);
        if(n - 1 >= 0)
        {
            write_char('+');
            write_signed_integer(write_char, n - 1, 10);
        }
        else
            write_signed_integer(write_char, n - 1, 10);
    }
}
}

std::string Number_value::append_double_to_string(double value, std::string buffer, unsigned base)
{
    write_number(
        [&](char ch)
        {
            buffer += ch;
        },
        value,
        base);
    return buffer;
}

std::size_t Number_value::double_to_buffer(double value,
                                           char *output_buffer,
                                           std::size_t output_buffer_size,
                                           bool require_null_terminator,
                                           unsigned base) noexcept
{
    if(output_buffer_size == 0)
        return 0;
    std::size_t used_buffer_size = 0;
    std::size_t output_buffer_size_without_terminator = output_buffer_size;
    if(require_null_terminator)
        output_buffer_size_without_terminator--;
    write_number(
        [&](char ch)
        {
            if(used_buffer_size < output_buffer_size_without_terminator)
                output_buffer[used_buffer_size++] = ch;
        },
        value,
        base);
    if(used_buffer_size < output_buffer_size)
        output_buffer[used_buffer_size] = '\0'; // add the null terminator if there is space
    return used_buffer_size; // report used buffer excluding the null terminator
}

std::string Number_value::append_unsigned_integer_to_string(std::uint64_t value,
                                                            std::string buffer,
                                                            unsigned base)
{
    write_unsigned_integer(
        [&](char ch)
        {
            buffer += ch;
        },
        value,
        base);
    return buffer;
}

std::size_t Number_value::unsigned_integer_to_buffer(std::uint64_t value,
                                                     char *output_buffer,
                                                     std::size_t output_buffer_size,
                                                     bool require_null_terminator,
                                                     unsigned base) noexcept
{
    if(output_buffer_size == 0)
        return 0;
    std::size_t used_buffer_size = 0;
    std::size_t output_buffer_size_without_terminator = output_buffer_size;
    if(require_null_terminator)
        output_buffer_size_without_terminator--;
    write_unsigned_integer(
        [&](char ch)
        {
            if(used_buffer_size < output_buffer_size_without_terminator)
                output_buffer[used_buffer_size++] = ch;
        },
        value,
        base);
    if(used_buffer_size < output_buffer_size)
        output_buffer[used_buffer_size] = '\0'; // add the null terminator if there is space
    return used_buffer_size; // report used buffer excluding the null terminator
}

std::string Number_value::append_signed_integer_to_string(std::int64_t value,
                                                          std::string buffer,
                                                          unsigned base)
{
    write_unsigned_integer(
        [&](char ch)
        {
            buffer += ch;
        },
        value,
        base);
    return buffer;
}

std::size_t Number_value::signed_integer_to_buffer(std::int64_t value,
                                                   char *output_buffer,
                                                   std::size_t output_buffer_size,
                                                   bool require_null_terminator,
                                                   unsigned base) noexcept
{
    if(output_buffer_size == 0)
        return 0;
    std::size_t used_buffer_size = 0;
    std::size_t output_buffer_size_without_terminator = output_buffer_size;
    if(require_null_terminator)
        output_buffer_size_without_terminator--;
    write_unsigned_integer(
        [&](char ch)
        {
            if(used_buffer_size < output_buffer_size_without_terminator)
                output_buffer[used_buffer_size++] = ch;
        },
        value,
        base);
    if(used_buffer_size < output_buffer_size)
        output_buffer[used_buffer_size] = '\0'; // add the null terminator if there is space
    return used_buffer_size; // report used buffer excluding the null terminator
}

void Number_value::write(std::ostream &os, Write_state &state, unsigned base) const
{
    write_number(
        [&](char ch)
        {
            os << ch;
        },
        value,
        base);
}

void Object::write(std::ostream &os, Write_state &state) const
{
    os << '{';
    if(!values.empty())
    {
        Write_state::Push_indent push_indent(state);
        auto seperator = "";
        auto write_entry = [&](const std::pair<std::string, Value> &entry)
        {
            const std::string &key = std::get<0>(entry);
            const Value &value = std::get<1>(entry);
            os << seperator;
            seperator = ",";
            if(state.options.composite_value_elements_on_seperate_lines)
            {
                os << '\n';
                state.write_indent(os);
            }
            String_value::write(os, key, state);
            os << ':';
            json::write(os, value, state);
        };
        if(state.options.sort_object_values)
        {
            auto compare_fn = [](decltype(values)::const_iterator a,
                                 decltype(values)::const_iterator b) -> bool
            {
                return std::get<0>(*a) < std::get<0>(*b);
            };
            std::vector<decltype(values)::const_iterator> entry_iterators;
            entry_iterators.reserve(values.size());
            for(auto i = values.begin(); i != values.end(); ++i)
                entry_iterators.push_back(i);
            std::sort(entry_iterators.begin(), entry_iterators.end(), compare_fn);
            for(auto &i : entry_iterators)
                write_entry(*i);
        }
        else
        {
            for(auto &entry : values)
                write_entry(entry);
        }
        push_indent.finish();
        if(state.options.composite_value_elements_on_seperate_lines)
        {
            os << '\n';
            state.write_indent(os);
        }
    }
    os << '}';
}

void Array::write(std::ostream &os, Write_state &state) const
{
    os << '[';
    if(!values.empty())
    {
        Write_state::Push_indent push_indent(state);
        auto seperator = "";
        for(const Value &v : values)
        {
            os << seperator;
            seperator = ",";
            if(state.options.composite_value_elements_on_seperate_lines)
            {
                os << '\n';
                state.write_indent(os);
            }
            json::write(os, v, state);
        }
        push_indent.finish();
        if(state.options.composite_value_elements_on_seperate_lines)
        {
            os << '\n';
            state.write_indent(os);
        }
    }
    os << ']';
}
}
}
}
