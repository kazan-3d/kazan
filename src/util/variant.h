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

#ifndef SOURCE_UTIL_VARIANT_H_
#define SOURCE_UTIL_VARIANT_H_

#include <utility>
#include <type_traits>
#include <functional>
#include <exception>
#include <limits>
#include <new>
#include "in_place.h"
#include "void_t.h"

namespace vulkan_cpu
{
namespace util
{
class bad_variant_access : public std::exception
{
    bad_variant_access() noexcept = default;
    virtual const char *what() const noexcept override
    {
        return "bad_variant_access";
    }
};

struct monostate
{
};

constexpr bool operator==(monostate, monostate) noexcept
{
    return true;
}

constexpr bool operator!=(monostate, monostate) noexcept
{
    return false;
}

constexpr bool operator<=(monostate, monostate) noexcept
{
    return true;
}

constexpr bool operator>=(monostate, monostate) noexcept
{
    return true;
}

constexpr bool operator<(monostate, monostate) noexcept
{
    return false;
}

constexpr bool operator>(monostate, monostate) noexcept
{
    return false;
}

constexpr std::size_t variant_npos = -1;

template <typename... Types>
class variant;

template <typename T>
struct variant_size;

template <typename... Types>
struct variant_size<variant<Types...>>
    : public std::integral_constant<std::size_t, sizeof...(Types)>
{
};

template <typename T>
struct variant_size<const T> : public variant_size<T>
{
};

template <typename T>
struct variant_size<volatile T> : public variant_size<T>
{
};

template <typename T>
struct variant_size<const volatile T> : public variant_size<T>
{
};

template <typename T>
constexpr std::size_t variant_size_v = variant_size<T>::value;

template <std::size_t Index, typename T>
struct variant_alternative;

template <std::size_t Index, typename T>
struct variant_alternative<Index, const T>
{
    typedef const typename variant_alternative<Index, T>::type type;
};

template <std::size_t Index, typename T>
struct variant_alternative<Index, const T>
{
    typedef const typename variant_alternative<Index, T>::type type;
};

template <std::size_t Index, typename T>
struct variant_alternative<Index, const volatile T>
{
    typedef const volatile typename variant_alternative<Index, T>::type type;
};

template <std::size_t Index, typename T, typename... Types>
struct variant_alternative<Index, variant<T, Types...>>
{
    typedef typename variant_alternative<Index - 1, variant<Types...>>::type type;
};

template <typename T, typename... Types>
struct variant_alternative<0, variant<T, Types...>>
{
    typedef T type;
};

template <std::size_t Index, typename T>
using variant_alternative_t = typename variant_alternative<Index, T>::type;

namespace detail
{
template <typename T>
struct variant_identity_type
{
    typedef T type;
};

template <typename... Types>
struct variant_hypothetical_overload_set;

template <typename T, typename... Types>
struct variant_hypothetical_overload_set<T, Types...>
    : public variant_hypothetical_overload_set<Types...>
{
    using variant_hypothetical_overload_set<Types...>::fn;
    static variant_identity_type<T> fn(T); // not implemented
};

template <>
struct variant_hypothetical_overload_set
{
    static void fn(); // not implemented
};

template <typename... Types>
union variant_values
{
    char value;
    static constexpr bool is_copy_constructible = true;
    static constexpr bool is_move_constructible = true;
    static constexpr bool is_nothrow_copy_constructible = true;
    static constexpr bool is_nothrow_move_constructible = true;
    static constexpr bool is_copy_assignable = true;
    static constexpr bool is_move_assignable = true;
    static constexpr bool is_nothrow_copy_assignable = true;
    static constexpr bool is_nothrow_move_assignable = true;
    static constexpr bool is_trivially_destructible = true;
    variant_values() = delete;
    template <std::size_t index>
    constexpr variant_values(in_place_index_t<index>) noexcept : value()
    {
    }
    template <typename U>
    static constexpr std::size_t index_from_type() noexcept
    {
        return variant_npos;
    }
    void copy_construct(const variant_values &rt, std::size_t index) noexcept
    {
    }
    void move_construct(variant_values &&rt, std::size_t index) noexcept
    {
    }
    void copy_assign(const variant_values &rt, std::size_t index) noexcept
    {
    }
    void move_assign(variant_values &&rt, std::size_t index) noexcept
    {
    }
    void destroy(std::size_t index) noexcept
    {
    }
};

template <typename T, typename... Types>
union variant_values<T, Types...>
{
    typedef T type_0;
    static_assert(!std::is_void<T>::value, "invalid variant member type");
    static_assert(!std::is_reference<T>::value, "invalid variant member type");
    static_assert(!std::is_array<T>::value, "invalid variant member type");
    T current_value;
    variant_values<Types...> other_values;
    static constexpr bool is_copy_constructible =
        std::is_copy_constructible<T>::value && variant_values<Types...>::is_copy_constructible;
    static constexpr bool is_move_constructible =
        std::is_move_constructible<T>::value && variant_values<Types...>::is_move_constructible;
    static constexpr bool is_nothrow_copy_constructible =
        std::is_nothrow_copy_constructible<T>::value
        && variant_values<Types...>::is_nothrow_copy_constructible;
    static constexpr bool is_nothrow_move_constructible =
        std::is_nothrow_move_constructible<T>::value
        && variant_values<Types...>::is_nothrow_move_constructible;
    static constexpr bool is_copy_assignable =
        std::is_copy_assignable<T>::value && variant_values<Types...>::is_copy_assignable;
    static constexpr bool is_move_assignable =
        std::is_move_assignable<T>::value && variant_values<Types...>::is_move_assignable;
    static constexpr bool is_nothrow_copy_assignable =
        std::is_nothrow_copy_assignable<T>::value
        && variant_values<Types...>::is_nothrow_copy_assignable;
    static constexpr bool is_nothrow_move_assignable =
        std::is_nothrow_move_assignable<T>::value
        && variant_values<Types...>::is_nothrow_move_assignable;
    static constexpr bool is_trivially_destructible =
        std::is_trivially_destructible<T>::value
        && variant_values<Types...>::is_trivially_destructible;
    template <typename T2 = T,
              typename = typename std::enable_if<std::is_default_constructible<T2>::value>::type>
    constexpr variant_values() noexcept(std::is_nothrow_default_constructible<T2>::value)
        : current_value()
    {
    }
    template <typename... Args,
              typename = typename std::enable_if<std::is_constructible<T, Args...>::value>::type>
    constexpr variant_values(in_place_index_t<0>, Args &&... args) noexcept(
        std::is_nothrow_constructible<T, Args...>::value)
        : current_value(std::forward<Args>(args)...)
    {
    }
    template <std::size_t index,
              typename... Args,
              typename = typename std::
                  enable_if<index != 0 && std::is_constructible<variant_values<Types...>,
                                                                in_place_index_t<index - 1>,
                                                                Args...>::value>::type>
    constexpr variant_values(in_place_index_t<index>, Args &&... args) noexcept(
        std::is_nothrow_constructible<variant_values<Types...>,
                                      in_place_index_t<index - 1>,
                                      Args...>::value)
        : other_values(in_place_index<index - 1>, std::forward<Args>(args)...)
    {
    }
    template <
        typename U,
        typename... Args,
        typename = typename std::
            enable_if<std::is_constructible<T, std::initializer_list<U>, Args...>::value>::type>
    constexpr variant_values(
        in_place_index_t<0>,
        std::initializer_list<U> il,
        Args &&... args) noexcept(std::is_nothrow_constructible<T,
                                                                std::initializer_list<U>,
                                                                Args...>::value)
        : current_value(il, std::forward<Args>(args)...)
    {
    }
    template <typename U>
    static constexpr std::size_t index_from_type() noexcept
    {
        std::size_t next = variant_values<Types...>::index_from_type<U>();
        if(std::is_same<U, T>::value && next == variant_npos)
            return 0;
        if(next == variant_npos)
            return variant_npos;
        return next + 1;
    }
    void copy_construct(const variant_values &rt,
                        std::size_t index) noexcept(is_nothrow_copy_constructible)
    {
        if(index == 0)
            new(const_cast<void *>(std::addressof(current_value))) T(rt.current_value);
        else
            other_values.copy_construct(rt.other_values, index - 1);
    }
    void move_construct(variant_values &&rt,
                        std::size_t index) noexcept(is_nothrow_move_constructible)
    {
        if(index == 0)
            new(const_cast<void *>(std::addressof(current_value))) T(std::move(rt.current_value));
        else
            other_values.move_construct(std::move(rt.other_values), index - 1);
    }
    void copy_assign(const variant_values &rt,
                     std::size_t index) noexcept(is_nothrow_copy_assignable)
    {
        if(index == 0)
            current_value = rt.current_value;
        else
            other_values.copy_assign(rt.other_values, index - 1);
    }
    void move_assign(variant_values &&rt, std::size_t index) noexcept(is_nothrow_move_assignable)
    {
        if(index == 0)
            current_value = std::move(rt.current_value);
        else
            other_values.move_assign(std::move(rt.other_values), index - 1);
    }
    void destruct(std::size_t index) noexcept
    {
        if(index == 0)
            current_value.~T();
        else
            other_values.destruct(index - 1);
    }
};

template <typename T,
          typename... Types,
          typename Deduced_Type = typename decltype(
              variant_hypothetical_overload_set<Types...>::fn(std::declval<T>()))::type,
          std::size_t Index = variant_values<Types...>::index_from_type<Deduced_Type>(),
          typename = typename std::enable_if<(Index < sizeof...(Types))>::type>
constexpr std::size_t variant_conversion_deduce_index() noexcept
{
    return Index;
}

template <typename T, typename... Types>
using variant_conversion_deduce_type =
    variant_alternative_t<variant_conversion_deduce_index<T, Types...>(), Types...>;

template <std::size_t Type_Count>
struct variant_index_type
{
    static constexpr std::size_t total_state_count =
        Type_Count + 1; // for valueless-by-exception state
    static constexpr bool is_unsigned_char_good =
        total_state_count <= std::numeric_limits<unsigned char>::max();
    static constexpr bool is_unsigned_short_good =
        total_state_count <= std::numeric_limits<unsigned short>::max();
    static constexpr bool is_unsigned_good =
        total_state_count <= std::numeric_limits<unsigned>::max();
    static constexpr bool is_unsigned_long_good =
        total_state_count <= std::numeric_limits<unsigned long>::max();
    static constexpr bool is_unsigned_long_long_good =
        total_state_count <= std::numeric_limits<unsigned long long>::max();
    typedef
        typename std::conditional<is_unsigned_long_long_good, unsigned long long, std::size_t>::type
            unsigned_long_long_or_larger;
    typedef typename std::conditional<is_unsigned_long_good,
                                      unsigned long,
                                      unsigned_long_long_or_larger>::type unsigned_long_or_larger;
    typedef typename std::conditional<is_unsigned_good, unsigned, unsigned_long_or_larger>::type
        unsigned_or_larger;
    typedef
        typename std::conditional<is_unsigned_short_good, unsigned short, unsigned_or_larger>::type
            unsigned_short_or_larger;
    typedef typename std::conditional<is_unsigned_char_good,
                                      unsigned char,
                                      unsigned_short_or_larger>::type type;
    static constexpr type npos = variant_npos;
    type index_value;
    constexpr variant_index_type() = delete;
    constexpr explicit variant_index_type(std::size_t index_value) noexcept
        : index_value(index_value)
    {
    }
    constexpr std::size_t get() const noexcept
    {
        return index_value == npos ? variant_npos : index_value;
    }
    constexpr void set(std::size_t new_value) noexcept
    {
        index_value = new_value;
    }
};

template <bool Is_Trivially_Destructible, typename... Types>
struct variant_base
{
    detail::variant_values<Types...> values;
    detail::variant_index_type<sizeof...(Types)> index_value;
    template <typename... Args>
    constexpr variant_base(std::size_t index_value, Args &&... args) //
        noexcept(noexcept(new(std::declval<void *>())
                              detail::variant_values<Types...>(std::declval<Args>()...)))
        : values(std::forward<Args>(args)...), index_value(index_value)
    {
    }
    ~variant_base()
    {
        values.destroy(index_value.get());
    }
};

template <typename... Types>
struct variant_base<true, Types...>
{
    detail::variant_values<Types...> values;
    detail::variant_index_type<sizeof...(Types)> index_value;
    template <typename... Args>
    constexpr variant_base(std::size_t index_value, Args &&... args) //
        noexcept(noexcept(new(std::declval<void *>())
                              detail::variant_values<Types...>(std::declval<Args>()...)))
        : values(std::forward<Args>(args)...), index_value(index_value)
    {
    }
    ~variant_base() = default;
};

template <typename T>
struct variant_is_in_place_index
{
    static constexpr bool value = false;
};

template <std::size_t I>
struct variant_is_in_place_index<in_place_index_t<I>>
{
    static constexpr bool value = true;
};

template <typename T>
struct variant_is_in_place_type
{
    static constexpr bool value = false;
};

template <typename T>
struct variant_is_in_place_type<in_place_type_t<T>>
{
    static constexpr bool value = true;
};
}

template <typename... Types>
class variant
    : private detail::variant_base<detail::variant_values<Types...>::is_trivially_destructible,
                                   Types...>
{
    static_assert(sizeof...(Types) > 0, "empty variant is not permitted");

private:
    typedef typename detail::variant_values<Types...>::type_0 type_0;
    typedef detail::variant_base<detail::variant_values<Types...>::is_trivially_destructible,
                                 Types...> base;

private:
    using base::values;
    using base::index_value;

public:
    template <
        typename = typename std::enable_if<std::is_default_constructible<type_0>::value>::value>
    constexpr variant() noexcept(std::is_nothrow_default_constructible<type_0>::value)
        : base(0)
    {
    }
    template <
        typename =
            typename std::enable_if<detail::variant_values<Types...>::is_copy_constructible>::type>
    variant(const variant &rt) noexcept(
        detail::variant_values<Types...>::is_nothrow_copy_constructible)
        : base(variant_npos, in_place_index<variant_npos>())
    {
        values.copy_construct(rt.values, rt.index_value.get());
        index_value = rt.index_value;
    }
    template <
        typename =
            typename std::enable_if<detail::variant_values<Types...>::is_move_constructible>::type>
    variant(variant &&rt) noexcept(detail::variant_values<Types...>::is_nothrow_move_constructible)
        : base(variant_npos, in_place_index<variant_npos>())
    {
        values.move_construct(std::move(rt.values), rt.index_value.get());
        index_value = rt.index_value;
    }
    template <
        typename T,
        std::size_t Index = detail::variant_conversion_deduce_index<T, Types...>(),
        typename = typename std::
            enable_if<!std::is_same<typename std::decay<T>::type, variant>::value
                      && !detail::variant_is_in_place_index<typename std::decay<T>::type>::value
                      && !detail::variant_is_in_place_type<typename std::decay<T>::type>::value>::
                type>
    constexpr variant(T &&value) noexcept(
        std::is_nothrow_constructible<variant_alternative_t<Index, variant<Types...>>, T>::value)
        : base(Index, in_place_index<Index>, std::forward<T>(value))
    {
    }
    template <std::size_t Index,
              typename... Args,
              typename = typename std::
                  enable_if<std::is_constructible<variant_alternative_t<Index, variant<Types...>>,
                                                  Args...>::value>::type>
    constexpr explicit variant(in_place_index_t<Index>, Args &&... args) noexcept(
        std::is_nothrow_constructible<variant_alternative_t<Index, variant<Types...>>,
                                      Args...>::value)
        : base(Index, in_place_index<Index>, std::forward<Args>(args)...)
    {
    }
    template <std::size_t Index,
              typename U,
              typename... Args,
              typename = typename std::
                  enable_if<std::is_constructible<variant_alternative_t<Index, variant<Types...>>,
                                                  std::initializer_list<U>,
                                                  Args...>::value>::type>
    constexpr explicit variant(in_place_index_t<Index>,
                               std::initializer_list<U> il,
                               Args &&... args) //
        noexcept(std::is_nothrow_constructible<variant_alternative_t<Index, variant<Types...>>,
                                               std::initializer_list<U>,
                                               Args...>::value)
        : base(Index, in_place_index<Index>, il, std::forward<Args>(args)...)
    {
    }
#error finish
    constexpr std::size_t index() const noexcept
    {
        return index_value.get();
    }
};
}
}

namespace std
{
template <>
struct hash<vulkan_cpu::util::monostate>
{
    constexpr std::size_t operator()(vulkan_cpu::util::monostate) const noexcept
    {
        return 5546275UL;
    }
};
}

#endif /* SOURCE_UTIL_VARIANT_H_ */
