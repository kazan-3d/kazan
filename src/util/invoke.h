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

#ifndef UTIL_INVOKE_H_
#define UTIL_INVOKE_H_

#include <type_traits>
#include <utility>
#include <functional>
#include "void_t.h"

namespace kazan
{
namespace util
{
namespace detail
{
template <typename Fn,
          typename... Args,
          typename = typename std::enable_if<!std::is_member_pointer<Fn>::value>::type>
constexpr decltype(auto) invoke_helper(Fn &&fn, Args &&... args) noexcept(
    noexcept(std::declval<Fn>()(std::declval<Args>()...)))
{
    return std::forward<Fn>(fn)(std::forward<Args>(args)...);
}

template <
    typename Fn,
    typename T,
    typename Arg1,
    typename... Args,
    typename =
        typename std::enable_if<std::is_base_of<T, typename std::decay<Arg1>::type>::value>::type>
constexpr decltype(auto) invoke_member_function_pointer(
    Fn T::*fn,
    Arg1 &&arg1,
    Args &&... args) noexcept(noexcept((std::forward<Arg1>(arg1).*fn)(std::forward<Args>(args)...)))
{
    return (std::forward<Arg1>(arg1).*fn)(std::forward<Args>(args)...);
}

template <typename T>
struct Invoke_is_reference_wrapper
{
    static constexpr bool value = false;
};

template <typename T>
struct Invoke_is_reference_wrapper<std::reference_wrapper<T>>
{
    static constexpr bool value = true;
};

template <
    typename Fn,
    typename T,
    typename Arg1,
    typename... Args,
    typename = int,
    typename = typename std::enable_if<!std::is_base_of<T, typename std::decay<Arg1>::type>::value
                                       && Invoke_is_reference_wrapper<
                                              typename std::decay<Arg1>::type>::value>::type>
constexpr decltype(auto) invoke_member_function_pointer(
    Fn T::*fn,
    Arg1 &&arg1,
    Args &&... args) noexcept(noexcept((arg1.get().*fn)(std::forward<Args>(args)...)))
{
    return (arg1.get().*fn)(std::forward<Args>(args)...);
}

template <
    typename Fn,
    typename T,
    typename Arg1,
    typename... Args,
    typename = int,
    typename = int,
    typename = typename std::enable_if<!std::is_base_of<T, typename std::decay<Arg1>::type>::value
                                       && !Invoke_is_reference_wrapper<
                                              typename std::decay<Arg1>::type>::value>::type>
constexpr decltype(auto) invoke_member_function_pointer(
    Fn T::*fn, Arg1 &&arg1, Args &&... args) noexcept(noexcept(((*std::forward<Arg1>(arg1))
                                                                .*fn)(std::forward<Args>(args)...)))
{
    return ((*std::forward<Arg1>(arg1)).*fn)(std::forward<Args>(args)...);
}

template <typename Fn,
          typename Arg1,
          typename... Args,
          typename = typename std::enable_if<std::is_member_function_pointer<Fn>::value>::type>
constexpr decltype(auto) invoke_helper(Fn fn, Args &&... args) noexcept(
    noexcept(invoke_member_function_pointer(fn, std::forward<Args>(args)...)))
{
    return invoke_member_function_pointer(fn, std::forward<Args>(args)...);
}

template <
    typename Fn,
    typename T,
    typename Arg,
    typename =
        typename std::enable_if<std::is_base_of<T, typename std::decay<Arg>::type>::value>::type>
constexpr decltype(auto) invoke_member_object_pointer(Fn T::*fn, Arg &&arg) noexcept(
    noexcept(std::forward<Arg>(arg).*fn))
{
    return std::forward<Arg>(arg).*fn;
}

template <
    typename Fn,
    typename T,
    typename Arg,
    typename = int,
    typename = typename std::enable_if<!std::is_base_of<T, typename std::decay<Arg>::type>::value
                                       && Invoke_is_reference_wrapper<
                                              typename std::decay<Arg>::type>::value>::type>
constexpr decltype(auto) invoke_member_object_pointer(Fn T::*fn,
                                                      Arg &&arg) noexcept(noexcept(arg.get().*fn))
{
    return arg.get().*fn;
}

template <
    typename Fn,
    typename T,
    typename Arg,
    typename = int,
    typename = int,
    typename = typename std::enable_if<!std::is_base_of<T, typename std::decay<Arg>::type>::value
                                       && !Invoke_is_reference_wrapper<
                                              typename std::decay<Arg>::type>::value>::type>
constexpr decltype(auto) invoke_member_object_pointer(Fn T::*fn, Arg &&arg) noexcept(
    noexcept((*std::forward<Arg>(arg)).*fn))
{
    return (*std::forward<Arg>(arg)).*fn;
}

template <typename Fn,
          typename Arg,
          typename = typename std::enable_if<std::is_member_object_pointer<Fn>::value>::type>
constexpr decltype(auto) invoke_helper(Fn fn, Arg &&arg) noexcept(
    noexcept(invoke_member_object_pointer(fn, std::forward<Arg>(arg))))
{
    return invoke_member_object_pointer(fn, std::forward<Arg>(arg));
}

template <typename Fn, typename = void, typename... Args>
struct Invoke_result_helper
{
    static constexpr bool is_invokable = false;
    static constexpr bool is_nothrow_invokable = false;
    template <typename R>
    static constexpr bool is_invokable_r() noexcept
    {
        return false;
    }
    template <typename R>
    static constexpr bool is_nothrow_invokable_r() noexcept
    {
        return false;
    }
};

template <typename Fn, typename... Args>
struct Invoke_result_helper<Fn,
                            void_t<decltype(
                                invoke_helper(std::declval<Fn>(), std::declval<Args>()...))>,
                            Args...>
{
    typedef decltype(invoke_helper(std::declval<Fn>(), std::declval<Args>()...)) type;
    static constexpr bool is_invokable = true;
    static constexpr bool is_nothrow_invokable =
        noexcept(invoke_helper(std::declval<Fn>(), std::declval<Args>()...));
    template <typename R>
    static constexpr bool is_invokable_r() noexcept
    {
        return std::is_void<R>::value || std::is_convertible<type, R>::value;
    }
    template <typename R>
    static constexpr bool is_nothrow_invokable_r_helper(...) noexcept
    {
        return false;
    }
    template <typename R,
              bool Is_Nothrow = noexcept(static_cast<R>(invoke_helper(std::declval<Fn>(),
                                                                      std::declval<Args>()...)))>
    static constexpr bool is_nothrow_invokable_r_helper(int) noexcept
    {
        return is_invokable_r<R>() && Is_Nothrow;
    }
    template <typename R>
    constexpr bool is_nothrow_invokable_r() noexcept
    {
        return is_nothrow_invokable_r_helper<R>(0);
    }
};
}

template <typename Fn, typename... Args>
struct invoke_result : public detail::Invoke_result_helper<Fn, void, Args...>
{
};

template <typename Fn, typename... Args>
using invoke_result_t = typename invoke_result<Fn, Args...>::type;

template <typename Fn, typename... Args>
constexpr invoke_result_t<Fn, Args...> invoke(Fn &&fn, Args &&... args) noexcept(
    noexcept(detail::invoke_helper(std::declval<Fn>(), std::declval<Args>()...)))
{
    return detail::invoke_helper(std::forward<Fn>(fn), std::forward<Args>(args)...);
}

template <typename Fn, typename... Args>
struct is_invocable
    : public std::integral_constant<bool, detail::Invoke_result_helper<Fn, Args...>::is_invokable>
{
};

template <typename Fn, typename... Args>
constexpr bool is_invocable_v = is_invocable<Fn, Args...>::value;

template <typename R, typename Fn, typename... Args>
struct is_invocable_r
    : public std::
          integral_constant<bool,
                            detail::Invoke_result_helper<Fn, Args...>::template is_invokable_r<R>()>
{
};

template <typename R, typename Fn, typename... Args>
constexpr bool is_invocable_r_v = is_invocable_r<R, Fn, Args...>::value;

template <typename Fn, typename... Args>
struct is_nothrow_invocable
    : public std::integral_constant<bool,
                                    detail::Invoke_result_helper<Fn, Args...>::is_nothrow_invokable>
{
};

template <typename Fn, typename... Args>
constexpr bool is_nothrow_invocable_v = is_nothrow_invocable<Fn, Args...>::value;

template <typename R, typename Fn, typename... Args>
struct is_nothrow_invocable_r
    : public std::integral_constant<bool,
                                    detail::Invoke_result_helper<Fn, Args...>::
                                        template is_nothrow_invokable_r<R>()>
{
};

template <typename R, typename Fn, typename... Args>
constexpr bool is_nothrow_invocable_r_v = is_nothrow_invocable_r<R, Fn, Args...>::value;
}
}

#endif /* UTIL_INVOKE_H_ */
