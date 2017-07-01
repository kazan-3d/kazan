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

#ifndef UTIL_ENUM_H_
#define UTIL_ENUM_H_

#include <type_traits>
#include <utility>
#include <iterator>
#include "constexpr_array.h"
#include "bitset.h"

namespace vulkan_cpu
{
namespace util
{
template <typename T>
void enum_traits_resolve_function(T) = delete;

template <typename T>
struct Enum_traits
{
private:
    typedef decltype(enum_traits_resolve_function(T())) base;

public:
    static constexpr std::size_t value_count = base::values.size();
    static constexpr Constexpr_array<T, value_count> values = base::values;
    typedef typename std::underlying_type<T>::type underlying_type;

private:
    static constexpr bool is_compact_helper() noexcept
    {
        for(std::size_t i = 0; i < value_count; i++)
            if(i
               != static_cast<std::size_t>(static_cast<underlying_type>(values[i]))
                      - static_cast<std::size_t>(static_cast<underlying_type>(values.front())))
                return false;
        return true;
    }

public:
    static constexpr bool is_compact = is_compact_helper();
    struct Value_and_index
    {
        T value;
        std::size_t index;
        constexpr Value_and_index() noexcept : value(), index()
        {
        }
        constexpr Value_and_index(T value, std::size_t index) noexcept : value(value), index(index)
        {
        }
    };

private:
    template <std::size_t N>
    static constexpr Constexpr_array<Value_and_index, N> sort_value_index_map(
        const Value_and_index *value_index_map) noexcept
    {
        // uses merge sort algorithm
        if(N == 0)
            return {};
        Constexpr_array<Value_and_index, N> retval{};
        if(N == 1)
        {
            retval[0] = value_index_map[0];
            return retval;
        }

        // split
        constexpr std::size_t split_index = N / 2;
        constexpr std::size_t part1_size = split_index;
        constexpr std::size_t part2_size = N - part1_size;
        auto part1 = sort_value_index_map<part1_size>(value_index_map);
        auto part2 = sort_value_index_map<part2_size>(value_index_map + split_index);

        // merge, preserving order of equal values
        std::size_t part1_index = 0;
        std::size_t part2_index = 0;
        std::size_t retval_index = 0;
        while(part1_index < part1_size && part2_index < part2_size)
        {
            // we want to copy from part1 if values are equal
            if(static_cast<underlying_type>(part2[part2_index].value)
               < static_cast<underlying_type>(part1[part1_index].value))
                retval[retval_index++] = part2[part2_index++];
            else
                retval[retval_index++] = part1[part1_index++];
        }
        while(part1_index < part1_size)
            retval[retval_index++] = part1[part1_index++];
        while(part2_index < part2_size)
            retval[retval_index++] = part2[part2_index++];
        return retval;
    }
    static constexpr Constexpr_array<Value_and_index, value_count>
        make_sorted_value_index_map() noexcept
    {
        Constexpr_array<Value_and_index, value_count> retval{};
        for(std::size_t i = 0; i < value_count; i++)
            retval[i] = {values[i], i};
        retval = sort_value_index_map<value_count>(retval.data());
        return retval;
    }

public:
    static constexpr Constexpr_array<Value_and_index, value_count> sorted_value_index_map =
        make_sorted_value_index_map();
    static constexpr std::size_t npos = -1;
    /** find first occurrence of value in values and return index if found, otherwise return npos */
    static constexpr std::size_t find_value(T value) noexcept
    {
        std::size_t retval{};
        constexpr std::size_t binary_search_transition = 8;
        if(is_compact)
        {
            retval = static_cast<std::size_t>(static_cast<underlying_type>(value))
                     - static_cast<std::size_t>(static_cast<underlying_type>(values.front()));
        }
        else if(value_count < binary_search_transition)
        {
            retval = npos;
            for(std::size_t i = 0; i < value_count; i++)
            {
                if(values[i] == value)
                {
                    retval = i;
                    break;
                }
            }
        }
        else
        {
            retval = 0;
            std::size_t count = value_count;
            while(count != 0)
            {
                std::size_t step = count / 2;
                if(static_cast<underlying_type>(values[retval + step])
                   < static_cast<underlying_type>(value))
                {
                    retval += step + 1;
                    count -= step + 1;
                }
                else
                {
                    count = step;
                }
            }
        }
        if(retval >= value_count)
            return npos;
        return retval;
    }
};

template <typename T>
constexpr std::size_t Enum_traits<T>::value_count;

template <typename T>
constexpr Constexpr_array<T, Enum_traits<T>::value_count> Enum_traits<T>::values;

template <typename T>
constexpr bool Enum_traits<T>::is_compact;

template <typename T>
constexpr Constexpr_array<typename Enum_traits<T>::Value_and_index, Enum_traits<T>::value_count>
    Enum_traits<T>::sorted_value_index_map;

template <typename T>
constexpr std::size_t Enum_traits<T>::npos;

namespace detail
{
template <typename Enum, Enum... Values>
struct Default_enum_traits
{
    static constexpr Constexpr_array<Enum, sizeof...(Values)> values = {{Values...}};
};

template <typename Enum, Enum... Values>
constexpr Constexpr_array<Enum, sizeof...(Values)> Default_enum_traits<Enum, Values...>::values;
/** generate code for Enum_traits instantiation; use like
 * <code>vulkan_cpu_util_generate_enum_traits(Enum, Enum::Value1, Enum::Value2, Enum::Value3,
 * <...>);</code> */
#define vulkan_cpu_util_generate_enum_traits(Enum, ...)                \
    ::vulkan_cpu::util::detail::Default_enum_traits<Enum, __VA_ARGS__> \
        enum_traits_resolve_function(Enum)
}

/** behaves like a std::set<T> */
template <typename T>
class Enum_set
{
private:
    typedef util::bitset<Enum_traits<T>::value_count> Bits;

public:
    typedef T key_type;
    typedef T value_type;
    typedef std::size_t size_type;
    typedef std::ptrdiff_t difference_type;
    typedef T &reference;
    typedef const T &const_reference;
    typedef T *pointer;
    typedef const T *const_pointer;
    class iterator
    {
        template <typename>
        friend class Enum_set;

    public:
        typedef std::ptrdiff_t difference_type;
        typedef T value_type;
        typedef const T *pointer;
        typedef const T &reference;
        std::bidirectional_iterator_tag iterator_category;

    private:
        const Enum_set *enum_set;
        std::size_t index;

    public:
        constexpr iterator() noexcept : enum_set(nullptr), index(0)
        {
        }

    private:
        constexpr iterator(const Enum_set *enum_set, std::size_t index) noexcept
            : enum_set(enum_set),
              index(index)
        {
        }
        static constexpr iterator first_at_or_after(const Enum_set *enum_set,
                                                    std::size_t index) noexcept
        {
            return iterator(enum_set, enum_set->bits.find_first(true, index));
        }
        static constexpr iterator first_at_or_before(const Enum_set *enum_set,
                                                     std::size_t index) noexcept
        {
            return iterator(enum_set, enum_set->bits.find_last(true, index));
        }

    public:
        constexpr bool operator==(const iterator &rt) const noexcept
        {
            return index == rt.index && enum_set == rt.enum_set;
        }
        constexpr bool operator!=(const iterator &rt) const noexcept
        {
            return !operator==(rt);
        }
        constexpr iterator &operator++() noexcept
        {
            *this = first_at_or_after(enum_set, index + 1);
            return *this;
        }
        constexpr iterator &operator--() noexcept
        {
            *this = first_at_or_before(enum_set, index - 1);
            return *this;
        }
        constexpr iterator operator++(int) noexcept
        {
            auto retval = *this;
            operator++();
            return retval;
        }
        constexpr iterator operator--(int) noexcept
        {
            auto retval = *this;
            operator--();
            return retval;
        }
        constexpr const T &operator*() const noexcept
        {
            return Enum_traits<T>::values[index];
        }
        constexpr const T *operator->() const noexcept
        {
            return &operator*();
        }
    };
    typedef iterator const_iterator;
    typedef std::reverse_iterator<iterator> reverse_iterator;
    typedef reverse_iterator reverse_const_iterator;

private:
    Bits bits;

public:
    constexpr Enum_set() noexcept : bits(0)
    {
    }
    template <typename Iter>
    constexpr Enum_set(Iter first, Iter last)
        : Enum_set()
    {
        insert(std::move(first), std::move(last));
    }
    constexpr Enum_set(std::initializer_list<T> il) noexcept : Enum_set(il.begin(), il.end())
    {
    }
    constexpr Enum_set &operator=(std::initializer_list<T> il) noexcept
    {
        *this = Enum_set(il);
        return *this;
    }
    constexpr iterator begin() const noexcept
    {
        return iterator::first_at_or_after(this, 0);
    }
    constexpr iterator end() const noexcept
    {
        return iterator(this, Bits::npos);
    }
    constexpr iterator cbegin() const noexcept
    {
        return begin();
    }
    constexpr iterator cend() const noexcept
    {
        return end();
    }
    constexpr bool empty() const noexcept
    {
        return bits.none();
    }
    constexpr std::size_t size() const noexcept
    {
        return bits.count();
    }
    constexpr std::size_t max_size() const noexcept
    {
        return bits.size();
    }
    constexpr void clear() noexcept
    {
        bits = Bits();
    }
    constexpr std::pair<iterator, bool> insert(T value) noexcept
    {
        std::size_t index = Enum_traits<T>::find_value(value);
        bool inserted = !bits[index];
        bits[index] = true;
        return {iterator(this, index), inserted};
    }
    constexpr iterator insert(iterator hint, T value) noexcept
    {
        std::size_t index = Enum_traits<T>::find_value(value);
        bits[index] = true;
        return iterator(this, index);
    }
    template <typename Iter>
    constexpr void insert(Iter first, Iter last)
    {
        for(; first != last; ++first)
            insert(*first);
    }
    constexpr void insert(std::initializer_list<T> il) noexcept
    {
        insert(il.begin(), il.end());
    }
    template <typename... Args>
    std::pair<iterator, bool> emplace(Args &&... args)
    {
        return insert(T(std::forward<Args>(args)...));
    }
    template <typename... Args>
    iterator emplace_hint(iterator hint, Args &&... args)
    {
        return insert(hint, T(std::forward<Args>(args)...));
    }
    constexpr std::size_t erase(T value) noexcept
    {
        std::size_t index = Enum_traits<T>::find_value(value);
        std::size_t retval = 0;
        if(index < bits.size())
        {
            retval = bits[index] ? 1 : 0;
            bits[index] = false;
        }
        return retval;
    }
    constexpr iterator erase(iterator pos) noexcept
    {
        auto retval = pos;
        ++retval;
        bits[pos.index] = false;
        return retval;
    }
    constexpr iterator erase(iterator first, iterator last) noexcept
    {
        while(first != last)
            first = erase(first);
        return first;
    }
    /** invalidates all iterators, all references still valid because they are bound to static
     * objects */
    constexpr void swap(Enum_set &other) noexcept
    {
        using std::swap;
        swap(bits, other.bits);
    }
    constexpr std::size_t count(T value) const noexcept
    {
        std::size_t index = Enum_traits<T>::find_value(value);
        if(index < bits.size() && bits[index])
            return 1;
        return 0;
    }
    constexpr iterator find(T value) const noexcept
    {
        std::size_t index = Enum_traits<T>::find_value(value);
        if(index < bits.size() && bits[index])
            return iterator(this, index);
        return 0;
    }
    std::pair<iterator, iterator> equal_range(T value) const noexcept
    {
        std::size_t index = Enum_traits<T>::find_value(value);
        if(index < bits.size() && bits[index])
        {
            auto first = iterator(this, index);
            auto last = first;
            ++last;
            return {first, last};
        }
        return {end(), end()};
    }
};

#if 1
#warning finish implementing Enum_map
#else
namespace detail
{
template <typename T,
          bool Is_Trivially_Destructible = std::is_trivially_destructible<T>::value,
          bool Is_Trivially_Copyable = std::is_trivially_copyable<T>::value>
struct Enum_map_base
{
    union Entry_type
    {
        T full_value;
        alignas(T) char empty_value[sizeof(T)];
    };
};
}

/** behaves like a std::map<K, V> */
template <typename K, typename V>
class Enum_map
{
public:
    typedef K key_type;
    typedef V mapped_type;
    typedef std::pair<const K, V> value_type;

private:
    union
    {
        T full_value;
        alignas(T) char empty_value[sizeof(T)];
    };
};
#endif
}
}

#endif /* UTIL_ENUM_H_ */
