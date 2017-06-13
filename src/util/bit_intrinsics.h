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

#ifndef UTIL_BIT_INTRINSICS_H_
#define UTIL_BIT_INTRINSICS_H_

#include <cstdint>
#include <limits>

#if defined(__clang__)
#if defined(__apple_build_version__)
#if __clang_major__ > 5 || (__clang_major__ == 5 && __clang_minor__ >= 1)
#define VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED 1
#endif
#else
#if __clang_major__ > 3 || (__clang_major__ == 3 && __clang_minor__ >= 4)
#define VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED 1
#endif
#endif
#elif defined(__INTEL_COMPILER)
#warning figure out icc version numbers for constexpr __builtin_clz and __builtin_ctz
#elif defined(__GNUC__)
// gcc supports constexpr __builtin_clz and __builtin_ctz before it supports c++14
#define VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED 1
#endif

#if 1
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
#undef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
#endif
#endif

namespace vulkan_cpu
{
namespace util
{
constexpr std::uint32_t clz4(std::uint8_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return v == 0 ? 4 : __builtin_clz(v) - __builtin_clz(0x8U);
#else
    typedef const std::uint_fast8_t LookupTableType[0x10];
    return LookupTableType
    {
        4, 3, 2, 2, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0
    }
    [v];
#endif
}

constexpr std::uint32_t clz8(std::uint8_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return v == 0 ? 8 : __builtin_clz(v) - __builtin_clz(0x80U);
#else
    return ((v & 0xF0) == 0) ? 4 + clz4(v) : clz4(v >> 4);
#endif
}

constexpr std::uint32_t clz16(std::uint16_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return v == 0 ? 16 : __builtin_clz(v) - (std::numeric_limits<int>::digits - 16);
#else
    return ((v & 0xFF00U) == 0) ? 8 + clz8(v) : clz8(v >> 8);
#endif
}

constexpr std::uint32_t clz32(std::uint32_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return v == 0 ? 32 : __builtin_clzl(v) - (std::numeric_limits<long>::digits - 32);
#else
    return ((v & 0xFFFF0000UL) == 0) ? 16 + clz16(v) : clz16(v >> 16);
#endif
}

constexpr std::uint32_t clz64(std::uint64_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return v == 0 ? 64 : __builtin_clzll(v) - (std::numeric_limits<long long>::digits - 64);
#else
    return ((v & 0xFFFFFFFF00000000ULL) == 0) ? 32 + clz32(v) : clz32(v >> 32);
#endif
}

constexpr std::uint32_t ctz4(std::uint8_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return v == 0 ? 4 : __builtin_ctz(v);
#else
    typedef const std::uint_fast8_t LookupTableType[0x10];
    return LookupTableType
    {
        4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
    }
    [v];
#endif
}

constexpr std::uint32_t ctz8(std::uint8_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return v == 0 ? 8 : __builtin_ctz(v);
#else
    return ((v & 0xF0) == 0) ? ctz4(v) : 4 + ctz4(v >> 4);
#endif
}

constexpr std::uint32_t ctz16(std::uint16_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return v == 0 ? 16 : __builtin_ctz(v);
#else
    return ((v & 0xFF00U) == 0) ? ctz8(v) : 8 + ctz8(v >> 8);
#endif
}

constexpr std::uint32_t ctz32(std::uint32_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return v == 0 ? 32 : __builtin_ctzl(v);
#else
    return ((v & 0xFFFF0000UL) == 0) ? ctz16(v) : 16 + ctz16(v >> 16);
#endif
}

constexpr std::uint32_t ctz64(std::uint64_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return v == 0 ? 64 : __builtin_ctzll(v);
#else
    return ((v & 0xFFFFFFFF00000000ULL) == 0) ? ctz32(v) : 32 + ctz32(v >> 32);
#endif
}

constexpr std::uint32_t popcount8(std::uint8_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return __builtin_popcount(v);
#else
    constexpr std::uint8_t lookup_table[0x10] = {
        0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4,
    };
    return lookup_table[v & 0xF] + lookup_table[v >> 4];
#endif
}

constexpr std::uint32_t popcount32(std::uint32_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return __builtin_popcountl(v);
#else
    constexpr std::uint32_t m1 = 0x5555'5555UL;
    constexpr std::uint32_t m2 = 0x3333'3333UL;
    constexpr std::uint32_t m4 = 0x0F0F'0F0FUL;
    v -= (v >> 1) & m1;
    v = (v & m2) + ((v >> 2) & m2);
    v = (v & m4) + ((v >> 4) & m4);
    return static_cast<std::uint32_t>(v * 0x0101'0101UL) >> 24;
#endif
}

constexpr std::uint32_t popcount16(std::uint16_t v) noexcept
{
    return popcount32(v);
}

constexpr std::uint32_t popcount64(std::uint64_t v) noexcept
{
#ifdef VULKAN_CPU_UTIL_CONSTEXPR_BUILTIN_CLZ_CTZ_POPCOUNT_SUPPORTED
    return __builtin_popcountll(v);
#else
    constexpr std::uint64_t m1 = 0x5555'5555'5555'5555ULL;
    constexpr std::uint64_t m2 = 0x3333'3333'3333'3333ULL;
    constexpr std::uint64_t m4 = 0x0F0F'0F0F'0F0F'0F0FULL;
    v -= (v >> 1) & m1;
    v = (v & m2) + ((v >> 2) & m2);
    v = (v & m4) + ((v >> 4) & m4);
    return static_cast<std::uint64_t>(v * 0x0101'0101'0101'0101ULL) >> 56;
#endif
}
}
}

#endif /* UTIL_BIT_INTRINSICS_H_ */
