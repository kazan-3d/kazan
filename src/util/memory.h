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
#ifndef UTIL_MEMORY_H_
#define UTIL_MEMORY_H_

#include <cstddef>
#include <new>
#include <cstdint>

namespace kazan
{
namespace util
{
namespace detail
{
constexpr std::size_t get_max_align_alignment() noexcept
{
    using namespace std;
    // not all standard libraries properly put max_align_t in std
    return alignof(max_align_t);
}

template <std::size_t Alignment, bool Needs_adjusting = (Alignment > get_max_align_alignment())>
struct Aligned_memory_allocator_base
{
    static_assert(Alignment != 0 && (Alignment & (Alignment - 1)) == 0, "non-power-of-2 Alignment");
    static void *allocate(std::size_t size)
    {
        return new unsigned char[size];
    }
    static void deallocate(void *p) noexcept
    {
        delete[] static_cast<unsigned char *>(p);
    }
};

template <std::size_t Alignment>
struct Aligned_memory_allocator_base<Alignment, true>
{
    static_assert(Alignment != 0 && (Alignment & (Alignment - 1)) == 0, "non-power-of-2 Alignment");
    typedef unsigned char *Base_pointer;
    static constexpr std::size_t extra_size = Alignment + sizeof(Base_pointer);
    static void *allocate(std::size_t size)
    {
        size += extra_size;
        Base_pointer base = new unsigned char[size];
        auto alignment_start = reinterpret_cast<std::uintptr_t>(base + sizeof(Base_pointer));
        auto retval =
            reinterpret_cast<unsigned char *>((alignment_start + Alignment - 1) & ~(Alignment - 1));
        auto base_location = reinterpret_cast<Base_pointer *>(retval) - 1;
        *base_location = base;
        return retval;
    }
    static void deallocate(void *p) noexcept
    {
        if(p != nullptr)
        {
            auto base_location = reinterpret_cast<Base_pointer *>(p) - 1;
            delete[] * base_location;
        }
    }
};
}

template <std::size_t Alignment>
struct Aligned_memory_allocator
{
    static void *allocate(std::size_t size)
    {
        return detail::Aligned_memory_allocator_base<Alignment>::allocate(size);
    }
    static void deallocate(void *p) noexcept
    {
        return detail::Aligned_memory_allocator_base<Alignment>::deallocate(p);
    }
    struct Deleter
    {
        void operator()(void *p) const noexcept
        {
            deallocate(p);
        }
    };
};
}
}

#endif // UTIL_MEMORY_H_
