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
#include <new>
#include <stdexcept>
#include <cassert>
#include <tuple>
#include "constexpr_array.h"
#include "bitset.h"
#include "is_swappable.h"

namespace kazan
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
 * <code>kazan_util_generate_enum_traits(Enum, Enum::Value1, Enum::Value2, Enum::Value3,
 * <...>);</code> */
#define kazan_util_generate_enum_traits(Enum, ...)                                \
    [[gnu::unused]] ::kazan::util::detail::Default_enum_traits<Enum, __VA_ARGS__> \
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
    constexpr iterator insert([[gnu::unused]] iterator hint, T value) noexcept
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
    iterator lower_bound(T value) const noexcept
    {
        return iterator::first_at_or_after(this, Enum_traits<T>::find_value(value));
    }
    iterator upper_bound(T value) const noexcept
    {
        std::size_t index = Enum_traits<T>::find_value(value);
        if(index >= bits.size())
            return end();
        auto retval = iterator::first_at_or_after(this, index);
        if(retval.index == index)
            ++retval;
        return retval;
    }
    std::pair<iterator, iterator> equal_range(T value) const noexcept
    {
        std::size_t index = Enum_traits<T>::find_value(value);
        if(index < bits.size())
        {
            auto first = iterator::first_at_or_after(this, index);
            auto last = first;
            if(first.index == index)
                ++last;
            return {first, last};
        }
        return {end(), end()};
    }
};

#if 0
#warning finish implementing Enum_map
#else
namespace detail
{
template <typename T,
          typename Mapped_type,
          std::size_t Entry_count,
          bool Is_Trivially_Destructible = std::is_trivially_destructible<T>::value,
          bool Is_Trivially_Copyable = std::is_trivially_copyable<T>::value>
struct Enum_map_base
{
    union Entry_type
    {
        T full_value;
        alignas(T) char empty_value[sizeof(T)];
        constexpr Entry_type() noexcept : empty_value{}
        {
        }
        ~Entry_type()
        {
        }
    };
    typedef util::bitset<Entry_count> Bits;
    Entry_type entries[Entry_count];
    Bits full_entries;
    void erase_at_index(std::size_t index) noexcept
    {
        entries[index].full_value.~T();
        full_entries[index] = false;
    }
    template <typename... Args>
    void emplace_at_index(std::size_t index,
                          Args &&... args) noexcept(noexcept(new(std::declval<void *>())
                                                                 T(std::declval<Args>()...)))
    {
        new(const_cast<void *>(static_cast<const void *>(
            std::addressof(entries[index].full_value)))) T(std::forward<Args>(args)...);
        full_entries[index] = true;
    }
    constexpr Enum_map_base() noexcept : entries{}, full_entries(0)
    {
    }
    Enum_map_base(const Enum_map_base &rt) noexcept(noexcept(new(std::declval<void *>())
                                                                 T(std::declval<const T &>())))
        : entries{}, full_entries(0)
    {
        try
        {
            operator=(rt);
        }
        catch(...)
        {
            clear();
            throw;
        }
    }
    Enum_map_base(Enum_map_base &&rt) noexcept(noexcept(new(std::declval<void *>())
                                                            T(std::declval<T &&>())))
        : entries{}, full_entries(0)
    {
        try
        {
            operator=(std::move(rt));
        }
        catch(...)
        {
            clear();
            throw;
        }
    }
    Enum_map_base &operator=(const Enum_map_base &rt) noexcept(
        noexcept(new(std::declval<void *>()) T(std::declval<const T &>()))
        && noexcept(std::declval<Mapped_type &>() = std::declval<const Mapped_type &>()))
    {
        auto either_full = full_entries | rt.full_entries;
        for(std::size_t index = either_full.find_first(true); index != Bits::npos;
            index = either_full.find_first(true, index + 1))
            if(rt.full_entries[index])
                if(full_entries[index])
                    std::get<1>(entries[index].full_value) =
                        std::get<1>(rt.entries[index].full_value);
                else
                    emplace_at_index(index, rt.entries[index].full_value);
            else
                erase_at_index(index);
        return *this;
    }
    Enum_map_base &operator=(Enum_map_base &&rt) noexcept(
        noexcept(new(std::declval<void *>()) T(std::declval<T &&>()))
        && noexcept(std::declval<Mapped_type &>() = std::declval<Mapped_type &&>()))
    {
        auto either_full = full_entries | rt.full_entries;
        for(std::size_t index = either_full.find_first(true); index != Bits::npos;
            index = either_full.find_first(true, index + 1))
            if(rt.full_entries[index])
                if(full_entries[index])
                    std::get<1>(entries[index].full_value) =
                        std::move(std::get<1>(rt.entries[index].full_value));
                else
                    emplace_at_index(index, std::move(rt.entries[index].full_value));
            else
                erase_at_index(index);
        return *this;
    }
    void clear() noexcept
    {
        for(std::size_t index = full_entries.find_first(true); index != Bits::npos;
            index = full_entries.find_first(true, index + 1))
            erase_at_index(index);
    }
    ~Enum_map_base()
    {
        clear();
    }
};

template <typename T, typename Mapped_type, std::size_t Entry_count>
struct Enum_map_base<T, Mapped_type, Entry_count, true, false>
{
    union Entry_type
    {
        T full_value;
        alignas(T) char empty_value[sizeof(T)];
        constexpr Entry_type() noexcept : empty_value{}
        {
        }
    };
    typedef util::bitset<Entry_count> Bits;
    Entry_type entries[Entry_count];
    Bits full_entries;
    constexpr void erase_at_index(std::size_t index) noexcept
    {
        full_entries[index] = false;
    }
    template <typename... Args>
    void emplace_at_index(std::size_t index,
                          Args &&... args) noexcept(noexcept(new(std::declval<void *>())
                                                                 T(std::declval<Args>()...)))
    {
        new(const_cast<void *>(static_cast<const void *>(
            std::addressof(entries[index].full_value)))) T(std::forward<Args>(args)...);
        full_entries[index] = true;
    }
    constexpr Enum_map_base() noexcept : entries{}, full_entries(0)
    {
    }
    Enum_map_base(const Enum_map_base &rt) noexcept(noexcept(new(std::declval<void *>())
                                                                 T(std::declval<const T &>())))
        : entries{}, full_entries(0)
    {
        try
        {
            operator=(rt);
        }
        catch(...)
        {
            clear();
            throw;
        }
    }
    Enum_map_base(Enum_map_base &&rt) noexcept(noexcept(new(std::declval<void *>())
                                                            T(std::declval<T &&>())))
        : entries{}, full_entries(0)
    {
        try
        {
            operator=(std::move(rt));
        }
        catch(...)
        {
            clear();
            throw;
        }
    }
    Enum_map_base &operator=(const Enum_map_base &rt) noexcept(
        noexcept(new(std::declval<void *>()) T(std::declval<const T &>()))
        && noexcept(std::declval<Mapped_type &>() = std::declval<const Mapped_type &>()))
    {
        auto either_full = full_entries | rt.full_entries;
        for(std::size_t index = either_full.find_first(true); index != Bits::npos;
            index = either_full.find_first(true, index + 1))
            if(rt.full_entries[index])
                if(full_entries[index])
                    std::get<1>(entries[index].full_value) =
                        std::get<1>(rt.entries[index].full_value);
                else
                    emplace_at_index(index, rt.entries[index].full_value);
            else
                erase_at_index(index);
        return *this;
    }
    Enum_map_base &operator=(Enum_map_base &&rt) noexcept(
        noexcept(new(std::declval<void *>()) T(std::declval<T &&>()))
        && noexcept(std::declval<Mapped_type &>() = std::declval<Mapped_type &&>()))
    {
        auto either_full = full_entries | rt.full_entries;
        for(std::size_t index = either_full.find_first(true); index != Bits::npos;
            index = either_full.find_first(true, index + 1))
            if(rt.full_entries[index])
                if(full_entries[index])
                    std::get<1>(entries[index].full_value) =
                        std::move(std::get<1>(rt.entries[index].full_value));
                else
                    emplace_at_index(index, std::move(rt.entries[index].full_value));
            else
                erase_at_index(index);
        return *this;
    }
    constexpr void clear() noexcept
    {
        for(std::size_t index = full_entries.find_first(true); index != Bits::npos;
            index = full_entries.find_first(true, index + 1))
            erase_at_index(index);
    }
    ~Enum_map_base() = default;
};

template <typename T, typename Mapped_type, std::size_t Entry_count>
struct Enum_map_base<T, Mapped_type, Entry_count, true, true>
{
    union Entry_type
    {
        T full_value;
        alignas(T) char empty_value[sizeof(T)];
        constexpr Entry_type() noexcept : empty_value{}
        {
        }
    };
    typedef util::bitset<Entry_count> Bits;
    Entry_type entries[Entry_count];
    Bits full_entries;
    constexpr void erase_at_index(std::size_t index) noexcept
    {
        full_entries[index] = false;
    }
    template <typename... Args>
    void emplace_at_index(std::size_t index,
                          Args &&... args) noexcept(noexcept(new(std::declval<void *>())
                                                                 T(std::declval<Args>()...)))
    {
        new(const_cast<void *>(static_cast<const void *>(
            std::addressof(entries[index].full_value)))) T(std::forward<Args>(args)...);
        full_entries[index] = true;
    }
    constexpr Enum_map_base() noexcept : entries{}, full_entries(0)
    {
    }
    constexpr Enum_map_base(const Enum_map_base &rt) noexcept = default;
    constexpr Enum_map_base(Enum_map_base &&rt) noexcept = default;
    Enum_map_base &operator=(const Enum_map_base &rt) noexcept(
        noexcept(new(std::declval<void *>()) T(std::declval<const T &>()))
        && noexcept(std::declval<Mapped_type &>() = std::declval<const Mapped_type &>()))
    {
        auto either_full = full_entries | rt.full_entries;
        for(std::size_t index = either_full.find_first(true); index != Bits::npos;
            index = either_full.find_first(true, index + 1))
            if(rt.full_entries[index])
                if(full_entries[index])
                    std::get<1>(entries[index].full_value) =
                        std::get<1>(rt.entries[index].full_value);
                else
                    emplace_at_index(index, rt.entries[index].full_value);
            else
                erase_at_index(index);
        return *this;
    }
    Enum_map_base &operator=(Enum_map_base &&rt) noexcept(
        noexcept(new(std::declval<void *>()) T(std::declval<T &&>()))
        && noexcept(std::declval<Mapped_type &>() = std::declval<Mapped_type &&>()))
    {
        auto either_full = full_entries | rt.full_entries;
        for(std::size_t index = either_full.find_first(true); index != Bits::npos;
            index = either_full.find_first(true, index + 1))
            if(rt.full_entries[index])
                if(full_entries[index])
                    std::get<1>(entries[index].full_value) =
                        std::move(std::get<1>(rt.entries[index].full_value));
                else
                    emplace_at_index(index, std::move(rt.entries[index].full_value));
            else
                erase_at_index(index);
        return *this;
    }
    constexpr void clear() noexcept
    {
        for(std::size_t index = full_entries.find_first(true); index != Bits::npos;
            index = full_entries.find_first(true, index + 1))
            erase_at_index(index);
    }
    ~Enum_map_base() = default;
};
}

/** behaves like a std::map<K, V> */
template <typename K, typename V>
class Enum_map
    : private detail::Enum_map_base<std::pair<const K, V>, V, Enum_traits<K>::value_count>
{
private:
    typedef detail::Enum_map_base<std::pair<const K, V>, V, Enum_traits<K>::value_count> Base;

public:
    typedef K key_type;
    typedef V mapped_type;
    typedef std::pair<const K, V> value_type;
    typedef std::size_t size_type;
    typedef std::ptrdiff_t difference_type;
    typedef value_type &reference;
    typedef const value_type &const_reference;
    typedef value_type *pointer;
    typedef const value_type *const_pointer;

private:
    using typename Base::Entry_type;
    using typename Base::Bits;
    using Base::entries;
    using Base::full_entries;
    using Base::emplace_at_index;
    using Base::erase_at_index;

public:
    constexpr Enum_map() noexcept = default;
    constexpr Enum_map(const Enum_map &) noexcept(noexcept(Base(std::declval<const Base &>()))) =
        default;
    constexpr Enum_map(Enum_map &&) noexcept(noexcept(Base(std::declval<Base &&>()))) = default;
    constexpr Enum_map &operator=(const Enum_map &) noexcept(
        noexcept(std::declval<Base &>() = std::declval<const Base &>())) = default;
    constexpr Enum_map &operator=(Enum_map &&) noexcept(
        noexcept(std::declval<Base &>() = std::declval<Base &&>())) = default;
    constexpr void clear() noexcept
    {
        Base::clear();
    }
    template <typename Input_iterator>
    Enum_map(Input_iterator first, Input_iterator last) noexcept(
        noexcept(std::declval<Enum_map &>().insert(std::move(first), std::move(last))))
        : Enum_map()
    {
        insert(std::move(first), std::move(last));
    }
    Enum_map(std::initializer_list<value_type> il) noexcept(
        noexcept(std::declval<Enum_map &>().insert(il.begin(), il.end())))
        : Enum_map()
    {
        insert(il.begin(), il.end());
    }
    Enum_map &operator=(std::initializer_list<value_type> il) noexcept(
        noexcept(std::declval<Enum_map &>().insert(il.begin(), il.end())))
    {
        clear();
        insert(il.begin(), il.end());
        return *this;
    }
    constexpr V &at(K key)
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        if(index >= full_entries.size() || !full_entries[index])
            throw std::out_of_range("Enum_map::at");
        return std::get<1>(entries[index].full_value);
    }
    constexpr const V &at(K key) const
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        if(index >= full_entries.size() || !full_entries[index])
            throw std::out_of_range("Enum_map::at");
        return std::get<1>(entries[index].full_value);
    }
    V &operator[](K key) noexcept(noexcept(std::declval<Enum_map &>().emplace_at_index(
        0, std::piecewise_construct, std::forward_as_tuple(key), std::make_tuple())))
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        assert(index < full_entries.size());
        if(!full_entries[index])
            emplace_at_index(
                index, std::piecewise_construct, std::forward_as_tuple(key), std::make_tuple());
        return std::get<1>(entries[index].full_value);
    }
    constexpr bool empty() const noexcept
    {
        return full_entries.none();
    }
    constexpr std::size_t size() const noexcept
    {
        return full_entries.count();
    }
    constexpr std::size_t max_size() const noexcept
    {
        return full_entries.size();
    }
    constexpr void swap(Enum_map &other) noexcept(noexcept(Base(std::declval<Base &&>()))
                                                  && util::is_nothrow_swappable<V>::value)
    {
        auto either_full = full_entries | other.full_entries;
        using std::swap;
        for(std::size_t index = either_full.find_first(true); index != Bits::npos;
            index = either_full.find_first(true, index + 1))
        {
            if(other.full_entries[index])
            {
                if(full_entries[index])
                {
                    swap(entries[index].full_value, other.entries[index].full_value);
                }
                else
                {
                    emplace_at_index(index, std::move(other.entries[index].full_value));
                    other.erase_at_index(index);
                }
            }
            else
            {
                other.emplace_at_index(index, std::move(entries[index].full_value));
                erase_at_index(index);
            }
        }
    }
    constexpr std::size_t count(K key) const noexcept
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        if(index >= full_entries.size() || !full_entries[index])
            return 0;
        return 1;
    }
    class const_iterator;
    class iterator
    {
        template <typename, typename>
        friend class Enum_map;
        friend class const_iterator;

    public:
        typedef std::ptrdiff_t difference_type;
        typedef std::pair<const K, V> value_type;
        typedef value_type *pointer;
        typedef value_type &reference;
        std::bidirectional_iterator_tag iterator_category;

    private:
        Enum_map *map;
        std::size_t index;

    private:
        constexpr iterator(Enum_map *map, std::size_t index) noexcept : map(map), index(index)
        {
        }
        static constexpr iterator first_at_or_after(Enum_map *map, std::size_t index) noexcept
        {
            return iterator(map, map->full_entries.find_first(true, index));
        }
        static constexpr iterator first_at_or_before(Enum_map *map, std::size_t index) noexcept
        {
            return iterator(map, map->full_entries.find_last(true, index));
        }

    public:
        constexpr iterator() noexcept : map(nullptr), index(0)
        {
        }
        friend constexpr bool operator==(const iterator &l, const iterator &r) noexcept
        {
            return l.index == r.index && l.map == r.map;
        }
        friend constexpr bool operator!=(const iterator &l, const iterator &r) noexcept
        {
            return !operator==(l, r);
        }
        constexpr iterator &operator++() noexcept
        {
            *this = first_at_or_after(map, index + 1);
            return *this;
        }
        constexpr iterator &operator--() noexcept
        {
            *this = first_at_or_before(map, index + 1);
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
        constexpr value_type &operator*() const noexcept
        {
            return map->entries[index].full_value;
        }
        constexpr value_type *operator->() const noexcept
        {
            return &operator*();
        }
    };
    class const_iterator
    {
        template <typename, typename>
        friend class Enum_map;

    public:
        typedef std::ptrdiff_t difference_type;
        typedef std::pair<const K, V> value_type;
        typedef const value_type *pointer;
        typedef const value_type &reference;
        std::bidirectional_iterator_tag iterator_category;

    private:
        const Enum_map *map;
        std::size_t index;

    private:
        constexpr const_iterator(const Enum_map *map, std::size_t index) noexcept : map(map),
                                                                                    index(index)
        {
        }
        static constexpr const_iterator first_at_or_after(const Enum_map *map,
                                                          std::size_t index) noexcept
        {
            return const_iterator(map, map->full_entries.find_first(true, index));
        }
        static constexpr const_iterator first_at_or_before(const Enum_map *map,
                                                           std::size_t index) noexcept
        {
            return const_iterator(map, map->full_entries.find_last(true, index));
        }

    public:
        constexpr const_iterator() noexcept : map(nullptr), index(0)
        {
        }
        constexpr const_iterator(const iterator &iter) noexcept : map(iter.map), index(iter.index)
        {
        }
        friend constexpr bool operator==(const const_iterator &l, const const_iterator &r) noexcept
        {
            return l.index == r.index && l.map == r.map;
        }
        friend constexpr bool operator!=(const const_iterator &l, const const_iterator &r) noexcept
        {
            return !operator==(l, r);
        }
        friend constexpr bool operator==(const iterator &l, const const_iterator &r) noexcept
        {
            return operator==(const_iterator(l), r);
        }
        friend constexpr bool operator!=(const iterator &l, const const_iterator &r) noexcept
        {
            return operator!=(const_iterator(l), r);
        }
        friend constexpr bool operator==(const const_iterator &l, const iterator &r) noexcept
        {
            return operator==(l, const_iterator(r));
        }
        friend constexpr bool operator!=(const const_iterator &l, const iterator &r) noexcept
        {
            return operator!=(l, const_iterator(r));
        }
        constexpr const_iterator &operator++() noexcept
        {
            *this = first_at_or_after(map, index + 1);
            return *this;
        }
        constexpr const_iterator &operator--() noexcept
        {
            *this = first_at_or_before(map, index + 1);
            return *this;
        }
        constexpr const_iterator operator++(int) noexcept
        {
            auto retval = *this;
            operator++();
            return retval;
        }
        constexpr const_iterator operator--(int) noexcept
        {
            auto retval = *this;
            operator--();
            return retval;
        }
        constexpr const value_type &operator*() const noexcept
        {
            return map->entries[index].full_value;
        }
        constexpr const value_type *operator->() const noexcept
        {
            return &operator*();
        }
    };
    typedef std::reverse_iterator<iterator> reverse_iterator;
    typedef std::reverse_iterator<const_iterator> const_reverse_iterator;
    constexpr iterator begin() noexcept
    {
        return iterator::first_at_or_after(this, 0);
    }
    constexpr const_iterator begin() const noexcept
    {
        return const_iterator::first_at_or_after(this, 0);
    }
    constexpr const_iterator cbegin() const noexcept
    {
        return const_iterator::first_at_or_after(this, 0);
    }
    constexpr iterator end() noexcept
    {
        return iterator(this, Bits::npos);
    }
    constexpr const_iterator end() const noexcept
    {
        return const_iterator(this, Bits::npos);
    }
    constexpr const_iterator cend() const noexcept
    {
        return const_iterator(this, Bits::npos);
    }
    constexpr reverse_iterator rbegin() noexcept
    {
        return reverse_iterator(end());
    }
    constexpr const_reverse_iterator rbegin() const noexcept
    {
        return const_reverse_iterator(end());
    }
    constexpr const_reverse_iterator crbegin() const noexcept
    {
        return const_reverse_iterator(end());
    }
    constexpr reverse_iterator rend() noexcept
    {
        return reverse_iterator(begin());
    }
    constexpr const_reverse_iterator rend() const noexcept
    {
        return const_reverse_iterator(begin());
    }
    constexpr const_reverse_iterator crend() const noexcept
    {
        return const_reverse_iterator(begin());
    }
    std::pair<iterator, bool> insert(const value_type &value) noexcept(
        noexcept(Base(std::declval<const Base &>())))
    {
        std::size_t index = Enum_traits<K>::find_value(std::get<0>(value));
        assert(index < full_entries.size());
        if(!full_entries[index])
        {
            emplace_at_index(index, value);
            return {iterator(this, index), true};
        }
        return {iterator(this, index), false};
    }
    std::pair<iterator, bool> insert(value_type &&value) noexcept(
        noexcept(Base(std::declval<Base &&>())))
    {
        std::size_t index = Enum_traits<K>::find_value(std::get<0>(value));
        assert(index < full_entries.size());
        if(!full_entries[index])
        {
            emplace_at_index(index, std::move(value));
            return {iterator(this, index), true};
        }
        return {iterator(this, index), false};
    }
    iterator insert([[gnu::unused]] const_iterator hint,
                    const value_type &value) noexcept(noexcept(Base(std::declval<const Base &>())))
    {
        return std::get<0>(insert(value));
    }
    iterator insert([[gnu::unused]] const_iterator hint,
                    value_type &&value) noexcept(noexcept(Base(std::declval<Base &&>())))
    {
        return std::get<0>(insert(std::move(value)));
    }
    template <typename... Args>
    std::pair<iterator, bool> emplace(Args &&... args) noexcept(
        noexcept(std::declval<Enum_map>().insert(value_type(std::declval<Args>()...))))
    {
        return insert(value_type(std::forward<Args>(args)...));
    }
    template <typename T>
    typename std::enable_if<std::is_constructible<value_type, T &&>::value,
                            std::pair<iterator, bool>>::type
        insert(T &&value) noexcept(noexcept(std::declval<Enum_map>().emplace(std::declval<T &&>())))
    {
        return emplace(std::forward<T>(value));
    }
    template <typename T>
    typename std::enable_if<std::is_constructible<value_type, T &&>::value, iterator>::type insert(
        [[gnu::unused]] const_iterator hint,
        T &&value) noexcept(noexcept(std::declval<Enum_map>().emplace(std::declval<T &&>())))
    {
        return std::get<0>(emplace(std::forward<T>(value)));
    }
    template <typename Input_iterator>
    void insert(Input_iterator first,
                Input_iterator last) noexcept(noexcept(std::declval<Enum_map>().emplace(*first))
                                              && noexcept(first != last ? 0 : 0)
                                              && noexcept(++first))
    {
        for(; first != last; ++first)
            emplace(*first);
    }
    void insert(std::initializer_list<value_type> il) noexcept(
        noexcept(std::declval<Enum_map>().insert(il.begin(), il.end())))
    {
        insert(il.begin(), il.end());
    }
    template <typename T>
    std::pair<iterator, bool> insert_or_assign(K key, T &&mapped_value) noexcept(
        noexcept(std::declval<V &>() = std::declval<T &&>())
        && noexcept(std::declval<Enum_map>().emplace_at_index(0, key, std::declval<T &&>()))
        && noexcept(Base(std::declval<Base &&>())))
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        assert(index < full_entries.size());
        if(!full_entries[index])
        {
            emplace_at_index(index, key, std::forward<T>(mapped_value));
            return {iterator(this, index), true};
        }
        std::get<1>(entries[index].full_value) = std::forward<T>(mapped_value);
        return {iterator(this, index), false};
    }
    template <typename T>
    iterator
        insert_or_assign([[gnu::unused]] const_iterator hint, K key, T &&mapped_value) noexcept(
            noexcept(std::declval<Enum_map>().insert_or_assign(key, std::declval<T &&>())))
    {
        return std::get<0>(insert_or_assign(key, std::forward<T>(mapped_value)));
    }
    template <typename... Args>
    iterator emplace_hint([[gnu::unused]] const_iterator hint, Args &&... args) noexcept(
        noexcept(std::declval<Enum_map>().emplace(std::declval<Args &&>()...)))
    {
        return std::get<0>(emplace(std::forward<Args>(args)...));
    }
    template <typename... Args>
    std::pair<iterator, bool> try_emplace(K key, Args &&... args) noexcept(
        noexcept(std::declval<Enum_map>().emplace_at_index(
            0,
            std::piecewise_construct,
            std::forward_as_tuple(key),
            std::forward_as_tuple(std::forward<Args>(args)...))))
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        assert(index < full_entries.size());
        if(!full_entries[index])
        {
            emplace_at_index(index,
                             std::piecewise_construct,
                             std::forward_as_tuple(key),
                             std::forward_as_tuple(std::forward<Args>(args)...));
            return {iterator(this, index), true};
        }
        return {iterator(this, index), false};
    }
    template <typename... Args>
    iterator try_emplace([[gnu::unused]] const_iterator hint, K key, Args &&... args) noexcept(
        noexcept(std::declval<Enum_map>().try_emplace(key, std::declval<Args &&>()...)))
    {
        return std::get<0>(try_emplace(key, std::forward<Args>(args)...));
    }
    iterator erase(const_iterator pos) noexcept
    {
        erase_at_index(pos.index);
        ++pos;
        return iterator(this, pos.index);
    }
    iterator erase(iterator pos) noexcept
    {
        return erase(const_iterator(pos));
    }
    iterator erase(const_iterator first, const_iterator last) noexcept
    {
        if(first == last)
            return end();
        iterator retval = erase(first);
        while(retval != last)
            retval = erase(retval);
        return retval;
    }
    std::size_t erase(K key) noexcept
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        if(index < full_entries.size() || !full_entries[index])
            return 0;
        erase_at_index(index);
        return 1;
    }
    iterator find(K key) noexcept
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        if(index < full_entries.size() || !full_entries[index])
            return end();
        return iterator(this, index);
    }
    const_iterator find(K key) const noexcept
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        if(index < full_entries.size() || !full_entries[index])
            return end();
        return const_iterator(this, index);
    }
    const_iterator lower_bound(K key) const noexcept
    {
        return const_iterator::first_at_or_after(this, Enum_traits<K>::find_value(key));
    }
    const_iterator upper_bound(K key) const noexcept
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        if(index >= full_entries.size())
            return end();
        auto retval = const_iterator::first_at_or_after(this, index);
        if(retval.index == index)
            ++retval;
        return retval;
    }
    std::pair<const_iterator, const_iterator> equal_range(K key) const noexcept
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        if(index < full_entries.size())
        {
            auto first = const_iterator::first_at_or_after(this, index);
            auto last = first;
            if(first.index == index)
                ++last;
            return {first, last};
        }
        return {end(), end()};
    }
    iterator lower_bound(K key) noexcept
    {
        return iterator::first_at_or_after(this, Enum_traits<K>::find_value(key));
    }
    iterator upper_bound(K key) noexcept
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        if(index >= full_entries.size())
            return end();
        auto retval = iterator::first_at_or_after(this, index);
        if(retval.index == index)
            ++retval;
        return retval;
    }
    std::pair<iterator, iterator> equal_range(K key) noexcept
    {
        std::size_t index = Enum_traits<K>::find_value(key);
        if(index < full_entries.size())
        {
            auto first = iterator::first_at_or_after(this, index);
            auto last = first;
            if(first.index == index)
                ++last;
            return {first, last};
        }
        return {end(), end()};
    }
};
#endif
}
}

#endif /* UTIL_ENUM_H_ */
