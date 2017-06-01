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
#include "copy_cv_ref.h"
#include "is_swappable.h"
#include "invoke.h"

namespace vulkan_cpu
{
namespace util
{
class bad_variant_access : public std::exception
{
public:
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
struct variant_alternative<Index, volatile T>
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
struct variant_hypothetical_overload_set<>
{
    static void fn(); // not implemented
};

template <typename T>
struct variant_is_equals_comparable
{
private:
    static std::false_type fn(...);
    template <typename T2,
              typename = decltype(static_cast<bool>(std::declval<const T2 &>()
                                                    == std::declval<const T2 &>()))>
    static std::true_type fn(const T2 &);

public:
    static constexpr bool value = decltype(fn(std::declval<const T &>()))::value;
};

template <typename T>
struct variant_is_less_comparable
{
private:
    static std::false_type fn(...);
    template <typename T2,
              typename = decltype(static_cast<bool>(std::declval<const T2 &>()
                                                    < std::declval<const T2 &>()))>
    static std::true_type fn(const T2 &);

public:
    static constexpr bool value = decltype(fn(std::declval<const T &>()))::value;
};

template <typename T>
struct variant_is_nothrow_equals_comparable
{
private:
    static std::false_type fn(...);
    template <typename T2>
    static std::integral_constant<bool,
                                  noexcept(static_cast<bool>(std::declval<const T2 &>()
                                                             == std::declval<const T2 &>()))>
        fn(const T2 &);

public:
    static constexpr bool value = decltype(fn(std::declval<const T &>()))::value;
};

template <typename T>
struct variant_is_nothrow_less_comparable
{
private:
    static std::false_type fn(...);
    template <typename T2>
    static std::integral_constant<bool,
                                  noexcept(static_cast<bool>(std::declval<const T2 &>()
                                                             < std::declval<const T2 &>()))>
        fn(const T2 &);

public:
    static constexpr bool value = decltype(fn(std::declval<const T &>()))::value;
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
    static constexpr bool is_swappable = true;
    static constexpr bool is_nothrow_swappable = true;
    static constexpr bool is_equals_comparable = true;
    static constexpr bool is_less_comparable = true;
    static constexpr bool is_nothrow_equals_comparable = true;
    static constexpr bool is_nothrow_less_comparable = true;
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
    void swap(variant_values &rt, std::size_t index) noexcept
    {
    }
    bool is_equal(const variant_values &rt, std::size_t index) const noexcept
    {
        return true;
    }
    bool is_less(const variant_values &rt, std::size_t index) const noexcept
    {
        return false;
    }
};

template <typename T, typename... Types>
union variant_values<T, Types...>
{
    typedef T type_0;
    static_assert(!std::is_void<T>::value, "invalid variant member type");
    static_assert(!std::is_reference<T>::value, "invalid variant member type");
    static_assert(!std::is_array<T>::value, "invalid variant member type");
    static_assert(!std::is_const<T>::value, "invalid variant member type");
    static_assert(!std::is_volatile<T>::value, "invalid variant member type");
    static_assert(std::is_object<T>::value, "invalid variant member type");
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
    static constexpr bool is_copy_assignable = std::is_copy_assignable<T>::value
                                               && std::is_copy_constructible<T>::value
                                               && variant_values<Types...>::is_copy_assignable;
    static constexpr bool is_move_assignable = std::is_move_assignable<T>::value
                                               && std::is_move_constructible<T>::value
                                               && variant_values<Types...>::is_move_assignable;
    static constexpr bool is_nothrow_copy_assignable =
        std::is_nothrow_copy_assignable<T>::value && std::is_nothrow_copy_constructible<T>::value
        && variant_values<Types...>::is_nothrow_copy_assignable;
    static constexpr bool is_nothrow_move_assignable =
        std::is_nothrow_move_assignable<T>::value && std::is_nothrow_move_constructible<T>::value
        && variant_values<Types...>::is_nothrow_move_assignable;
    static constexpr bool is_trivially_destructible =
        std::is_trivially_destructible<T>::value
        && variant_values<Types...>::is_trivially_destructible;
    static constexpr bool is_swappable = is_swappable_v<T> && std::is_move_constructible<T>::value
                                         && variant_values<Types...>::is_swappable;
    static constexpr bool is_nothrow_swappable =
        is_nothrow_swappable_v<T> && std::is_nothrow_move_constructible<T>::value
        && variant_values<Types...>::is_nothrow_swappable;
    static constexpr bool is_equals_comparable =
        variant_is_equals_comparable<T>::value && variant_values<Types...>::is_equals_comparable;
    static constexpr bool is_less_comparable =
        variant_is_less_comparable<T>::value && variant_values<Types...>::is_less_comparable;
    static constexpr bool is_nothrow_equals_comparable =
        variant_is_nothrow_equals_comparable<T>::value
        && variant_values<Types...>::is_nothrow_equals_comparable;
    static constexpr bool is_nothrow_less_comparable =
        variant_is_nothrow_less_comparable<T>::value
        && variant_values<Types...>::is_nothrow_less_comparable;
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
        std::size_t next = variant_values<Types...>::template index_from_type<U>();
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
    void swap(variant_values &rt, std::size_t index) noexcept(is_nothrow_swappable)
    {
        using std::swap;
        if(index == 0)
            swap(current_value, rt.current_value);
        else
            other_values.swap(rt.other_values, index - 1);
    }
    bool is_equal(const variant_values &rt, std::size_t index) const
        noexcept(is_nothrow_equals_comparable)
    {
        if(index == 0)
            return static_cast<bool>(current_value == rt.current_value);
        return other_values.is_equal(rt.other_values, index - 1);
    }
    bool is_less(const variant_values &rt, std::size_t index) const
        noexcept(is_nothrow_less_comparable)
    {
        if(index == 0)
            return static_cast<bool>(current_value < rt.current_value);
        return other_values.is_equal(rt.other_values, index - 1);
    }
};

template <std::size_t Index, typename... Types>
struct variant_get;

template <std::size_t Index, typename T, typename... Types>
struct variant_get<Index, T, Types...>
{
    static constexpr auto get(const variant_values<T, Types...> &values) noexcept
        -> decltype(variant_get<Index - 1, Types...>::get(values.other_values))
    {
        return variant_get<Index - 1, Types...>::get(values.other_values);
    }
    static constexpr auto get(variant_values<T, Types...> &values) noexcept
        -> decltype(variant_get<Index - 1, Types...>::get(values.other_values))
    {
        return variant_get<Index - 1, Types...>::get(values.other_values);
    }
    static constexpr auto get(const variant_values<T, Types...> &&values) noexcept
        -> decltype(variant_get<Index - 1, Types...>::get(std::move(values.other_values)))
    {
        return variant_get<Index - 1, Types...>::get(std::move(values.other_values));
    }
    static constexpr auto get(variant_values<T, Types...> &&values) noexcept
        -> decltype(variant_get<Index - 1, Types...>::get(std::move(values.other_values)))
    {
        return variant_get<Index - 1, Types...>::get(std::move(values.other_values));
    }
};

template <typename T, typename... Types>
struct variant_get<0, T, Types...>
{
    static constexpr const T &get(const variant_values<T, Types...> &values) noexcept
    {
        return values.current_value;
    }
    static constexpr T &get(variant_values<T, Types...> &values) noexcept
    {
        return values.current_value;
    }
    static constexpr const T &&get(const variant_values<T, Types...> &&values) noexcept
    {
        return std::move(values.current_value);
    }
    static constexpr T &&get(variant_values<T, Types...> &&values) noexcept
    {
        return std::move(values.current_value);
    }
};

#define VULKAN_CPU_UTIL_VARIANT_DISPATCH(Const, Ref)                                             \
    template <typename Fn,                                                                       \
              typename... Args,                                                                  \
              typename... Types,                                                                 \
              std::size_t... Indexes,                                                            \
              typename Return_Type = typename std::common_type<decltype(std::declval<Fn>()(      \
                  std::declval<Const Types Ref>(), std::declval<Args>()...))...>::type>          \
    constexpr Return_Type variant_dispatch_helper(Fn &&fn,                                       \
                                                  Const variant_values<Types...> Ref values,     \
                                                  std::size_t index,                             \
                                                  std::index_sequence<Indexes...>,               \
                                                  Args &&... args)                               \
    {                                                                                            \
        typedef Return_Type (*Dispatch_Function)(                                                \
            Fn && fn, Const variant_values<Types...> & values, Args && ... args);                \
        const Dispatch_Function dispatch_functions[sizeof...(Types)] = {                         \
            static_cast<Dispatch_Function>(                                                      \
                [](Fn &&fn, Const variant_values<Types...> &values, Args &&... args)             \
                    -> Return_Type                                                               \
                {                                                                                \
                    return std::forward<Fn>(fn)(                                                 \
                        variant_get<Indexes, Types...>::get(                                     \
                            std::forward<Const variant_values<Types...> Ref>(values)),           \
                        std::forward<Args>(args)...);                                            \
                })...,                                                                           \
        };                                                                                       \
        if(index < sizeof...(Types))                                                             \
            return dispatch_functions[index](                                                    \
                std::forward<Fn>(fn), values, std::forward<Args>(args)...);                      \
        throw bad_variant_access();                                                              \
    }                                                                                            \
                                                                                                 \
    template <typename Fn, typename... Args, typename... Types>                                  \
    constexpr auto variant_dispatch(                                                             \
        Fn &&fn, Const variant_values<Types...> Ref values, std::size_t index, Args &&... args)  \
        ->decltype(                                                                              \
            variant_dispatch_helper(std::forward<Fn>(fn),                                        \
                                    std::forward<Const variant_values<Types...> Ref>(values),    \
                                    index,                                                       \
                                    std::index_sequence_for<Types...>{},                         \
                                    std::forward<Args>(args)...))                                \
    {                                                                                            \
        return variant_dispatch_helper(std::forward<Fn>(fn),                                     \
                                       std::forward<Const variant_values<Types...> Ref>(values), \
                                       index,                                                    \
                                       std::index_sequence_for<Types...>{},                      \
                                       std::forward<Args>(args)...);                             \
    }                                                                                            \
                                                                                                 \
    template <typename Fn,                                                                       \
              typename... Args,                                                                  \
              typename... Types,                                                                 \
              std::size_t... Indexes,                                                            \
              typename Return_Type = typename std::common_type<decltype(std::declval<Fn>()(      \
                  std::declval<Const Types Ref>(), std::declval<Args>()...))...>::type>          \
    constexpr Return_Type variant_dispatch_helper_nothrow(                                       \
        Fn &&fn,                                                                                 \
        Const variant_values<Types...> Ref values,                                               \
        std::size_t index,                                                                       \
        std::index_sequence<Indexes...>,                                                         \
        Args &&... args)                                                                         \
    {                                                                                            \
        typedef Return_Type (*Dispatch_Function)(                                                \
            Fn && fn, Const variant_values<Types...> & values, Args && ... args);                \
        const Dispatch_Function dispatch_functions[sizeof...(Types)] = {                         \
            static_cast<Dispatch_Function>(                                                      \
                [](Fn &&fn, Const variant_values<Types...> &values, Args &&... args)             \
                    -> Return_Type                                                               \
                {                                                                                \
                    return std::forward<Fn>(fn)(                                                 \
                        variant_get<Indexes, Types...>::get(                                     \
                            std::forward<Const variant_values<Types...> Ref>(values)),           \
                        std::forward<Args>(args)...);                                            \
                })...,                                                                           \
        };                                                                                       \
        if(index < sizeof...(Types))                                                             \
            return dispatch_functions[index](                                                    \
                std::forward<Fn>(fn), values, std::forward<Args>(args)...);                      \
        return {};                                                                               \
    }                                                                                            \
                                                                                                 \
    template <typename Fn, typename... Args, typename... Types>                                  \
    constexpr auto variant_dispatch_nothrow(                                                     \
        Fn &&fn, Const variant_values<Types...> Ref values, std::size_t index, Args &&... args)  \
        ->decltype(variant_dispatch_helper_nothrow(                                              \
            std::forward<Fn>(fn),                                                                \
            std::forward<Const variant_values<Types...> Ref>(values),                            \
            index,                                                                               \
            std::index_sequence_for<Types...>{},                                                 \
            std::forward<Args>(args)...))                                                        \
    {                                                                                            \
        return variant_dispatch_helper_nothrow(                                                  \
            std::forward<Fn>(fn),                                                                \
            std::forward<Const variant_values<Types...> Ref>(values),                            \
            index,                                                                               \
            std::index_sequence_for<Types...>{},                                                 \
            std::forward<Args>(args)...);                                                        \
    }

VULKAN_CPU_UTIL_VARIANT_DISPATCH(, &)
VULKAN_CPU_UTIL_VARIANT_DISPATCH(const, &)
VULKAN_CPU_UTIL_VARIANT_DISPATCH(, &&)
VULKAN_CPU_UTIL_VARIANT_DISPATCH(const, &&)
#undef VULKAN_CPU_UTIL_VARIANT_DISPATCH

template <typename T,
          typename... Types,
          typename Deduced_Type = typename decltype(
              variant_hypothetical_overload_set<Types...>::fn(std::declval<T>()))::type,
          std::size_t Index = variant_values<Types...>::template index_from_type<Deduced_Type>(),
          typename = typename std::enable_if<(Index < sizeof...(Types))>::type>
constexpr std::size_t variant_conversion_deduce_index() noexcept
{
    return Index;
}

template <typename T, typename... Types>
using variant_conversion_deduce_type =
    variant_alternative_t<variant_conversion_deduce_index<T, Types...>(), variant<Types...>>;

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

template <typename Fn, typename... Types, typename... Args>
typename std::common_type<decltype(std::declval<Fn>()(std::declval<Types &>()))...>::type
    variant_dispatch(Fn &&fn, variant<Types...> &v, Args &&... args);

template <typename Fn, typename... Types, typename... Args>
typename std::common_type<decltype(std::declval<Fn>()(std::declval<const Types &>()))...>::type
    variant_dispatch(Fn &&fn, const variant<Types...> &v, Args &&... args);

template <typename Fn, typename... Types, typename... Args>
typename std::common_type<decltype(std::declval<Fn>()(std::declval<Types &&>()))...>::type
    variant_dispatch(Fn &&fn, variant<Types...> &&v, Args &&... args);

template <typename Fn, typename... Types, typename... Args>
typename std::common_type<decltype(std::declval<Fn>()(std::declval<const Types &&>()))...>::type
    variant_dispatch(Fn &&fn, const variant<Types...> &&v, Args &&... args);
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
        : base(variant_npos, in_place_index<variant_npos>)
    {
        values.copy_construct(rt.values, rt.index_value.get());
        index_value = rt.index_value;
    }
    template <
        typename =
            typename std::enable_if<detail::variant_values<Types...>::is_move_constructible>::type>
    variant(variant &&rt) noexcept(detail::variant_values<Types...>::is_nothrow_move_constructible)
        : base(variant_npos, in_place_index<variant_npos>)
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
                      && !detail::variant_is_in_place_type<typename std::decay<T>::type>::value
                      && std::is_constructible<variant_alternative_t<Index, variant<Types...>>,
                                               T>::value>::type>
    constexpr variant(T &&value) noexcept(
        std::is_nothrow_constructible<variant_alternative_t<Index, variant<Types...>>, T>::value)
        : base(Index, in_place_index<Index>, std::forward<T>(value))
    {
    }
    template <typename T,
              typename... Args,
              std::size_t Index = detail::variant_values<Types...>::template index_from_type<T>(),
              typename = typename std::enable_if<(Index < sizeof...(Types))
                                                 && std::is_constructible<T, Args...>::value>::type>
    constexpr explicit variant(in_place_type_t<T>, Args &&... args) noexcept(
        std::is_nothrow_constructible<T, Args...>::value)
        : base(Index, in_place_index<Index>, std::forward<Args>(args)...)
    {
    }
    template <
        typename T,
        typename U,
        typename... Args,
        std::size_t Index = detail::variant_values<Types...>::template index_from_type<T>(),
        typename = typename std::
            enable_if<(Index < sizeof...(Types))
                      && std::is_constructible<T, std::initializer_list<U>, Args...>::value>::type>
    constexpr explicit variant(
        in_place_type_t<T>,
        std::initializer_list<U> il,
        Args &&... args) noexcept(std::is_nothrow_constructible<T,
                                                                std::initializer_list<U>,
                                                                Args...>::value)
        : base(Index, in_place_index<Index>, il, std::forward<Args>(args)...)
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
    template <
        typename =
            typename std::enable_if<detail::variant_values<Types...>::is_copy_assignable>::type>
    variant &operator=(const variant &rt) noexcept(
        detail::variant_values<Types...>::is_nothrow_copy_assignable)
    {
        if(index_value.get() == rt.index_value.get())
        {
            values.copy_assign(rt.values, index_value.get());
        }
        else
        {
            values.destruct(index_value.get());
            index_value.set(variant_npos); // in case copy_construct throws
            values.copy_construct(rt, rt.index_value.get());
            index_value = rt.index_value;
        }
        return *this;
    }
    template <
        typename =
            typename std::enable_if<detail::variant_values<Types...>::is_move_assignable>::type>
    variant &operator=(variant &&rt) noexcept(
        detail::variant_values<Types...>::is_nothrow_move_assignable)
    {
        if(index_value.get() == rt.index_value.get())
        {
            values.move_assign(std::move(rt.values), index_value.get());
        }
        else
        {
            values.destruct(index_value.get());
            index_value.set(variant_npos); // in case move_construct throws
            values.move_construct(std::move(rt), rt.index_value.get());
            index_value = rt.index_value;
        }
        return *this;
    }
    template <
        typename T,
        std::size_t Index = detail::variant_conversion_deduce_index<T, Types...>(),
        typename = typename std::
            enable_if<!std::is_same<typename std::decay<T>::type, variant>::value
                      && !detail::variant_is_in_place_index<typename std::decay<T>::type>::value
                      && !detail::variant_is_in_place_type<typename std::decay<T>::type>::value
                      && std::is_constructible<variant_alternative_t<Index, variant<Types...>>,
                                               T>::value
                      && std::is_assignable<variant_alternative_t<Index, variant<Types...>>,
                                            T>::value>::type>
    variant &operator=(T &&new_value) noexcept(
        std::is_nothrow_constructible<variant_alternative_t<Index, variant<Types...>>, T>::value
            &&std::is_nothrow_assignable<variant_alternative_t<Index, variant<Types...>>, T>::value)
    {
        if(index_value.get() == Index)
        {
            detail::variant_get<Index, Types...>::get(values) = std::forward<T>(new_value);
        }
        else
        {
            values.destruct(index_value.get());
            index_value.set(variant_npos); // in case construction throws
            auto &value = detail::variant_get<Index, Types...>::get(values);
            new(const_cast<void *>(std::addressof(value)))
                variant_alternative_t<Index, variant<Types...>>(std::forward<T>(new_value));
            index_value.set(Index);
        }
        return *this;
    }
    template <typename T,
              typename... Args,
              std::size_t Index = detail::variant_values<Types...>::template index_from_type<T>(),
              typename = typename std::enable_if<(Index < sizeof...(Types))
                                                 && std::is_constructible<T, Args...>::value>::type>
    void emplace(Args &&... args)
    {
        emplace<Index>(std::forward<Args>(args)...);
    }
    template <
        typename T,
        typename U,
        typename... Args,
        std::size_t Index = detail::variant_values<Types...>::template index_from_type<T>(),
        typename = typename std::
            enable_if<(Index < sizeof...(Types))
                      && std::is_constructible<T, std::initializer_list<U>, Args...>::value>::type>
    void emplace(std::initializer_list<U> il, Args &&... args)
    {
        emplace<Index>(il, std::forward<Args>(args)...);
    }
    template <std::size_t Index,
              typename... Args,
              typename = typename std::
                  enable_if<(Index < sizeof...(Types))
                            && std::is_constructible<variant_alternative_t<Index,
                                                                           variant<Types...>>,
                                                     Args...>::value>::type>
    void emplace(Args &&... args)
    {
        values.destruct(index_value.get());
        index_value.set(variant_npos); // in case construction throws
        auto &value = detail::variant_get<Index, Types...>::get(values);
        new(const_cast<void *>(std::addressof(value)))
            variant_alternative_t<Index, variant<Types...>>(std::forward<Args>(args)...);
        index_value.set(Index);
    }
    template <std::size_t Index,
              typename U,
              typename... Args,
              typename = typename std::
                  enable_if<(Index < sizeof...(Types))
                            && std::is_constructible<variant_alternative_t<Index,
                                                                           variant<Types...>>,
                                                     std::initializer_list<U>,
                                                     Args...>::value>::type>
    void emplace(std::initializer_list<U> il, Args &&... args)
    {
        values.destruct(index_value.get());
        index_value.set(variant_npos); // in case construction throws
        auto &value = detail::variant_get<Index, Types...>::get(values);
        new(const_cast<void *>(std::addressof(value)))
            variant_alternative_t<Index, variant<Types...>>(il, std::forward<Args>(args)...);
        index_value.set(Index);
    }
    constexpr bool valueless_by_exception() const noexcept
    {
        return index_value.get() == variant_npos;
    }
    constexpr std::size_t index() const noexcept
    {
        return index_value.get();
    }
    template <
        typename = typename std::enable_if<detail::variant_values<Types...>::is_swappable>::type>
    void swap(variant &rt) noexcept(detail::variant_values<Types...>::is_nothrow_swappable)
    {
        if(index_value.get() == rt.index_value.get())
            values.swap(rt.values, index_value.get());
        else
        {
            variant temp = std::move(rt);
            rt = std::move(*this);
            *this = std::move(temp);
        }
    }
    template <typename T, typename... Types2>
    friend constexpr bool holds_alternative(const variant<Types2...> &v) noexcept;
    template <std::size_t Index, typename... Types2>
    friend constexpr variant_alternative_t<Index, variant<Types2...>> &get(variant<Types2...> &v);
    template <std::size_t Index, typename... Types2>
    friend constexpr const variant_alternative_t<Index, variant<Types2...>> &get(
        const variant<Types2...> &v);
    template <typename... Types2>
    friend constexpr
        typename std::enable_if<detail::variant_values<Types2...>::is_equals_comparable, bool>::type
        operator==(const variant<Types2...> &l, const variant<Types2...> &r) noexcept(
            detail::variant_values<Types2...>::is_nothrow_equals_comparable);
    template <typename... Types2>
    friend constexpr
        typename std::enable_if<detail::variant_values<Types2...>::is_less_comparable, bool>::type
        operator<(const variant<Types2...> &l, const variant<Types2...> &r) noexcept(
            detail::variant_values<Types2...>::is_nothrow_less_comparable);
    template <typename Fn, typename... Types2, typename... Args>
    friend typename std::common_type<decltype(std::declval<Fn>()(std::declval<Types2 &>()))...>::type
        detail::variant_dispatch(Fn &&fn, variant<Types2...> &v, Args &&... args);
    template <typename Fn, typename... Types2, typename... Args>
    friend typename std::common_type<decltype(
        std::declval<Fn>()(std::declval<const Types2 &>()))...>::type
        detail::variant_dispatch(Fn &&fn, const variant<Types2...> &v, Args &&... args);
    template <typename Fn, typename... Types2, typename... Args>
    friend
        typename std::common_type<decltype(std::declval<Fn>()(std::declval<Types2 &&>()))...>::type
        detail::variant_dispatch(Fn &&fn, variant<Types2...> &&v, Args &&... args);
    template <typename Fn, typename... Types2, typename... Args>
    friend typename std::common_type<decltype(
        std::declval<Fn>()(std::declval<const Types2 &&>()))...>::type
        detail::variant_dispatch(Fn &&fn, const variant<Types2...> &&v, Args &&... args);
};

template <typename T, typename... Types>
constexpr bool holds_alternative(const variant<Types...> &v) noexcept
{
    constexpr std::size_t index = detail::variant_values<Types...>::template index_from_type<T>();
    static_assert(index != variant_npos, "");
    return v.index_value.get() == index;
}

template <std::size_t Index, typename... Types>
constexpr variant_alternative_t<Index, variant<Types...>> &get(variant<Types...> &v)
{
    static_assert(Index < sizeof...(Types), "");
    if(v.index_value.get() == Index)
        return detail::variant_get<Index, Types...>::get(v.values);
    throw bad_variant_access();
}

template <std::size_t Index, typename... Types>
constexpr const variant_alternative_t<Index, variant<Types...>> &get(const variant<Types...> &v)
{
    static_assert(Index < sizeof...(Types), "");
    if(v.index_value.get() == Index)
        return detail::variant_get<Index, Types...>::get(v.values);
    throw bad_variant_access();
}

template <std::size_t Index, typename... Types>
constexpr const variant_alternative_t<Index, variant<Types...>> &&get(const variant<Types...> &&v)
{
    return std::move(get<Index>(v));
}

template <std::size_t Index, typename... Types>
constexpr variant_alternative_t<Index, variant<Types...>> &&get(variant<Types...> &&v)
{
    return std::move(get<Index>(v));
}

template <typename T, typename... Types>
constexpr const T &get(const variant<Types...> &v)
{
    return get<detail::variant_values<Types...>::template index_from_type<T>()>(v);
}

template <typename T, typename... Types>
constexpr T &get(variant<Types...> &v)
{
    return get<detail::variant_values<Types...>::template index_from_type<T>()>(v);
}

template <typename T, typename... Types>
constexpr const T &&get(const variant<Types...> &&v)
{
    return get<detail::variant_values<Types...>::template index_from_type<T>()>(std::move(v));
}

template <typename T, typename... Types>
constexpr T &&get(variant<Types...> &&v)
{
    return get<detail::variant_values<Types...>::template index_from_type<T>()>(std::move(v));
}

template <std::size_t Index, typename... Types>
constexpr const variant_alternative_t<Index, variant<Types...>> *get_if(
    const variant<Types...> *v) noexcept
{
    static_assert(Index < sizeof...(Types), "");
    if(!v || v->index() != Index)
        return nullptr;
    return std::addressof(get<Index>(*v));
}

template <std::size_t Index, typename... Types>
constexpr variant_alternative_t<Index, variant<Types...>> *get_if(variant<Types...> *v) noexcept
{
    static_assert(Index < sizeof...(Types), "");
    if(!v || v->index() != Index)
        return nullptr;
    return std::addressof(get<Index>(*v));
}

template <typename T, typename... Types>
constexpr const T *get_if(const variant<Types...> *v) noexcept
{
    return get_if<detail::variant_values<Types...>::template index_from_type<T>()>(v);
}

template <typename T, typename... Types>
constexpr T *get_if(variant<Types...> *v) noexcept
{
    return get_if<detail::variant_values<Types...>::template index_from_type<T>()>(v);
}

template <typename... Types>
constexpr
    typename std::enable_if<detail::variant_values<Types...>::is_equals_comparable, bool>::type
    operator==(const variant<Types...> &l, const variant<Types...> &r) noexcept(
        detail::variant_values<Types...>::is_nothrow_equals_comparable)
{
    return l.index_value.get() == r.index_value.get()
           && l.values.is_equal(r.values, l.index_value.get());
}

template <typename... Types>
constexpr
    typename std::enable_if<detail::variant_values<Types...>::is_equals_comparable, bool>::type
    operator!=(const variant<Types...> &l, const variant<Types...> &r) noexcept(
        detail::variant_values<Types...>::is_nothrow_equals_comparable)
{
    return !operator==(l, r);
}

template <typename... Types>
constexpr typename std::enable_if<detail::variant_values<Types...>::is_less_comparable, bool>::type
    operator<(const variant<Types...> &l, const variant<Types...> &r) noexcept(
        detail::variant_values<Types...>::is_nothrow_less_comparable)
{
    if(l.index_value.get() != r.index_value.get())
        return l.index_value.get() < r.index_value.get();
    return l.values.is_less(r.values, l.index_value.get());
}

template <typename... Types>
constexpr typename std::enable_if<detail::variant_values<Types...>::is_less_comparable, bool>::type
    operator>(const variant<Types...> &l, const variant<Types...> &r) noexcept(
        detail::variant_values<Types...>::is_nothrow_less_comparable)
{
    return operator<(r, l);
}

template <typename... Types>
constexpr typename std::enable_if<detail::variant_values<Types...>::is_less_comparable, bool>::type
    operator>=(const variant<Types...> &l, const variant<Types...> &r) noexcept(
        detail::variant_values<Types...>::is_nothrow_less_comparable)
{
    return !operator<(l, r);
}

template <typename... Types>
constexpr typename std::enable_if<detail::variant_values<Types...>::is_less_comparable, bool>::type
    operator<=(const variant<Types...> &l, const variant<Types...> &r) noexcept(
        detail::variant_values<Types...>::is_nothrow_less_comparable)
{
    return !operator<(r, l);
}

namespace detail
{
template <typename Fn, typename... Types, typename... Args>
typename std::common_type<decltype(std::declval<Fn>()(std::declval<Types &>()))...>::type
    variant_dispatch(Fn &&fn, variant<Types...> &v, Args &&... args)
{
    return variant_dispatch(
        std::forward<Fn>(fn), v.values, v.index_value.get(), std::forward<Args>(args)...);
}

template <typename Fn, typename... Types, typename... Args>
typename std::common_type<decltype(std::declval<Fn>()(std::declval<const Types &>()))...>::type
    variant_dispatch(Fn &&fn, const variant<Types...> &v, Args &&... args)
{
    return variant_dispatch(
        std::forward<Fn>(fn), v.values, v.index_value.get(), std::forward<Args>(args)...);
}

template <typename Fn, typename... Types, typename... Args>
typename std::common_type<decltype(std::declval<Fn>()(std::declval<Types &&>()))...>::type
    variant_dispatch(Fn &&fn, variant<Types...> &&v, Args &&... args)
{
    return variant_dispatch(std::forward<Fn>(fn),
                            std::move(v.values),
                            v.index_value.get(),
                            std::forward<Args>(args)...);
}

template <typename Fn, typename... Types, typename... Args>
typename std::common_type<decltype(std::declval<Fn>()(std::declval<const Types &&>()))...>::type
    variant_dispatch(Fn &&fn, const variant<Types...> &&v, Args &&... args)
{
    return variant_dispatch(std::forward<Fn>(fn),
                            std::move(v.values),
                            v.index_value.get(),
                            std::forward<Args>(args)...);
}

template <typename Fn, typename... Types>
decltype(variant_dispatch(std::declval<Fn>(), std::declval<variant<Types...> &>())) variant_visit(
    Fn &&fn, variant<Types...> &v)
{
    return variant_dispatch(std::forward<Fn>(fn), v);
}

template <typename Fn, typename... Types>
decltype(variant_dispatch(std::declval<Fn>(), std::declval<const variant<Types...> &>()))
    variant_visit(Fn &&fn, const variant<Types...> &v)
{
    return variant_dispatch(std::forward<Fn>(fn), v);
}

template <typename Fn, typename... Types>
decltype(variant_dispatch(std::declval<Fn>(), std::declval<variant<Types...> &&>())) variant_visit(
    Fn &&fn, variant<Types...> &&v)
{
    return variant_dispatch(std::forward<Fn>(fn), std::move(v));
}

template <typename Fn, typename... Types>
decltype(variant_dispatch(std::declval<Fn>(), std::declval<const variant<Types...> &&>()))
    variant_visit(Fn &&fn, const variant<Types...> &&v)
{
    return variant_dispatch(std::forward<Fn>(fn), std::move(v));
}

template <typename Fn, typename... Types, typename Variant2, typename... Variants>
auto variant_visit(Fn &&fn,
                   const variant<Types...> &&v,
                   Variant2 &&variant2,
                   Variants &&... variants)
{
    return variant_dispatch(
        [&](auto &&value)
        {
            return variant_visit(
                [&](auto &&... args)
                {
                    return std::forward<Fn>(fn)(std::forward<decltype(value)>(value),
                                                std::forward<decltype(args)>(args)...);
                },
                std::forward<Variant2>(variant2),
                std::forward<Variants>(variants)...);
        },
        std::move(v));
}
}

template <typename Fn, typename... Variants>
auto visit(Fn &&fn, Variants &&... variants)
{
    return detail::variant_visit(std::forward<Fn>(fn), std::forward<Variants>(variants)...);
}
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

template <typename... Types>
struct hash<vulkan_cpu::util::variant<Types...>>
{
    constexpr std::size_t operator()(const vulkan_cpu::util::variant<Types...> &v) const
    {
        if(v.valueless_by_exception())
            return 10285473UL;
        return v.index() * 1414729UL
               + vulkan_cpu::util::visit(
                     [](const auto &v) -> std::size_t
                     {
                         return std::hash<typename std::decay<decltype(v)>::type>()(v);
                     },
                     v);
    }
};

template <typename... Types,
          typename = typename std::
              enable_if<vulkan_cpu::util::detail::variant_values<Types...>::is_swappable>::type>
inline void
    swap(vulkan_cpu::util::variant<Types...> &l, vulkan_cpu::util::variant<Types...> &r) noexcept(
        vulkan_cpu::util::detail::variant_values<Types...>::is_nothrow_swappable)
{
    l.swap(r);
}
}

#endif /* SOURCE_UTIL_VARIANT_H_ */
