/*
 * Copyright 2012-2017 Jacob Lifshay
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
/* translated from
 * https://github.com/programmerjake/hashlife-voxels/blob/5dda3bc240e1e89f43606316d1c3202221e3b06b/util/text.h
 */

#ifndef UTIL_TEXT_H_
#define UTIL_TEXT_H_

#include <type_traits>
#include <utility>
#include <limits>
#include <cstdint>
#include <string>
#include <ostream>
#include <cassert>
#include "string_view.h"

namespace kazan
{
namespace util
{
namespace text
{
constexpr char32_t replacement_character = U'\uFFFD';

template <typename Input_iterator, typename Sentinel>
typename std::char_traits<char32_t>::int_type decode_utf8(
    Input_iterator &iter,
    Sentinel sentinel,
    bool allow_surrogate_code_points = true,
    bool allow_2_byte_null = false,
    typename std::char_traits<char32_t>::int_type error_value =
        replacement_character) noexcept(noexcept(++iter) && noexcept(static_cast<char>(*iter))
                                        && noexcept(iter == sentinel ? 0 : 0))
{
    if(iter == sentinel)
        return std::char_traits<char32_t>::eof();
    auto byte0 = static_cast<std::uint8_t>(static_cast<char>(*iter));
    ++iter;
    if(byte0 < 0x80)
        return byte0;
    if(allow_2_byte_null && byte0 == 0xC0)
    {
        if(iter == sentinel)
            return error_value;
        auto byte1 = static_cast<std::uint8_t>(static_cast<char>(*iter));
        ++iter;
        if(byte1 != 0x80)
            return error_value;
        return 0;
    }
    if(byte0 > 0xF4 || byte0 < 0xC2)
        return error_value;
    if(iter == sentinel)
        return error_value;
    auto byte1 = static_cast<std::uint8_t>(static_cast<char>(*iter));
    if(byte1 < 0x80 || byte1 >= 0xC0)
        return error_value;
    if(byte0 < 0xE0)
    {
        ++iter;
        return (static_cast<std::uint_fast32_t>(byte0 & 0x1F) << 6) | (byte1 & 0x3F);
    }
    if(byte0 == 0xE0 && byte1 < 0xA0)
        return error_value;
    if(byte0 == 0xF0 && byte1 < 0x90)
        return error_value;
    if(byte0 == 0xF4 && byte1 >= 0x90)
        return error_value;
    if(!allow_surrogate_code_points && byte0 == 0xED && byte1 >= 0xA0)
        return error_value;
    if(iter == sentinel)
        return error_value;
    ++iter;
    auto byte2 = static_cast<std::uint8_t>(static_cast<char>(*iter));
    ++iter;
    if(byte2 < 0x80 || byte2 >= 0xC0)
        return error_value;
    if(byte0 < 0xF0)
        return (static_cast<std::uint_fast32_t>(byte0 & 0xF) << 12)
               | (static_cast<std::uint_fast32_t>(byte1 & 0x3F) << 6) | (byte2 & 0x3F);
    if(iter == sentinel)
        return error_value;
    auto byte3 = static_cast<std::uint8_t>(static_cast<char>(*iter));
    ++iter;
    if(byte3 < 0x80 || byte3 >= 0xC0)
        return error_value;
    return (static_cast<std::uint_fast32_t>(byte0 & 0x7) << 18)
           | (static_cast<std::uint_fast32_t>(byte1 & 0x3F) << 12)
           | (static_cast<std::uint_fast32_t>(byte2 & 0x3F) << 6) | (byte3 & 0x3F);
}

template <typename T, std::size_t N>
struct Encoded_character final
{
    static constexpr std::size_t max_Chars = N;
    typedef T Char_type;
    static_assert(max_Chars != 0, "");
    Char_type chars[max_Chars];
    std::size_t used;
    constexpr Char_type &front()
    {
        return chars[0];
    }
    constexpr const Char_type &front() const
    {
        return chars[0];
    }
    constexpr Char_type &back()
    {
        return chars[0];
    }
    constexpr const Char_type &back() const
    {
        return chars[0];
    }
    typedef const Char_type *const_iterator;
    typedef Char_type *iterator;
    constexpr const_iterator begin() const
    {
        return &chars[0];
    }
    constexpr const_iterator end() const
    {
        return begin() + used;
    }
    constexpr const_iterator cbegin() const
    {
        return &chars[0];
    }
    constexpr const_iterator cend() const
    {
        return begin() + used;
    }
    constexpr iterator begin()
    {
        return &chars[0];
    }
    constexpr iterator end()
    {
        return begin() + used;
    }
    constexpr std::size_t capacity() const
    {
        return max_Chars;
    }
    constexpr std::size_t size() const
    {
        return used;
    }
    constexpr const Char_type &operator[](std::size_t index) const
    {
        assert(index < used);
        return chars[index];
    }
    constexpr Char_type &operator[](std::size_t index)
    {
        assert(index < used);
        return chars[index];
    }
    constexpr Encoded_character() : chars(), used(0)
    {
    }

private:
    static constexpr Char_type implicit_conversion_helper(Char_type ch) noexcept
    {
        return ch;
    }

public:
    template <typename... Args>
    constexpr Encoded_character(Args &&... args)
        : chars{implicit_conversion_helper(std::forward<Args>(args))...}, used(sizeof...(args))
    {
        static_assert(sizeof...(args) <= max_Chars, "");
    }
    template <typename Char_traits, typename Allocator>
    operator std::basic_string<Char_type, Char_traits, Allocator>() const
    {
        return std::basic_string<Char_type, Char_traits, Allocator>(begin(), end());
    }
    template <typename Char_traits>
    constexpr operator basic_string_view<Char_type, Char_traits>() const noexcept
    {
        return basic_string_view<Char_type, Char_traits>(chars, used);
    }
    template <typename Char_traits, typename Allocator>
    friend std::basic_string<Char_type, Char_traits, Allocator> operator+(
        std::basic_string<Char_type, Char_traits, Allocator> a, const Encoded_character &b)
    {
        a.append(b.begin(), b.end());
        return a;
    }
    template <typename Char_traits, typename Allocator>
    friend std::basic_string<Char_type, Char_traits, Allocator> operator+(
        const Encoded_character &a, std::basic_string<Char_type, Char_traits, Allocator> b)
    {
        b.insert(b.begin(), a.begin(), a.end());
        return b;
    }
    template <std::size_t N2>
    friend std::basic_string<Char_type> operator+(const Encoded_character &a,
                                                  const Encoded_character<Char_type, N2> &b)
    {
        std::basic_string<Char_type> retval;
        retval.reserve(a.size() + b.size());
        retval.append(a.begin(), a.end());
        retval.append(b.begin(), b.end());
        return retval;
    }
    template <typename Traits>
    friend std::basic_ostream<Char_type, Traits> &operator<<(
        std::basic_ostream<Char_type, Traits> &os, const Encoded_character &a)
    {
        os << static_cast<std::basic_string<Char_type, Traits>>(a);
        return os;
    }
};

constexpr Encoded_character<char, 4> encode_utf8(char32_t ch, bool use_2_byte_null = false) noexcept
{
    assert(ch < 0x10FFFFUL && ch >= 0);
    if(use_2_byte_null && ch == 0)
        return Encoded_character<char, 4>(0xC0U, 0x80U);
    if(ch < 0x80)
        return Encoded_character<char, 4>(ch);
    if(ch < 0x800)
        return Encoded_character<char, 4>(0xC0 | (ch >> 6), 0x80 | (ch & 0x3F));
    if(ch < 0x10000UL)
        return Encoded_character<char, 4>(
            0xE0 | (ch >> 12), 0x80 | ((ch >> 6) & 0x3F), 0x80 | (ch & 0x3F));
    return Encoded_character<char, 4>(0xF0 | (ch >> 18),
                                      0x80 | ((ch >> 12) & 0x3F),
                                      0x80 | ((ch >> 6) & 0x3F),
                                      0x80 | (ch & 0x3F));
}

template <typename Input_iterator, typename Sentinel>
typename std::char_traits<char32_t>::int_type decode_utf16(
    Input_iterator &iter,
    Sentinel sentinel,
    bool allow_unpaired_surrogate_code_units = true,
    typename std::char_traits<char32_t>::int_type error_value =
        replacement_character) noexcept(noexcept(++iter) && noexcept(static_cast<char16_t>(*iter))
                                        && noexcept(iter == sentinel ? 0 : 0))
{
    if(iter == sentinel)
        return std::char_traits<char32_t>::eof();
    auto unit0 = static_cast<std::uint16_t>(static_cast<char16_t>(*iter));
    ++iter;
    if(unit0 >= 0xD800U && unit0 < 0xDC00U)
    {
        if(iter == sentinel)
            return allow_unpaired_surrogate_code_units ? unit0 : error_value;
        auto unit1 = static_cast<std::uint16_t>(static_cast<char16_t>(*iter));
        if(unit1 < 0xDC00U || unit1 >= 0xE000U)
            return allow_unpaired_surrogate_code_units ? unit0 : error_value;
        ++iter;
        return 0x10000UL + ((unit0 & 0x3FF) << 10) + (unit1 & 0x3FF);
    }
    return unit0;
}

constexpr Encoded_character<char16_t, 2> encode_utf16(char32_t ch) noexcept
{
    assert(ch < 0x10FFFFUL && ch >= 0);
    if(ch < 0x10000UL)
        return Encoded_character<char16_t, 2>(ch);
    return Encoded_character<char16_t, 2>(0xD800U | ((ch - 0x10000UL) >> 10),
                                          0xDC00U | ((ch - 0x10000UL) & 0x3FF));
}

template <typename Input_iterator, typename Sentinel>
typename std::char_traits<char32_t>::int_type decode_utf32(
    Input_iterator &iter,
    Sentinel sentinel,
    bool allow_Surrogate_Code_Units = true,
    typename std::char_traits<char32_t>::int_type error_value =
        replacement_character) noexcept(noexcept(++iter) && noexcept(static_cast<char32_t>(*iter))
                                        && noexcept(iter == sentinel ? 0 : 0))
{
    if(iter == sentinel)
        return std::char_traits<char32_t>::eof();
    auto retval = static_cast<std::uint32_t>(static_cast<char32_t>(*iter));
    ++iter;
    if(retval > 0x10FFFFUL)
        return error_value;
    if(!allow_Surrogate_Code_Units && retval >= 0xD800U && retval < 0xE000U)
        return error_value;
    return retval;
}

constexpr Encoded_character<char32_t, 1> encode_utf32(char32_t ch) noexcept
{
    return Encoded_character<char32_t, 1>(ch);
}

static_assert(std::numeric_limits<wchar_t>::radix == 2, "");
static_assert(std::numeric_limits<wchar_t>::digits
                      + static_cast<std::size_t>(std::is_signed<wchar_t>::value)
                  >= 16,
              "");

constexpr bool is_wide_character_utf16 = std::numeric_limits<wchar_t>::digits <= 16;

constexpr Encoded_character<wchar_t, 2> encode_wide(char32_t ch) noexcept
{
    if(is_wide_character_utf16)
    {
        auto result = encode_utf16(ch);
        Encoded_character<wchar_t, 2> retval;
        retval.used = result.used;
        for(std::size_t i = 0; i < result.size(); i++)
        {
            retval[i] = static_cast<wchar_t>(result[i]);
        }
        return retval;
    }
    return Encoded_character<wchar_t, 2>(static_cast<wchar_t>(ch));
}

template <typename Input_iterator, typename Sentinel>
typename std::char_traits<char32_t>::int_type decode_wide(
    Input_iterator &iter,
    Sentinel sentinel,
    bool allow_unpaired_surrogate_code_units = true,
    typename std::char_traits<char32_t>::int_type error_value =
        replacement_character) noexcept(noexcept(++iter) && noexcept(static_cast<wchar_t>(*iter))
                                        && noexcept(iter == sentinel ? 0 : 0))
{
    struct Iterator_wrapper
    {
        Input_iterator &iter;
        Iterator_wrapper(Input_iterator &iter) : iter(iter)
        {
        }
        void operator++()
        {
            ++iter;
        }
        wchar_t operator*()
        {
            return static_cast<wchar_t>(*iter);
        }
        bool operator==(Sentinel &sentinel)
        {
            return iter == sentinel;
        }
    };
    Iterator_wrapper iterator_wrapper(iter);
    if(is_wide_character_utf16)
        return decode_utf16(iterator_wrapper,
                            std::move(sentinel),
                            allow_unpaired_surrogate_code_units,
                            error_value);
    return decode_utf32(
        iterator_wrapper, std::move(sentinel), allow_unpaired_surrogate_code_units, error_value);
}

struct Convert_options final
{
    typename std::char_traits<char32_t>::int_type error_value = replacement_character;
    bool allow_unpaired_surrogate_code_points = true;
    bool allow_2_byte_null = false;
    bool use_2_byte_null = false;
    constexpr Convert_options()
    {
    }
    constexpr Convert_options(typename std::char_traits<char32_t>::int_type error_value,
                              bool allow_unpaired_surrogate_code_points,
                              bool allow_2_byte_null,
                              bool use_2_byte_null)
        : error_value(error_value),
          allow_unpaired_surrogate_code_points(allow_unpaired_surrogate_code_points),
          allow_2_byte_null(allow_2_byte_null),
          use_2_byte_null(use_2_byte_null)
    {
    }
    static constexpr Convert_options strict(
        typename std::char_traits<char32_t>::int_type error_value = replacement_character)
    {
        return Convert_options(error_value, false, false, false);
    }
    static constexpr Convert_options java(
        typename std::char_traits<char32_t>::int_type error_value = replacement_character)
    {
        return Convert_options(error_value, true, true, true);
    }
};

template <typename Char_type>
struct Decode_encode_functions
{
    template <typename Input_iterator, typename Sentinel>
    static typename std::char_traits<char32_t>::int_type decode(
        Input_iterator &iter, Sentinel sentinel, const Convert_options &convert_options) = delete;
    static Encoded_character<Char_type, 1> encode(
        char32_t ch, const Convert_options &convert_options) noexcept = delete;
};

template <>
struct Decode_encode_functions<char>
{
    template <typename Input_iterator, typename Sentinel>
    static typename std::char_traits<char32_t>::int_type decode(
        Input_iterator &iter,
        Sentinel sentinel,
        const Convert_options
            &convert_options) noexcept(noexcept(decode_utf8(std::declval<Input_iterator &>(),
                                                            std::declval<Sentinel &&>())))
    {
        return decode_utf8(iter,
                           std::move(sentinel),
                           convert_options.allow_unpaired_surrogate_code_points,
                           convert_options.allow_2_byte_null,
                           convert_options.error_value);
    }
    static Encoded_character<char, 4> encode(char32_t ch,
                                             const Convert_options &convert_options) noexcept
    {
        return encode_utf8(ch, convert_options.use_2_byte_null);
    }
};

template <>
struct Decode_encode_functions<char16_t>
{
    template <typename Input_iterator, typename Sentinel>
    static typename std::char_traits<char32_t>::int_type decode(
        Input_iterator &iter,
        Sentinel sentinel,
        const Convert_options
            &convert_options) noexcept(noexcept(decode_utf16(std::declval<Input_iterator &>(),
                                                             std::declval<Sentinel &&>())))
    {
        return decode_utf16(iter,
                            std::move(sentinel),
                            convert_options.allow_unpaired_surrogate_code_points,
                            convert_options.error_value);
    }
    static Encoded_character<char16_t, 2> encode(char32_t ch,
                                                 const Convert_options &convert_options) noexcept
    {
        return encode_utf16(ch);
    }
};

template <>
struct Decode_encode_functions<char32_t>
{
    template <typename Input_iterator, typename Sentinel>
    static typename std::char_traits<char32_t>::int_type decode(
        Input_iterator &iter,
        Sentinel sentinel,
        const Convert_options
            &convert_options) noexcept(noexcept(decode_utf32(std::declval<Input_iterator &>(),
                                                             std::declval<Sentinel &&>())))
    {
        return decode_utf32(iter,
                            std::move(sentinel),
                            convert_options.allow_unpaired_surrogate_code_points,
                            convert_options.error_value);
    }
    static Encoded_character<char32_t, 1> encode(char32_t ch,
                                                 const Convert_options &convert_options) noexcept
    {
        return encode_utf32(ch);
    }
};

template <>
struct Decode_encode_functions<wchar_t>
{
    template <typename Input_iterator, typename Sentinel>
    static typename std::char_traits<char32_t>::int_type decode(
        Input_iterator &iter,
        Sentinel sentinel,
        const Convert_options
            &convert_options) noexcept(noexcept(decode_wide(std::declval<Input_iterator &>(),
                                                            std::declval<Sentinel &&>())))
    {
        return decode_wide(iter,
                           std::move(sentinel),
                           convert_options.allow_unpaired_surrogate_code_points,
                           convert_options.error_value);
    }
    static Encoded_character<wchar_t, 2> encode(char32_t ch,
                                                const Convert_options &convert_options) noexcept
    {
        return encode_wide(ch);
    }
};

namespace detail
{
template <typename Target, typename Source>
struct String_cast_helper;

template <typename Target_Char_type,
          typename Target_Traits,
          typename Target_Allocator,
          typename Source_Char_type,
          typename Source_Traits>
struct String_cast_helper<std::basic_string<Target_Char_type, Target_Traits, Target_Allocator>,
                          basic_string_view<Source_Char_type, Source_Traits>>
{
    static std::basic_string<Target_Char_type, Target_Traits, Target_Allocator> run(
        basic_string_view<Source_Char_type, Source_Traits> source,
        const Convert_options &convert_options)
    {
        std::basic_string<Target_Char_type, Target_Traits, Target_Allocator> retval;
        for(auto iter = source.begin(); iter != source.end();)
        {
            retval = std::move(retval) + Decode_encode_functions<Target_Char_type>::encode(
                                             Decode_encode_functions<Source_Char_type>::decode(
                                                 iter, source.end(), convert_options),
                                             convert_options);
        }
        return retval;
    }
};

template <typename Char_type,
          typename Target_Traits,
          typename Target_Allocator,
          typename Source_Traits>
struct String_cast_helper<std::basic_string<Char_type, Target_Traits, Target_Allocator>,
                          basic_string_view<Char_type, Source_Traits>>
{
    static std::basic_string<Char_type, Target_Traits, Target_Allocator> run(
        basic_string_view<Char_type, Source_Traits> source, const Convert_options &)
    {
        return std::basic_string<Char_type, Target_Traits, Target_Allocator>(source.begin(),
                                                                             source.end());
    }
};
}

template <typename Target, typename Source_Char_type, typename Source_Traits>
Target string_cast(basic_string_view<Source_Char_type, Source_Traits> source,
                   const Convert_options &convert_options)
{
    return detail::String_cast_helper<Target, basic_string_view<Source_Char_type, Source_Traits>>::
        run(source, convert_options);
}

template <typename Target, typename Source_Char_type, typename Source_Traits>
Target string_cast(basic_string_view<Source_Char_type, Source_Traits> source)
{
    return detail::String_cast_helper<Target, basic_string_view<Source_Char_type, Source_Traits>>::
        run(source, Convert_options());
}
}
}
}

#endif /* UTIL_TEXT_H_ */
