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

#ifndef UTIL_CONSTEXPR_ARRAY_H_
#define UTIL_CONSTEXPR_ARRAY_H_

#include "is_swappable.h"
#include <type_traits>
#include <utility>
#include <iterator>
#include <stdexcept>

namespace vulkan_cpu
{
namespace util
{
namespace detail
{
template <typename T, std::size_t N>
struct Constexpr_array_helper
{
    typedef T values_type[N];
    static constexpr T *get_values_pointer(values_type &values) noexcept
    {
        return values;
    }
    static constexpr const T *get_values_pointer(const values_type &values) noexcept
    {
        return values;
    }
};

template <typename T>
struct Constexpr_array_helper<T, 0>
{
    struct values_type
    {
    };
    static constexpr T *get_values_pointer(values_type &values) noexcept
    {
        return nullptr;
    }
    static constexpr const T *get_values_pointer(const values_type &values) noexcept
    {
        return nullptr;
    }
};
}

template <typename T, std::size_t N>
struct Constexpr_array
{
private:
    typedef detail::Constexpr_array_helper<T, N> Helper;
    typedef typename Helper::values_type values_type;
    constexpr T *get_values_pointer() noexcept
    {
        return Helper::get_values_pointer(values);
    }
    constexpr const T *get_values_pointer() const noexcept
    {
        return Helper::get_values_pointer(values);
    }

public:
    values_type values;
    typedef T value_type;
    typedef std::size_t size_type;
    typedef std::ptrdiff_t difference_type;
    typedef T &reference;
    typedef const T &const_reference;
    typedef T *pointer;
    typedef const T *const_pointer;
    typedef T *iterator;
    typedef const T *const_iterator;
    typedef std::reverse_iterator<iterator> reverse_iterator;
    typedef std::reverse_iterator<const_iterator> const_reverse_iterator;
    constexpr T &at(std::size_t index)
    {
        if(index >= N)
            throw std::out_of_range("Constexpr_array::at");
        return get_values_pointer()[index];
    }
    constexpr const T &at(std::size_t index) const
    {
        if(index >= N)
            throw std::out_of_range("Constexpr_array::at");
        return get_values_pointer()[index];
    }
    constexpr T &operator[](std::size_t index) noexcept
    {
        return get_values_pointer()[index];
    }
    constexpr const T &operator[](std::size_t index) const noexcept
    {
        return get_values_pointer()[index];
    }
    constexpr T &front() noexcept
    {
        return get_values_pointer()[0];
    }
    constexpr const T &front() const noexcept
    {
        return get_values_pointer()[0];
    }
    constexpr T &back() noexcept
    {
        return get_values_pointer()[N - 1];
    }
    constexpr const T &back() const noexcept
    {
        return get_values_pointer()[N - 1];
    }
    constexpr T *data() noexcept
    {
        return get_values_pointer();
    }
    constexpr const T *data() const noexcept
    {
        return get_values_pointer();
    }
    constexpr iterator begin() noexcept
    {
        return get_values_pointer();
    }
    constexpr const_iterator begin() const noexcept
    {
        return get_values_pointer();
    }
    constexpr const_iterator cbegin() const noexcept
    {
        return get_values_pointer();
    }
    constexpr iterator end() noexcept
    {
        return get_values_pointer() + N;
    }
    constexpr const_iterator end() const noexcept
    {
        return get_values_pointer() + N;
    }
    constexpr const_iterator cend() const noexcept
    {
        return get_values_pointer() + N;
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
    constexpr bool empty() const noexcept
    {
        return size() == 0;
    }
    constexpr std::size_t size() const noexcept
    {
        return N;
    }
    constexpr std::size_t max_size() const noexcept
    {
        return N;
    }
    constexpr void fill(const T &value) noexcept(std::is_nothrow_copy_assignable<T>::value)
    {
        for(auto &i : *this)
            i = value;
    }
    constexpr void swap(Constexpr_array &other) noexcept(is_nothrow_swappable_v<T>)
    {
        using std::swap;
        for(std::size_t index = 0; index < size(); index++)
            swap(get_values_pointer()[index], other.get_values_pointer()[index]);
    }
};

template <typename T, std::size_t N>
constexpr void swap(Constexpr_array<T, N> &a,
                    Constexpr_array<T, N> &b) noexcept(is_nothrow_swappable_v<T>)
{
    a.swap(b);
}

template <std::size_t I, typename T, std::size_t N>
constexpr T &get(Constexpr_array<T, N> &v) noexcept
{
    static_assert(I < N, "");
    return v[I];
}

template <std::size_t I, typename T, std::size_t N>
constexpr const T &get(const Constexpr_array<T, N> &v) noexcept
{
    static_assert(I < N, "");
    return v[I];
}

template <std::size_t I, typename T, std::size_t N>
constexpr const T &&get(const Constexpr_array<T, N> &&v) noexcept
{
    static_assert(I < N, "");
    return std::move(v[I]);
}

template <std::size_t I, typename T, std::size_t N>
constexpr T &&get(Constexpr_array<T, N> &&v) noexcept
{
    static_assert(I < N, "");
    return std::move(v[I]);
}
}
}

#endif /* UTIL_CONSTEXPR_ARRAY_H_ */
