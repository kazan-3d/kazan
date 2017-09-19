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
#ifndef UTIL_CIRCULAR_QUEUE_H_
#define UTIL_CIRCULAR_QUEUE_H_

#include <new>
#include <utility>
#include <type_traits>
#include <cassert>

namespace kazan
{
namespace util
{
template <typename T, std::size_t Capacity>
class Static_circular_deque
{
    static_assert(Capacity != 0, "");
    static_assert(std::is_nothrow_destructible<T>::value, "");

private:
    union
    {
        T objects[Capacity];
        alignas(T) char bytes[sizeof(T) * Capacity];
    };
    std::size_t front_index = 0;
    std::size_t back_index = Capacity - 1;
    std::size_t enqueued_count = 0;
    static constexpr std::size_t prev_index(std::size_t index) noexcept
    {
        return index == 0 ? Capacity - 1 : index - 1;
    }
    static constexpr std::size_t next_index(std::size_t index) noexcept
    {
        return index == Capacity - 1 ? 0 : index + 1;
    }

public:
    constexpr Static_circular_deque() noexcept : bytes{}
    {
    }
    ~Static_circular_deque()
    {
        while(!empty())
            pop_back();
    }
    Static_circular_deque(Static_circular_deque &&rt) noexcept(
        std::is_nothrow_move_constructible<T>::value)
        : Static_circular_deque()
    {
        try
        {
            while(!rt.empty())
            {
                push_back(std::move(rt.front()));
                rt.pop_front();
            }
        }
        catch(...)
        {
            while(!empty())
                pop_back();
            throw;
        }
    }
    Static_circular_deque &operator=(Static_circular_deque &&rt) noexcept(
        std::is_nothrow_move_constructible<T>::value)
    {
        if(this == &rt)
            return *this;
        while(!empty())
            pop_back();
        while(!rt.empty())
        {
            push_back(std::move(rt.front()));
            rt.pop_front();
        }
        return *this;
    }
    std::size_t size() const noexcept
    {
        return enqueued_count;
    }
    std::size_t capacity() const noexcept
    {
        return Capacity;
    }
    bool empty() const noexcept
    {
        return enqueued_count == 0;
    }
    bool full() const noexcept
    {
        return enqueued_count == Capacity;
    }
    T &front() noexcept
    {
        assert(!empty());
        return objects[front_index];
    }
    T &back() noexcept
    {
        assert(!empty());
        return objects[back_index];
    }
    void pop_back() noexcept
    {
        assert(!empty());
        std::size_t new_index = prev_index(back_index);
        objects[back_index].~T();
        enqueued_count--;
        back_index = new_index;
    }
    void pop_front() noexcept
    {
        assert(!empty());
        std::size_t new_index = next_index(front_index);
        objects[front_index].~T();
        enqueued_count--;
        front_index = new_index;
    }
    template <typename... Args>
    void emplace_back(Args &&... args) noexcept(std::is_nothrow_constructible<T, Args &&...>::value)
    {
        assert(!full());
        std::size_t new_index = next_index(back_index);
        ::new(std::addressof(objects[new_index])) T(std::forward<Args>(args)...);
        enqueued_count++;
        back_index = new_index;
    }
    template <typename... Args>
    void emplace_front(Args &&... args) noexcept(std::is_nothrow_constructible<T, Args &&...>::value)
    {
        assert(!full());
        std::size_t new_index = prev_index(front_index);
        ::new(std::addressof(objects[new_index])) T(std::forward<Args>(args)...);
        enqueued_count++;
        front_index = new_index;
    }
    void push_back(const T &new_value) noexcept(std::is_nothrow_copy_constructible<T>::value)
    {
        emplace_back(new_value);
    }
    void push_back(T &&new_value) noexcept(std::is_nothrow_move_constructible<T>::value)
    {
        emplace_back(std::move(new_value));
    }
    void push_front(const T &new_value) noexcept(std::is_nothrow_copy_constructible<T>::value)
    {
        emplace_front(new_value);
    }
    void push_front(T &&new_value) noexcept(std::is_nothrow_move_constructible<T>::value)
    {
        emplace_front(std::move(new_value));
    }
};
}
}

#endif

