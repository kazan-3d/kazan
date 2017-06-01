/*
 * Copyright 2016-2017 Jacob Lifshay
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
// https://github.com/programmerjake/javascript-tasklets/blob/master/javascript_tasklets/soft_float.cpp

#if 1
#include "soft_float.h"
#if 0
#include <iostream>
#include <cstdlib>
#include <string>
#include <sstream>
#include <cstdio>
namespace
{
using namespace vulkan_cpu::util::soft_float;
std::string hexValue(const ExtendedFloat &v)
{
    if(v.isNaN())
    {
        return "NaN";
    }
    if(v.isInfinite())
    {
        if(v.sign)
            return "-Infinity";
        return "+Infinity";
    }
    std::ostringstream ss;
    ss << std::hex << std::uppercase;
    ss.fill('0');
    if(v.sign)
        ss << "-";
    else
        ss << "+";
    ss << "0x";
    std::int32_t exponent = v.exponent;
    exponent -= ExtendedFloat::exponentBias();
    if(v.isZero())
        exponent = 0;
    std::uint64_t mantissa = v.mantissa;
    unsigned firstDigitBits = 1 + (exponent & 3);
    ss << (mantissa >> (64 - firstDigitBits));
    mantissa <<= firstDigitBits;
    exponent &= ~3;
    ss << ".";
    ss.width(16);
    ss << mantissa;
    ss << "p";
    ss << std::dec << std::showpos;
    ss << exponent;
    return ss.str();
}
std::string hexValue(long double v)
{
    if(std::isnan(v))
    {
        return "NaN";
    }
    if(std::isinf(v))
    {
        if(v < 0)
            return "-Infinity";
        return "+Infinity";
    }
    const std::size_t strSize = 64;
    char str[strSize];
    std::snprintf(str, sizeof(str), "%+1.16LA", v);
    for(char &ch : str)
    {
        if(ch == '\0')
            break;
        if(ch == 'X')
            ch = 'x';
        else if(ch == 'P')
            ch = 'p';
    }
    return str;
}
std::string hexValue(std::int64_t v)
{
    std::ostringstream ss;
    ss << std::hex << std::uppercase;
    ss.fill('0');
    if(v < 0)
        ss << "-";
    else
        ss << "+";
    ss << "0x";
    ss.width(16);
    if(v < 0)
        ss << -static_cast<std::uint64_t>(v);
    else
        ss << static_cast<std::uint64_t>(v);
    return ss.str();
}
std::string hexValue(std::uint64_t v)
{
    std::ostringstream ss;
    ss << std::hex << std::uppercase;
    ss.fill('0');
    ss << "0x";
    ss.width(16);
    ss << static_cast<std::uint64_t>(v);
    return ss.str();
}
bool sameValue(long double a, long double b)
{
    if(std::isnan(a))
        return std::isnan(b);
    if(a == 0)
    {
        return b == 0 && std::signbit(a) == std::signbit(b);
    }
    return a == b;
}
void writeArgs()
{
}
template <typename Arg, typename... Args>
void writeArgs(Arg arg, Args... args)
{
    std::cout << " " << hexValue(arg);
    writeArgs(args...);
}
constexpr bool displayPassedTests = true;
template <typename TestFn1, typename TestFn2, typename... Args>
void testCase(const char *name, TestFn1 &&testFn1, TestFn2 &&testFn2, Args... args)
{
    long double result1 = static_cast<long double>(testFn1(args...));
    long double result2 = static_cast<long double>(testFn2(args...));
    if(!sameValue(result1, result2))
    {
        std::cout << name;
        writeArgs(args...);
        std::cout << " -> ";
        std::cout << hexValue(result1) << " != " << hexValue(result2) << std::endl;
    }
    else if(displayPassedTests)
    {
        std::cout << name;
        writeArgs(args...);
        std::cout << " -> ";
        std::cout << hexValue(result1) << std::endl;
    }
}
template <typename TestFn1, typename TestFn2, typename... Args>
void testCaseI(const char *name, TestFn1 &&testFn1, TestFn2 &&testFn2, Args... args)
{
    auto result1 = testFn1(args...);
    auto result2 = testFn2(args...);
    if(result1 != result2)
    {
        std::cout << name;
        writeArgs(args...);
        std::cout << " -> ";
        std::cout << hexValue(result1) << " != " << hexValue(result2) << std::endl;
    }
    else if(displayPassedTests)
    {
        std::cout << name;
        writeArgs(args...);
        std::cout << " -> ";
        std::cout << hexValue(result1) << std::endl;
    }
}
template <typename TestFn1, typename TestFn2, typename... Args>
void roundTestCases(const char *name, TestFn1 &&testFn1, TestFn2 &&testFn2)
{
    const long double NaN = std::numeric_limits<long double>::quiet_NaN();
    const long double Infinity = std::numeric_limits<long double>::infinity();
    auto testBothSigns = [&](long double value)
    {
        testCase(name, testFn1, testFn2, value);
        testCase(name, testFn1, testFn2, -value);
    };
    testCase(name, testFn1, testFn2, NaN);
    testBothSigns(0.0L);
    testBothSigns(Infinity);
    testBothSigns(1.0L);
    testBothSigns(0x1.0p-1L);
    testBothSigns(0x1.8p0L);
    testBothSigns(0x1.Fp0L);
    testBothSigns(0x1.Fp-30L);
    testBothSigns(0x1.Fp30L);
    testBothSigns(0x1.Fp62L);
    testBothSigns(0x1.Fp63L);
    testBothSigns(0x1.Fp64L);
    testBothSigns(0x1.Fp65L);
    testBothSigns(0x1.Fp62L + 0.5L);
    testBothSigns(0x1.Fp63L + 0.5L);
    testBothSigns(0x1.Fp64L + 0.5L);
    testBothSigns(0x1.Fp65L + 0.5L);
    testBothSigns(0x1.Fp62L + 1);
    testBothSigns(0x1.Fp63L + 1);
    testBothSigns(0x1.Fp64L + 1);
    testBothSigns(0x1.Fp65L + 1);
}
template <typename TestFn1, typename TestFn2, typename... Args>
void toIntTestCases(const char *name, TestFn1 &&testFn1, TestFn2 &&testFn2)
{
    const long double NaN = std::numeric_limits<long double>::quiet_NaN();
    const long double Infinity = std::numeric_limits<long double>::infinity();
    auto testBothSigns = [&](long double value)
    {
        testCaseI(name, testFn1, testFn2, value);
        testCaseI(name, testFn1, testFn2, -value);
    };
    testCaseI(name, testFn1, testFn2, NaN);
    testBothSigns(0.0L);
    testBothSigns(Infinity);
    testBothSigns(1.0L);
    testBothSigns(0x1.0p-1L);
    testBothSigns(0x1.8p0L);
    testBothSigns(0x1.Fp0L);
    testBothSigns(0x1.Fp-30L);
    testBothSigns(0x1.Fp30L);
    testBothSigns(0x1.Fp62L);
    testBothSigns(0x1.Fp63L);
    testBothSigns(0x1.Fp64L);
    testBothSigns(0x1.Fp65L);
    testBothSigns(0x1.Fp62L + 0.5L);
    testBothSigns(0x1.Fp63L + 0.5L);
    testBothSigns(0x1.Fp64L + 0.5L);
    testBothSigns(0x1.Fp65L + 0.5L);
    testBothSigns(0x1.Fp62L + 1);
    testBothSigns(0x1.Fp63L + 1);
    testBothSigns(0x1.Fp64L + 1);
    testBothSigns(0x1.Fp65L + 1);
}
void mainFn()
{
    auto add1 = [](long double a, long double b) -> long double
    {
        return a + b;
    };
    auto add2 = [](long double a, long double b) -> ExtendedFloat
    {
        return ExtendedFloat(a) + ExtendedFloat(b);
    };
    auto mul1 = [](long double a, long double b) -> long double
    {
        return a * b;
    };
    auto mul2 = [](long double a, long double b) -> ExtendedFloat
    {
        return ExtendedFloat(a) * ExtendedFloat(b);
    };
    auto div1 = [](long double a, long double b) -> long double
    {
        return a / b;
    };
    auto div2 = [](long double a, long double b) -> ExtendedFloat
    {
        return ExtendedFloat(a) / ExtendedFloat(b);
    };
    auto floor1 = [](long double a) -> long double
    {
        return std::floor(a);
    };
    auto floor2 = [](long double a) -> ExtendedFloat
    {
        return floor(ExtendedFloat(a));
    };
    auto ceil1 = [](long double a) -> long double
    {
        return std::ceil(a);
    };
    auto ceil2 = [](long double a) -> ExtendedFloat
    {
        return ceil(ExtendedFloat(a));
    };
    auto round1 = [](long double a) -> long double
    {
        return std::round(a);
    };
    auto round2 = [](long double a) -> ExtendedFloat
    {
        return round(ExtendedFloat(a));
    };
    auto trunc1 = [](long double a) -> long double
    {
        return std::trunc(a);
    };
    auto trunc2 = [](long double a) -> ExtendedFloat
    {
        return trunc(ExtendedFloat(a));
    };
    auto toUInt1 = [](long double a) -> std::uint64_t
    {
        if(std::isnan(a))
            return 0;
        if(a < std::numeric_limits<std::uint64_t>::min())
            return std::numeric_limits<std::uint64_t>::min();
        if(a > std::numeric_limits<std::uint64_t>::max())
            return std::numeric_limits<std::uint64_t>::max();
        return static_cast<std::uint64_t>(a);
    };
    auto toUInt2 = [](long double a) -> std::uint64_t
    {
        return static_cast<std::uint64_t>(ExtendedFloat(a));
    };
    auto toInt1 = [](long double a) -> std::int64_t
    {
        if(std::isnan(a))
            return 0;
        if(a < std::numeric_limits<std::int64_t>::min())
            return std::numeric_limits<std::int64_t>::min();
        if(a > std::numeric_limits<std::int64_t>::max())
            return std::numeric_limits<std::int64_t>::max();
        return static_cast<std::int64_t>(a);
    };
    auto toInt2 = [](long double a) -> std::int64_t
    {
        return static_cast<std::int64_t>(ExtendedFloat(a));
    };
    auto pow1 = [](long double base, int exponent) -> long double
    {
        if(exponent < 0)
        {
            base = 1 / base;
            exponent = -exponent;
        }
        else if(exponent == 0)
            return 1;
        long double retval = 1;
        for(;;)
        {
            if(exponent == 0)
                return retval;
            else if(exponent == 1)
                return retval * base;
            if(exponent & 1)
            {
                retval *= base;
            }
            base *= base;
            exponent >>= 1;
        }
    };
    auto pow2 = [](long double base, int exponent) -> ExtendedFloat
    {
        return pow(ExtendedFloat(base), static_cast<std::int64_t>(exponent));
    };
    auto scalbn1 = [](long double a, std::int64_t exponent) -> long double
    {
        return std::scalbln(a, static_cast<long>(exponent));
    };
    auto scalbn2 = [](long double a, std::int64_t exponent) -> ExtendedFloat
    {
        return scalbn(ExtendedFloat(a), exponent);
    };
    auto log2_1 = [](long double a) -> long double
    {
        return std::log2(a);
    };
    auto log2_2 = [](long double a) -> ExtendedFloat
    {
        return log2(ExtendedFloat(a));
    };
    auto log10_1 = [](long double a) -> long double
    {
        return std::log10(a);
    };
    auto log10_2 = [](long double a) -> ExtendedFloat
    {
        return log10(ExtendedFloat(a));
    };
    const long double NaN = std::numeric_limits<long double>::quiet_NaN();
    const long double Infinity = std::numeric_limits<long double>::infinity();
    testCase("add", add1, add2, +0.0L, +0.0L);
    testCase("add", add1, add2, +0.0L, -0.0L);
    testCase("add", add1, add2, -0.0L, +0.0L);
    testCase("add", add1, add2, -0.0L, -0.0L);
    testCase("add", add1, add2, 0.0L, NaN);
    testCase("add", add1, add2, NaN, 0.0L);
    testCase("add", add1, add2, NaN, NaN);
    testCase("add", add1, add2, +Infinity, +Infinity);
    testCase("add", add1, add2, +Infinity, -Infinity);
    testCase("add", add1, add2, -Infinity, +Infinity);
    testCase("add", add1, add2, -Infinity, -Infinity);
    testCase("add", add1, add2, 0x1.0000000000000002p0L, -0x1.0p-64L);
    testCase("add", add1, add2, 0x1.p0L, -0x1.0p-65L);
    testCase("add", add1, add2, 0x1.p0L, -0x0.Fp-65L);
    testCase("add", add1, add2, 0x1.p0L, -0x1.1p-65L);
    testCase("add", add1, add2, 0x1.0000000000000002p0L, -0x2.0p-65L);
    testCase("add", add1, add2, 0x1.0000000000000002p0L, -0x1.Fp-65L);
    testCase("add", add1, add2, 0x1.0000000000000002p0L, -0x2.1p-65L);
    testCase("add", add1, add2, 0x1p-16445L, 0x1p-16445L);
    testCase("add", add1, add2, 0x1p+16383L, 0x1p+16383L);
    testCase("mul", mul1, mul2, +0.0L, +0.0L);
    testCase("mul", mul1, mul2, +0.0L, -0.0L);
    testCase("mul", mul1, mul2, -0.0L, +0.0L);
    testCase("mul", mul1, mul2, -0.0L, -0.0L);
    testCase("mul", mul1, mul2, 0.0L, NaN);
    testCase("mul", mul1, mul2, NaN, 0.0L);
    testCase("mul", mul1, mul2, NaN, NaN);
    testCase("mul", mul1, mul2, +Infinity, +Infinity);
    testCase("mul", mul1, mul2, +Infinity, -Infinity);
    testCase("mul", mul1, mul2, -Infinity, +Infinity);
    testCase("mul", mul1, mul2, -Infinity, -Infinity);
    testCase("mul", mul1, mul2, 0x1p0L, 0x1p0L);
    testCase("mul", mul1, mul2, 0x1p16000L, 0x1p383L);
    testCase("mul", mul1, mul2, 0x1p16000L, 0x1p384L);
    testCase("mul", mul1, mul2, 0x1p-16000L, 0x1p-445L);
    testCase("mul", mul1, mul2, 0x1p-16000L, 0x1p-446L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.000000001p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.0000000018p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.000000002p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.0000000028p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.000000003p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.0000000038p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.000000004p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.0000000048p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.000000005p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.0000000058p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.000000006p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.0000000068p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.000000007p0L);
    testCase("mul", mul1, mul2, 0x1.0000001p0L, 0x1.0000000078p0L);
    testCase("mul",
             mul1,
             mul2,
             3.1415926535897932384626433832795L,
             0.318309886183790671537767526745028724L);
    testCase("mul",
             mul1,
             mul2,
             2.718281828459045235360287471352662497757L,
             0.3678794411714423215955237701614608674458L);
    testCase("div", div1, div2, +0.0L, +0.0L);
    testCase("div", div1, div2, +1.0L, +0.0L);
    testCase("div", div1, div2, +1.0L, -0.0L);
    testCase("div", div1, div2, -1.0L, +0.0L);
    testCase("div", div1, div2, -1.0L, -0.0L);
    testCase("div", div1, div2, +0.0L, +1.0L);
    testCase("div", div1, div2, +0.0L, -1.0L);
    testCase("div", div1, div2, -0.0L, +1.0L);
    testCase("div", div1, div2, -0.0L, -1.0L);
    testCase("div", div1, div2, 0.0L, NaN);
    testCase("div", div1, div2, NaN, 0.0L);
    testCase("div", div1, div2, NaN, NaN);
    testCase("div", div1, div2, +Infinity, +Infinity);
    testCase("div", div1, div2, +1.0L, +Infinity);
    testCase("div", div1, div2, +1.0L, -Infinity);
    testCase("div", div1, div2, -1.0L, +Infinity);
    testCase("div", div1, div2, -1.0L, -Infinity);
    testCase("div", div1, div2, 1.0L, 3.0L);
    testCase("div", div1, div2, 1.0L, 5.0L);
    testCase("div", div1, div2, 1.0L, 7.0L);
    testCase("div", div1, div2, 1.0L, 9.0L);
    testCase("div", div1, div2, 1.0L, 11.0L);
    testCase("div", div1, div2, 1.0L, 3.1415926535897932384626433832795L);
    testCase("div", div1, div2, 1.0L, 2.718281828459045235360287471352662497757L);
    testCase("div", div1, div2, 0x1p16000L, 0x1p-383L);
    testCase("div", div1, div2, 0x1p16000L, 0x1p-384L);
    testCase("div", div1, div2, 0x1p-16000L, 0x1p445L);
    testCase("div", div1, div2, 0x1p-16000L, 0x1p446L);
    roundTestCases("floor", floor1, floor2);
    roundTestCases("round", round1, round2);
    roundTestCases("ceil", ceil1, ceil2);
    roundTestCases("trunc", trunc1, trunc2);
    toIntTestCases("uint64", toUInt1, toUInt2);
    toIntTestCases("int64", toInt1, toInt2);
    testCase("pow", pow1, pow2, 1.0L, static_cast<std::int64_t>(0));
    testCase("pow", pow1, pow2, 1.0L, static_cast<std::int64_t>(5000));
    testCase("pow", pow1, pow2, 1.0L, static_cast<std::int64_t>(-5000));
    testCase("pow", pow1, pow2, 2.0L, static_cast<std::int64_t>(3000));
    testCase("pow", pow1, pow2, 2.0L, static_cast<std::int64_t>(-3000));
    testCase("pow", pow1, pow2, 3.0L, static_cast<std::int64_t>(3000));
    testCase("pow", pow1, pow2, 3.0L, static_cast<std::int64_t>(-3000));
    testCase("pow", pow1, pow2, 10.0L, static_cast<std::int64_t>(3000));
    testCase("pow", pow1, pow2, 10.0L, static_cast<std::int64_t>(-3000));
    testCase("pow", pow1, pow2, 36.0L, static_cast<std::int64_t>(3000));
    testCase("pow", pow1, pow2, 36.0L, static_cast<std::int64_t>(-3000));
    testCase("scalbn", scalbn1, scalbn2, 1.0L, static_cast<std::int64_t>(16384));
    testCase("scalbn", scalbn1, scalbn2, 1.0L, static_cast<std::int64_t>(16383));
    testCase("scalbn", scalbn1, scalbn2, 1.0L, static_cast<std::int64_t>(3000));
    testCase("scalbn", scalbn1, scalbn2, 1.0L, static_cast<std::int64_t>(-3000));
    testCase("scalbn", scalbn1, scalbn2, 1.0L, static_cast<std::int64_t>(-16383));
    testCase("scalbn", scalbn1, scalbn2, 1.0L, static_cast<std::int64_t>(-16384));
    testCase("scalbn", scalbn1, scalbn2, 1.0L, static_cast<std::int64_t>(-16445));
    testCase("scalbn", scalbn1, scalbn2, 1.0L, static_cast<std::int64_t>(-16446));
    testCase("log2", log2_1, log2_2, NaN);
    testCase("log2", log2_1, log2_2, Infinity);
    testCase("log2", log2_1, log2_2, -Infinity);
    testCase("log2", log2_1, log2_2, 0.0L);
    testCase("log2", log2_1, log2_2, -0.0L);
    testCase("log2", log2_1, log2_2, -1.0L);
    testCase("log2", log2_1, log2_2, 1.0L);
    testCase("log2", log2_1, log2_2, 2.0L);
    testCase("log2", log2_1, log2_2, 0x1.0p-16445L);
    testCase("log2", log2_1, log2_2, 0x1.0p16383L);
    testCase("log2", log2_1, log2_2, 3.0L);
    testCase("log2", log2_1, log2_2, 5.0L);
    testCase("log2", log2_1, log2_2, 7.0L);
    testCase("log2", log2_1, log2_2, 9.0L);
    testCase("log2", log2_1, log2_2, 11.0L);
    testCase("log2", log2_1, log2_2, 1e100L);
    testCase("log2", log2_1, log2_2, 1e-1L);
    testCase("log2", log2_1, log2_2, 1e-2L);
    testCase("log2", log2_1, log2_2, 1.5L);
    testCase("log2", log2_1, log2_2, 0.693147180559945309417232121458176568L);
    testCase("log2", log2_1, log2_2, static_cast<long double>(ExtendedFloat::Log10Of2()));
    testCase("log2", log2_1, log2_2, static_cast<long double>(ExtendedFloat::LogOf2()));
    testCase("log10", log10_1, log10_2, 1e1001L);
    testCase("log10", log10_1, log10_2, 1.5L);
}
struct Init
{
    Init()
    {
        mainFn();
        std::exit(0);
    }
};
Init init;
}
#endif
#else
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <thread>
#include <list>
#include <sstream>
#include <cassert>
namespace
{
unsigned clz8(std::uint8_t v)
{
    return __builtin_clz(v) - __builtin_clz(0x80U);
}
unsigned ctz8(std::uint8_t v)
{
    return v == 0 ? 8 : __builtin_ctz(v);
}
struct UInt16 final
{
    std::uint8_t high;
    std::uint8_t low;
    explicit UInt16(std::uint8_t low = 0) : high(0), low(low)
    {
    }
    UInt16(std::uint8_t high, std::uint8_t low) : high(high), low(low)
    {
    }
    friend unsigned clz16(UInt16 v)
    {
        return v.high == 0 ? 8 + clz8(v.low) : clz8(v.high);
    }
    friend unsigned ctz16(UInt16 v)
    {
        return v.low == 0 ? 8 + ctz8(v.high) : ctz8(v.low);
    }
    static UInt16 mul8x8(std::uint8_t a, std::uint8_t b)
    {
        unsigned v = a;
        v *= b;
        return UInt16(v >> 8, v & 0xFFU);
    }
    static bool addCarry(std::uint8_t a, std::uint8_t b)
    {
        return static_cast<std::uint16_t>(a) + b > 0xFFU;
    }
    static bool addCarry(std::uint8_t a, std::uint8_t b, bool carry)
    {
        return static_cast<unsigned>(a) + b + carry > 0xFFU;
    }
    static bool subBorrow(std::uint8_t a, std::uint8_t b)
    {
        return a < b;
    }
    static bool subBorrow(std::uint8_t a, std::uint8_t b, bool borrow)
    {
        return a < b || (a == b && borrow);
    }
    friend UInt16 operator+(UInt16 a, UInt16 b)
    {
        return UInt16(a.high + b.high + addCarry(a.low, b.low), a.low + b.low);
    }
    friend UInt16 operator-(UInt16 a, UInt16 b)
    {
        return UInt16(a.high - b.high - subBorrow(a.low, b.low), a.low - b.low);
    }
    friend UInt16 operator<<(UInt16 v, unsigned shiftAmount)
    {
        return shiftAmount == 0 ? v : shiftAmount < 8 ?
                                  UInt16((v.high << shiftAmount) | (v.low >> (8 - shiftAmount)),
                                         v.low << shiftAmount) :
                                  UInt16(v.low << (shiftAmount - 8), 0);
    }
    friend UInt16 operator>>(UInt16 v, unsigned shiftAmount)
    {
        return shiftAmount == 0 ? v : shiftAmount < 8 ?
                                  UInt16(v.high >> shiftAmount,
                                         (v.low >> shiftAmount) | (v.high << (8 - shiftAmount))) :
                                  UInt16(v.high >> (shiftAmount - 8));
    }
    struct DivModResult8 final
    {
        std::uint8_t divResult;
        std::uint8_t modResult;
        DivModResult8(std::uint8_t divResult, std::uint8_t modResult)
            : divResult(divResult), modResult(modResult)
        {
        }
    };
    static DivModResult8 divMod16x8(UInt16 n, std::uint8_t d)
    {
        assert(d != 0);
        std::uint16_t v = n.high;
        v <<= 8;
        v |= n.low;
        std::uint16_t divResult = v / d;
        std::uint16_t modResult = v % d;
        assert(divResult <= 0xFFU);
        assert(modResult <= 0xFFU);
        return DivModResult8(divResult, modResult);
    }
    struct DivModResult;
    static DivModResult divMod(UInt16 uIn, UInt16 vIn);
    static DivModResult divMod2(UInt16 n, UInt16 d);
    friend bool operator==(UInt16 a, UInt16 b) noexcept
    {
        return a.high == b.high && a.low == b.low;
    }
    friend bool operator!=(UInt16 a, UInt16 b) noexcept
    {
        return a.high != b.high || a.low != b.low;
    }
    friend bool operator<(UInt16 a, UInt16 b) noexcept
    {
        return a.high < b.high || (a.high == b.high && a.low < b.low);
    }
    friend bool operator<=(UInt16 a, UInt16 b) noexcept
    {
        return a.high < b.high || (a.high == b.high && a.low <= b.low);
    }
    friend bool operator>(UInt16 a, UInt16 b) noexcept
    {
        return a.high > b.high || (a.high == b.high && a.low > b.low);
    }
    friend bool operator>=(UInt16 a, UInt16 b) noexcept
    {
        return a.high > b.high || (a.high == b.high && a.low >= b.low);
    }
};
struct UInt16::DivModResult final
{
    UInt16 divResult;
    UInt16 modResult;
    DivModResult(UInt16 divResult, UInt16 modResult) : divResult(divResult), modResult(modResult)
    {
    }
};
UInt16::DivModResult UInt16::divMod2(UInt16 n, UInt16 d)
{
    std::uint16_t nv = n.high;
    nv <<= 8;
    nv |= n.low;
    std::uint16_t dv = d.high;
    dv <<= 8;
    dv |= d.low;
    std::uint16_t qv = nv / dv;
    std::uint16_t rv = nv % dv;
    return DivModResult(UInt16(qv >> 8, qv & 0xFF), UInt16(rv >> 8, rv & 0xFF));
}
template <std::size_t NumberSizes,
          typename Digit,
          typename DoubleDigit,
          unsigned DigitBitCount,
          typename DigitCLZFn>
void divMod(const Digit(&numerator)[NumberSizes],
            const Digit(&denominator)[NumberSizes],
            Digit(&quotient)[NumberSizes],
            Digit(&remainder)[NumberSizes])
{
    constexpr Digit DigitMax = (static_cast<DoubleDigit>(1) << DigitBitCount) - 1;
    static_assert(NumberSizes != 0, "bad size");
    std::size_t m = NumberSizes;
    for(std::size_t i = 0; i < NumberSizes; i++)
    {
        if(denominator[i] != 0)
        {
            m = i;
            break;
        }
    }
    const std::size_t n = NumberSizes - m;
    if(n <= 1)
    {
        assert(denominator[NumberSizes - 1] != 0);
        for(std::size_t i = 0; i < NumberSizes - 1; i++)
        {
            remainder[i] = 0;
        }
        Digit currentRemainder = 0;
        for(std::size_t i = 0; i < NumberSizes; i++)
        {
            DoubleDigit n = currentRemainder;
            n <<= DigitBitCount;
            n |= numerator[i];
            quotient[i] = n / denominator[NumberSizes - 1];
            currentRemainder = n % denominator[NumberSizes - 1];
        }
        remainder[NumberSizes - 1] = currentRemainder;
        return;
    }
    // from algorithm D, section 4.3.1 in Art of Computer Programming volume 2 by Knuth.
    unsigned log2D = DigitCLZFn()(denominator[m]);
    Digit u[NumberSizes + 1];
    u[NumberSizes] = (numerator[NumberSizes - 1] << log2D) & DigitMax;
    u[0] = ((static_cast<DoubleDigit>(numerator[0]) << log2D) >> DigitBitCount) & DigitMax;
    for(std::size_t i = 1; i < NumberSizes; i++)
    {
        DoubleDigit value = numerator[i - 1];
        value <<= DigitBitCount;
        value |= numerator[i];
        value <<= log2D;
        u[i] = (value >> DigitBitCount) & DigitMax;
    }
    Digit v[NumberSizes + 1] = {};
    v[n] = (denominator[NumberSizes - 1] << log2D) & DigitMax;
    for(std::size_t i = 1; i < n; i++)
    {
        DoubleDigit value = denominator[m + i - 1];
        value <<= DigitBitCount;
        value |= denominator[m + i];
        value <<= log2D;
        v[i] = (value >> DigitBitCount) & DigitMax;
        quotient[i - 1] = 0;
    }
    for(std::size_t j = 0; j <= m; j++)
    {
        DoubleDigit qHat;
        if(u[j] == v[1])
        {
            qHat = DigitMax;
        }
        else
        {
            qHat = ((static_cast<DoubleDigit>(u[j]) << DigitBitCount) | u[j + 1]) / v[1];
        }
        {
            DoubleDigit lhs = v[2] * qHat;
            DoubleDigit rhsHigh =
                ((static_cast<DoubleDigit>(u[j]) << DigitBitCount) | u[j + 1]) - qHat * v[1];
            Digit rhsLow = u[j + 2];
            if(rhsHigh < static_cast<DoubleDigit>(1) << DigitBitCount
               && lhs > ((rhsHigh << DigitBitCount) | rhsLow))
            {
                qHat--;
                lhs -= v[2];
                rhsHigh += v[1];
                if(rhsHigh < static_cast<DoubleDigit>(1) << DigitBitCount
                   && lhs > ((rhsHigh << DigitBitCount) | rhsLow))
                {
                    qHat--;
                }
            }
        }
        bool borrow = false;
        {
            Digit mulCarry = 0;
            for(std::size_t i = n; i > 0; i--)
            {
                assert(i <= NumberSizes);
                DoubleDigit product = qHat * v[i] + mulCarry;
                mulCarry = product >> DigitBitCount;
                product &= DigitMax;
                bool prevBorrow = borrow;
                DoubleDigit digit = u[j + i] - product - prevBorrow;
                borrow = digit != (digit & DigitMax);
                digit &= DigitMax;
                u[j + i] = digit;
            }
            bool prevBorrow = borrow;
            DoubleDigit digit = u[j] - mulCarry - prevBorrow;
            borrow = digit != (digit & DigitMax);
            digit &= DigitMax;
            u[j] = digit;
        }
        Digit qj = qHat;
        if(borrow)
        {
            qj--;
            bool carry = false;
            for(std::size_t i = n; i > 0; i--)
            {
                bool prevCarry = carry;
                assert(i + j <= NumberSizes);
                DoubleDigit digit = u[j + i] + v[i] + prevCarry;
                carry = digit != (digit & DigitMax);
                digit &= DigitMax;
                u[j + i] = digit;
            }
            u[j] = (u[j] + carry) & DigitMax;
        }
        quotient[j + n - 1] = qj;
    }
    for(std::size_t i = 0; i < NumberSizes; i++)
    {
        DoubleDigit value = u[i];
        value <<= DigitBitCount;
        value |= u[i + 1];
        remainder[i] = value >> log2D;
    }
}
struct OpClz4 final
{
    constexpr unsigned operator()(std::uint16_t value) const noexcept
    {
        return __builtin_clz(value) - (__builtin_clz(0) - 4);
    }
};
UInt16::DivModResult UInt16::divMod(UInt16 uIn, UInt16 vIn)
{
    constexpr std::size_t NumberSizes = 4;
    typedef std::uint16_t Digit;
    typedef unsigned DoubleDigit;
    constexpr unsigned DigitBitCount = 4;
    Digit numerator[NumberSizes], denominator[NumberSizes], quotient[NumberSizes],
        remainder[NumberSizes];
    numerator[0] = uIn.high >> 4;
    numerator[1] = uIn.high & 0xF;
    numerator[2] = uIn.low >> 4;
    numerator[3] = uIn.low & 0xF;
    denominator[0] = vIn.high >> 4;
    denominator[1] = vIn.high & 0xF;
    denominator[2] = vIn.low >> 4;
    denominator[3] = vIn.low & 0xF;
    ::divMod<NumberSizes, Digit, DoubleDigit, DigitBitCount, OpClz4>(
        numerator, denominator, quotient, remainder);
    return DivModResult(
        UInt16((quotient[0] << 4) | quotient[1], (quotient[2] << 4) | quotient[3]),
        UInt16((remainder[0] << 4) | remainder[1], (remainder[2] << 4) | remainder[3]));
}
void mainFn(std::uint8_t start, std::uint8_t end)
{
    for(unsigned dHigh = start; dHigh <= end; dHigh++)
    {
        if(start == 0)
        {
            std::ostringstream ss;
            ss << dHigh * 100 / (end + 1) << "%\n";
            std::cout << ss.str() << std::flush;
        }
        for(unsigned dLow = 0; dLow < 0x100U; dLow++)
        {
            UInt16 d(dHigh, dLow);
            if(d == UInt16(0))
                continue;
#if 0
            if(d < UInt16(2, 0))
                continue;
#endif
            for(unsigned nHigh = 0; nHigh < 0x100U; nHigh++)
            {
                for(unsigned nLow = 0; nLow < 0x100U; nLow++)
                {
                    UInt16 n(nHigh, nLow);
                    auto result = UInt16::divMod(n, d);
                    auto result2 = UInt16::divMod2(n, d);
                    if(result.divResult != result2.divResult
                       || result.modResult != result2.modResult)
                    {
                        std::ostringstream ss;
                        ss << std::hex << std::uppercase;
                        ss.fill('0');
                        ss.width(2);
                        ss << static_cast<unsigned>(n.high);
                        ss.width(2);
                        ss << static_cast<unsigned>(n.low);
                        ss << " / ";
                        ss.width(2);
                        ss << static_cast<unsigned>(d.high);
                        ss.width(2);
                        ss << static_cast<unsigned>(d.low);
                        ss << " == ";
                        ss.width(2);
                        ss << static_cast<unsigned>(result.divResult.high);
                        ss.width(2);
                        ss << static_cast<unsigned>(result.divResult.low);
                        ss << ", ";
                        ss.width(2);
                        ss << static_cast<unsigned>(result2.divResult.high);
                        ss.width(2);
                        ss << static_cast<unsigned>(result2.divResult.low);
                        ss << std::endl;
                        ss.width(2);
                        ss << static_cast<unsigned>(n.high);
                        ss.width(2);
                        ss << static_cast<unsigned>(n.low);
                        ss << " % ";
                        ss.width(2);
                        ss << static_cast<unsigned>(d.high);
                        ss.width(2);
                        ss << static_cast<unsigned>(d.low);
                        ss << " == ";
                        ss.width(2);
                        ss << static_cast<unsigned>(result.modResult.high);
                        ss.width(2);
                        ss << static_cast<unsigned>(result.modResult.low);
                        ss << ", ";
                        ss.width(2);
                        ss << static_cast<unsigned>(result2.modResult.high);
                        ss.width(2);
                        ss << static_cast<unsigned>(result2.modResult.low);
                        std::cout << ss.str() << std::endl;
                        return;
                    }
                }
            }
        }
    }
}
struct Init
{
    Init()
    {
        const std::size_t splitCount = 6;
        std::list<std::thread> threads;
        for(std::size_t i = 0; i < splitCount; i++)
        {
            auto start = i * 0x100 / splitCount;
            auto end = (i + 1) * 0x100 / splitCount - 1;
            threads.push_back(std::thread([=]()
                                          {
                                              mainFn(start, end);
                                          }));
        }
        for(std::thread &thread : threads)
            thread.join();
        std::exit(0);
    }
};
Init init;
}
#endif
