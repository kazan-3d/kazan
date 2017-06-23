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

#error finish

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
};

template <>
struct Path_traits<Path_traits_kind::windows>
{
    typedef wchar_t value_type;
    static constexpr value_type preferred_separator = L'\\';
};

enum class Path_kind
{
    root_name,
    root_dir,
    file_name,
    multiple_parts,
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
            return *base_iterator == value_type();
        }
        if(rt.base_iterator)
            return *rt.base_iterator == value_type();
        return true;
    }
    bool operator!=(const Path_convert_single_iterator_adaptor &rt) const
    {
        return !operator==(rt);
    }
    bool operator==(Path_iterator_sentinel) const
    {
        if(base_iterator)
            return *base_iterator == value_type();
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
            encode_result = text::Decode_encode_functions<Dest_char_type>::encode(ch);
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
    const char32_t &operator*() const noexcept
    {
        return encode_result[encode_result_index];
    }
    const char32_t *operator->() const noexcept
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
    template <typename Sentinel>
    static std::basic_string<Path_char_type> to_string(Iterator iterator, Sentinel sentinel)
    {
        typedef Path_convert_iterator<Path_char_type, Iterator, Sentinel> Convert_iterator;
        return std::basic_string<Path_char_type>(Convert_iterator(iterator, sentinel),
                                                 Convert_iterator());
    }
};

template <typename Iterator>
struct Path_convert_range<typename Path_is_convertable_iterator_type<Iterator>::Char_type,
                          Iterator,
                          void>
{
    static constexpr bool is_convertible = true;
    typedef typename Path_is_convertable_iterator_type<Iterator>::Char_type Char_type;
    static std::basic_string<Char_type> to_string(Iterator iterator, Iterator sentinel)
    {
        return std::basic_string<Char_type>(iterator, sentinel);
    }
    template <typename Sentinel>
    static std::basic_string<Char_type> to_string(Iterator iterator, Sentinel sentinel)
    {
        std::basic_string<Char_type> retval;
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
    static std::basic_string<Path_char_type> to_string(
        const std::basic_string<Source_char_type, Traits, Allocator> &source)
    {
        return Convert_range::to_string(source.begin(), source.end());
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
    static std::basic_string<Path_char_type> to_string(
        const basic_string_view<Source_char_type, Traits> &source)
    {
        return Convert_range::to_string(source.begin(), source.end());
    }
};

template <typename Char_type>
struct Path_convert_source<Char_type,
                           std::basic_string<Char_type>,
                           typename std::
                               enable_if<Path_convert_range<Char_type,
                                                            typename std::basic_string<Char_type>::
                                                                const_iterator>::is_convertible>::
                                   type>
{
    static constexpr bool is_convertible = true;
    static std::basic_string<Char_type> to_string(const std::basic_string<Char_type> &source)
    {
        return source;
    }
};

template <typename Char_type, typename Iterator>
struct Path_convert_source<Char_type,
                           Iterator,
                           void_t<typename Path_is_convertable_iterator_type<Iterator>::Char_type>>
{
    static constexpr bool is_convertible = true;
    typedef Path_convert_range<Char_type, Path_convert_single_iterator_adaptor<Iterator>>
        Convert_range;
    static std::basic_string<Char_type> to_string(Iterator iterator)
    {
        return Convert_range::to_string(Path_convert_single_iterator_adaptor<Iterator>(iterator),
                                        Path_iterator_sentinel());
    }
};

template <typename Char_type>
struct Path_convert_source<Char_type, const Char_type *, void>
{
    static constexpr bool is_convertible = true;
    static std::basic_string<Char_type> to_string(const Char_type *source)
    {
        return source;
    }
};

#error finish

template <Path_traits_kind Traits_kind = default_path_traits_kind,
          typename Char_type = typename Path_traits<Traits_kind>::value_type,
          Char_type Preferred_separator = Path_traits<Traits_kind>::preferred_separator>
class basic_path
{
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
    Path_kind kind;

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
    static constexpr bool is_separator(Char_type v) noexcept
    {
        return v == static_cast<Char_type>('/') || v == preferred_separator;
    }
    template <typename Fn>
    static void parse(string_view_type value, Fn callback, format fmt = auto_format) noexcept(
        noexcept(callback(typename string_view_type::iterator(),
                          typename string_view_type::iterator(),
                          Path_kind())))
    {
        constexpr Char_type dot = '.';
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
            return;
        if(Traits_kind == Path_traits_kind::windows && value.size() >= 2
           && is_ascii_letter(value[0])
           && value[1] == colon)
        {
            char_iter += 2;
            callback(value.begin(), char_iter, Path_kind::root_name);
        }
        else if(Traits_kind == Path_traits_kind::windows && value.size() >= 2
                && is_separator(value[0])
                && is_separator(value[1]))
        {
            while(peek() != eof && is_separator(peek()))
                get();
            while(peek() != eof && !is_separator(peek()))
                get();
            callback(value.begin(), char_iter, Path_kind::root_name);
        }
        if(peek() != eof && is_separator(peek()))
        {
            auto start_iter = char_iter;
            do
            {
                get();
            } while(peek() != eof && is_separator(peek()));
            callback(start_iter, char_iter, Path_kind::root_dir);
        }
        if(peek() != eof && !is_separator(peek()))
        {
            auto start_iter = char_iter;
            do
            {
                get();
            } while(peek() != eof && !is_separator(peek()));
            callback(start_iter, char_iter, Path_kind::file_name);
        }
        while(peek() != eof)
        {
            do
            {
                get();
            } while(peek() != eof && is_separator(peek()));
            auto start_iter = char_iter;
            while(peek() != eof && !is_separator(peek()))
                get();
            callback(start_iter, char_iter, Path_kind::file_name);
        }
    }
    void parse(format fmt = auto_format)
    {
        auto last_part_kind = Path_kind::multiple_parts;
        std::size_t part_count = 0;
        parse(value,
              [&]([[gnu::unused]] typename string_view_type::iterator part_string_begin,
                  [[gnu::unused]] typename string_view_type::iterator part_string_end,
                  Path_kind part_kind) noexcept
              {
                  last_part_kind = part_kind;
                  part_count++;
              },
              fmt);
        if(part_count == 1)
        {
            kind = last_part_kind;
            parts.clear();
            return;
        }
        else
        {
            kind = Path_kind::multiple_parts;
        }
        while(parts.size() > part_count)
            parts.pop_back();
        parts.reserve(part_count);
        std::size_t part_index = 0;
        parse(value,
              [&](typename string_view_type::iterator part_string_begin,
                  typename string_view_type::iterator part_string_end,
                  Path_kind part_kind) noexcept
              {
                  if(part_index >= parts.size())
                      parts.emplace_back();
                  parts[part_index].value.assign(part_string_begin, part_string_end);
                  parts[part_index].kind = part_kind;
                  part_index++;
              },
              fmt);
    }
    static void convert_source(string_type &output_value, const string_type &source)
    {
        output_value = source;
    }
    template <typename Char_type2, typename Traits, typename Allocator>
    static void convert_source(string_type &output_value,
                               const std::basic_string<Char_type2, Traits, Allocator> &source)
    {
        convert_source(output_value, source.begin(), source.end());
    }
    template <typename Char_type2, typename Traits>
    static void convert_source(string_type &output_value,
                               const basic_string_view<Char_type2, Traits> &source)
    {
        convert_source(output_value, source.begin(), source.end());
    }
    template <typename Char_type2>
    static void convert_source(string_type &output_value, const Char_type2 *source)
    {
        convert_source(output_value, basic_string_view<Char_type2>(source));
    }
    template <

        public : basic_path() noexcept : parts(),
        value(),
        kind(Path_kind::multiple_parts)
    {
    }
    basic_path(const basic_path &) = default;
    basic_path(basic_path &&) noexcept = default;
    basic_path(string_type &&source, format fmt = auto_format)
        : parts(), value(std::move(source)), kind()
    {
        parse(fmt);
    }
    template <typename Source>
    basic_path(const Source &source, format fmt = auto_format)
        : basic_path()
    {
        convert_source(value, source);
        parse(fmt);
    }
    template <typename Input_iterator>
    basic_path(Input_iterator first, Input_iterator last, format fmt = auto_format)
        : basic_path()
    {
        convert_source(value, first, last);
        parse(fmt);
    }
};

template <Path_traits_kind Traits_kind, typename Char_type, Char_type Preferred_separator>
constexpr Char_type basic_path<Traits_kind, Char_type, Preferred_separator>::preferred_separator;

template <Path_traits_kind Traits_kind, typename Char_type, Char_type Preferred_separator>
basic_path<Traits_kind, Char_type, Preferred_separator>
    *basic_path<Traits_kind, Char_type, Preferred_separator>::Parts::allocate(std::size_t count)
{
    if(count == 0)
        return nullptr;
    return std::allocator<basic_path>::allocate(count);
}

template <Path_traits_kind Traits_kind, typename Char_type, Char_type Preferred_separator>
void basic_path<Traits_kind, Char_type, Preferred_separator>::Parts::deallocate(
    basic_path *values, std::size_t count) noexcept
{
    if(count != 0)
        std::allocator<basic_path>::deallocate(values, count);
}
}
}
}
}

#endif /* UTIL_FILESYSTEM_H_ */
