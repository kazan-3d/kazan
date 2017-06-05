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

#ifndef UTIL_IS_SWAPPABLE_H_
#define UTIL_IS_SWAPPABLE_H_

#include <utility>
#include <type_traits>
#include "is_referenceable.h"

namespace vulkan_cpu_util_is_swappable_unrelated_namespace
{
using std::swap;
template <typename T, typename U, bool Is_Void = std::is_void<T>::value || std::is_void<T>::value>
class Is_swappable_with
{
private:
    template <typename L,
              typename R,
              typename = decltype(swap(std::declval<L>(), std::declval<R>()))>
    static void fn(int);
    template <typename L, typename R>
    static char fn(...);

public:
    static constexpr bool value =
        std::is_void<decltype(fn<T, U>(0))>::value && std::is_void<decltype(fn<U, T>(0))>::value;
};

template <typename T, typename U>
class Is_swappable_with<T, U, true>
{
public:
    static constexpr bool value = false;
};

template <typename T, typename U, bool Is_Swappable_With = Is_swappable_with<T, U>::value>
struct Is_nothrow_swappable_with
{
    static constexpr bool value = noexcept(swap(std::declval<T>(), std::declval<U>()))
                                  && noexcept(swap(std::declval<U>(), std::declval<T>()));
};

template <typename T, typename U>
struct Is_nothrow_swappable_with<T, U, false>
{
    static constexpr bool value = false;
};
}

namespace vulkan_cpu
{
namespace util
{
template <typename T, typename U>
struct is_swappable_with
    : public std::integral_constant<bool,
                                    vulkan_cpu_util_is_swappable_unrelated_namespace::
                                        Is_swappable_with<T, U>::value>
{
};

template <typename T, typename U>
constexpr bool is_swappable_with_v = is_swappable_with<T, U>::value;

template <typename T, typename U>
struct is_nothrow_swappable_with
    : public std::integral_constant<bool,
                                    vulkan_cpu_util_is_swappable_unrelated_namespace::
                                        Is_nothrow_swappable_with<T, U>::value>
{
};

template <typename T, typename U>
constexpr bool is_nothrow_swappable_with_v = is_nothrow_swappable_with<T, U>::value;

namespace detail
{
template <typename T, bool Is_Referenceable = is_referenceable<T>::value>
struct is_swappable_helper
{
    static constexpr bool value = is_swappable_with<T &, T &>::value;
};

template <typename T>
struct is_swappable_helper<T, false>
{
    static constexpr bool value = false;
};

template <typename T, bool Is_Referenceable = is_referenceable<T>::value>
struct is_nothrow_swappable_helper
{
    static constexpr bool value = is_nothrow_swappable_with<T &, T &>::value;
};

template <typename T>
struct is_nothrow_swappable_helper<T, false>
{
    static constexpr bool value = false;
};
}

template <typename T>
struct is_swappable : public std::integral_constant<bool, detail::is_swappable_helper<T>::value>
{
};

template <typename T>
constexpr bool is_swappable_v = is_swappable<T>::value;

template <typename T>
struct is_nothrow_swappable
    : public std::integral_constant<bool, detail::is_nothrow_swappable_helper<T>::value>
{
};

template <typename T>
constexpr bool is_nothrow_swappable_v = is_nothrow_swappable<T>::value;
}
}

#endif /* UTIL_IS_SWAPPABLE_H_ */
