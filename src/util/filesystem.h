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
#ifndef UTIL_FILESYSTEM_H_
#define UTIL_FILESYSTEM_H_

#include <string>
#include <utility>
#include <new>
#include <memory>
#include <cassert>
#include <limits>
#include "bit_intrinsics.h"
#include "string_view.h"
#include "void_t.h"
#include <type_traits>
#include <iterator>
#include "optional.h"
#include "text.h"
#include <cstdint>
#include <iomanip>
#include <istream>
#include <ostream>

namespace vulkan_cpu
{
namespace util
{
namespace filesystem
{
namespace detail
{
enum class Path_traits_kind
{
    posix,
    windows,
};

#ifdef _WIN32
constexpr Path_traits_kind default_path_traits_kind = Path_traits_kind::windows;
#else
constexpr Path_traits_kind default_path_traits_kind = Path_traits_kind::posix;
#endif

template <Path_traits_kind Kind>
struct Path_traits
{
    typedef char value_type;
    static constexpr value_type preferred_separator = '/';
    static constexpr bool needs_root_name_to_be_absolute = false;
};

template <>
struct Path_traits<Path_traits_kind::windows>
{
    typedef wchar_t value_type;
    static constexpr value_type preferred_separator = L'\\';
    static constexpr bool needs_root_name_to_be_absolute = true;
};

enum class Path_part_kind
{
    relative_root_name, // root name that has a current directory, like "C:" in windows
    absolute_root_name, // root name that can't have a current directory, like "\\ABC" in windows
    root_dir,
    file_name,
    multiple_parts,
    path_separator,
};

template <typename T>
struct Path_is_convertable_char_type
{
    static constexpr bool value = false;
};

template <typename T>
struct Path_is_convertable_char_type<const T> : public Path_is_convertable_char_type<T>
{
};

template <>
struct Path_is_convertable_char_type<char>
{
    static constexpr bool value = true;
    typedef char Char_type;
};

template <>
struct Path_is_convertable_char_type<wchar_t>
{
    static constexpr bool value = true;
    typedef wchar_t Char_type;
};

template <>
struct Path_is_convertable_char_type<char16_t>
{
    static constexpr bool value = true;
    typedef char16_t Char_type;
};

template <>
struct Path_is_convertable_char_type<char32_t>
{
    static constexpr bool value = true;
    typedef char32_t Char_type;
};

template <typename T, typename = void>
struct Path_is_convertable_iterator_type
{
    static constexpr bool value = false;
};

template <typename T>
struct Path_is_convertable_iterator_type<T,
                                         typename std::enable_if<Path_is_convertable_char_type<
                                             typename std::iterator_traits<T>::value_type>::value>::
                                             type>
{
    static constexpr bool value = true;
    typedef typename Path_is_convertable_char_type<
        typename std::iterator_traits<T>::value_type>::Char_type Char_type;
};

struct Path_iterator_sentinel
{
};

template <typename Iterator>
class Path_convert_single_iterator_adaptor
{
private:
    typedef std::iterator_traits<Iterator> Traits;
    optional<Iterator> base_iterator;

public:
    typedef typename Traits::value_type value_type;
    typedef typename Traits::pointer pointer;
    typedef typename Traits::reference reference;
    typedef typename Traits::difference_type difference_type;
    typedef std::input_iterator_tag iterator_category;
    constexpr Path_convert_single_iterator_adaptor() noexcept : base_iterator()
    {
    }
    constexpr explicit Path_convert_single_iterator_adaptor(Iterator iterator)
        : base_iterator(std::move(iterator))
    {
    }
    bool operator==(const Path_convert_single_iterator_adaptor &rt) const
    {
        if(base_iterator)
        {
            assert(!rt.base_iterator);
            return **base_iterator == value_type();
        }
        if(rt.base_iterator)
            return **rt.base_iterator == value_type();
        return true;
    }
    bool operator!=(const Path_convert_single_iterator_adaptor &rt) const
    {
        return !operator==(rt);
    }
    bool operator==(Path_iterator_sentinel) const
    {
        if(base_iterator)
            return **base_iterator == value_type();
        return true;
    }
    bool operator!=(Path_iterator_sentinel) const
    {
        return !operator==(Path_iterator_sentinel());
    }
    Path_convert_single_iterator_adaptor &operator++()
    {
        if(base_iterator)
            ++(*base_iterator);
        return *this;
    }
    Path_convert_single_iterator_adaptor operator++(int)
    {
        if(base_iterator)
            return Path_convert_single_iterator_adaptor((*base_iterator)++);
        return {};
    }
    reference operator*() const
    {
        return **base_iterator;
    }
    pointer operator->() const
    {
        return std::addressof(operator*());
    }
};

template <typename Iterator, typename Sentinel>
struct Iterator_and_sentinel
{
    Iterator iterator;
    Sentinel sentinel;
    Iterator_and_sentinel(Iterator iterator, Sentinel sentinel)
        : iterator(std::move(iterator)), sentinel(std::move(sentinel))
    {
    }
};

template <typename Dest_char_type,
          typename Iterator,
          typename Sentinel = Iterator,
          typename Source_char_type =
              typename Path_is_convertable_iterator_type<Iterator>::Char_type>
class Path_convert_iterator
{
private:
    typedef decltype(text::Decode_encode_functions<Dest_char_type>::encode(
        char32_t(), text::Convert_options())) Encode_result;
    static_assert(std::is_same<typename Encode_result::Char_type, Dest_char_type>::value, "");

public:
    typedef Dest_char_type value_type;
    typedef const Dest_char_type *pointer;
    typedef const Dest_char_type &reference;
    typedef std::ptrdiff_t difference_type;
    typedef std::input_iterator_tag iterator_category;

private:
    Encode_result encode_result;
    std::size_t encode_result_index;
    util::optional<Iterator_and_sentinel<Iterator, Sentinel>> iterator_and_sentinel;
    void convert_next()
    {
        std::char_traits<char32_t>::int_type ch =
            text::Decode_encode_functions<Source_char_type>::decode(iterator_and_sentinel->iterator,
                                                                    iterator_and_sentinel->sentinel,
                                                                    text::Convert_options());
        if(ch == std::char_traits<char32_t>::eof())
            *this = Path_convert_iterator();
        else
        {
            encode_result =
                text::Decode_encode_functions<Dest_char_type>::encode(ch, text::Convert_options());
            encode_result_index = 0;
        }
    }

public:
    constexpr Path_convert_iterator() noexcept : encode_result(),
                                                 encode_result_index(),
                                                 iterator_and_sentinel()
    {
    }
    Path_convert_iterator(Iterator iterator, Sentinel sentinel)
        : encode_result(),
          encode_result_index(),
          iterator_and_sentinel(in_place, std::move(iterator), std::move(sentinel))
    {
        convert_next();
    }
    Path_convert_iterator &operator++()
    {
        if(++encode_result_index >= encode_result.size())
            convert_next();
        return *this;
    }
    Path_convert_iterator operator++(int)
    {
        auto retval = *this;
        operator++();
        return retval;
    }
    const Dest_char_type &operator*() const noexcept
    {
        return encode_result[encode_result_index];
    }
    const Dest_char_type *operator->() const noexcept
    {
        return &encode_result[encode_result_index];
    }
    bool operator==(const Path_convert_iterator &rt) const noexcept
    {
        return iterator_and_sentinel.has_value() == rt.iterator_and_sentinel.has_value();
    }
    bool operator!=(const Path_convert_iterator &rt) const noexcept
    {
        return !operator==(rt);
    }
    bool operator==(Path_iterator_sentinel) const noexcept
    {
        return !iterator_and_sentinel;
    }
    bool operator!=(Path_iterator_sentinel) const noexcept
    {
        return !operator==(Path_iterator_sentinel());
    }
};

template <typename Path_char_type, typename Iterator, typename = void>
struct Path_convert_range
{
    static constexpr bool is_convertible = false;
};

template <typename Path_char_type, typename Iterator>
struct Path_convert_range<Path_char_type,
                          Iterator,
                          typename std::
                              enable_if<Path_is_convertable_iterator_type<Iterator>::value>::type>
{
    static constexpr bool is_convertible = true;
    template <typename Traits = std::char_traits<Path_char_type>,
              typename Allocator = std::allocator<Path_char_type>,
              typename Sentinel>
    static std::basic_string<Path_char_type, Traits, Allocator> to_string(
        Iterator iterator, Sentinel sentinel, const Allocator &a = Allocator())
    {
        typedef Path_convert_iterator<Path_char_type, Iterator, Sentinel> Convert_iterator;
        return std::basic_string<Path_char_type, Traits, Allocator>(
            Convert_iterator(iterator, sentinel), Convert_iterator(), a);
    }
};

template <typename Iterator>
struct Path_convert_range<typename Path_is_convertable_iterator_type<Iterator>::Char_type,
                          Iterator,
                          void>
{
    static constexpr bool is_convertible = true;
    typedef typename Path_is_convertable_iterator_type<Iterator>::Char_type Char_type;
    static std::basic_string<Char_type> to_string(
        Iterator iterator,
        Iterator sentinel,
        const std::allocator<Char_type> &a = std::allocator<Char_type>())
    {
        return std::basic_string<Char_type>(iterator, sentinel, a);
    }
    template <typename Traits = std::char_traits<Char_type>,
              typename Allocator = std::allocator<Char_type>,
              typename Sentinel>
    static std::basic_string<Char_type, Traits, Allocator> to_string(
        Iterator iterator, Sentinel sentinel, const Allocator &a = Allocator())
    {
        std::basic_string<Char_type, Traits, Allocator> retval(a);
        while(iterator != sentinel)
            retval += *iterator++;
        return retval;
    }
};

template <typename Path_char_type, typename Source, typename = void>
struct Path_convert_source
{
    static constexpr bool is_convertible = false;
};

template <typename Path_char_type, typename Source_char_type, typename Traits, typename Allocator>
struct Path_convert_source<Path_char_type,
                           std::basic_string<Source_char_type, Traits, Allocator>,
                           typename std::
                               enable_if<Path_convert_range<Path_char_type,
                                                            typename std::
                                                                basic_string<Source_char_type,
                                                                             Traits,
                                                                             Allocator>::
                                                                    const_iterator>::
                                             is_convertible>::type>
{
    typedef Path_convert_range<Path_char_type,
                               typename std::basic_string<Source_char_type, Traits, Allocator>::
                                   const_iterator> Convert_range;
    static constexpr bool is_convertible = true;
    template <typename Dest_traits = std::char_traits<Path_char_type>,
              typename Dest_allocator = std::allocator<Path_char_type>>
    static std::basic_string<Path_char_type, Dest_traits, Dest_allocator> to_string(
        const std::basic_string<Source_char_type, Traits, Allocator> &source,
        const Allocator &a = Allocator())
    {
        return Convert_range::template to_string<Dest_traits, Dest_allocator>(
            source.begin(), source.end(), a);
    }
};

template <typename Path_char_type, typename Source_char_type, typename Traits>
struct
    Path_convert_source<Path_char_type,
                        basic_string_view<Source_char_type, Traits>,
                        typename std::
                            enable_if<Path_convert_range<Path_char_type,
                                                         typename basic_string_view<Source_char_type,
                                                                                    Traits>::
                                                             const_iterator>::is_convertible>::type>
{
    typedef Path_convert_range<Path_char_type,
                               typename basic_string_view<Source_char_type, Traits>::const_iterator>
        Convert_range;
    static constexpr bool is_convertible = true;
    template <typename Dest_traits = std::char_traits<Path_char_type>,
              typename Allocator = std::allocator<Path_char_type>>
    static std::basic_string<Path_char_type, Dest_traits, Allocator> to_string(
        const basic_string_view<Source_char_type, Traits> &source, const Allocator &a = Allocator())
    {
        return Convert_range::template to_string<Dest_traits, Allocator>(
            source.begin(), source.end(), a);
    }
};

template <typename Char_type>
struct Path_convert_source<Char_type, std::basic_string<Char_type>, void>
{
    static constexpr bool is_convertible = true;
    template <typename Traits = std::char_traits<Char_type>,
              typename Allocator = std::allocator<Char_type>>
    static std::basic_string<Char_type, Traits, Allocator> to_string(
        const std::basic_string<Char_type> &source, const Allocator &a = Allocator())
    {
        return std::basic_string<Char_type, Traits, Allocator>(source.begin(), source.end(), a);
    }
    static std::basic_string<Char_type> to_string(
        const std::basic_string<Char_type> &source,
        const std::allocator<Char_type> & = std::allocator<Char_type>())
    {
        return source;
    }
};

template <typename Char_type, typename Iterator>
struct Path_convert_source<Char_type,
                           Iterator,
                           typename std::enable_if<!std::is_same<
                               typename Path_is_convertable_iterator_type<Iterator>::Char_type,
                               Char_type>::value>::type>
{
    static constexpr bool is_convertible = true;
    typedef Path_convert_range<Char_type, Path_convert_single_iterator_adaptor<Iterator>>
        Convert_range;
    template <typename Traits = std::char_traits<Char_type>,
              typename Allocator = std::allocator<Char_type>>
    static std::basic_string<Char_type, Traits, Allocator> to_string(
        Iterator iterator, const Allocator &a = Allocator())
    {
        return Convert_range::template to_string<Traits, Allocator>(
            Path_convert_single_iterator_adaptor<Iterator>(iterator), Path_iterator_sentinel(), a);
    }
};

template <typename Char_type>
struct Path_convert_source<Char_type, const Char_type *, void>
{
    static constexpr bool is_convertible = true;
    template <typename Traits = std::char_traits<Char_type>,
              typename Allocator = std::allocator<Char_type>>
    static std::basic_string<Char_type, Traits, Allocator> to_string(
        const Char_type *source, const Allocator &a = Allocator())
    {
        return {source, a};
    }
};

template <typename Path_char_type, typename Source_char_type, std::size_t N>
struct Path_convert_source<Path_char_type, Source_char_type[N], void>
    : public Path_convert_source<Path_char_type, const Source_char_type *, void>
{
};

struct Path_index_range
{
    std::size_t begin;
    std::size_t end;
    constexpr Path_index_range() noexcept : begin(0), end(0)
    {
    }
    constexpr Path_index_range(std::size_t begin, std::size_t end) noexcept : begin(begin), end(end)
    {
    }
    template <typename Char_type, typename Traits>
    constexpr Path_index_range(basic_string_view<Char_type, Traits> str,
                               typename basic_string_view<Char_type, Traits>::iterator begin,
                               typename basic_string_view<Char_type, Traits>::iterator end) noexcept
        : begin(begin - str.begin()),
          end(end - str.begin())
    {
    }
    constexpr bool empty() const noexcept
    {
        return begin == end;
    }
    constexpr std::size_t size() const noexcept
    {
        return end - begin;
    }
};

struct Path_tester;
}

template <detail::Path_traits_kind Traits_kind = detail::default_path_traits_kind,
          typename Char_type = typename detail::Path_traits<Traits_kind>::value_type,
          Char_type Preferred_separator = detail::Path_traits<Traits_kind>::preferred_separator,
          bool Needs_root_name_to_be_absolute =
              detail::Path_traits<Traits_kind>::needs_root_name_to_be_absolute>
class basic_path
{
    friend struct detail::Path_tester;
    static_assert(std::is_same<Char_type, char>::value || std::is_same<Char_type, wchar_t>::value,
                  "");

private:
    typedef detail::Path_traits_kind Path_traits_kind;
    typedef detail::Path_part_kind Path_part_kind;
    typedef detail::Path_index_range Path_index_range;

public:
    typedef Char_type value_type;
    typedef std::basic_string<Char_type> string_type;
    enum format
    {
        native_format,
        generic_format,
        auto_format
    };
    static constexpr Char_type preferred_separator = Preferred_separator;

private:
    typedef basic_string_view<Char_type> string_view_type;
    class Parts
    {
    private:
        std::size_t allocated_count;
        std::size_t used_count;
        basic_path *values;

    private:
        static basic_path *allocate(std::size_t count);
        template <typename... Args>
        static void construct(basic_path &value, Args &&... args)
        {
            ::new(static_cast<void *>(&value)) basic_path(std::forward<Args>(args)...);
        }
        static void destruct(basic_path &value) noexcept
        {
            value.~basic_path();
        }
        static void deallocate(basic_path *values, std::size_t count) noexcept;
        void reallocate(std::size_t new_allocated_count)
        {
            assert(new_allocated_count >= used_count);
            if(used_count == 0)
            {
                deallocate(values, allocated_count);
                values = nullptr;
                allocated_count = 0; // set now in case allocate throws
                values = allocate(new_allocated_count);
                allocated_count = new_allocated_count;
            }
            else
            {
                Parts new_parts;
                new_parts.reserve(new_allocated_count);
                for(std::size_t i = 0; i < used_count; i++)
                    new_parts.push_back(std::move(values[i]));
                swap(new_parts);
            }
        }
        static constexpr std::uint64_t get_expanded_count_64(std::uint64_t count) noexcept
        {
            constexpr std::uint64_t high_bit = 1ULL << 63;
            if(count == 0 || count >= high_bit)
                return count;
            return 1ULL << (64 - clz64(count - 1));
        }
        static constexpr std::uint32_t get_expanded_count_32(std::uint32_t count) noexcept
        {
            constexpr std::uint32_t high_bit = 1UL << 31;
            if(count == 0 || count >= high_bit)
                return count;
            return 1UL << (32 - clz32(count - 1));
        }
        static constexpr std::size_t get_expanded_count(std::size_t count) noexcept
        {
            constexpr bool is_size_t_uint32_t = std::is_same<std::size_t, std::uint32_t>::value,
                           is_size_t_uint64_t = std::is_same<std::size_t, std::uint64_t>::value;
            static_assert(is_size_t_uint32_t || is_size_t_uint64_t, "");
            if(is_size_t_uint32_t)
                return get_expanded_count_32(static_cast<std::uint32_t>(count));
            return static_cast<std::size_t>(get_expanded_count_64(count));
        }

    public:
        constexpr Parts() noexcept : allocated_count(0), used_count(0), values(nullptr)
        {
        }
        Parts(const Parts &rt) : Parts()
        {
            Parts new_parts;
            new_parts.reserve(rt.used_count);
            for(std::size_t i = 0; i < rt.used_count; i++)
                push_back(rt.values[i]);
            swap(new_parts);
        }
        Parts(Parts &&rt) noexcept : Parts()
        {
            swap(rt);
        }
        Parts &operator=(Parts &&rt) noexcept
        {
            Parts(std::move(rt)).swap(*this);
            return *this;
        }
        Parts &operator=(const Parts &rt)
        {
            if(this == &rt)
                return *this;
            if(allocated_count < rt.used_count)
            {
                Parts(rt).swap(*this);
                return *this;
            }
            while(used_count > rt.used_count)
                pop_back();
            for(std::size_t i = 0; i < used_count; i++)
                values[i] = rt[i];
            while(used_count < rt.used_count)
                push_back(rt[used_count]);
            return *this;
        }
        ~Parts() noexcept
        {
            while(used_count > 0)
                destruct(values[--used_count]);
            deallocate(values, allocated_count);
        }
        void swap(Parts &rt) noexcept
        {
            using std::swap;
            swap(allocated_count, rt.allocated_count);
            swap(used_count, rt.used_count);
            swap(values, rt.values);
        }
        void reserve(std::size_t new_allocated_count)
        {
            if(new_allocated_count > allocated_count)
                reallocate(new_allocated_count);
        }
        bool empty() const noexcept
        {
            return used_count == 0;
        }
        std::size_t size() const noexcept
        {
            return used_count;
        }
        std::size_t capacity() const noexcept
        {
            return allocated_count;
        }
        typedef basic_path *iterator;
        typedef const basic_path *const_iterator;
        iterator begin() noexcept
        {
            return values;
        }
        iterator end() noexcept
        {
            return values + used_count;
        }
        const_iterator begin() const noexcept
        {
            return values;
        }
        const_iterator end() const noexcept
        {
            return values + used_count;
        }
        const_iterator cbegin() const noexcept
        {
            return values;
        }
        const_iterator cend() const noexcept
        {
            return values + used_count;
        }
        template <typename... Args>
        void emplace_back(Args &&... args)
        {
            if(used_count >= allocated_count)
                reallocate(get_expanded_count(used_count + 1));
            construct(values[used_count], std::forward<Args>(args)...);
            used_count++;
        }
        void push_back(const basic_path &v)
        {
            emplace_back(v);
        }
        void push_back(basic_path &&v)
        {
            emplace_back(v);
        }
        void pop_back() noexcept
        {
            assert(used_count > 0);
            destruct(values[--used_count]);
        }
        void clear() noexcept
        {
            while(used_count > 0)
                pop_back();
        }
        basic_path &operator[](std::size_t index) noexcept
        {
            assert(index < used_count);
            return values[index];
        }
        const basic_path &operator[](std::size_t index) const noexcept
        {
            assert(index < used_count);
            return values[index];
        }
    };

private:
    Parts parts;
    string_type value;
    detail::Path_part_kind kind;

public:
    class iterator
    {
        template <detail::Path_traits_kind, typename Char_type2, Char_type2, bool>
        friend class basic_path;

    public:
        typedef basic_path value_type;
        typedef const basic_path *pointer;
        typedef const basic_path &reference;
        typedef std::ptrdiff_t difference_type;
        typedef std::bidirectional_iterator_tag iterator_category;

    private:
        const basic_path *path;
        std::size_t index;
        constexpr iterator(const basic_path *path, std::size_t index) noexcept : path(path),
                                                                                 index(index)
        {
        }

    public:
        constexpr iterator() noexcept : path(nullptr), index()
        {
        }
        constexpr iterator &operator++() noexcept
        {
            index++;
            return *this;
        }
        constexpr iterator &operator--() noexcept
        {
            index--;
            return *this;
        }
        constexpr iterator operator++(int) noexcept
        {
            return iterator(path, index++);
        }
        constexpr iterator operator--(int) noexcept
        {
            return iterator(path, index--);
        }
        const basic_path *operator->() const
        {
            assert(path);
            if(path->kind == Path_part_kind::multiple_parts)
                return &path->parts[index];
            return path;
        }
        const basic_path &operator*() const
        {
            return *operator->();
        }
        constexpr bool operator==(const iterator &rt) const noexcept
        {
            return index == rt.index;
        }
        constexpr bool operator!=(const iterator &rt) const noexcept
        {
            return index != rt.index;
        }
    };
    typedef iterator const_iterator;

private:
    static constexpr bool is_ascii_letter(Char_type v) noexcept
    {
        auto ch = static_cast<unsigned char>(v);
        if(static_cast<Char_type>(ch) != v)
            return false;
        if(ch >= 'a' && ch <= 'z')
            return true;
        if(ch >= 'A' && ch <= 'Z')
            return true;
        return false;
    }
    template <typename Char_type2>
    static constexpr bool is_separator(Char_type2 v) noexcept
    {
        return v == static_cast<Char_type2>('/')
               || v == static_cast<Char_type2>(preferred_separator);
    }
    template <bool Ignore_root_parts = false, typename Fn>
    static bool parse(string_view_type value, Fn callback, format fmt = auto_format) noexcept(
        noexcept(callback(Path_index_range(), Path_part_kind())))
    {
        constexpr Char_type colon = ':';
        typedef typename std::char_traits<Char_type>::int_type Int_type;
        constexpr Int_type eof = std::char_traits<Char_type>::eof();
        auto char_iter = value.begin();
        auto peek = [&]() -> Int_type
        {
            if(char_iter == value.end())
                return eof;
            return std::char_traits<Char_type>::to_int_type(*char_iter);
        };
        auto get = [&]() -> Int_type
        {
            if(char_iter == value.end())
                return eof;
            return std::char_traits<Char_type>::to_int_type(*char_iter++);
        };
        if(value.empty())
            return true;
        if(!Ignore_root_parts && Traits_kind == Path_traits_kind::windows && value.size() >= 2
           && is_ascii_letter(value[0])
           && value[1] == colon)
        {
            char_iter += 2;
            if(!callback(Path_index_range(value, value.begin(), char_iter),
                         Path_part_kind::relative_root_name))
                return false;
        }
        else if(!Ignore_root_parts && Traits_kind == Path_traits_kind::windows && value.size() >= 2
                && is_separator(value[0])
                && is_separator(value[1]))
        {
            while(peek() != eof && is_separator(peek()))
                get();
            while(peek() != eof && !is_separator(peek()))
                get();
            if(!callback(Path_index_range(value, value.begin(), char_iter),
                         Path_part_kind::absolute_root_name))
                return false;
        }
        if(!Ignore_root_parts && peek() != eof && is_separator(peek()))
        {
            auto start_iter = char_iter;
            do
            {
                get();
            } while(peek() != eof && is_separator(peek()));
            if(!callback(Path_index_range(value, start_iter, char_iter), Path_part_kind::root_dir))
                return false;
        }
        if(Ignore_root_parts && peek() != eof && is_separator(peek()))
        {
            if(!callback(Path_index_range(value, char_iter, char_iter), Path_part_kind::file_name))
                return false;
        }
        if(peek() != eof && !is_separator(peek()))
        {
            auto start_iter = char_iter;
            do
            {
                get();
            } while(peek() != eof && !is_separator(peek()));
            if(!callback(Path_index_range(value, start_iter, char_iter), Path_part_kind::file_name))
                return false;
        }
        while(peek() != eof)
        {
            auto start_iter = char_iter;
            do
            {
                get();
            } while(peek() != eof && is_separator(peek()));
            if(!callback(Path_index_range(value, start_iter, char_iter),
                         Path_part_kind::path_separator))
                return false;
            start_iter = char_iter;
            while(peek() != eof && !is_separator(peek()))
                get();
            if(!callback(Path_index_range(value, start_iter, char_iter), Path_part_kind::file_name))
                return false;
        }
        return true;
    }
    void parse(format fmt = auto_format)
    {
        constexpr Char_type generic_separator = '/';
        auto last_part_kind = Path_part_kind::multiple_parts;
        std::size_t part_count = 0;
        bool need_generic_conversion = false;
        parse(value,
              [&](Path_index_range index_range, Path_part_kind part_kind) noexcept
              {
                  if(part_kind == Path_part_kind::path_separator)
                      return true;
                  if(generic_separator != preferred_separator && !need_generic_conversion)
                  {
                      for(std::size_t i = index_range.begin; i < index_range.end; i++)
                      {
                          if(is_separator(value[i]) && value[i] != generic_separator)
                          {
                              need_generic_conversion = true;
                              break;
                          }
                      }
                  }
                  last_part_kind = part_kind;
                  part_count++;
                  return true;
              },
              fmt);
        if(part_count == 1 && !need_generic_conversion)
        {
            kind = last_part_kind;
            parts.clear();
            return;
        }
        else
        {
            kind = Path_part_kind::multiple_parts;
        }
        while(parts.size() > part_count)
            parts.pop_back();
        parts.reserve(part_count);
        std::size_t part_index = 0;
        parse(value,
              [&](Path_index_range index_range, Path_part_kind part_kind)
              {
                  if(part_kind == Path_part_kind::path_separator)
                      return true;
                  if(part_index >= parts.size())
                      parts.emplace_back();
                  parts[part_index].value.assign(value.data() + index_range.begin,
                                                 index_range.size());
                  parts[part_index].kind = part_kind;
                  change_separator(parts[part_index].value, generic_separator);
                  part_index++;
                  return true;
              },
              fmt);
    }
    static Path_index_range get_filename_index_range(string_view_type value) noexcept
    {
        Path_index_range retval(value.size(), value.size());
        parse(value,
              [&](Path_index_range index_range, Path_part_kind part_kind) noexcept
              {
                  if(part_kind == Path_part_kind::file_name)
                      retval = index_range;
                  return true;
              });
        return retval;
    }
    static Path_index_range get_stem_index_range(string_view_type value) noexcept
    {
        return get_stem_index_range(value, get_filename_index_range(value));
    }
    static Path_index_range get_stem_index_range(string_view_type value,
                                                 Path_index_range filename_index_range) noexcept
    {
        constexpr Char_type dot = '.';
        if(filename_index_range.size() <= 1)
            return filename_index_range;
        for(std::size_t i = filename_index_range.end; i > filename_index_range.begin; i--)
        {
            if(value[i - 1] == dot)
            {
                if(i == filename_index_range.begin + 1)
                    return filename_index_range;
                if(i == filename_index_range.begin + 2 && value[filename_index_range.begin] == dot)
                    return filename_index_range;
                return Path_index_range(filename_index_range.begin, i - 1);
            }
        }
        return filename_index_range;
    }
    static Path_index_range get_extension_index_range(string_view_type value) noexcept
    {
        return get_extension_index_range(value, get_filename_index_range(value));
    }
    static Path_index_range get_extension_index_range(
        string_view_type value, Path_index_range filename_index_range) noexcept
    {
        return get_extension_index_range(
            value, filename_index_range, get_stem_index_range(value, filename_index_range));
    }
    static Path_index_range get_extension_index_range([[gnu::unused]] string_view_type value,
                                                      Path_index_range filename_index_range,
                                                      Path_index_range stem_index_range) noexcept
    {
        return Path_index_range(stem_index_range.end, filename_index_range.end);
    }
    static Path_index_range get_root_name_index_range(string_view_type value) noexcept
    {
        Path_index_range retval(0, 0);
        parse(value,
              [&](Path_index_range index_range, Path_part_kind part_kind) noexcept
              {
                  if(part_kind == Path_part_kind::absolute_root_name
                     || part_kind == Path_part_kind::relative_root_name)
                      retval = index_range;
                  return false;
              });
        return retval;
    }
    static Path_index_range get_root_dir_index_range(string_view_type value) noexcept
    {
        Path_index_range retval(0, 0);
        parse(value,
              [&](Path_index_range index_range, Path_part_kind part_kind) noexcept
              {
                  if(part_kind == Path_part_kind::root_dir)
                  {
                      retval = index_range;
                  }
                  else if(part_kind == Path_part_kind::absolute_root_name
                          || part_kind == Path_part_kind::relative_root_name)
                  {
                      retval = Path_index_range(index_range.end, index_range.end);
                      return true;
                  }
                  return false;
              });
        return retval;
    }
    static Path_index_range get_root_path_index_range(string_view_type value) noexcept
    {
        Path_index_range retval(0, 0);
        parse(value,
              [&](Path_index_range index_range, Path_part_kind part_kind) noexcept
              {
                  if(part_kind == Path_part_kind::absolute_root_name
                     || part_kind == Path_part_kind::relative_root_name)
                  {
                      retval = index_range;
                      return true;
                  }
                  else if(part_kind == Path_part_kind::root_dir)
                  {
                      retval.end = index_range.end;
                      return false;
                  }
                  return false;
              });
        return retval;
    }
    static Path_index_range get_relative_path_index_range(string_view_type value) noexcept
    {
        Path_index_range retval(value.size(), value.size());
        parse(value,
              [&](Path_index_range index_range, Path_part_kind part_kind) noexcept
              {
                  if(part_kind == Path_part_kind::absolute_root_name
                     || part_kind == Path_part_kind::relative_root_name
                     || part_kind == Path_part_kind::root_dir)
                  {
                      return true;
                  }
                  retval.begin = index_range.begin;
                  return false;
              });
        return retval;
    }
    static Path_index_range get_parent_path_index_range(string_view_type value) noexcept
    {
        Path_index_range retval(0, 0);
        std::size_t last_file_name_end_index = 0;
        parse(value,
              [&](Path_index_range index_range, Path_part_kind part_kind) noexcept
              {
                  switch(part_kind)
                  {
                  case Path_part_kind::path_separator:
                      return true;
                  case Path_part_kind::absolute_root_name:
                  case Path_part_kind::relative_root_name:
                  case Path_part_kind::root_dir:
                      retval.end = index_range.end;
                      return true;
                  case Path_part_kind::file_name:
                      if(last_file_name_end_index != 0)
                          retval.end = last_file_name_end_index;
                      last_file_name_end_index = index_range.end;
                      return true;
                  case Path_part_kind::multiple_parts:
                      break;
                  }
                  assert(false);
                  return false;
              });
        return retval;
    }

public:
    basic_path() noexcept : parts(), value(), kind(Path_part_kind::multiple_parts)
    {
    }
    basic_path(const basic_path &) = default;
    basic_path(basic_path &&rt) noexcept : parts(), value(), kind()
    {
        swap(rt);
    }
    basic_path(string_type &&source, format fmt = auto_format)
        : parts(), value(std::move(source)), kind()
    {
        parse(fmt);
    }
    template <typename Source>
    basic_path(const Source &source, format fmt = auto_format)
        : basic_path(detail::Path_convert_source<Char_type, Source>::to_string(source), fmt)
    {
    }
    template <typename Input_iterator>
    basic_path(Input_iterator first, Input_iterator last, format fmt = auto_format)
        : basic_path(detail::Path_convert_range<Char_type, Input_iterator>::to_string(first, last),
                     fmt)
    {
    }
    basic_path &operator=(const basic_path &rt) = default;
    basic_path &operator=(basic_path &&rt) noexcept
    {
        basic_path(std::move(rt)).swap(*this);
        return *this;
    }
    basic_path &operator=(string_type &&new_value)
    {
        value = std::move(new_value);
        parse();
        return *this;
    }
    template <typename Source>
    basic_path &operator=(const Source &source)
    {
        assign(source);
        return *this;
    }
    basic_path &assign(string_type &&new_value)
    {
        return operator=(new_value);
    }
    basic_path &assign(const string_type &new_value)
    {
        value = new_value;
        parse();
        return *this;
    }
    basic_path &assign(const string_view_type &new_value)
    {
        // use assign to prevent allocating a temporary string_type
        value.assign(new_value.data(), new_value.size());
        parse();
        return *this;
    }
    template <typename Source>
    basic_path &assign(const Source &source)
    {
        assign(detail::Path_convert_source<Char_type, Source>::to_string(source));
        return *this;
    }
    template <typename Input_iterator>
    basic_path &assign(Input_iterator first, Input_iterator last)
    {
        assign(detail::Path_convert_range<Char_type, Input_iterator>::to_string(first, last));
        return *this;
    }

private:
    template <bool Check_for_shared_memory = true>
    void append_string(string_view_type str)
    {
        bool just_need_to_assign = is_absolute(str);
        if(!just_need_to_assign && !get_root_name_index_range(str).empty())
        {
            auto my_root_name_index_range = get_root_name_index_range(value);
            auto str_root_name_index_range = get_root_name_index_range(str);
            if(my_root_name_index_range.empty()
               || string_view_type(value)
                          .substr(my_root_name_index_range.begin, my_root_name_index_range.size())
                      == str.substr(str_root_name_index_range.begin,
                                    str_root_name_index_range.size()))
                just_need_to_assign = true;
        }
        if(just_need_to_assign)
        {
            assign(str);
            return;
        }
        static_assert(std::is_same<typename string_view_type::iterator, const Char_type *>::value,
                      "");
        if(Check_for_shared_memory && str.begin() <= value.data() + value.size()
           && str.end() >= value.data())
        {
            // value and str share memory, reallocate str
            append_string<false>(static_cast<string_type>(str));
            return;
        }
        auto str_root_name_index_range = get_root_name_index_range(str);
        assert(str_root_name_index_range.begin == 0);
        str.remove_prefix(str_root_name_index_range.end);
        if(!get_root_dir_index_range(str).empty())
        {
            auto my_root_name_index_range = get_root_name_index_range(value);
            assert(my_root_name_index_range.begin == 0);
            value.resize(my_root_name_index_range.end);
        }
        else if(!get_filename_index_range(value).empty()
                || (get_root_dir_index_range(value).empty() && is_absolute()))
        {
            value.reserve(value.size() + 1 + str.size());
            value += preferred_separator;
        }
        value += str;
        parse();
    }

public:
    basic_path &operator/=(const basic_path &p)
    {
        append_string(p.value);
        return *this;
    }
    basic_path &operator/=(const string_type &p)
    {
        append_string(p);
    }
    basic_path &operator/=(const string_view_type &p)
    {
        append_string(p);
    }
    template <typename Source>
    basic_path &operator/=(const Source &source)
    {
        append_string(detail::Path_convert_source<Char_type, Source>::to_string(source));
        return *this;
    }
    template <typename Source>
    basic_path &append(const Source &source)
    {
        operator/=(source);
        return *this;
    }
    template <typename Input_iterator>
    basic_path &append(Input_iterator first, Input_iterator last)
    {
        append_string(
            detail::Path_convert_range<Char_type, Input_iterator>::to_string(first, last));
        return *this;
    }
    basic_path &operator+=(const basic_path &p)
    {
        value += p.value;
        parse();
        return *this;
    }
    basic_path &operator+=(const string_type &p)
    {
        value += p;
        parse();
        return *this;
    }
    basic_path &operator+=(const string_view_type &p)
    {
        value += p;
        parse();
        return *this;
    }
    template <typename Source>
    basic_path &operator+=(const Source &source)
    {
        value += detail::Path_convert_source<Char_type, Source>::to_string(source);
        parse();
        return *this;
    }
    template <typename Source>
    basic_path &concat(const Source &source)
    {
        operator+=(source);
        return *this;
    }
    template <typename Input_iterator>
    basic_path &concat(Input_iterator first, Input_iterator last)
    {
        value += detail::Path_convert_range<Char_type, Input_iterator>::to_string(first, last);
        parse();
        return *this;
    }
    const Char_type *c_str() const noexcept
    {
        return value.c_str();
    }
    const string_type &native() const noexcept
    {
        return value;
    }
    operator string_type() const
    {
        return value;
    }
    void clear() noexcept
    {
        value.clear();
        parse();
    }

private:
    template <typename Char_type2, typename Traits, typename Allocator>
    static void change_separator(std::basic_string<Char_type2, Traits, Allocator> &str,
                                 Char_type2 separator) noexcept
    {
        for(auto &ch : str)
        {
            if(is_separator(ch))
                ch = separator;
        }
    }
    basic_path &change_separator(Char_type separator) noexcept
    {
        change_separator(value, separator);
        for(auto &part : parts)
            change_separator(part.value, separator);
        return *this;
    }

public:
    basic_path &make_preferred() noexcept
    {
        change_separator(preferred_separator);
        return *this;
    }
    basic_path &remove_filename()
    {
        auto filename_index_range = get_filename_index_range(value);
        if(!filename_index_range.empty())
        {
            value.erase(filename_index_range.begin, filename_index_range.size());
            parse();
        }
        return *this;
    }
    basic_path &replace_filename(const basic_path &replacement)
    {
        remove_filename();
        operator/=(replacement);
        return *this;
    }
    basic_path &replace_extension(const basic_path &replacement = basic_path())
    {
        constexpr Char_type dot = '.';
        auto extension_index_range = get_extension_index_range(value);
        if(!extension_index_range.empty())
            value.erase(extension_index_range.begin, extension_index_range.size());
        else if(replacement.value.empty())
            return *this;
        if(!replacement.value.empty() && replacement.value.front() != dot)
        {
            value.reserve(value.size() + 1 + replacement.value.size());
            value += dot;
            value += replacement.value;
        }
        else
        {
            value += replacement.value;
        }
        parse();
        return *this;
    }
    void swap(basic_path &other) noexcept
    {
        using std::swap;
        swap(value, other.value);
        parts.swap(other.parts);
        swap(kind, other.kind);
    }
    bool has_root_path() const noexcept
    {
        return !get_root_path_index_range(value).empty();
    }
    bool has_root_name() const noexcept
    {
        return !get_root_name_index_range(value).empty();
    }
    bool has_root_directory() const noexcept
    {
        return !get_root_dir_index_range(value).empty();
    }
    bool has_relative_path() const noexcept
    {
        return !get_relative_path_index_range(value).empty();
    }
    bool has_parent_path() const noexcept
    {
        return !get_parent_path_index_range(value).empty();
    }
    bool has_filename() const noexcept
    {
        return !get_filename_index_range(value).empty();
    }
    bool has_stem() const noexcept
    {
        return !get_stem_index_range(value).empty();
    }
    bool has_extension() const noexcept
    {
        return !get_extension_index_range(value).empty();
    }

private:
    static bool is_absolute(string_view_type value) noexcept
    {
        bool has_root_dir = false;
        bool has_relative_root_name = false;
        bool has_absolute_root_name = false;
        parse(value,
              [&]([[gnu::unused]] Path_index_range index_range, Path_part_kind part_kind) noexcept
              {
                  if(part_kind == Path_part_kind::relative_root_name)
                  {
                      has_relative_root_name = true;
                      return true;
                  }
                  else if(part_kind == Path_part_kind::absolute_root_name)
                  {
                      has_absolute_root_name = true;
                      return false;
                  }
                  else if(part_kind == Path_part_kind::root_dir)
                  {
                      has_root_dir = true;
                  }
                  return false;
              });
        if(has_absolute_root_name)
            return true;
        if(has_root_dir)
        {
            if(Needs_root_name_to_be_absolute)
                return has_relative_root_name;
            return true;
        }
        return false;
    }

public:
    bool is_absolute() const noexcept
    {
        return is_absolute(value);
    }
    bool is_relative() const noexcept
    {
        return !is_absolute(value);
    }
    template <typename String_char_type,
              typename String_traits_type = std::char_traits<String_char_type>,
              typename Allocator = std::allocator<String_char_type>>
    std::basic_string<String_char_type, String_traits_type, Allocator> string(
        const Allocator &a = Allocator()) const
    {
        return detail::Path_convert_source<String_char_type,
                                           string_type>::template to_string<String_traits_type,
                                                                            Allocator>(value, a);
    }
    std::string string() const
    {
        return string<char>();
    }
    std::wstring wstring() const
    {
        return string<wchar_t>();
    }
    std::string u8string() const
    {
        return string<char>();
    }
    std::u16string u16string() const
    {
        return string<char16_t>();
    }
    std::u32string u32string() const
    {
        return string<char32_t>();
    }
    template <typename String_char_type,
              typename String_traits_type = std::char_traits<String_char_type>,
              typename Allocator = std::allocator<String_char_type>>
    std::basic_string<String_char_type, String_traits_type, Allocator> generic_string(
        const Allocator &a = Allocator()) const
    {
        auto retval =
            detail::Path_convert_source<String_char_type,
                                        string_type>::template to_string<String_traits_type,
                                                                         Allocator>(value, a);
        change_separator(retval, static_cast<String_char_type>('/'));
        return retval;
    }
    std::string generic_string() const
    {
        return generic_string<char>();
    }
    std::wstring generic_wstring() const
    {
        return generic_string<wchar_t>();
    }
    std::string generic_u8string() const
    {
        return generic_string<char>();
    }
    std::u16string generic_u16string() const
    {
        return generic_string<char16_t>();
    }
    std::u32string generic_u32string() const
    {
        return generic_string<char32_t>();
    }
    template <typename Stream_char_type, typename Stream_traits_type>
    friend std::basic_ostream<Stream_char_type, Stream_traits_type> &operator<<(
        std::basic_ostream<Stream_char_type, Stream_traits_type> &os, const basic_path &p)
    {
        os << std::quoted(p.string<Stream_char_type, Stream_traits_type>());
        return os;
    }
    template <typename Stream_char_type, typename Stream_traits_type>
    friend std::basic_istream<Stream_char_type, Stream_traits_type> &operator>>(
        std::basic_istream<Stream_char_type, Stream_traits_type> &is, basic_path &p)
    {
        std::basic_string<Stream_char_type, Stream_traits_type> str;
        is >> std::quoted(str);
        p = std::move(str);
        return is;
    }

private:
    static int compare_part(string_view_type a,
                            Path_part_kind a_kind,
                            string_view_type b,
                            Path_part_kind b_kind) noexcept
    {
        constexpr Char_type generic_separator_char = '/';
        constexpr string_view_type generic_separator(&generic_separator_char, 1);
        if(a_kind == Path_part_kind::root_dir)
            a = generic_separator;
        if(b_kind == Path_part_kind::root_dir)
            b = generic_separator;
        for(std::size_t i = 0; i < a.size() && i < b.size(); i++)
        {
            Char_type a_char = a[i];
            Char_type b_char = b[i];
            if(a_char == preferred_separator)
                a_char = generic_separator_char;
            if(b_char == preferred_separator)
                b_char = generic_separator_char;
            if(a_char < b_char)
                return -1;
            if(a_char > b_char)
                return 1;
        }
        if(a.size() < b.size())
            return -1;
        if(a.size() > b.size())
            return 1;
        return 0;
    }

public:
    int compare(string_view_type str) const noexcept
    {
        int retval;
        if(kind != Path_part_kind::multiple_parts)
        {
            retval = 1; // non-empty is more than empty
            parse(str,
                  [&](Path_index_range index_range, Path_part_kind part_kind) noexcept
                  {
                      if(part_kind == Path_part_kind::path_separator)
                          return true;
                      if(retval == 1) // initial value
                          retval = compare_part(value,
                                                kind,
                                                str.substr(index_range.begin, index_range.size()),
                                                part_kind);
                      else
                          retval = -1; // one-element is less than two-elements
                      return retval == 0;
                  });
        }
        else
        {
            retval = 0;
            auto part_iter = parts.begin();
            parse(str,
                  [&](Path_index_range index_range, Path_part_kind part_kind) noexcept
                  {
                      if(part_kind == Path_part_kind::path_separator)
                          return true;
                      if(part_iter == parts.end())
                      {
                          retval = -1; // empty is less than non-empty
                      }
                      else
                      {
                          retval = compare_part(part_iter->value,
                                                part_iter->kind,
                                                str.substr(index_range.begin, index_range.size()),
                                                part_kind);
                          ++part_iter;
                      }
                      return retval == 0;
                  });
            if(retval == 0 && part_iter != parts.end())
                retval = 1; // more-elements is more than fewer-elements
        }
        return retval;
    }
    int compare(const string_type &str) const noexcept
    {
        return compare(string_view_type(str));
    }
    int compare(const Char_type *str) const
    {
        return compare(string_view_type(str));
    }
    int compare(const basic_path &rt) const noexcept
    {
        return compare(rt.value);
    }
    iterator begin() const noexcept
    {
        return iterator(this, 0);
    }
    iterator end() const noexcept
    {
        return iterator(this, kind == Path_part_kind::multiple_parts ? parts.size() : 1);
    }
    basic_path root_name() const
    {
        auto iter = begin();
        if(iter == end())
            return {};
        if(iter->kind == Path_part_kind::relative_root_name
           || iter->kind == Path_part_kind::absolute_root_name)
            return *iter;
        return {};
    }
    basic_path root_directory() const
    {
        auto iter = begin();
        if(iter == end())
            return {};
        if(iter->kind == Path_part_kind::relative_root_name
           || iter->kind == Path_part_kind::absolute_root_name)
            ++iter;
        if(iter == end())
            return {};
        if(iter->kind == Path_part_kind::root_dir)
            return *iter;
        return {};
    }
    basic_path root_path() const
    {
        auto index_range = get_root_path_index_range(value);
        if(index_range.empty())
            return {};
        return value.substr(index_range.begin, index_range.size());
    }
    basic_path relative_path() const
    {
        auto index_range = get_relative_path_index_range(value);
        if(index_range.empty())
            return {};
        return value.substr(index_range.begin, index_range.size());
    }
    basic_path parent_path() const
    {
        auto index_range = get_parent_path_index_range(value);
        if(index_range.empty())
            return {};
        return value.substr(index_range.begin, index_range.size());
    }
    basic_path filename() const
    {
        auto iter = end();
        if(iter == begin())
            return {};
        --iter;
        if(iter->kind == Path_part_kind::file_name)
            return *iter;
        return {};
    }
    basic_path stem() const
    {
        auto index_range = get_stem_index_range(value);
        if(index_range.empty())
            return {};
        return value.substr(index_range.begin, index_range.size());
    }
    basic_path extension() const
    {
        auto index_range = get_extension_index_range(value);
        if(index_range.empty())
            return {};
        return value.substr(index_range.begin, index_range.size());
    }
    bool empty() const noexcept
    {
        return begin() == end();
    }
    basic_path lexically_normal() const
    {
        constexpr Char_type dot = '.';
        constexpr std::size_t dot_dot_size = 2;
        constexpr Char_type dot_dot_storage[dot_dot_size + 1] = {dot, dot};
        string_view_type dot_dot(dot_dot_storage, dot_dot_size);
        if(empty())
            return {};
        auto relative_path_index_range = get_relative_path_index_range(value);
        auto root_name_index_range = get_root_name_index_range(value);
        bool has_root_dir = has_root_directory();
        basic_path retval;
        retval.value.reserve(value.size());
        retval.value.assign(value.data() + relative_path_index_range.begin,
                            relative_path_index_range.size());
        std::size_t new_size = 0;
        for(std::size_t i = 0; i < retval.value.size(); i++)
        {
            if(is_separator(retval.value[i]))
            {
                while(i + 1 < retval.value.size() && is_separator(retval.value[i + 1]))
                    i++;
                retval.value[new_size++] = preferred_separator;
            }
            else
            {
                retval.value[new_size++] = retval.value[i];
            }
        }
        retval.value.resize(new_size);
        new_size = 0;
        bool last_was_separator = true;
        for(std::size_t i = 0; i < retval.value.size(); i++)
        {
            if(last_was_separator && retval.value[i] == dot)
            {
                if(i + 1 >= retval.value.size())
                    break; // don't write the dot
                if(retval.value[i + 1] == preferred_separator)
                {
                    i++;
                    last_was_separator = true;
                    continue; // skip the dot and separator
                }
            }
            if(retval.value[i] == preferred_separator)
                last_was_separator = true;
            else
                last_was_separator = false;
            retval.value[new_size++] = retval.value[i];
        }
        retval.value.resize(new_size);
        retval.parts.reserve(parts.size());
        new_size = 0;
        parse<true>(retval.value,
                    [&](Path_index_range index_range, Path_part_kind part_kind) noexcept
                    {
                        if(part_kind == Path_part_kind::path_separator)
                            return true;
                        assert(part_kind == Path_part_kind::file_name);
                        if(index_range.size() == 2 && retval.value[index_range.begin] == dot
                           && retval.value[index_range.begin + 1] == dot)
                        {
                            if(new_size == 0 && has_root_dir)
                                return true;
                            if(new_size != 0)
                            {
                                new_size--;
                                return true;
                            }
                        }
                        if(new_size >= retval.parts.size())
                            retval.parts.emplace_back();
                        retval.parts[new_size].value.assign(retval.value.data() + index_range.begin,
                                                            index_range.size());
                        retval.parts[new_size].kind = Path_part_kind::file_name;
                        new_size++;
                        return true;
                    });
        if(new_size >= 2 && retval.parts[new_size - 1].value.empty()
           && retval.parts[new_size - 2].value == dot_dot)
            new_size--;
        std::size_t needed_space = 0;
        if(!root_name_index_range.empty())
            needed_space++;
        if(has_root_dir)
            needed_space++;
        if(needed_space > 0)
        {
            while(retval.parts.size() < new_size + needed_space)
                retval.parts.emplace_back();
            for(std::size_t source = new_size - 1, target = new_size + needed_space - 1, i = 0;
                i < new_size;
                source--, target--, i++)
                retval.parts[target] = std::move(retval.parts[source]);
            std::size_t root_part_index = 0;
            if(!root_name_index_range.empty())
            {
                retval.parts[root_part_index].value.assign(
                    value.data() + root_name_index_range.begin, root_name_index_range.size());
                change_separator(retval.parts[root_part_index].value, static_cast<Char_type>('/'));
                retval.parts[root_part_index].parts = Parts();
                retval.parts[root_part_index].kind = begin()->kind;
                root_part_index++;
            }
            if(has_root_dir)
            {
                retval.parts[root_part_index].value.assign(1, static_cast<Char_type>('/'));
                retval.parts[root_part_index].parts = Parts();
                retval.parts[root_part_index].kind = Path_part_kind::root_dir;
            }
        }
        if(new_size + needed_space == 0)
        {
            if(retval.parts.empty())
                retval.parts.emplace_back();
            retval.parts[new_size].value.assign(1, dot);
            retval.parts[new_size].parts = Parts();
            retval.parts[new_size].kind = Path_part_kind::file_name;
            new_size++;
        }
        while(retval.parts.size() > new_size + needed_space)
            retval.parts.pop_back();
        retval.value.clear();
        bool need_seperator = false;
        for(auto &part : retval.parts)
        {
            switch(part.kind)
            {
            case Path_part_kind::absolute_root_name:
            case Path_part_kind::relative_root_name:
                retval.value += part.value;
                change_separator(retval.value, preferred_separator);
                need_seperator = false;
                // absolute_root_name will be followed by root_dir if we need a seperator
                continue;
            case Path_part_kind::file_name:
                if(need_seperator)
                    retval.value += preferred_separator;
                retval.value += part.value;
                need_seperator = true;
                continue;
            case Path_part_kind::root_dir:
                retval.value += preferred_separator;
                need_seperator = false;
                continue;
            case Path_part_kind::path_separator:
            case Path_part_kind::multiple_parts:
                break;
            }
            assert(false);
        }
        retval.parse();
        return retval;
    }
#warning finish
};

template <detail::Path_traits_kind Traits_kind,
          typename Char_type,
          Char_type Preferred_separator,
          bool Needs_root_name_to_be_absolute>
void swap(
    basic_path<Traits_kind, Char_type, Preferred_separator, Needs_root_name_to_be_absolute> &l,
    basic_path<Traits_kind, Char_type, Preferred_separator, Needs_root_name_to_be_absolute>
        &r) noexcept
{
    l.swap(r);
}

template <detail::Path_traits_kind Traits_kind,
          typename Char_type,
          Char_type Preferred_separator,
          bool Needs_root_name_to_be_absolute>
constexpr Char_type basic_path<Traits_kind,
                               Char_type,
                               Preferred_separator,
                               Needs_root_name_to_be_absolute>::preferred_separator;

template <detail::Path_traits_kind Traits_kind,
          typename Char_type,
          Char_type Preferred_separator,
          bool Needs_root_name_to_be_absolute>
basic_path<Traits_kind, Char_type, Preferred_separator, Needs_root_name_to_be_absolute>
    *basic_path<Traits_kind, Char_type, Preferred_separator, Needs_root_name_to_be_absolute>::
        Parts::allocate(std::size_t count)
{
    if(count == 0)
        return nullptr;
    return std::allocator<basic_path>().allocate(count);
}

template <detail::Path_traits_kind Traits_kind,
          typename Char_type,
          Char_type Preferred_separator,
          bool Needs_root_name_to_be_absolute>
void basic_path<Traits_kind, Char_type, Preferred_separator, Needs_root_name_to_be_absolute>::
    Parts::deallocate(basic_path *values, std::size_t count) noexcept
{
    if(count != 0)
        std::allocator<basic_path>().deallocate(values, count);
}

typedef basic_path<> path;
}
}
}

#endif /* UTIL_FILESYSTEM_H_ */
