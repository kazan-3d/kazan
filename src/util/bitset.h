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

#ifndef UTIL_BITSET_H_
#define UTIL_BITSET_H_

#include "bit_intrinsics.h"
#include <cstdint>
#include <limits>
#include <type_traits>
#include <utility>
#include <string>
#include <iosfwd>
#include <stdexcept>

// I have my own bitset implementation because std::bitset is not completely constexpr.

// derived from
// https://github.com/programmerjake/hashlife-voxels/blob/0dd91021a5b9caeffb7849b2114dca89204876bd/util/bitset.h

namespace vulkan_cpu
{
namespace util
{
template <std::size_t Bit_count>
class bitset;
namespace detail
{
class Bitset_nontemplate_base_helper
{
protected:
    static constexpr bool is_power_of_2(std::size_t v) noexcept
    {
        return (v & (v - 1)) == 0;
    }
};

struct Bitset_nontemplate_base : public Bitset_nontemplate_base_helper
{
protected:
    struct Tester;

public:
    typedef std::uintptr_t Word_type;
    static constexpr std::size_t word_bit_count = std::numeric_limits<Word_type>::digits;
    static_assert(is_power_of_2(word_bit_count), "word_bit_count is not a power of 2");
    static constexpr std::size_t constexpr_min(std::size_t a, std::size_t b) noexcept
    {
        return a > b ? b : a;
    }
    static constexpr std::size_t get_word_index(std::size_t bit_index) noexcept
    {
        return bit_index / word_bit_count;
    }
    static constexpr Word_type get_word_mask(std::size_t bit_index) noexcept
    {
        return static_cast<Word_type>(1) << (bit_index % word_bit_count);
    }
    static constexpr std::size_t get_word_count(std::size_t bit_count) noexcept
    {
        return (bit_count + (word_bit_count - 1)) / word_bit_count;
    }
};

template <std::size_t Word_count>
class Bitset_base : protected Bitset_nontemplate_base
{
protected:
    static constexpr std::size_t word_count = Word_count;
    Word_type words[Word_count]; // little endian order
    constexpr Bitset_base() noexcept : words{}
    {
    }
    constexpr Bitset_base(unsigned long long val) noexcept
        : Bitset_base(
              val,
              std::make_index_sequence<constexpr_min(
                  get_word_count(std::numeric_limits<unsigned long long>::digits), Word_count)>())
    {
    }
    constexpr Word_type get_word(std::size_t word_index) const noexcept
    {
        return words[word_index];
    }
    constexpr void set_word(std::size_t word_index, Word_type word_value) noexcept
    {
        words[word_index] = word_value;
    }
    constexpr bool equals(const Bitset_base &rt) const noexcept
    {
        for(std::size_t i = 0; i < word_count; i++)
            if(words[i] != rt.words[i])
                return false;
        return true;
    }

private:
    template <std::size_t... Indexes>
    constexpr Bitset_base(unsigned long long val, std::index_sequence<Indexes...>) noexcept
        : words{
              static_cast<Word_type>(val >> Indexes * word_bit_count)...,
          }
    {
    }
};

template <>
class Bitset_base<0> : protected Bitset_nontemplate_base
{
protected:
    static constexpr std::size_t word_count = 0;
    constexpr Bitset_base() noexcept
    {
    }
    constexpr Bitset_base(unsigned long long) noexcept
    {
    }
    constexpr Word_type get_word(std::size_t word_index) const noexcept
    {
        return (static_cast<void>(word_index), 0);
    }
    constexpr void set_word(std::size_t word_index, Word_type word_value) noexcept
    {
        static_cast<void>(word_index);
        static_cast<void>(word_value);
    }
    constexpr bool equals(const Bitset_base &rt) const noexcept
    {
        return true;
    }

public:
    constexpr unsigned long long to_ullong() const
    {
        return 0;
    }
};
}

template <std::size_t Bit_count>
class bitset final
    : public detail::Bitset_base<detail::Bitset_nontemplate_base::get_word_count(Bit_count)>
{
private:
    friend struct detail::Bitset_nontemplate_base::Tester;
    static constexpr std::size_t bit_count = Bit_count;
    typedef detail::Bitset_base<detail::Bitset_nontemplate_base::get_word_count(Bit_count)> Base;
    using typename Base::Word_type;
    using Base::word_count;
    using Base::get_word;
    using Base::set_word;
    using Base::get_word_count;
    using Base::get_word_mask;
    using Base::get_word_index;
    using Base::word_bit_count;

private:
    constexpr Word_type get_word_checked(std::size_t word_index) const noexcept
    {
        return word_index < word_count ? get_word(word_index) : 0;
    }

public:
    constexpr bitset() noexcept : Base()
    {
    }
    constexpr bitset(unsigned long long val) noexcept
        : Base(bit_count >= std::numeric_limits<unsigned long long>::digits ?
                   val :
                   val & ((1ULL << bit_count) - 1ULL))
    {
    }
    class reference final
    {
        template <std::size_t>
        friend class bitset;

    private:
        bitset *base;
        std::size_t bit_index;
        constexpr reference(bitset *base, std::size_t bit_index) noexcept : base(base),
                                                                            bit_index(bit_index)
        {
        }

    public:
        constexpr reference &operator=(
            const reference &rt) noexcept // assigns referenced value, not this class
        {
            return operator=(static_cast<bool>(rt));
        }
        constexpr reference &operator=(bool value) noexcept
        {
            auto mask = get_word_mask(bit_index);
            auto word = base->get_word(get_word_index(bit_index));
            if(value)
                word |= mask;
            else
                word &= ~mask;
            base->set_word(get_word_index(bit_index), word);
            return *this;
        }
        constexpr operator bool() const noexcept
        {
            return static_cast<const bitset *>(base)->operator[](bit_index);
        }
        constexpr bool operator~() const noexcept
        {
            return !operator bool();
        }
        constexpr reference &flip() noexcept
        {
            auto mask = get_word_mask(bit_index);
            auto word = base->get_word(get_word_index(bit_index));
            word ^= mask;
            base->set_word(get_word_index(bit_index), word);
            return *this;
        }
    };
    constexpr bool operator==(const bitset &rt) const noexcept
    {
        return this->equals(rt);
    }
    constexpr bool operator!=(const bitset &rt) const noexcept
    {
        return !this->equals(rt);
    }
    constexpr reference operator[](std::size_t bit_index) noexcept
    {
        return reference(this, bit_index);
    }
    constexpr bool operator[](std::size_t bit_index) const noexcept
    {
        return get_word_mask(bit_index) & get_word(get_word_index(bit_index));
    }
    constexpr bool test(std::size_t bit_index) const
    {
        if(bit_index >= bit_count)
            throw std::out_of_range("bit_index out of range in bitset::test");
        return operator[](bit_index);
    }
    constexpr bool all() const noexcept
    {
        if(bit_count == 0)
            return true;
        for(std::size_t i = 0; i < get_word_index(bit_count - 1); i++)
            if(get_word(i) != static_cast<Word_type>(-1))
                return false;
        return get_word(get_word_index(bit_count - 1))
               == static_cast<Word_type>(static_cast<Word_type>(get_word_mask(bit_count - 1) << 1)
                                         - 1);
    }
    constexpr bool any() const noexcept
    {
        if(bit_count == 0)
            return false;
        for(std::size_t i = 0; i < get_word_index(bit_count - 1); i++)
            if(get_word(i) != 0)
                return true;
        return get_word(get_word_index(bit_count - 1)) != 0;
    }
    constexpr bool none() const noexcept
    {
        return !any();
    }
    constexpr std::size_t count() const noexcept
    {
        std::size_t retval = 0;
        for(std::size_t i = 0; i < word_count; i++)
        {
            static_assert(
                std::numeric_limits<Word_type>::max() <= std::numeric_limits<std::uint64_t>::max(),
                "");
            if(std::numeric_limits<Word_type>::max() <= std::numeric_limits<std::uint32_t>::max())
                retval += popcount32(get_word(i));
            else
                retval += popcount64(get_word(i));
        }
        return retval;
    }
    constexpr std::size_t size() const noexcept
    {
        return bit_count;
    }
    constexpr bitset &operator&=(const bitset &rt) noexcept
    {
        for(std::size_t i = 0; i < word_count; i++)
            set_word(i, get_word(i) & rt.get_word(i));
        return *this;
    }
    friend constexpr bitset operator&(bitset a, const bitset &b) noexcept
    {
        return a &= b;
    }
    constexpr bitset &operator|=(const bitset &rt) noexcept
    {
        for(std::size_t i = 0; i < word_count; i++)
            set_word(i, get_word(i) | rt.get_word(i));
        return *this;
    }
    friend constexpr bitset operator|(bitset a, const bitset &b) noexcept
    {
        return a |= b;
    }
    constexpr bitset &operator^=(const bitset &rt) noexcept
    {
        for(std::size_t i = 0; i < word_count; i++)
            set_word(i, get_word(i) ^ rt.get_word(i));
        return *this;
    }
    friend constexpr bitset operator^(bitset a, const bitset &b) noexcept
    {
        return a ^= b;
    }
    constexpr bitset &flip() noexcept
    {
        for(std::size_t i = 0; i < get_word_index(bit_count - 1); i++)
            set_word(i, ~get_word(i));
        set_word(get_word_index(bit_count - 1),
                 get_word(get_word_index(bit_count - 1))
                     ^ static_cast<Word_type>(
                           static_cast<Word_type>(get_word_mask(bit_count - 1) << 1) - 1));
        return *this;
    }
    constexpr bitset operator~() const noexcept
    {
        bitset retval = *this;
        retval.flip();
        return retval;
    }
    constexpr bitset &operator<<=(std::size_t shiftCount) noexcept
    {
        if(shiftCount >= bit_count)
            return reset();
        std::size_t shiftWord_count = shiftCount / word_bit_count;
        std::size_t shiftBitCount = shiftCount % word_bit_count;
        if(shiftBitCount == 0)
        {
            for(std::size_t i = word_count; i > 0; i--)
            {
                std::size_t index = i - 1;
                set_word(index, get_word_checked(index - shiftWord_count));
            }
        }
        else
        {
            for(std::size_t i = word_count; i > 0; i--)
            {
                std::size_t index = i - 1;
                Word_type highWord = get_word_checked(index - shiftWord_count);
                Word_type lowWord = get_word_checked(index - 1 - shiftWord_count);
                set_word(
                    index,
                    (lowWord >> (word_bit_count - shiftBitCount)) | (highWord << shiftBitCount));
            }
        }
        if(word_count != 0)
            set_word(word_count - 1,
                     get_word(word_count - 1)
                         & static_cast<Word_type>(
                               static_cast<Word_type>(get_word_mask(bit_count - 1) << 1) - 1));
        return *this;
    }
    constexpr bitset operator<<(std::size_t shiftCount) const noexcept
    {
        bitset retval = *this;
        retval <<= shiftCount;
        return retval;
    }
    constexpr bitset &operator>>=(std::size_t shiftCount) noexcept
    {
        if(shiftCount >= bit_count)
            return reset();
        std::size_t shiftWord_count = shiftCount / word_bit_count;
        std::size_t shiftBitCount = shiftCount % word_bit_count;
        if(shiftBitCount == 0)
        {
            for(std::size_t index = 0; index < word_count; index++)
            {
                set_word(index, get_word_checked(index + shiftWord_count));
            }
        }
        else
        {
            for(std::size_t index = 0; index < word_count; index++)
            {
                Word_type highWord = get_word_checked(index + 1 + shiftWord_count);
                Word_type lowWord = get_word_checked(index + shiftWord_count);
                set_word(
                    index,
                    (lowWord >> shiftBitCount) | (highWord << (word_bit_count - shiftBitCount)));
            }
        }
        return *this;
    }
    constexpr bitset operator>>(std::size_t shiftCount) const noexcept
    {
        bitset retval = *this;
        retval >>= shiftCount;
        return retval;
    }
    constexpr bitset &set() noexcept
    {
        if(word_count == 0)
            return *this;
        for(std::size_t i = 0; i < get_word_index(bit_count - 1); i++)
            set_word(i, static_cast<Word_type>(-1));
        set_word(
            get_word_index(bit_count - 1),
            static_cast<Word_type>(static_cast<Word_type>(get_word_mask(bit_count - 1) << 1) - 1));
        return *this;
    }
    constexpr bitset &set(std::size_t bit_index, bool value = true)
    {
        if(bit_index >= bit_count)
            throw std::out_of_range("bit_index out of range in bitset::set");
        operator[](bit_index) = value;
        return *this;
    }
    constexpr bitset &reset() noexcept
    {
        for(std::size_t i = 0; i < word_count; i++)
            set_word(i, 0);
        return *this;
    }
    constexpr bitset &reset(std::size_t bit_index)
    {
        if(bit_index >= bit_count)
            throw std::out_of_range("bit_index out of range in bitset::reset");
        operator[](bit_index) = false;
        return *this;
    }
    constexpr bitset &flip(std::size_t bit_index)
    {
        if(bit_index >= bit_count)
            throw std::out_of_range("bit_index out of range in bitset::flip");
        operator[](bit_index).flip();
        return *this;
    }
    constexpr unsigned long long to_ullong() const
    {
        unsigned long long retval = 0;
        constexpr std::size_t ullBitCount = std::numeric_limits<unsigned long long>::digits;
        for(std::size_t i = 0; i < word_count; i++)
        {
            if(i * word_bit_count >= ullBitCount)
            {
                if(get_word(i) != 0)
                    throw std::overflow_error("bit set value too large in bitset::to_ullong");
            }
            else
            {
                auto word = get_word(i);
                auto shiftedWord = static_cast<unsigned long long>(word) << i * word_bit_count;
                if((shiftedWord >> i * word_bit_count) != word)
                    throw std::overflow_error("bit set value too large in bitset::to_ullong");
                retval |= shiftedWord;
            }
        }
        return retval;
    }
    constexpr unsigned long to_ulong() const
    {
        unsigned long long retval = to_ullong();
        if(retval > std::numeric_limits<unsigned long>::max())
            throw std::overflow_error("bit set value too large in bitset::to_ulong");
        return retval;
    }
    static constexpr std::size_t npos = -1; // not in std::bitset
    constexpr std::size_t find_first(bool value, std::size_t start = 0) const
        noexcept // not in std::bitset
    {
        if(start >= bit_count)
            return npos;
        constexpr std::size_t endWordIndex = get_word_index(bit_count - 1);
        std::size_t startWordIndex = get_word_index(start);
        auto startWord = get_word(startWordIndex);
        if(!value)
        {
            if(startWordIndex == endWordIndex)
                startWord ^= static_cast<Word_type>(
                    static_cast<Word_type>(get_word_mask(bit_count - 1) << 1) - 1);
            else
                startWord = ~startWord;
        }
        auto mask = get_word_mask(start);
        for(std::size_t retval = start; mask != 0; mask <<= 1, retval++)
        {
            if(startWord & mask)
                return retval;
        }
        if(startWordIndex == endWordIndex)
            return npos;
        for(std::size_t word_index = startWordIndex + 1; word_index < endWordIndex; word_index++)
        {
            auto word = get_word(word_index);
            if(word == static_cast<Word_type>(value ? 0 : -1))
                continue;
            if(!value)
                word = ~word;
            mask = 1;
            std::size_t retval = word_index * word_bit_count;
            for(; mask != 0; mask <<= 1, retval++)
            {
                if(word & mask)
                    break;
            }
            return retval;
        }
        auto endWord = get_word(endWordIndex);
        if(!value)
            endWord ^= static_cast<Word_type>(
                static_cast<Word_type>(get_word_mask(bit_count - 1) << 1) - 1);
        if(endWord == 0)
            return npos;
        mask = 1;
        std::size_t retval = endWordIndex * word_bit_count;
        for(; mask != 0; mask <<= 1, retval++)
        {
            if(endWord & mask)
                break;
        }
        return retval;
    }
    constexpr std::size_t find_last(bool value, std::size_t start = npos) const
        noexcept // not in std::bitset
    {
        if(bit_count == 0)
            return npos;
        if(start >= bit_count)
            start = bit_count - 1;
        std::size_t startWordIndex = get_word_index(start);
        auto startWord = get_word(startWordIndex);
        if(!value)
        {
            if(startWordIndex == get_word_index(bit_count - 1))
                startWord ^= static_cast<Word_type>(
                    static_cast<Word_type>(get_word_mask(bit_count - 1) << 1) - 1);
            else
                startWord = ~startWord;
        }
        auto mask = get_word_mask(start);
        for(std::size_t retval = start; mask != 0; mask >>= 1, retval--)
        {
            if(startWord & mask)
                return retval;
        }
        for(std::size_t word_index = startWordIndex - 1, i = 0; i < startWordIndex;
            word_index--, i++)
        {
            auto word = get_word(word_index);
            if(word == static_cast<Word_type>(value ? 0 : -1))
                continue;
            if(!value)
                word = ~word;
            mask = get_word_mask(word_bit_count - 1);
            std::size_t retval = word_index * word_bit_count + (word_bit_count - 1);
            for(; mask != 0; mask >>= 1, retval--)
            {
                if(word & mask)
                    break;
            }
            return retval;
        }
        return npos;
    }
    constexpr std::size_t hash() const noexcept // not in std::bitset
    {
        std::size_t retval = 0;
        for(std::size_t i = 0; i < word_count; i++)
        {
            retval ^= get_word(i);
        }
        return retval;
    }
};

template <std::size_t Bit_count>
constexpr std::size_t bitset<Bit_count>::npos;
}
}

namespace std
{
template <std::size_t Bit_count>
struct hash<vulkan_cpu::util::bitset<Bit_count>>
{
    constexpr std::size_t operator()(const vulkan_cpu::util::bitset<Bit_count> &v) const noexcept
    {
        return v.hash();
    }
};
}

#endif /* UTIL_BITSET_H_ */
