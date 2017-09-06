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
#ifndef SPIRV_LITERAL_STRING_H_
#define SPIRV_LITERAL_STRING_H_

#include <iterator>
#include <string>
#include <cstdint>
#include <type_traits>
#include <stdexcept>
#include <cassert>
#include <ostream>
#include "word.h"
#include "util/endian.h"
#include "util/string_view.h"

namespace kazan
{
namespace spirv
{
/** reference to a SPIR-V string */
class Literal_string
{
    static_assert(sizeof(Word) == 4, "");
    static_assert(std::is_same<unsigned char, std::uint8_t>::value, "");

public:
    constexpr Literal_string() noexcept : begin_iter(), byte_count(0)
    {
    }
    constexpr Literal_string(const Word *word_array, std::size_t byte_count) noexcept
        : begin_iter(word_array, 0),
          byte_count(byte_count)
    {
    }
    class const_iterator
    {
        friend class Literal_string;

    public:
        typedef std::ptrdiff_t difference_type;
        typedef char value_type;
        typedef const char *pointer;
        typedef const char &reference;
        typedef std::random_access_iterator_tag iterator_category;

    private:
        const Word *word_array;
        std::size_t index;

    private:
        static constexpr std::size_t get_memory_offset(std::size_t index) noexcept
        {
            switch(util::endian)
            {
            case util::Endian::Big:
                static_assert((sizeof(Word) & (sizeof(Word) - 1)) == 0,
                              "sizeof(Word) is not a power of 2");
                return index ^ (sizeof(Word) - 1);
            case util::Endian::Little:
                return index;
            }
        }

    private:
        constexpr explicit const_iterator(const Word *word_array, std::size_t index) noexcept
            : word_array(word_array),
              index(index)
        {
        }
        static const char *get_memory_pointer(const Word *word_array, std::size_t index) noexcept
        {
            return reinterpret_cast<const char *>(word_array) + get_memory_offset(index);
        }

    public:
        constexpr const_iterator() noexcept : word_array(nullptr), index(0)
        {
        }
        const char *operator->() const noexcept
        {
            return get_memory_pointer(word_array, index);
        }
        const char &operator*() const noexcept
        {
            return *get_memory_pointer(word_array, index);
        }
        constexpr const_iterator &operator++() noexcept
        {
            index++;
            return *this;
        }
        constexpr const_iterator &operator--() noexcept
        {
            index--;
            return *this;
        }
        constexpr const_iterator operator++(int) noexcept
        {
            return const_iterator(word_array, index++);
        }
        constexpr const_iterator operator--(int) noexcept
        {
            auto retval = *this;
            operator--();
            return retval;
        }
        constexpr const_iterator &operator+=(std::ptrdiff_t offset) noexcept
        {
            index += offset;
            return *this;
        }
        constexpr const_iterator &operator-=(std::ptrdiff_t offset) noexcept
        {
            index -= offset;
            return *this;
        }
        friend constexpr const_iterator operator+(std::ptrdiff_t offset,
                                                  const_iterator iter) noexcept
        {
            return iter += offset;
        }
        friend constexpr const_iterator operator+(const_iterator iter,
                                                  std::ptrdiff_t offset) noexcept
        {
            return iter += offset;
        }
        friend constexpr const_iterator operator-(const_iterator iter,
                                                  std::ptrdiff_t offset) noexcept
        {
            return iter -= offset;
        }
        friend constexpr std::ptrdiff_t operator-(const_iterator l, const_iterator r) noexcept
        {
            return static_cast<std::ptrdiff_t>(l.index) - static_cast<std::ptrdiff_t>(r.index);
        }
        const char &operator[](std::ptrdiff_t offset) const noexcept
        {
            return *get_memory_pointer(word_array, index + offset);
        }
        constexpr bool operator==(const const_iterator &r) noexcept
        {
            return index == r.index;
        }
        constexpr bool operator!=(const const_iterator &r) noexcept
        {
            return index != r.index;
        }
        constexpr bool operator<=(const const_iterator &r) noexcept
        {
            return index <= r.index;
        }
        constexpr bool operator>=(const const_iterator &r) noexcept
        {
            return index >= r.index;
        }
        constexpr bool operator<(const const_iterator &r) noexcept
        {
            return index < r.index;
        }
        constexpr bool operator>(const const_iterator &r) noexcept
        {
            return index > r.index;
        }
    };
    typedef const_iterator iterator;
    typedef std::reverse_iterator<const_iterator> const_reverse_iterator;
    typedef const_reverse_iterator reverse_iterator;
    constexpr std::size_t size() const noexcept
    {
        return byte_count;
    }
    constexpr const_iterator begin() const noexcept
    {
        return begin_iter;
    }
    constexpr const_iterator end() const noexcept
    {
        return begin_iter + byte_count;
    }
    constexpr const_iterator cbegin() const noexcept
    {
        return begin_iter;
    }
    constexpr const_iterator cend() const noexcept
    {
        return begin_iter + byte_count;
    }
    const_reverse_iterator rbegin() const noexcept
    {
        return const_reverse_iterator(end());
    }
    const_reverse_iterator rend() const noexcept
    {
        return const_reverse_iterator(begin());
    }
    const_reverse_iterator crbegin() const noexcept
    {
        return const_reverse_iterator(end());
    }
    const_reverse_iterator crend() const noexcept
    {
        return const_reverse_iterator(begin());
    }
    const char &operator[](std::size_t index) const noexcept
    {
        assert(index < byte_count);
        return begin()[index];
    }
    const char &front() const noexcept
    {
        return operator[](0);
    }
    const char &back() const noexcept
    {
        return operator[](byte_count - 1);
    }
    constexpr bool empty() const noexcept
    {
        return byte_count == 0;
    }
    constexpr void swap(Literal_string &rt) noexcept
    {
        auto temp = *this;
        *this = rt;
        rt = temp;
    }
    constexpr void remove_prefix(std::size_t count) noexcept
    {
        assert(count <= byte_count);
        begin_iter += count;
        byte_count -= count;
    }
    constexpr void remove_suffix(std::size_t count) noexcept
    {
        assert(count <= byte_count);
        byte_count -= count;
    }
    static constexpr std::size_t npos = -1;
    constexpr Literal_string substr(std::size_t pos = 0, std::size_t count = npos) const
    {
        if(pos > byte_count)
            throw std::out_of_range("Literal_string::substr");
        auto retval = *this;
        retval.remove_prefix(pos);
        if(count < retval.byte_count)
            retval.byte_count = count;
        return retval;
    }

private:
    template <typename T>
    constexpr int compare_implementation(T rt) const noexcept
    {
        auto l_iter = begin();
        auto r_iter = rt.begin();
        for(; l_iter != end() && r_iter != rt.end(); ++l_iter, ++r_iter)
        {
            unsigned char l_char = *l_iter;
            unsigned char r_char = *r_iter;
            if(l_char < r_char)
                return -1;
            if(l_char > r_char)
                return 1;
        }
        if(l_iter != end())
            return 1;
        if(r_iter != rt.end())
            return -1;
        return 0;
    }

public:
    constexpr int compare(Literal_string rt) const noexcept
    {
        return compare_implementation(rt);
    }
    constexpr int compare(util::string_view rt) const noexcept
    {
        return compare_implementation(rt);
    }
    int compare(const char *rt) const noexcept
    {
        return compare(util::string_view(rt));
    }
    constexpr int compare(std::size_t l_pos, std::size_t l_count, Literal_string rt) const
    {
        return substr(l_pos, l_count).compare(rt);
    }
    constexpr int compare(std::size_t l_pos, std::size_t l_count, util::string_view rt) const
    {
        return substr(l_pos, l_count).compare(rt);
    }
    constexpr int compare(std::size_t l_pos, std::size_t l_count, const char *rt) const
    {
        return substr(l_pos, l_count).compare(rt);
    }
    constexpr int compare(std::size_t l_pos,
                          std::size_t l_count,
                          Literal_string rt,
                          std::size_t r_pos,
                          std::size_t r_count) const
    {
        return substr(l_pos, l_count).compare(rt.substr(r_pos, r_count));
    }
    constexpr int compare(std::size_t l_pos,
                          std::size_t l_count,
                          util::string_view rt,
                          std::size_t r_pos,
                          std::size_t r_count) const
    {
        return substr(l_pos, l_count).compare(rt.substr(r_pos, r_count));
    }
    constexpr int compare(std::size_t l_pos,
                          std::size_t l_count,
                          const char *rt,
                          std::size_t r_count) const
    {
        return substr(l_pos, l_count).compare(util::string_view(rt, r_count));
    }
    template <typename Allocator>
    explicit operator std::basic_string<char, std::char_traits<char>, Allocator>() const
    {
        return std::basic_string<char, std::char_traits<char>, Allocator>(begin(), end());
    }
    friend constexpr bool operator==(Literal_string a, Literal_string b) noexcept
    {
        return a.compare(b) == 0;
    }
    friend constexpr bool operator==(util::string_view a, Literal_string b) noexcept
    {
        return b.compare(a) == 0;
    }
    friend constexpr bool operator==(Literal_string a, util::string_view b) noexcept
    {
        return a.compare(b) == 0;
    }
    friend constexpr bool operator!=(Literal_string a, Literal_string b) noexcept
    {
        return a.compare(b) != 0;
    }
    friend constexpr bool operator!=(util::string_view a, Literal_string b) noexcept
    {
        return b.compare(a) != 0;
    }
    friend constexpr bool operator!=(Literal_string a, util::string_view b) noexcept
    {
        return a.compare(b) != 0;
    }
    friend constexpr bool operator>(Literal_string a, Literal_string b) noexcept
    {
        return a.compare(b) > 0;
    }
    friend constexpr bool operator>(util::string_view a, Literal_string b) noexcept
    {
        return b.compare(a) < 0;
    }
    friend constexpr bool operator>(Literal_string a, util::string_view b) noexcept
    {
        return a.compare(b) > 0;
    }
    friend constexpr bool operator<(Literal_string a, Literal_string b) noexcept
    {
        return a.compare(b) < 0;
    }
    friend constexpr bool operator<(util::string_view a, Literal_string b) noexcept
    {
        return b.compare(a) > 0;
    }
    friend constexpr bool operator<(Literal_string a, util::string_view b) noexcept
    {
        return a.compare(b) < 0;
    }
    friend constexpr bool operator>=(Literal_string a, Literal_string b) noexcept
    {
        return a.compare(b) >= 0;
    }
    friend constexpr bool operator>=(util::string_view a, Literal_string b) noexcept
    {
        return b.compare(a) <= 0;
    }
    friend constexpr bool operator>=(Literal_string a, util::string_view b) noexcept
    {
        return a.compare(b) >= 0;
    }
    friend constexpr bool operator<=(Literal_string a, Literal_string b) noexcept
    {
        return a.compare(b) <= 0;
    }
    friend constexpr bool operator<=(util::string_view a, Literal_string b) noexcept
    {
        return b.compare(a) >= 0;
    }
    friend constexpr bool operator<=(Literal_string a, util::string_view b) noexcept
    {
        return a.compare(b) <= 0;
    }
    friend std::ostream &operator<<(std::ostream &os, Literal_string v)
    {
        os << static_cast<std::string>(v);
        return os;
    }

private:
    const_iterator begin_iter;
    std::size_t byte_count;
};
}
}

#endif /* SPIRV_LITERAL_STRING_H_ */
