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
#ifndef UTIL_OBSERVER_PTR_H_
#define UTIL_OBSERVER_PTR_H_

#include <cstddef>
#include <type_traits>
#include <functional>
#include <utility>

namespace kazan
{
namespace util
{
template <typename T>
class observer_ptr
{
public:
    typedef T element_type;

private:
    T *value;

public:
    constexpr observer_ptr() noexcept : value(nullptr)
    {
    }
    constexpr observer_ptr(std::nullptr_t) noexcept : value(nullptr)
    {
    }
    explicit constexpr observer_ptr(T *value) noexcept : value(value)
    {
    }
    template <typename T2,
              typename = typename std::enable_if<std::is_convertible<T2 *, T *>::value>::type>
    observer_ptr(observer_ptr<T2> rt) noexcept : value(rt.get())
    {
    }
    constexpr T *release() noexcept
    {
        T *retval = value;
        value = nullptr;
        return retval;
    }
    constexpr void reset(T *new_value = nullptr) noexcept
    {
        value = new_value;
    }
    constexpr void swap(observer_ptr &other) noexcept
    {
        T *temp = value;
        value = other.value;
        other.value = temp;
    }
    constexpr T *get() const noexcept
    {
        return value;
    }
    constexpr explicit operator bool() const noexcept
    {
        return value != nullptr;
    }
    constexpr typename std::add_lvalue_reference<T>::type operator*() const noexcept
    {
        return *value;
    }
    constexpr T *operator->() const noexcept
    {
        return value;
    }
    constexpr explicit operator T *() const noexcept
    {
        return value;
    }
};

template <typename T>
constexpr observer_ptr<T> make_observer(T *value) noexcept
{
    return observer_ptr<T>(value);
}

template <typename T1, typename T2>
constexpr bool operator==(const observer_ptr<T1> &l, const observer_ptr<T2> &r) noexcept
{
    return l.get() == r.get();
}

template <typename T1, typename T2>
constexpr bool operator!=(const observer_ptr<T1> &l, const observer_ptr<T2> &r) noexcept
{
    return !(l == r);
}

template <typename T>
constexpr bool operator==(const observer_ptr<T> &p, std::nullptr_t) noexcept
{
    return !p;
}

template <typename T>
constexpr bool operator==(std::nullptr_t, const observer_ptr<T> &p) noexcept
{
    return !p;
}

template <typename T>
constexpr bool operator!=(const observer_ptr<T> &p, std::nullptr_t) noexcept
{
    return static_cast<bool>(p);
}

template <typename T>
constexpr bool operator!=(std::nullptr_t, const observer_ptr<T> &p) noexcept
{
    return static_cast<bool>(p);
}

template <typename T1, typename T2>
constexpr bool operator<(const observer_ptr<T1> &l, const observer_ptr<T2> &r) noexcept
{
    return std::less<typename std::common_type<T1 *, T2 *>::type>()(l.get(), r.get());
}

template <typename T1, typename T2>
constexpr bool operator>(const observer_ptr<T1> &l, const observer_ptr<T2> &r) noexcept
{
    return r < l;
}

template <typename T1, typename T2>
constexpr bool operator<=(const observer_ptr<T1> &l, const observer_ptr<T2> &r) noexcept
{
    return !(r < l);
}

template <typename T1, typename T2>
constexpr bool operator>=(const observer_ptr<T1> &l, const observer_ptr<T2> &r) noexcept
{
    return !(l < r);
}
}
}

namespace std
{
template <typename T>
constexpr void swap(kazan::util::observer_ptr<T> &l, kazan::util::observer_ptr<T> &r) noexcept
{
    l.swap(r);
}

template <typename T>
struct hash<kazan::util::observer_ptr<T>>
{
    constexpr std::size_t operator()(kazan::util::observer_ptr<T> v) const noexcept
    {
        return std::hash<T *>()(v);
    }
};
}

#endif // UTIL_OBSERVER_PTR_H_
