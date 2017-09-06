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
// derived from
// https://github.com/programmerjake/hashlife-voxels/blob/0dd91021a5b9caeffb7849b2114dca89204876bd/util/bitset.cpp

#include "bitset.h"
#include <cassert>
#include <cstdlib>
#include <iostream>
#include <random>
#include <vector>
#include <string>
#include <utility>

namespace kazan
{
namespace util
{
namespace detail
{
#if 0
#warning testing bitset
struct Bitset_nontemplate_base::Tester final
{
    template <std::size_t Bit_count>
    static void check_unused_bits(const bitset<Bit_count> &value)
    {
        constexpr Word_type unused_bits =
            Bit_count % word_bit_count ? ~((1ULL << (Bit_count % word_bit_count)) - 1ULL) : 0;
        assert((value.get_word(value.word_count - 1) & unused_bits) == 0);
    }
    static void check_unused_bits(const bitset<0> &)
    {
    }
    template <std::size_t Bit_count>
    static void test_default_construct()
    {
        bitset<Bit_count> value;
        for(std::size_t i = 0; i < value.word_count; i++)
            assert(value.get_word(i) == 0);
        check_unused_bits(value);
    }
    template <std::size_t Bit_count>
    static void test_construct_from_ull()
    {
        for(std::size_t i = 0; i < std::numeric_limits<unsigned long long>::digits; i++)
        {
            bitset<Bit_count> value(1ULL << i);
            check_unused_bits(value);
            assert(bitset<Bit_count>(1ULL << i).to_ullong() == (1ULL << i) || i >= Bit_count);
        }
    }
    template <std::size_t Bit_count>
    static void test_reference_assign()
    {
        std::default_random_engine re;
        std::uniform_int_distribution<unsigned long long> distribution;
        for(std::size_t i = 0; i < 1000; i++)
        {
            bitset<Bit_count> src(distribution(re)), dest;
            for(std::size_t j = 0; j < Bit_count; j++)
            {
                dest[j] = src[j];
                check_unused_bits(src);
                check_unused_bits(dest);
            }
            assert(src == dest);
        }
    }
    template <std::size_t Bit_count>
    static void test_reference_flip()
    {
        if(Bit_count == 0)
            return;
        std::default_random_engine re;
        std::vector<bool> vector;
        vector.resize(Bit_count, false);
        bitset<Bit_count> value;
        for(std::size_t i = 0; i < 1000; i++)
        {
            std::size_t index = std::uniform_int_distribution<std::size_t>(0, Bit_count - 1)(re);
            vector[index].flip();
            value[index].flip();
            check_unused_bits(value);
            for(std::size_t j = 0; j < Bit_count; j++)
                assert(value[index] == static_cast<bool>(vector[index]));
        }
    }
    template <std::size_t Bit_count>
    static void test_test()
    {
        std::default_random_engine re;
        std::vector<bool> vector;
        vector.resize(Bit_count, false);
        bitset<Bit_count> value;
        if(Bit_count != 0)
        {
            for(std::size_t i = 0; i < 1000; i++)
            {
                std::size_t index =
                    std::uniform_int_distribution<std::size_t>(0, Bit_count - 1)(re);
                vector[index].flip();
                value[index].flip();
                check_unused_bits(value);
            }
        }
        for(std::size_t i = 0; i < Bit_count + 1000; i++)
        {
            bool threw = false;
            bool result = false;
            try
            {
                result = value.test(i);
            }
            catch(std::out_of_range &)
            {
                threw = true;
            }
            if(i >= Bit_count)
            {
                assert(threw);
            }
            else
            {
                assert(!threw);
                assert(result == vector[i]);
            }
        }
    }
    template <std::size_t Bit_count>
    static void test_all_none_any_and_count_helper(const std::vector<bool> &vector,
                                                   const bitset<Bit_count> &value)
    {
        std::size_t set_bit_count = 0;
        for(std::size_t i = 0; i < Bit_count; i++)
            if(vector[i])
                set_bit_count++;
        assert(value.all() == (set_bit_count == Bit_count));
        assert(value.any() == (set_bit_count != 0));
        assert(value.none() == (set_bit_count == 0));
        assert(value.count() == set_bit_count);
    }
    template <std::size_t Bit_count>
    static void test_all_none_any_and_count()
    {
        std::default_random_engine re;
        std::vector<bool> vector;
        vector.resize(Bit_count, false);
        bitset<Bit_count> value;
        test_all_none_any_and_count_helper(vector, value);
        if(Bit_count != 0)
        {
            for(std::size_t i = 0; i < 1000; i++)
            {
                std::size_t index =
                    std::uniform_int_distribution<std::size_t>(0, Bit_count - 1)(re);
                vector[index].flip();
                value[index].flip();
                check_unused_bits(value);
                test_all_none_any_and_count_helper(vector, value);
            }
        }
        for(std::size_t i = 0; i < Bit_count; i++)
        {
            value[i] = true;
            vector[i] = true;
            check_unused_bits(value);
            test_all_none_any_and_count_helper(vector, value);
        }
    }
    template <std::size_t Bit_count>
    static void test_and_or_and_xor_helper(const std::vector<bool> &vector1,
                                           const std::vector<bool> &vector2,
                                           const bitset<Bit_count> &bitset1,
                                           const bitset<Bit_count> &bitset2)
    {
        bitset<Bit_count> dest_bitset_and = bitset1 & bitset2;
        bitset<Bit_count> dest_bitset_or = bitset1 | bitset2;
        bitset<Bit_count> dest_bitset_xor = bitset1 ^ bitset2;
        check_unused_bits(dest_bitset_and);
        check_unused_bits(dest_bitset_or);
        check_unused_bits(dest_bitset_xor);
        for(std::size_t i = 0; i < Bit_count; i++)
        {
            assert(dest_bitset_and[i] == (vector1[i] && vector2[i]));
            assert(dest_bitset_or[i] == (vector1[i] || vector2[i]));
            assert(dest_bitset_xor[i] == (static_cast<bool>(vector1[i]) != vector2[i]));
        }
    }
    template <std::size_t Bit_count>
    static void test_and_or_and_xor()
    {
        std::default_random_engine re;
        std::vector<bool> vector1, vector2;
        vector1.resize(Bit_count, false);
        vector2.resize(Bit_count, false);
        bitset<Bit_count> bitset1, bitset2;
        test_and_or_and_xor_helper(vector1, vector2, bitset1, bitset2);
        if(Bit_count != 0)
        {
            for(std::size_t i = 0; i < 2000; i++)
            {
                std::size_t index =
                    std::uniform_int_distribution<std::size_t>(0, Bit_count * 2 - 1)(re);
                bool is_second_bit_set = index >= Bit_count;
                index %= Bit_count;
                if(is_second_bit_set)
                {
                    vector2[index].flip();
                    bitset2[index].flip();
                }
                else
                {
                    vector1[index].flip();
                    bitset1[index].flip();
                }
                check_unused_bits(bitset1);
                check_unused_bits(bitset2);
                test_and_or_and_xor_helper(vector1, vector2, bitset1, bitset2);
            }
        }
        for(std::size_t i = 0; i < Bit_count; i++)
        {
            bitset1[i] = true;
            vector1[i] = true;
            check_unused_bits(bitset1);
            check_unused_bits(bitset2);
            test_and_or_and_xor_helper(vector1, vector2, bitset1, bitset2);
        }
        for(std::size_t i = 0; i < Bit_count; i++)
        {
            bitset2[i] = true;
            vector2[i] = true;
            check_unused_bits(bitset1);
            check_unused_bits(bitset2);
            test_and_or_and_xor_helper(vector1, vector2, bitset1, bitset2);
        }
    }
    template <std::size_t Bit_count>
    static void test_not()
    {
        std::default_random_engine re;
        std::vector<bool> vector;
        vector.resize(Bit_count, false);
        bitset<Bit_count> value;
        if(Bit_count != 0)
        {
            for(std::size_t i = 0; i < 1000; i++)
            {
                std::size_t index =
                    std::uniform_int_distribution<std::size_t>(0, Bit_count - 1)(re);
                vector[index].flip();
                value[index].flip();
                check_unused_bits(value);
                bitset<Bit_count> bitset_not = ~value;
                check_unused_bits(bitset_not);
                for(std::size_t j = 0; j < Bit_count; j++)
                    assert(vector[j] == !bitset_not[j]);
            }
        }
    }
    template <std::size_t Bit_count>
    static void test_shift_helper(const std::vector<bool> &vector, const bitset<Bit_count> &value)
    {
        for(std::size_t shift_count = 0; shift_count < Bit_count * 2 + 1; shift_count++)
        {
            bitset<Bit_count> bitset_shifted_left = value << shift_count;
            bitset<Bit_count> bitset_shifted_right = value >> shift_count;
            check_unused_bits(bitset_shifted_left);
            check_unused_bits(bitset_shifted_right);
            for(std::size_t i = 0; i < Bit_count; i++)
            {
                assert(bitset_shifted_left[i]
                       == (i < shift_count ? false : static_cast<bool>(vector[i - shift_count])));
                assert(bitset_shifted_right[i] == (shift_count >= Bit_count - i ?
                                                       false :
                                                       static_cast<bool>(vector[i + shift_count])));
            }
        }
    }
    template <std::size_t Bit_count>
    static void test_shift()
    {
        std::default_random_engine re;
        std::vector<bool> vector;
        vector.resize(Bit_count, false);
        bitset<Bit_count> value;
        test_shift_helper(vector, value);
        if(Bit_count != 0)
        {
            for(std::size_t i = 0; i < 1000; i++)
            {
                std::size_t index =
                    std::uniform_int_distribution<std::size_t>(0, Bit_count - 1)(re);
                vector[index].flip();
                value[index].flip();
                check_unused_bits(value);
                test_shift_helper(vector, value);
            }
        }
        for(std::size_t i = 0; i < Bit_count; i++)
        {
            value[i] = true;
            vector[i] = true;
            check_unused_bits(value);
            test_shift_helper(vector, value);
        }
    }
    template <std::size_t Bit_count>
    static void test_global_set_and_reset()
    {
        bitset<Bit_count> value;
        value.reset();
        check_unused_bits(value);
        assert(value.none());
        value.set();
        check_unused_bits(value);
        assert(value.all());
    }
    template <std::size_t Bit_count>
    static void test_find_helper(const std::string &string, const bitset<Bit_count> &value)
    {
        for(std::size_t i = 0; i < Bit_count; i++)
            assert(string[i] == (value[i] ? '1' : '0'));
        for(std::size_t start = 0; start <= Bit_count; start++)
        {
            assert(string.find_first_of('1', start) == value.find_first(true, start));
            assert(string.find_first_not_of('1', start) == value.find_first(false, start));
            assert(string.find_last_of('1', start) == value.find_last(true, start));
            assert(string.find_last_not_of('1', start) == value.find_last(false, start));
        }
    }
    template <std::size_t Bit_count>
    static void test_find()
    {
        std::default_random_engine re;
        std::string string;
        string.resize(Bit_count, '0');
        bitset<Bit_count> value;
        test_find_helper(string, value);
        if(Bit_count != 0)
        {
            for(std::size_t i = 0; i < 1000; i++)
            {
                std::size_t index =
                    std::uniform_int_distribution<std::size_t>(0, Bit_count - 1)(re);
                string[index] = (string[index] == '0' ? '1' : '0');
                value[index].flip();
                check_unused_bits(value);
                test_find_helper(string, value);
            }
        }
        for(std::size_t i = 0; i < Bit_count; i++)
        {
            value[i] = true;
            string[i] = '1';
            check_unused_bits(value);
            test_find_helper(string, value);
        }
        if(Bit_count != 0)
        {
            for(std::size_t i = 0; i < 1000; i++)
            {
                std::size_t index =
                    std::uniform_int_distribution<std::size_t>(0, Bit_count - 1)(re);
                string[index] = (string[index] == '0' ? '1' : '0');
                value[index].flip();
                check_unused_bits(value);
                test_find_helper(string, value);
            }
        }
    }
    template <std::size_t Bit_count>
    static char test()
    {
        std::cout << "testing bitset<" << Bit_count << ">" << std::endl;
        test_default_construct<Bit_count>();
        test_construct_from_ull<Bit_count>();
        test_reference_assign<Bit_count>();
        test_reference_flip<Bit_count>();
        test_test<Bit_count>();
        test_all_none_any_and_count<Bit_count>();
        test_and_or_and_xor<Bit_count>();
        test_not<Bit_count>();
        test_shift<Bit_count>();
        test_global_set_and_reset<Bit_count>();
        test_find<Bit_count>();
        return 0;
    }
    template <typename... Args>
    static void test_helper(Args...)
    {
    }
    template <std::size_t... Bit_counts>
    static void test(std::index_sequence<Bit_counts...>)
    {
        test_helper(test<Bit_counts>()...);
    }
    struct Test_runner final
    {
        Test_runner()
        {
            test(std::make_index_sequence<128>());
            std::exit(0);
        }
    };
    static Test_runner test_runner;
};
Bitset_nontemplate_base::Tester::Test_runner Bitset_nontemplate_base::Tester::test_runner;
#endif
}
}
}
