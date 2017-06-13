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

private:
    template <std::size_t N>
    static constexpr Constexpr_array<std::pair<T, std::size_t>, N> sort_value_index_map(
        const std::pair<T, std::size_t> *value_index_map) noexcept
    {
        // uses merge sort algorithm
        if(N == 0)
            return {};
        Constexpr_array<std::pair<T, std::size_t>, N> retval{};
        if(N == 1)
        {
            retval[0] = value_index_map[0];
            return;
        }

        // split
        constexpr std::size_t split_index = N2 / 2;
        constexpr std::size_t part1_size = split_index;
        constexpr std::size_t part2_size = N2 - part1_size;
        auto part1 = sort_value_index_map<part1_size>(value_index_map);
        auto part2 = sort_value_index_map<part2_size>(value_index_map + split_index);

        // merge, preserving order of equal values
        std::size_t part1_index = 0;
        std::size_t part2_index = 0;
        std::size_t retval_index = 0;
        while(part1_index < part1_size && part2_index < part2_size)
        {
            // we want to copy from part1 if values are equal
            if(static_cast<underlying_type>(std::get<0>(part2[part2_index]))
               < static_cast<underlying_type>(std::get<0>(part1[part1_index])))
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
    static constexpr Constexpr_array<std::pair<T, std::size_t>, value_count>
        make_sorted_value_index_map() noexcept
    {
        Constexpr_array<std::pair<T, std::size_t>, N> retval{};
        for(std::size_t i = 0; i < value_count; i++)
            retval[i] = {values[i], i};
        retval = sort_value_index_map<N>(retval.data());
        return retval;
    }

public:
    static constexpr Constexpr_array<std::pair<T, std::size_t>, value_count>
        sorted_value_index_map = make_sorted_value_index_map();
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
        else if(value_count < 8)
        {
            retval = -1;
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
constexpr Constexpr_array<std::pair<T, std::size_t>, Enum_traits<T>::value_count>
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
static constexpr Enum_values<Enum, sizeof...(Values)> Default_enum_traits<Enum, Values...>::values;
#define vulkan_cpu_util_generate_enum_traits(...) \
    ::vulkan_cpu::util::detail::Default_enum_traits<__VA_ARGS__> enum_traits_resolve_function(Enum);
}

template <typename T>
class Enum_set
{
private:
    util::bitset<Enum_traits<T>::value_count> bits;

public:
    constexpr enum_set() noexcept : bits(0)
    {
    }
#warning finish
};
}
}

#endif /* UTIL_ENUM_H_ */
