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

#ifndef UTIL_OPTIONAL_H_
#define UTIL_OPTIONAL_H_

#include <type_traits>
#include <new>
#include <memory>
#include <initializer_list>
#include <utility>
#include <cassert>
#include <exception>
#include <functional>
#include "in_place.h"
#include "is_swappable.h"

namespace kazan
{
namespace util
{
struct nullopt_t
{
    constexpr explicit nullopt_t(int)
    {
    }
};

constexpr nullopt_t nullopt(0);

class bad_optional_access : public std::exception
{
public:
    virtual const char *what() const noexcept override
    {
        return "bad_optional_access";
    }
};

namespace detail
{
template <typename T,
          bool Is_Trivially_Destructible = std::is_trivially_destructible<T>::value,
          bool Is_Trivially_Copyable = std::is_trivially_copyable<T>::value>
struct Optional_base
{
    union
    {
        T full_value;
        alignas(T) char empty_value[sizeof(T)];
    };
    bool is_full;
    constexpr Optional_base() noexcept : empty_value{}, is_full(false)
    {
    }
    constexpr Optional_base(nullopt_t) noexcept : empty_value{}, is_full(false)
    {
    }
    void reset() noexcept
    {
        if(is_full)
            full_value.~T();
        is_full = false;
    }
    template <typename... Types>
    T &emplace(Types &&... args) noexcept(std::is_nothrow_constructible<T, Types...>::value)
    {
        reset();
        ::new(static_cast<void *>(std::addressof(full_value))) T(std::forward<Types>(args)...);
        is_full = true;
        return full_value;
    }
    template <
        typename U,
        typename... Types,
        typename = typename std::
            enable_if<std::is_constructible<T, std::initializer_list<U>, Types...>::value>::type>
    T &emplace(std::initializer_list<U> init_list, Types &&... args) noexcept(
        std::is_nothrow_constructible<T, std::initializer_list<U>, Types...>::value)
    {
        reset();
        ::new(static_cast<void *>(std::addressof(full_value)))
            T(init_list, std::forward<Types>(args)...);
        is_full = true;
        return full_value;
    }
    Optional_base(const Optional_base &rt) noexcept(std::is_nothrow_copy_constructible<T>::value)
        : empty_value{}, is_full(false)
    {
        if(rt.is_full)
            emplace(rt.full_value);
    }
    Optional_base(Optional_base &&rt) noexcept(std::is_nothrow_move_constructible<T>::value)
        : empty_value{}, is_full(false)
    {
        if(rt.is_full)
            emplace(std::move(rt.full_value));
    }
    template <typename... Types,
              typename = typename std::enable_if<std::is_constructible<T, Types...>::value>::type>
    constexpr explicit Optional_base(in_place_t, Types &&... args) noexcept(
        std::is_nothrow_constructible<T, Types...>::value)
        : full_value(std::forward<Types>(args)...), is_full(true)
    {
    }
    template <
        typename U,
        typename... Types,
        typename = typename std::
            enable_if<std::is_constructible<T, std::initializer_list<U>, Types...>::value>::type>
    constexpr explicit Optional_base(
        in_place_t,
        std::initializer_list<U> init_list,
        Types &&... args) noexcept(std::is_nothrow_constructible<T, Types...>::value)
        : full_value(init_list, std::forward<Types>(args)...), is_full(true)
    {
    }
    ~Optional_base()
    {
        reset();
    }
    Optional_base &operator=(const Optional_base &rt) noexcept(
        std::is_nothrow_copy_assignable<T>::value)
    {
        if(!rt.is_full)
            reset();
        else if(!is_full)
            emplace(rt.full_value);
        else
            full_value = rt.full_value;
        return *this;
    }
    Optional_base &operator=(Optional_base &&rt) noexcept(std::is_nothrow_move_assignable<T>::value)
    {
        if(!rt.is_full)
            reset();
        else if(!is_full)
            emplace(std::move(rt.full_value));
        else
            full_value = std::move(rt.full_value);
        return *this;
    }
};

template <typename T>
struct Optional_base<T, true, false>
{
    union
    {
        T full_value;
        alignas(T) char empty_value[sizeof(T)];
    };
    bool is_full;
    constexpr Optional_base() noexcept : empty_value{}, is_full(false)
    {
    }
    constexpr Optional_base(nullopt_t) noexcept : empty_value{}, is_full(false)
    {
    }
    void reset() noexcept
    {
        // full_value.~T() not needed
        is_full = false;
    }
    template <typename... Types>
    T &emplace(Types &&... args) noexcept(std::is_nothrow_constructible<T, Types...>::value)
    {
        reset();
        ::new(static_cast<void *>(std::addressof(full_value))) T(std::forward<Types>(args)...);
        is_full = true;
        return full_value;
    }
    template <
        typename U,
        typename... Types,
        typename = typename std::
            enable_if<std::is_constructible<T, std::initializer_list<U>, Types...>::value>::type>
    T &emplace(std::initializer_list<U> init_list, Types &&... args) noexcept(
        std::is_nothrow_constructible<T, std::initializer_list<U>, Types...>::value)
    {
        reset();
        ::new(static_cast<void *>(std::addressof(full_value)))
            T(init_list, std::forward<Types>(args)...);
        is_full = true;
        return full_value;
    }
    Optional_base(const Optional_base &rt) noexcept(std::is_nothrow_copy_constructible<T>::value)
        : empty_value{}, is_full(false)
    {
        if(rt.is_full)
            emplace(rt.full_value);
    }
    Optional_base(Optional_base &&rt) noexcept(std::is_nothrow_move_constructible<T>::value)
        : empty_value{}, is_full(false)
    {
        if(rt.is_full)
            emplace(std::move(rt.full_value));
    }
    template <typename... Types,
              typename = typename std::enable_if<std::is_constructible<T, Types...>::value>::type>
    constexpr explicit Optional_base(in_place_t, Types &&... args) noexcept(
        std::is_nothrow_constructible<T, Types...>::value)
        : full_value(std::forward<Types>(args)...), is_full(true)
    {
    }
    template <
        typename U,
        typename... Types,
        typename = typename std::
            enable_if<std::is_constructible<T, std::initializer_list<U>, Types...>::value>::type>
    constexpr explicit Optional_base(
        in_place_t,
        std::initializer_list<U> init_list,
        Types &&... args) noexcept(std::is_nothrow_constructible<T, Types...>::value)
        : full_value(init_list, std::forward<Types>(args)...), is_full(true)
    {
    }
    ~Optional_base() = default;
    Optional_base &operator=(const Optional_base &rt) noexcept(
        std::is_nothrow_copy_assignable<T>::value)
    {
        if(!rt.is_full)
            reset();
        else if(!is_full)
            emplace(rt.full_value);
        else
            full_value = rt.full_value;
        return *this;
    }
    Optional_base &operator=(Optional_base &&rt) noexcept(std::is_nothrow_move_assignable<T>::value)
    {
        if(!rt.is_full)
            reset();
        else if(!is_full)
            emplace(std::move(rt.full_value));
        else
            full_value = std::move(rt.full_value);
        return *this;
    }
};

template <typename T>
struct Optional_base<T, true, true>
{
    union
    {
        T full_value;
        alignas(T) char empty_value[sizeof(T)];
    };
    bool is_full;
    constexpr Optional_base() noexcept : empty_value{}, is_full(false)
    {
    }
    constexpr Optional_base(nullopt_t) noexcept : empty_value{}, is_full(false)
    {
    }
    void reset() noexcept
    {
        // full_value.~T() not needed
        is_full = false;
    }
    template <typename... Types>
    T &emplace(Types &&... args) noexcept(std::is_nothrow_constructible<T, Types...>::value)
    {
        reset();
        ::new(static_cast<void *>(std::addressof(full_value))) T(std::forward<Types>(args)...);
        is_full = true;
        return full_value;
    }
    template <
        typename U,
        typename... Types,
        typename = typename std::
            enable_if<std::is_constructible<T, std::initializer_list<U>, Types...>::value>::type>
    T &emplace(std::initializer_list<U> init_list, Types &&... args) noexcept(
        std::is_nothrow_constructible<T, std::initializer_list<U>, Types...>::value)
    {
        reset();
        ::new(static_cast<void *>(std::addressof(full_value)))
            T(init_list, std::forward<Types>(args)...);
        is_full = true;
        return full_value;
    }
    constexpr Optional_base(const Optional_base &rt) noexcept = default;
    constexpr Optional_base(Optional_base &&rt) noexcept = default;
    template <typename... Types,
              typename = typename std::enable_if<std::is_constructible<T, Types...>::value>::type>
    constexpr explicit Optional_base(in_place_t, Types &&... args) noexcept(
        std::is_nothrow_constructible<T, Types...>::value)
        : full_value(std::forward<Types>(args)...), is_full(true)
    {
    }
    template <
        typename U,
        typename... Types,
        typename = typename std::
            enable_if<std::is_constructible<T, std::initializer_list<U>, Types...>::value>::type>
    constexpr explicit Optional_base(
        in_place_t,
        std::initializer_list<U> init_list,
        Types &&... args) noexcept(std::is_nothrow_constructible<T, Types...>::value)
        : full_value(init_list, std::forward<Types>(args)...), is_full(true)
    {
    }
    ~Optional_base() = default;
    Optional_base &operator=(const Optional_base &rt) noexcept = default;
    Optional_base &operator=(Optional_base &&rt) noexcept = default;
};
}

template <typename T>
class optional;

namespace detail
{
template <typename T, typename U, typename U_Ref>
constexpr bool optional_needs_conversion_constructors() noexcept
{
    if(!std::is_constructible<T, U_Ref>::value)
        return false;
    if(std::is_constructible<T, optional<U> &>::value)
        return false;
    if(std::is_constructible<T, const optional<U> &>::value)
        return false;
    if(std::is_constructible<T, optional<U> &&>::value)
        return false;
    if(std::is_constructible<T, const optional<U> &&>::value)
        return false;
    if(std::is_convertible<optional<U> &, T>::value)
        return false;
    if(std::is_convertible<const optional<U> &, T>::value)
        return false;
    if(std::is_convertible<optional<U> &&, T>::value)
        return false;
    if(std::is_convertible<const optional<U> &&, T>::value)
        return false;
    return true;
}

template <typename T, typename U, typename U_Ref>
constexpr bool optional_needs_conversion_from_optional_assign_operators() noexcept
{
    if(!std::is_constructible<T, U_Ref>::value)
        return false;
    if(!std::is_assignable<T &, U_Ref>::value)
        return false;
    if(std::is_constructible<T, optional<U> &>::value)
        return false;
    if(std::is_constructible<T, const optional<U> &>::value)
        return false;
    if(std::is_constructible<T, optional<U> &&>::value)
        return false;
    if(std::is_constructible<T, const optional<U> &&>::value)
        return false;
    if(std::is_convertible<optional<U> &, T>::value)
        return false;
    if(std::is_convertible<const optional<U> &, T>::value)
        return false;
    if(std::is_convertible<optional<U> &&, T>::value)
        return false;
    if(std::is_convertible<const optional<U> &&, T>::value)
        return false;
    if(std::is_assignable<T &, optional<U> &>::value)
        return false;
    if(std::is_assignable<T &, const optional<U> &>::value)
        return false;
    if(std::is_assignable<T &, optional<U> &&>::value)
        return false;
    if(std::is_assignable<T &, const optional<U> &&>::value)
        return false;
    return true;
}
}

template <typename T>
class optional : private detail::Optional_base<T>
{
private:
    typedef detail::Optional_base<T> Base;
    using Base::is_full;
    using Base::full_value;

public:
    using Base::Base;
    using Base::reset;
    using Base::emplace;
    constexpr optional() noexcept = default;
    constexpr optional(const optional &) noexcept(std::is_nothrow_copy_constructible<T>::value) =
        default;
    constexpr optional(optional &&) noexcept(std::is_nothrow_move_constructible<T>::value) =
        default;
    template <typename U,
              typename = typename std::
                  enable_if<detail::optional_needs_conversion_constructors<T, U, const U &>()
                            && std::is_convertible<const U &, T>::value>::type>
    optional(const optional<U> &rt) noexcept(std::is_nothrow_constructible<T, const U &>::value)
        : Base()
    {
        if(rt)
            emplace(*rt);
    }
    template <typename U,
              typename = typename std::
                  enable_if<detail::optional_needs_conversion_constructors<T, U, const U &>()
                            && !std::is_convertible<const U &, T>::value>::type,
              typename = void>
    explicit optional(const optional<U> &rt) noexcept(
        std::is_nothrow_constructible<T, const U &>::value)
        : Base()
    {
        if(rt)
            emplace(*rt);
    }
    template <
        typename U,
        typename =
            typename std::enable_if<detail::optional_needs_conversion_constructors<T, U, U &&>()
                                    && std::is_convertible<U &&, T>::value>::type>
    optional(optional<U> &&rt) noexcept(std::is_nothrow_constructible<T, U &&>::value)
        : Base()
    {
        if(rt)
            emplace(std::move(*rt));
    }
    template <
        typename U,
        typename =
            typename std::enable_if<detail::optional_needs_conversion_constructors<T, U, U &&>()
                                    && !std::is_convertible<U &&, T>::value>::type,
        typename = void>
    explicit optional(optional<U> &&rt) noexcept(std::is_nothrow_constructible<T, U &&>::value)
        : Base()
    {
        if(rt)
            emplace(std::move(*rt));
    }
    template <typename U,
              typename = typename std::
                  enable_if<std::is_constructible<T, U &&>::value
                            && !std::is_same<typename std::decay<U>::type, in_place_t>::value
                            && !std::is_same<typename std::decay<U>::type, optional>::value
                            && std::is_convertible<U &&, T>::value>::type,
              typename = void>
    constexpr optional(U &&value) noexcept(std::is_nothrow_constructible<T, U &&>::value)
        : Base(in_place, std::forward<U>(value))
    {
    }
    template <typename U,
              typename = typename std::
                  enable_if<std::is_constructible<T, U &&>::value
                            && !std::is_same<typename std::decay<U>::type, in_place_t>::value
                            && !std::is_same<typename std::decay<U>::type, optional>::value
                            && !std::is_convertible<U &&, T>::value>::type>
    explicit constexpr optional(U &&value) noexcept(std::is_nothrow_constructible<T, U &&>::value)
        : Base(in_place, std::forward<U>(value))
    {
    }
    constexpr optional &operator=(const optional &) noexcept(
        std::is_nothrow_copy_assignable<T>::value) = default;
    constexpr optional &operator=(optional &&) noexcept(std::is_nothrow_move_assignable<T>::value) =
        default;
    template <typename U = T,
              typename = typename std::
                  enable_if<!std::is_same<typename std::decay<U>::type, optional>::value
                            && std::is_constructible<T, U>::value
                            && std::is_assignable<T &, U>::value
                            && (!std::is_scalar<T>::value
                                || !std::is_same<typename std::decay<U>::type, T>::value)>::type>
    optional &operator=(U &&value) noexcept(std::is_nothrow_constructible<T, U &&>::value
                                                &&std::is_nothrow_assignable<T &, U &&>::value)
    {
        if(is_full)
            full_value = std::forward<U>(value);
        else
            emplace(std::forward<U>(value));
        return *this;
    }
    optional &operator=(nullopt_t) noexcept
    {
        reset();
        return *this;
    }
    template <
        typename U,
        typename = typename std::enable_if< //
            detail::optional_needs_conversion_from_optional_assign_operators<T, U, const U &>()>::
            type>
    optional &operator=(const optional<U> &rt) noexcept(
        std::is_nothrow_constructible<T, const U &>::value
            &&std::is_nothrow_assignable<T &, const U &>::value)
    {
        if(!rt)
            reset();
        else if(!is_full)
            emplace(*rt);
        else
            full_value = *rt;
        return *this;
    }
    template <
        typename U,
        typename = typename std::enable_if< //
            detail::optional_needs_conversion_from_optional_assign_operators<T, U, U &&>()>::type>
    optional &operator=(optional<U> &&rt) noexcept(std::is_nothrow_constructible<T, U &&>::value &&
                                                       std::is_nothrow_assignable<T &, U &&>::value)
    {
        if(!rt)
            reset();
        else if(!is_full)
            emplace(std::move(*rt));
        else
            full_value = std::move(*rt);
        return *this;
    }
    constexpr const T *operator->() const noexcept
    {
        assert(is_full);
        return std::addressof(full_value);
    }
    constexpr T *operator->() noexcept
    {
        assert(is_full);
        return std::addressof(full_value);
    }
    constexpr const T &operator*() const &noexcept
    {
        assert(is_full);
        return full_value;
    }
    constexpr T &operator*() & noexcept
    {
        assert(is_full);
        return full_value;
    }
    constexpr const T &&operator*() const &&noexcept
    {
        assert(is_full);
        return std::move(full_value);
    }
    constexpr T &&operator*() && noexcept
    {
        assert(is_full);
        return std::move(full_value);
    }
    constexpr explicit operator bool() const noexcept
    {
        return is_full;
    }
    constexpr bool has_value() const noexcept
    {
        return is_full;
    }
    constexpr T &value() &
    {
        if(!is_full)
            throw bad_optional_access();
        return full_value;
    }
    constexpr const T &value() const &
    {
        if(!is_full)
            throw bad_optional_access();
        return full_value;
    }
    constexpr T &&value() &&
    {
        if(!is_full)
            throw bad_optional_access();
        return std::move(full_value);
    }
    constexpr const T &&value() const &&
    {
        if(!is_full)
            throw bad_optional_access();
        return std::move(full_value);
    }
    template <typename U>
    constexpr T value_or(U &&default_value) const &noexcept(
        std::is_nothrow_copy_constructible<T>::value //
            &&noexcept(static_cast<T>(std::declval<U>())))
    {
        return is_full ? full_value : static_cast<T>(std::forward<U>(default_value));
    }
    template <typename U>
        constexpr T value_or(U &&default_value)
        && noexcept(std::is_nothrow_copy_constructible<T>::value //
                        &&noexcept(static_cast<T>(std::declval<U>())))
    {
        return is_full ? std::move(full_value) : static_cast<T>(std::forward<U>(default_value));
    }
    void swap(optional &other) noexcept(
        std::is_nothrow_move_constructible<T>::value &&util::is_nothrow_swappable<T>::value)
    {
        if(is_full)
        {
            if(other.is_full)
            {
                using std::swap;
                swap(full_value, other.full_value);
            }
            else
            {
                other.emplace(std::move(full_value));
                reset();
            }
        }
        else if(other.is_full)
        {
            emplace(std::move(other.full_value));
            other.reset();
        }
    }
};

template <typename T, typename U>
constexpr bool operator==(const optional<T> &l, const optional<U> &r) noexcept(noexcept(*l == *r))
{
    if(!l.has_value() || !r.has_value())
        return !r.has_value();
    return *l == *r;
}

template <typename T, typename U>
constexpr bool operator!=(const optional<T> &l, const optional<U> &r) noexcept(noexcept(*l == *r))
{
    if(!l.has_value() || !r.has_value())
        return r.has_value();
    return *l != *r;
}

template <typename T, typename U>
constexpr bool operator<(const optional<T> &l, const optional<U> &r) noexcept(noexcept(*l == *r))
{
    if(!l.has_value() || !r.has_value())
        return r.has_value();
    return *l < *r;
}

template <typename T, typename U>
constexpr bool operator>(const optional<T> &l, const optional<U> &r) noexcept(noexcept(*l == *r))
{
    if(!l.has_value() || !r.has_value())
        return l.has_value();
    return *l > *r;
}

template <typename T, typename U>
constexpr bool operator<=(const optional<T> &l, const optional<U> &r) noexcept(noexcept(*l == *r))
{
    if(!l.has_value() || !r.has_value())
        return !l.has_value();
    return *l <= *r;
}

template <typename T, typename U>
constexpr bool operator>=(const optional<T> &l, const optional<U> &r) noexcept(noexcept(*l == *r))
{
    if(!l.has_value() || !r.has_value())
        return !r.has_value();
    return *l >= *r;
}

template <typename T>
constexpr bool operator==(const optional<T> &v, nullopt_t) noexcept
{
    return !v.has_value();
}

template <typename T>
constexpr bool operator!=(const optional<T> &v, nullopt_t) noexcept
{
    return v.has_value();
}

template <typename T>
constexpr bool operator<(const optional<T> &v, nullopt_t) noexcept
{
    return false;
}

template <typename T>
constexpr bool operator>(const optional<T> &v, nullopt_t) noexcept
{
    return v.has_value();
}

template <typename T>
constexpr bool operator<=(const optional<T> &v, nullopt_t) noexcept
{
    return !v.has_value();
}

template <typename T>
constexpr bool operator>=(const optional<T> &v, nullopt_t) noexcept
{
    return true;
}

template <typename T>
constexpr bool operator==(nullopt_t, const optional<T> &v) noexcept
{
    return !v.has_value();
}

template <typename T>
constexpr bool operator!=(nullopt_t, const optional<T> &v) noexcept
{
    return v.has_value();
}

template <typename T>
constexpr bool operator<(nullopt_t, const optional<T> &v) noexcept
{
    return v.has_value();
}

template <typename T>
constexpr bool operator>(nullopt_t, const optional<T> &v) noexcept
{
    return false;
}

template <typename T>
constexpr bool operator<=(nullopt_t, const optional<T> &v) noexcept
{
    return true;
}

template <typename T>
constexpr bool operator>=(nullopt_t, const optional<T> &v) noexcept
{
    return !v.has_value();
}

template <typename T, typename U>
constexpr bool operator==(const optional<T> &l, const U &r) noexcept(
    noexcept(static_cast<bool>(std::declval<const T &>() == std::declval<const U &>())))
{
    if(l)
        return *l == r;
    return false;
}

template <typename T, typename U>
constexpr bool operator==(const U &l, const optional<T> &r) noexcept(
    noexcept(static_cast<bool>(std::declval<const U &>() == std::declval<const T &>())))
{
    if(r)
        return l == *r;
    return false;
}

template <typename T, typename U>
constexpr bool operator!=(const optional<T> &l, const U &r) noexcept(
    noexcept(static_cast<bool>(std::declval<const T &>() != std::declval<const U &>())))
{
    if(l)
        return *l != r;
    return true;
}

template <typename T, typename U>
constexpr bool operator!=(const U &l, const optional<T> &r) noexcept(
    noexcept(static_cast<bool>(std::declval<const U &>() != std::declval<const T &>())))
{
    if(r)
        return l != *r;
    return true;
}

template <typename T, typename U>
constexpr bool operator<(const optional<T> &l, const U &r) noexcept(
    noexcept(static_cast<bool>(std::declval<const T &>() < std::declval<const U &>())))
{
    if(l)
        return *l < r;
    return true;
}

template <typename T, typename U>
constexpr bool operator<(const U &l, const optional<T> &r) noexcept(
    noexcept(static_cast<bool>(std::declval<const U &>() < std::declval<const T &>())))
{
    if(r)
        return l < *r;
    return false;
}

template <typename T, typename U>
constexpr bool operator>(const optional<T> &l, const U &r) noexcept(
    noexcept(static_cast<bool>(std::declval<const T &>() > std::declval<const U &>())))
{
    if(l)
        return *l > r;
    return false;
}

template <typename T, typename U>
constexpr bool operator>(const U &l, const optional<T> &r) noexcept(
    noexcept(static_cast<bool>(std::declval<const U &>() > std::declval<const T &>())))
{
    if(r)
        return l > *r;
    return true;
}

template <typename T, typename U>
constexpr bool operator<=(const optional<T> &l, const U &r) noexcept(
    noexcept(static_cast<bool>(std::declval<const T &>() <= std::declval<const U &>())))
{
    if(l)
        return *l <= r;
    return true;
}

template <typename T, typename U>
constexpr bool operator<=(const U &l, const optional<T> &r) noexcept(
    noexcept(static_cast<bool>(std::declval<const U &>() <= std::declval<const T &>())))
{
    if(r)
        return l <= *r;
    return false;
}

template <typename T, typename U>
constexpr bool operator>=(const optional<T> &l, const U &r) noexcept(
    noexcept(static_cast<bool>(std::declval<const T &>() >= std::declval<const U &>())))
{
    if(l)
        return *l >= r;
    return false;
}

template <typename T, typename U>
constexpr bool operator>=(const U &l, const optional<T> &r) noexcept(
    noexcept(static_cast<bool>(std::declval<const U &>() >= std::declval<const T &>())))
{
    if(r)
        return l >= *r;
    return true;
}

template <typename T>
constexpr optional<typename std::decay<T>::type> make_optional(T &&value)
{
    return optional<typename std::decay<T>::type>(in_place, std::forward<T>(value));
}

template <typename T, typename... Args>
constexpr optional<T> make_optional(Args &&... args)
{
    return optional<T>(in_place, std::forward<T>(args)...);
}

template <typename T, typename U, typename... Args>
constexpr optional<T> make_optional(std::initializer_list<U> init_list, Args &&... args)
{
    return optional<T>(in_place, init_list, std::forward<T>(args)...);
}

template <typename T,
          typename = typename std::enable_if<std::is_move_constructible<T>::value
                                             && is_swappable<T>::value>::type>
void swap(optional<T> &l, optional<T> &r) noexcept(noexcept(l.swap(r)))
{
    l.swap(r);
}

namespace detail
{
template <typename T, bool Is_Enabled = std::is_default_constructible<std::hash<T>>::value>
struct optional_hash
{
    constexpr std::size_t operator()(const optional<T> &value) const
        noexcept(noexcept(static_cast<std::size_t>(std::hash<T>()(std::declval<const T &>()))))
    {
        if(value)
            return std::hash<T>()(*value);
        return 0;
    }
};

template <typename T>
struct optional_hash<T, false>
{
    optional_hash() noexcept = delete;
    ~optional_hash() = delete;
    optional_hash(const optional_hash &) noexcept = delete;
    optional_hash &operator=(const optional_hash &) noexcept = delete;
    std::size_t operator()(const optional<T> &value) const noexcept = delete;
};
}
}
}

namespace std
{
template <typename T>
struct hash<kazan::util::optional<T>> : public kazan::util::detail::optional_hash<T>
{
};
}

#endif /* UTIL_OPTIONAL_H_ */
