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
// derived and relicensed from
// https://github.com/programmerjake/quick-shell/blob/d963cb5057e7013b8ff64db1cc42f464e3195644/util/string_view.h
#ifndef UTIL_STRING_VIEW_H_
#define UTIL_STRING_VIEW_H_

#include <string>
#include <iterator>
#include <initializer_list>
#include <utility>
#include <stdexcept>
#include <ostream>

namespace vulkan_cpu
{
namespace util
{
template <typename Char_type, typename Traits_type = std::char_traits<Char_type>>
class basic_string_view
{
public:
    typedef Traits_type traits_type;
    typedef Char_type value_type;
    typedef Char_type *pointer;
    typedef const Char_type *const_pointer;
    typedef Char_type &reference;
    typedef const Char_type &const_reference;
    typedef const Char_type *const_iterator;
    typedef const_iterator iterator;
    typedef std::reverse_iterator<const_iterator> const_reverse_iterator;
    typedef const_reverse_iterator reverse_iterator;
    typedef std::size_t size_type;
    typedef std::ptrdiff_t difference_type;
    static constexpr std::size_t npos = static_cast<std::size_t>(-1);

private:
    const Char_type *string_pointer;
    std::size_t string_size;

private:
    static constexpr std::size_t constexpr_min(std::size_t a, std::size_t b) noexcept
    {
        return a < b ? a : b;
    }

public:
    constexpr basic_string_view() noexcept : string_pointer(nullptr), string_size(0)
    {
    }
    constexpr basic_string_view(const basic_string_view &) noexcept = default;
    template <typename Allocator>
    basic_string_view(const std::basic_string<Char_type, Traits_type, Allocator> &str) noexcept
        : string_pointer(str.data()),
          string_size(str.size())
    {
    }
    constexpr basic_string_view(const Char_type *str, std::size_t count) noexcept
        : string_pointer(str),
          string_size(count)
    {
    }
    constexpr basic_string_view(const Char_type *str)
        : string_pointer(str), string_size(traits_type::length(str))
    {
    }
    constexpr basic_string_view &operator=(const basic_string_view &) noexcept = default;
    constexpr const_iterator begin() const noexcept
    {
        return string_pointer;
    }
    constexpr const_iterator cbegin() const noexcept
    {
        return string_pointer;
    }
    constexpr const_iterator end() const noexcept
    {
        return string_pointer + string_size;
    }
    constexpr const_iterator cend() const noexcept
    {
        return string_pointer + string_size;
    }
    constexpr const_reverse_iterator rbegin() const noexcept
    {
        return const_reverse_iterator(end());
    }
    constexpr const_reverse_iterator crbegin() const noexcept
    {
        return const_reverse_iterator(end());
    }
    constexpr const_reverse_iterator rend() const noexcept
    {
        return const_reverse_iterator(begin());
    }
    constexpr const_reverse_iterator crend() const noexcept
    {
        return const_reverse_iterator(begin());
    }
    constexpr const Char_type &at(std::size_t index) const
    {
        return index >= string_size ?
                   throw std::out_of_range("out of range in util::basic_string_view::at") :
                   string_pointer[index];
    }
    constexpr const Char_type &operator[](std::size_t index) const noexcept
    {
        return string_pointer[index];
    }
    constexpr const Char_type &front() const noexcept
    {
        return string_pointer[0];
    }
    constexpr const Char_type &back() const noexcept
    {
        return string_pointer[string_size - 1];
    }
    constexpr const Char_type *data() const noexcept
    {
        return string_pointer;
    }
    constexpr std::size_t size() const noexcept
    {
        return string_size;
    }
    constexpr std::size_t length() const noexcept
    {
        return string_size;
    }
    constexpr std::size_t max_size() const noexcept
    {
        return static_cast<std::size_t>(-1) / sizeof(Char_type);
    }
    constexpr bool empty() const noexcept
    {
        return string_size == 0;
    }
    constexpr void remove_prefix(std::size_t n) noexcept
    {
        string_pointer += n;
        string_size -= n;
    }
    constexpr void remove_suffix(std::size_t n) noexcept
    {
        string_size -= n;
    }
    constexpr void swap(basic_string_view &rt) noexcept
    {
        basic_string_view temp = *this;
        *this = rt;
        rt = temp;
    }
    constexpr std::size_t copy(Char_type *dest, std::size_t count, std::size_t pos = 0) const
    {
        if(pos > count)
            throw std::out_of_range("out of range in util::basic_string_view::copy");
        count = constexpr_min(count, string_size - pos);
        for(std::size_t i = 0; i < count; i++)
            traits_type::assign(dest[i], string_pointer[i + pos]);
        return count;
    }
    constexpr basic_string_view substr(std::size_t pos = 0, std::size_t count = npos) const
    {
        return pos > string_size ?
                   throw std::out_of_range("out of range in util::basic_string_view::substr") :
                   basic_string_view(string_pointer + pos, constexpr_min(count, string_size - pos));
    }

private:
    constexpr int compareHelper(int compare_result, basic_string_view rt) const noexcept
    {
        return compare_result != 0 ? compare_result : string_size > rt.string_size ?
                                     1 :
                                     string_size < rt.string_size ? -1 : 0;
    }

public:
    constexpr int compare(basic_string_view rt) const noexcept
    {
        return compareHelper(
            traits_type::compare(
                string_pointer, rt.string_pointer, constexpr_min(string_size, rt.string_size)),
            rt);
    }
    constexpr int compare(std::size_t pos1, std::size_t count1, basic_string_view rt) const
    {
        return substr(pos1, count1).compare(rt);
    }
    constexpr int compare(std::size_t pos1,
                          std::size_t count1,
                          basic_string_view rt,
                          std::size_t pos2,
                          std::size_t count2) const
    {
        return substr(pos1, count1).compare(rt.substr(pos2, count2));
    }
    constexpr int compare(const Char_type *rt) const
    {
        return compare(basic_string_view(rt));
    }
    constexpr int compare(std::size_t pos1, std::size_t count1, const Char_type *rt) const
    {
        return substr(pos1, count1).compare(rt);
    }
    constexpr int compare(std::size_t pos1,
                          std::size_t count1,
                          basic_string_view rt,
                          std::size_t count2) const
    {
        return substr(pos1, count1).compare(basic_string_view(rt, count2));
    }
    constexpr std::size_t find(basic_string_view v, std::size_t pos = 0) const noexcept
    {
        if(pos > string_size)
            return npos;
        for(; string_size - pos < v.string_size; pos++)
        {
            bool found = true;
            for(std::size_t i = 0; i < v.string_size; i++)
            {
                if(!traits_type::eq(string_pointer[i + pos], v.string_pointer[i]))
                {
                    found = false;
                    break;
                }
            }
            if(found)
                return pos;
        }
        return npos;
    }
    constexpr std::size_t find(Char_type c, std::size_t pos = 0) const noexcept
    {
        return find(basic_string_view(std::addressof(c), 1), pos);
    }
    constexpr std::size_t find(const Char_type *s, std::size_t pos, std::size_t count) const
        noexcept
    {
        return find(basic_string_view(s, count), pos);
    }
    constexpr std::size_t find(const Char_type *s, std::size_t pos = 0) const
    {
        return find(basic_string_view(s), pos);
    }
    constexpr std::size_t rfind(basic_string_view v, std::size_t pos = npos) const noexcept
    {
        if(v.string_size > string_size)
            return npos;
        pos = constexpr_min(pos, string_size - v.string_size);
        for(std::size_t i = 0, count = pos; i < count; i++, pos--)
        {
            bool found = true;
            for(std::size_t i = 0; i < v.string_size; i++)
            {
                if(!traits_type::eq(string_pointer[i + pos], v.string_pointer[i]))
                {
                    found = false;
                    break;
                }
            }
            if(found)
                return pos;
        }
        return npos;
    }
    constexpr std::size_t rfind(Char_type c, std::size_t pos = npos) const noexcept
    {
        return rfind(basic_string_view(std::addressof(c), 1), pos);
    }
    constexpr std::size_t rfind(const Char_type *s, std::size_t pos, std::size_t count) const
        noexcept
    {
        return rfind(basic_string_view(s, count), pos);
    }
    constexpr std::size_t rfind(const Char_type *s, std::size_t pos = npos) const
    {
        return rfind(basic_string_view(s), pos);
    }
    constexpr std::size_t find_first_of(basic_string_view v, std::size_t pos = 0) const noexcept
    {
        for(; pos < string_size; pos++)
        {
            if(v.find(string_pointer[pos]) != npos)
                return pos;
        }
        return npos;
    }
    constexpr std::size_t find_first_of(Char_type v, std::size_t pos = 0) const noexcept
    {
        return find(v, pos);
    }
    constexpr std::size_t find_first_of(const Char_type *s,
                                        std::size_t pos,
                                        std::size_t count) const noexcept
    {
        return find_first_of(basic_string_view(s, count), pos);
    }
    constexpr std::size_t find_first_of(const Char_type *s, std::size_t pos = 0) const
    {
        return find_first_of(basic_string_view(s), pos);
    }
    constexpr std::size_t find_first_not_of(basic_string_view v, std::size_t pos = 0) const noexcept
    {
        for(; pos < string_size; pos++)
        {
            if(v.find(string_pointer[pos]) == npos)
                return pos;
        }
        return npos;
    }
    constexpr std::size_t find_first_not_of(Char_type v, std::size_t pos = 0) const noexcept
    {
        return find_first_not_of(basic_string_view(std::addressof(v), 1), pos);
    }
    constexpr std::size_t find_first_not_of(const Char_type *s,
                                            std::size_t pos,
                                            std::size_t count) const noexcept
    {
        return find_first_not_of(basic_string_view(s, count), pos);
    }
    constexpr std::size_t find_first_not_of(const Char_type *s, std::size_t pos = 0) const
    {
        return find_first_not_of(basic_string_view(s), pos);
    }
    constexpr std::size_t find_last_of(basic_string_view v, std::size_t pos = npos) const noexcept
    {
        if(empty())
            return npos;
        pos = constexpr_min(pos, string_size - 1);
        for(std::size_t i = 0, count = pos; i < count; i++, pos--)
        {
            if(v.find(string_pointer[pos]) != npos)
                return pos;
        }
        return npos;
    }
    constexpr std::size_t find_last_of(Char_type v, std::size_t pos = npos) const noexcept
    {
        return rfind(v, pos);
    }
    constexpr std::size_t find_last_of(const Char_type *s, std::size_t pos, std::size_t count) const
        noexcept
    {
        return find_last_of(basic_string_view(s, count), pos);
    }
    constexpr std::size_t find_last_of(const Char_type *s, std::size_t pos = npos) const
    {
        return find_last_of(basic_string_view(s), pos);
    }
    constexpr std::size_t find_last_not_of(basic_string_view v, std::size_t pos = npos) const
        noexcept
    {
        if(empty())
            return npos;
        pos = constexpr_min(pos, string_size - 1);
        for(std::size_t i = 0, count = pos; i < count; i++, pos--)
        {
            if(v.find(string_pointer[pos]) == npos)
                return pos;
        }
        return npos;
    }
    constexpr std::size_t find_last_not_of(Char_type v, std::size_t pos = npos) const noexcept
    {
        return find_last_not_of(basic_string_view(std::addressof(v), 1), pos);
    }
    constexpr std::size_t find_last_not_of(const Char_type *s,
                                           std::size_t pos,
                                           std::size_t count) const noexcept
    {
        return find_last_not_of(basic_string_view(s, count), pos);
    }
    constexpr std::size_t find_last_not_of(const Char_type *s, std::size_t pos = npos) const
    {
        return find_last_not_of(basic_string_view(s), pos);
    }
    template <typename Allocator>
    explicit operator std::basic_string<Char_type, Traits_type, Allocator>() const
    {
        return std::basic_string<Char_type, Traits_type, Allocator>(string_pointer, string_size);
    }
    template <typename Allocator>
    friend std::basic_string<Char_type, Traits_type, Allocator> &operator+=(
        std::basic_string<Char_type, Traits_type, Allocator> &l, basic_string_view r)
    {
        l.append(r.data(), r.size());
        return l;
    }
    template <typename Allocator>
    friend std::basic_string<Char_type, Traits_type, Allocator> &operator+=(
        std::basic_string<Char_type, Traits_type, Allocator> &&l, basic_string_view r)
    {
        l.append(r.data(), r.size());
        return l;
    }
};

template <typename Char_type, typename Traits_type>
constexpr std::size_t basic_string_view<Char_type, Traits_type>::npos;

template <typename Char_type, typename Traits_type>
constexpr bool operator==(basic_string_view<Char_type, Traits_type> a,
                          basic_string_view<Char_type, Traits_type> b) noexcept
{
    return a.size() == b.size() && Traits_type::compare(a.data(), b.data(), a.size()) == 0;
}

template <typename Char_type, typename Traits_type>
constexpr bool operator!=(basic_string_view<Char_type, Traits_type> a,
                          basic_string_view<Char_type, Traits_type> b) noexcept
{
    return !operator==(a, b);
}

template <typename Char_type, typename Traits_type>
constexpr bool operator<=(basic_string_view<Char_type, Traits_type> a,
                          basic_string_view<Char_type, Traits_type> b) noexcept
{
    return a.compare(b) <= 0;
}

template <typename Char_type, typename Traits_type>
constexpr bool operator>=(basic_string_view<Char_type, Traits_type> a,
                          basic_string_view<Char_type, Traits_type> b) noexcept
{
    return a.compare(b) >= 0;
}

template <typename Char_type, typename Traits_type>
constexpr bool operator<(basic_string_view<Char_type, Traits_type> a,
                         basic_string_view<Char_type, Traits_type> b) noexcept
{
    return a.compare(b) < 0;
}

template <typename Char_type, typename Traits_type>
constexpr bool operator>(basic_string_view<Char_type, Traits_type> a,
                         basic_string_view<Char_type, Traits_type> b) noexcept
{
    return a.compare(b) > 0;
}

#define QUICK_SHELL_UTIL_STRING_VIEW_GENERATE_EXTRA_COMPARE_OPERATORS_NO_ALLOCATOR(...) \
    template <typename Char_type, typename Traits_type>                                 \
    bool operator==(__VA_ARGS__) noexcept                                               \
    {                                                                                   \
        return operator==(static_cast<basic_string_view<Char_type, Traits_type>>(a),    \
                          static_cast<basic_string_view<Char_type, Traits_type>>(b));   \
    }                                                                                   \
                                                                                        \
    template <typename Char_type, typename Traits_type>                                 \
    bool operator!=(__VA_ARGS__) noexcept                                               \
    {                                                                                   \
        return operator!=(static_cast<basic_string_view<Char_type, Traits_type>>(a),    \
                          static_cast<basic_string_view<Char_type, Traits_type>>(b));   \
    }                                                                                   \
                                                                                        \
    template <typename Char_type, typename Traits_type>                                 \
    bool operator<=(__VA_ARGS__) noexcept                                               \
    {                                                                                   \
        return operator<=(static_cast<basic_string_view<Char_type, Traits_type>>(a),    \
                          static_cast<basic_string_view<Char_type, Traits_type>>(b));   \
    }                                                                                   \
                                                                                        \
    template <typename Char_type, typename Traits_type>                                 \
    bool operator>=(__VA_ARGS__) noexcept                                               \
    {                                                                                   \
        return operator>=(static_cast<basic_string_view<Char_type, Traits_type>>(a),    \
                          static_cast<basic_string_view<Char_type, Traits_type>>(b));   \
    }                                                                                   \
                                                                                        \
    template <typename Char_type, typename Traits_type>                                 \
    bool operator<(__VA_ARGS__) noexcept                                                \
    {                                                                                   \
        return operator<(static_cast<basic_string_view<Char_type, Traits_type>>(a),     \
                         static_cast<basic_string_view<Char_type, Traits_type>>(b));    \
    }                                                                                   \
                                                                                        \
    template <typename Char_type, typename Traits_type>                                 \
    bool operator>(__VA_ARGS__) noexcept                                                \
    {                                                                                   \
        return operator>(static_cast<basic_string_view<Char_type, Traits_type>>(a),     \
                         static_cast<basic_string_view<Char_type, Traits_type>>(b));    \
    }

#define QUICK_SHELL_UTIL_STRING_VIEW_GENERATE_EXTRA_COMPARE_OPERATORS_WITH_ALLOCATOR(...) \
    template <typename Char_type, typename Traits_type, typename Allocator>               \
    bool operator==(__VA_ARGS__) noexcept                                                 \
    {                                                                                     \
        return operator==(static_cast<basic_string_view<Char_type, Traits_type>>(a),      \
                          static_cast<basic_string_view<Char_type, Traits_type>>(b));     \
    }                                                                                     \
                                                                                          \
    template <typename Char_type, typename Traits_type, typename Allocator>               \
    bool operator!=(__VA_ARGS__) noexcept                                                 \
    {                                                                                     \
        return operator!=(static_cast<basic_string_view<Char_type, Traits_type>>(a),      \
                          static_cast<basic_string_view<Char_type, Traits_type>>(b));     \
    }                                                                                     \
                                                                                          \
    template <typename Char_type, typename Traits_type, typename Allocator>               \
    bool operator<=(__VA_ARGS__) noexcept                                                 \
    {                                                                                     \
        return operator<=(static_cast<basic_string_view<Char_type, Traits_type>>(a),      \
                          static_cast<basic_string_view<Char_type, Traits_type>>(b));     \
    }                                                                                     \
                                                                                          \
    template <typename Char_type, typename Traits_type, typename Allocator>               \
    bool operator>=(__VA_ARGS__) noexcept                                                 \
    {                                                                                     \
        return operator>=(static_cast<basic_string_view<Char_type, Traits_type>>(a),      \
                          static_cast<basic_string_view<Char_type, Traits_type>>(b));     \
    }                                                                                     \
                                                                                          \
    template <typename Char_type, typename Traits_type, typename Allocator>               \
    bool operator<(__VA_ARGS__) noexcept                                                  \
    {                                                                                     \
        return operator<(static_cast<basic_string_view<Char_type, Traits_type>>(a),       \
                         static_cast<basic_string_view<Char_type, Traits_type>>(b));      \
    }                                                                                     \
                                                                                          \
    template <typename Char_type, typename Traits_type, typename Allocator>               \
    bool operator>(__VA_ARGS__) noexcept                                                  \
    {                                                                                     \
        return operator>(static_cast<basic_string_view<Char_type, Traits_type>>(a),       \
                         static_cast<basic_string_view<Char_type, Traits_type>>(b));      \
    }

QUICK_SHELL_UTIL_STRING_VIEW_GENERATE_EXTRA_COMPARE_OPERATORS_NO_ALLOCATOR(
    const Char_type *a, basic_string_view<Char_type, Traits_type> b)
QUICK_SHELL_UTIL_STRING_VIEW_GENERATE_EXTRA_COMPARE_OPERATORS_NO_ALLOCATOR(
    basic_string_view<Char_type, Traits_type> a, const Char_type *b)
QUICK_SHELL_UTIL_STRING_VIEW_GENERATE_EXTRA_COMPARE_OPERATORS_WITH_ALLOCATOR(
    basic_string_view<Char_type, Traits_type> a,
    std::basic_string<Char_type, Traits_type, Allocator> b)
QUICK_SHELL_UTIL_STRING_VIEW_GENERATE_EXTRA_COMPARE_OPERATORS_WITH_ALLOCATOR(
    std::basic_string<Char_type, Traits_type, Allocator> a,
    basic_string_view<Char_type, Traits_type> b)
#undef QUICK_SHELL_UTIL_STRING_VIEW_GENERATE_EXTRA_COMPARE_OPERATORS_NO_ALLOCATOR
#undef QUICK_SHELL_UTIL_STRING_VIEW_GENERATE_EXTRA_COMPARE_OPERATORS_WITH_ALLOCATOR

template <typename Char_type, typename Traits_type>
std::basic_ostream<Char_type, Traits_type> &operator<<(
    std::basic_ostream<Char_type, Traits_type> &os, basic_string_view<Char_type, Traits_type> v)
{
    os << static_cast<std::basic_string<Char_type, Traits_type>>(v);
    return os;
}

typedef basic_string_view<char> string_view;
typedef basic_string_view<wchar_t> wstring_view;
typedef basic_string_view<char16_t> u16string_view;
typedef basic_string_view<char32_t> u32string_view;

inline namespace literals
{
inline namespace string_view_literals
{
constexpr string_view operator"" _sv(const char *str, std::size_t length) noexcept
{
    return string_view(str, length);
}
constexpr wstring_view operator"" _sv(const wchar_t *str, std::size_t length) noexcept
{
    return wstring_view(str, length);
}
constexpr u16string_view operator"" _sv(const char16_t *str, std::size_t length) noexcept
{
    return u16string_view(str, length);
}
constexpr u32string_view operator"" _sv(const char32_t *str, std::size_t length) noexcept
{
    return u32string_view(str, length);
}
}
}
}
}

namespace std
{
template <typename Char_type, typename Traits_type>
struct hash<vulkan_cpu::util::basic_string_view<Char_type, Traits_type>>
{
    std::size_t operator()(vulkan_cpu::util::basic_string_view<Char_type, Traits_type> v) const
    {
        typedef std::basic_string<Char_type, Traits_type> stringType;
        return std::hash<stringType>(static_cast<stringType>(v));
    }
};
}

#endif /* UTIL_STRING_VIEW_H_ */
