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
// https://github.com/programmerjake/javascript-tasklets/blob/master/javascript_tasklets/soft_float.h

#ifndef UTIL_SOFT_FLOAT_H_
#define UTIL_SOFT_FLOAT_H_

#include <cstdint>
#include <cmath>
#include <cassert>
#include "bit_intrinsics.h"

namespace vulkan_cpu
{
namespace util
{
namespace soft_float
{
struct UInt128 final
{
    std::uint64_t low;
    std::uint64_t high;
    constexpr UInt128(std::uint64_t high, std::uint64_t low) noexcept : low(low), high(high)
    {
    }
    constexpr explicit UInt128(std::uint64_t low = 0) noexcept : low(low), high(0)
    {
    }
    static constexpr bool addCarries(std::uint64_t a, std::uint64_t b) noexcept
    {
        return static_cast<std::uint64_t>(a + b) < a;
    }
    static constexpr bool subBorrows(std::uint64_t a, std::uint64_t b) noexcept
    {
        return static_cast<std::uint64_t>(a - b) > a;
    }
    friend constexpr UInt128 operator+(UInt128 a, UInt128 b) noexcept
    {
        return UInt128(a.high + b.high + addCarries(a.low, b.low), a.low + b.low);
    }
    constexpr UInt128 &operator+=(UInt128 v) noexcept
    {
        return *this = *this + v;
    }
    friend constexpr UInt128 operator-(UInt128 a, UInt128 b) noexcept
    {
        return UInt128(a.high - b.high - subBorrows(a.low, b.low), a.low - b.low);
    }
    constexpr UInt128 &operator-=(UInt128 v) noexcept
    {
        return *this = *this - v;
    }

private:
    static constexpr std::uint64_t multiplyHighHelper2(std::uint64_t h,
                                                       std::uint64_t m1,
                                                       std::uint64_t m2,
                                                       std::uint64_t l) noexcept
    {
        return (UInt128(h, l) + UInt128(m1 >> 32, m1 << 32) + UInt128(m2 >> 32, m2 << 32)).high;
    }
    static constexpr std::uint64_t multiplyHighHelper1(std::uint32_t ah,
                                                       std::uint32_t al,
                                                       std::uint32_t bh,
                                                       std::uint32_t bl) noexcept
    {
        return multiplyHighHelper2(static_cast<std::uint64_t>(ah) * bh,
                                   static_cast<std::uint64_t>(ah) * bl,
                                   static_cast<std::uint64_t>(al) * bh,
                                   static_cast<std::uint64_t>(al) * bl);
    }

public:
    static constexpr std::uint64_t multiplyHigh(std::uint64_t a, std::uint64_t b) noexcept
    {
        return multiplyHighHelper1(a >> 32, a, b >> 32, b);
    }
    friend constexpr UInt128 operator*(UInt128 a, UInt128 b) noexcept
    {
        return UInt128(a.high * b.low + a.low * b.high + multiplyHigh(a.low, b.low), a.low * b.low);
    }
    constexpr UInt128 &operator*=(UInt128 v) noexcept
    {
        return *this = *this * v;
    }
    struct DivModResult;
    static constexpr DivModResult divmod(UInt128 a, UInt128 b) noexcept;
    static constexpr UInt128 div(UInt128 a, UInt128 b) noexcept;
    static constexpr UInt128 mod(UInt128 a, UInt128 b) noexcept;
    friend constexpr UInt128 operator/(UInt128 a, UInt128 b) noexcept
    {
        return div(a, b);
    }
    friend constexpr UInt128 operator%(UInt128 a, UInt128 b) noexcept
    {
        return mod(a, b);
    }
    constexpr UInt128 &operator/=(UInt128 v) noexcept
    {
        return *this = *this / v;
    }
    constexpr UInt128 &operator%=(UInt128 v) noexcept
    {
        return *this = *this % v;
    }
    friend constexpr UInt128 operator&(UInt128 a, UInt128 b) noexcept
    {
        return UInt128(a.high & b.high, a.low & b.low);
    }
    constexpr UInt128 &operator&=(UInt128 v) noexcept
    {
        return *this = *this & v;
    }
    friend constexpr UInt128 operator|(UInt128 a, UInt128 b) noexcept
    {
        return UInt128(a.high | b.high, a.low | b.low);
    }
    constexpr UInt128 &operator|=(UInt128 v) noexcept
    {
        return *this = *this | v;
    }
    friend constexpr UInt128 operator^(UInt128 a, UInt128 b) noexcept
    {
        return UInt128(a.high ^ b.high, a.low ^ b.low);
    }
    constexpr UInt128 &operator^=(UInt128 v) noexcept
    {
        return *this = *this ^ v;
    }
    friend constexpr UInt128 operator<<(UInt128 v, unsigned shiftAmount) noexcept
    {
        assert(shiftAmount < 128);
        return shiftAmount == 0 ? v : shiftAmount < 64 ?
                                  UInt128((v.high << shiftAmount) | (v.low >> (64 - shiftAmount)),
                                          v.low << shiftAmount) :
                                  shiftAmount == 64 ? UInt128(v.low, 0) :
                                                      UInt128(v.low << (shiftAmount - 64), 0);
    }
    constexpr UInt128 &operator<<=(unsigned shiftAmount) noexcept
    {
        return *this = *this << shiftAmount;
    }
    friend constexpr UInt128 operator>>(UInt128 v, unsigned shiftAmount) noexcept
    {
        assert(shiftAmount < 128);
        return shiftAmount == 0 ? v : shiftAmount < 64 ?
                                  UInt128(v.high >> shiftAmount,
                                          (v.low >> shiftAmount) | (v.high << (64 - shiftAmount))) :
                                  shiftAmount == 64 ? UInt128(0, v.high) :
                                                      UInt128(0, v.high >> (shiftAmount - 64));
    }
    constexpr UInt128 &operator>>=(unsigned shiftAmount) noexcept
    {
        return *this = *this >> shiftAmount;
    }
    constexpr UInt128 operator+() noexcept
    {
        return *this;
    }
    constexpr UInt128 operator~() noexcept
    {
        return UInt128(~high, ~low);
    }
    constexpr UInt128 operator-() noexcept
    {
        return low != 0 ? UInt128(~high, -low) : UInt128(-high, 0);
    }
    friend constexpr bool operator==(UInt128 a, UInt128 b) noexcept
    {
        return a.high == b.high && a.low == b.low;
    }
    friend constexpr bool operator!=(UInt128 a, UInt128 b) noexcept
    {
        return a.high != b.high || a.low != b.low;
    }
    friend constexpr bool operator<(UInt128 a, UInt128 b) noexcept
    {
        return a.high < b.high || (a.high == b.high && a.low < b.low);
    }
    friend constexpr bool operator<=(UInt128 a, UInt128 b) noexcept
    {
        return a.high < b.high || (a.high == b.high && a.low <= b.low);
    }
    friend constexpr bool operator>(UInt128 a, UInt128 b) noexcept
    {
        return a.high > b.high || (a.high == b.high && a.low > b.low);
    }
    friend constexpr bool operator>=(UInt128 a, UInt128 b) noexcept
    {
        return a.high > b.high || (a.high == b.high && a.low >= b.low);
    }
    friend constexpr unsigned clz128(UInt128 v) noexcept
    {
        return v.high == 0 ? 64 + clz64(v.low) : clz64(v.high);
    }
    friend constexpr unsigned ctz128(UInt128 v) noexcept
    {
        return v.low == 0 ? 64 + ctz64(v.high) : ctz64(v.low);
    }
};

struct UInt128::DivModResult final
{
    UInt128 divResult;
    UInt128 modResult;
    constexpr DivModResult(UInt128 divResult, UInt128 modResult) noexcept : divResult(divResult),
                                                                            modResult(modResult)
    {
    }
};

constexpr UInt128::DivModResult UInt128::divmod(UInt128 a, UInt128 b) noexcept
{
    constexpr std::size_t NumberSizes = 4;
    typedef std::uint32_t Digit;
    typedef std::uint64_t DoubleDigit;
    constexpr unsigned DigitBitCount = 32;
    struct DigitCLZFn final
    {
        constexpr unsigned operator()(Digit v) const noexcept
        {
            return clz32(v);
        }
    };
    constexpr Digit DigitMax = (static_cast<DoubleDigit>(1) << DigitBitCount) - 1;
    const Digit numerator[NumberSizes] = {
        static_cast<Digit>(a.high >> DigitBitCount),
        static_cast<Digit>(a.high & DigitMax),
        static_cast<Digit>(a.low >> DigitBitCount),
        static_cast<Digit>(a.low & DigitMax),
    };
    const Digit denominator[NumberSizes] = {
        static_cast<Digit>(b.high >> DigitBitCount),
        static_cast<Digit>(b.high & DigitMax),
        static_cast<Digit>(b.low >> DigitBitCount),
        static_cast<Digit>(b.low & DigitMax),
    };
    Digit quotient[NumberSizes]{};
    Digit remainder[NumberSizes]{};
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
    }
    else
    {
        // from algorithm D, section 4.3.1 in Art of Computer Programming volume 2 by Knuth.
        unsigned log2D = DigitCLZFn()(denominator[m]);
        Digit u[NumberSizes + 1]{};
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
            DoubleDigit qHat{};
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
    return DivModResult(
        UInt128((static_cast<DoubleDigit>(quotient[0]) << DigitBitCount) | quotient[1],
                (static_cast<DoubleDigit>(quotient[2]) << DigitBitCount) | quotient[3]),
        UInt128((static_cast<DoubleDigit>(remainder[0]) << DigitBitCount) | remainder[1],
                (static_cast<DoubleDigit>(remainder[2]) << DigitBitCount) | remainder[3]));
}

constexpr UInt128 UInt128::div(UInt128 a, UInt128 b) noexcept
{
    return divmod(a, b).divResult;
}

constexpr UInt128 UInt128::mod(UInt128 a, UInt128 b) noexcept
{
    return divmod(a, b).modResult;
}

struct ExtendedFloat final // modeled after IEEE754 standard
{
    std::uint64_t mantissa;
    std::uint16_t exponent;
    bool sign;
    static constexpr std::uint16_t infinityNaNExponent() noexcept
    {
        return 0xFFFFU;
    }
    static constexpr std::uint16_t exponentBias() noexcept
    {
        return 0x7FFFU;
    }
    static constexpr std::uint64_t normalizedMantissaMax() noexcept
    {
        return 0xFFFFFFFFFFFFFFFFULL;
    }
    static constexpr std::uint64_t normalizedMantissaMin() noexcept
    {
        return 0x8000000000000000ULL;
    }
    struct NormalizedTag final
    {
    };
    static constexpr ExtendedFloat normalizeHelper(const ExtendedFloat &v,
                                                   unsigned shiftAmount) noexcept
    {
        return shiftAmount > 0 && v.exponent >= shiftAmount ?
                   ExtendedFloat(NormalizedTag{},
                                 v.mantissa << shiftAmount,
                                 v.exponent - shiftAmount,
                                 v.sign) :
                   v;
    }
    static constexpr ExtendedFloat normalizeHelper(UInt128 mantissa,
                                                   std::int32_t exponent,
                                                   bool sign,
                                                   int shiftAmount) noexcept
    {
        return shiftAmount > 0 && exponent >= shiftAmount ?
                   ExtendedFloat(NormalizedTag{},
                                 (mantissa << shiftAmount).high,
                                 exponent - shiftAmount,
                                 sign) :
                   ExtendedFloat(NormalizedTag{}, mantissa.high, exponent, sign);
    }
    static constexpr ExtendedFloat normalize(const ExtendedFloat &v) noexcept
    {
        return v.exponent == infinityNaNExponent() ? v : v.mantissa == 0 ?
                                                     Zero(v.sign) :
                                                     normalizeHelper(v, clz64(v.mantissa));
    }
    static constexpr ExtendedFloat normalize(UInt128 mantissa,
                                             std::uint16_t exponent,
                                             bool sign) noexcept
    {
        return exponent == infinityNaNExponent() ?
                   ExtendedFloat(
                       NormalizedTag{}, mantissa != UInt128(0), infinityNaNExponent(), sign) :
                   mantissa == UInt128(0) ?
                   Zero(sign) :
                   normalizeHelper(mantissa, exponent, sign, clz128(mantissa));
    }
    constexpr ExtendedFloat() noexcept : mantissa(0), exponent(0), sign(false)
    {
    }
    constexpr ExtendedFloat(NormalizedTag,
                            std::uint64_t mantissa,
                            std::uint16_t exponent,
                            bool sign = false) noexcept : mantissa(mantissa),
                                                          exponent(exponent),
                                                          sign(sign)
    {
    }
    explicit constexpr ExtendedFloat(std::uint64_t mantissa,
                                     std::uint16_t exponent = exponentBias() + 63,
                                     bool sign = false) noexcept
        : ExtendedFloat(normalize(ExtendedFloat(NormalizedTag{}, mantissa, exponent, sign)))
    {
    }
    explicit constexpr ExtendedFloat(UInt128 mantissa,
                                     std::uint16_t exponent = exponentBias() + 127,
                                     bool sign = false) noexcept
        : ExtendedFloat(normalize(mantissa, exponent, sign))
    {
    }
    explicit constexpr ExtendedFloat(std::int64_t mantissa) noexcept
        : ExtendedFloat(mantissa < 0 ? -static_cast<std::uint64_t>(mantissa) :
                                       static_cast<std::uint64_t>(mantissa),
                        exponentBias() + 63,
                        mantissa < 0)
    {
    }
    explicit ExtendedFloat(double value) noexcept : mantissa(0),
                                                    exponent(0),
                                                    sign(std::signbit(value))
    {
        value = std::fabs(value);
        if(std::isnan(value))
        {
            mantissa = 1;
            exponent = infinityNaNExponent();
            return;
        }
        if(std::isinf(value))
        {
            exponent = infinityNaNExponent();
            mantissa = 0;
            return;
        }
        if(value == 0)
        {
            exponent = 0;
            mantissa = 0;
            return;
        }
        int log2Value = std::ilogb(value);
        if(log2Value <= -static_cast<int>(exponentBias()))
            exponent = 0;
        else
            exponent = log2Value + exponentBias();
        value = std::scalbn(value, 63 - static_cast<long>(exponent) + exponentBias());
        mantissa = value;
    }
    explicit ExtendedFloat(long double value) noexcept : mantissa(0),
                                                         exponent(0),
                                                         sign(std::signbit(value))
    {
        value = std::fabs(value);
        if(std::isnan(value))
        {
            mantissa = 1;
            exponent = infinityNaNExponent();
            return;
        }
        if(std::isinf(value))
        {
            exponent = infinityNaNExponent();
            mantissa = 0;
            return;
        }
        if(value == 0)
        {
            exponent = 0;
            mantissa = 0;
            return;
        }
        int log2Value = std::ilogb(value);
        if(log2Value <= -static_cast<int>(exponentBias()))
            exponent = 0;
        else
            exponent = log2Value + exponentBias();
        value = std::scalbn(value, 63 - static_cast<long>(exponent) + exponentBias());
        mantissa = value;
    }
    static constexpr ExtendedFloat fromHalfPrecision(std::uint16_t value) noexcept
    {
        bool sign = (value & 0x8000U) != 0;
        std::uint16_t exponentField = (value & 0x7C00U) >> 10;
        std::uint16_t mantissaField = value & 0x3FFU;
        if(exponentField == 0x1FU)
        {
            if(mantissaField != 0)
                return NaN();
            return Infinity(sign);
        }
        if(exponentField != 0)
            mantissaField |= 0x400U; // add in implicit 1
        else
            exponentField = 1;
        return ExtendedFloat(mantissaField, static_cast<int>(exponentField) - 15 - 10 + exponentBias() + 63, sign);
    }
    explicit operator long double() const noexcept
    {
        if(exponent == infinityNaNExponent())
        {
            double retval = std::numeric_limits<double>::infinity();
            if(mantissa)
                retval = std::numeric_limits<double>::quiet_NaN();
            if(sign)
                return -retval;
            return retval;
        }
        if(isZero())
        {
            if(sign)
                return -0.0;
            return 0;
        }
        long double value = std::scalbln(static_cast<long double>(mantissa),
                                         static_cast<long>(exponent) - exponentBias() - 63);
        if(sign)
            return -value;
        return value;
    }
    explicit operator double() const noexcept
    {
        if(exponent == infinityNaNExponent())
        {
            double retval = std::numeric_limits<double>::infinity();
            if(mantissa)
                retval = std::numeric_limits<double>::quiet_NaN();
            if(sign)
                return -retval;
            return retval;
        }
        if(isZero())
        {
            if(sign)
                return -0.0;
            return 0;
        }
        double value = std::scalbln(static_cast<double>(mantissa),
                                    static_cast<long>(exponent) - exponentBias() - 63);
        if(sign)
            return -value;
        return value;
    }
    constexpr bool isNaN() const noexcept
    {
        return exponent == infinityNaNExponent() && mantissa != 0;
    }
    constexpr bool isInfinite() const noexcept
    {
        return exponent == infinityNaNExponent() && mantissa == 0;
    }
    constexpr bool isFinite() const noexcept
    {
        return exponent != infinityNaNExponent();
    }
    constexpr bool isNormal() const noexcept
    {
        return exponent != infinityNaNExponent() && exponent != 0;
    }
    constexpr bool isDenormal() const noexcept
    {
        return exponent == 0 && mantissa != 0;
    }
    constexpr bool isZero() const noexcept
    {
        return exponent == 0 && mantissa == 0;
    }
    constexpr bool signBit() const noexcept
    {
        return sign;
    }
    static constexpr ExtendedFloat NaN() noexcept
    {
        return ExtendedFloat(NormalizedTag{}, 1, infinityNaNExponent());
    }
    static constexpr ExtendedFloat One() noexcept
    {
        return ExtendedFloat(NormalizedTag{}, 0x8000000000000000ULL, exponentBias());
    }
    static constexpr ExtendedFloat TwoToThe64() noexcept
    {
        return ExtendedFloat(NormalizedTag{}, 0x8000000000000000ULL, exponentBias() + 64);
    }
    static constexpr ExtendedFloat Infinity(bool sign = false) noexcept
    {
        return ExtendedFloat(NormalizedTag{}, 0, infinityNaNExponent(), sign);
    }
    static constexpr ExtendedFloat Zero(bool sign = false) noexcept
    {
        return ExtendedFloat(NormalizedTag{}, 0, 0, sign);
    }
    constexpr ExtendedFloat operator+() const noexcept
    {
        return *this;
    }
    constexpr ExtendedFloat operator-() const noexcept
    {
        return ExtendedFloat(NormalizedTag{}, mantissa, exponent, !sign);
    }
    static constexpr UInt128 shiftHelper(std::uint64_t a, unsigned shift) noexcept
    {
        return shift >= 128 ? UInt128(0) : UInt128(a, 0) >> shift;
    }
    static constexpr UInt128 finalRoundHelper(UInt128 v) noexcept
    {
        return v.low == 0x8000000000000000ULL && (v.high & 1) == 0 ?
                   UInt128(v.high) :
                   ((v >> 1) + UInt128(0x4000000000000000ULL)) >> 63;
    }
    static constexpr ExtendedFloat subtractHelper6(UInt128 mantissa,
                                                   std::uint16_t exponent,
                                                   bool sign,
                                                   unsigned shift)
    {
        return ExtendedFloat(finalRoundHelper(mantissa << shift), exponent - shift + 64, sign);
    }
    static constexpr ExtendedFloat subtractHelper5(UInt128 mantissa,
                                                   std::uint16_t exponent,
                                                   bool sign,
                                                   unsigned shift)
    {
        return subtractHelper6(mantissa, exponent, sign, shift > exponent ? exponent : shift);
    }
    static constexpr ExtendedFloat subtractHelper4(UInt128 mantissa,
                                                   std::uint16_t exponent,
                                                   bool sign)
    {
        return subtractHelper5(mantissa, exponent, sign, clz128(mantissa));
    }
    static constexpr ExtendedFloat subtractHelper3(UInt128 aMantissa,
                                                   UInt128 bMantissa,
                                                   std::uint16_t exponent) noexcept
    {
        return aMantissa == bMantissa ? Zero() : aMantissa < bMantissa ?
                                        subtractHelper4(bMantissa - aMantissa, exponent, true) :
                                        subtractHelper4(aMantissa - bMantissa, exponent, false);
    }
    static constexpr ExtendedFloat subtractHelper2(std::uint64_t aMantissa,
                                                   std::uint16_t aExponent,
                                                   std::uint64_t bMantissa,
                                                   std::uint16_t bExponent,
                                                   std::uint16_t maxExponent) noexcept
    {
        return subtractHelper3(shiftHelper(aMantissa, maxExponent - aExponent),
                               shiftHelper(bMantissa, maxExponent - bExponent),
                               maxExponent);
    }
    static constexpr ExtendedFloat subtractHelper(std::uint64_t aMantissa,
                                                  std::uint16_t aExponent,
                                                  std::uint64_t bMantissa,
                                                  std::uint16_t bExponent) noexcept
    {
        return subtractHelper2(aMantissa,
                               aExponent,
                               bMantissa,
                               bExponent,
                               aExponent < bExponent ? bExponent : aExponent);
    }
    static constexpr ExtendedFloat addHelper3(UInt128 mantissa,
                                              std::uint16_t exponent,
                                              bool sign) noexcept
    {
        return mantissa >= UInt128(0x8000000000000000ULL, 0) ?
                   (exponent + 1 == infinityNaNExponent() ?
                        Infinity(sign) :
                        ExtendedFloat(finalRoundHelper(mantissa), exponent + 65, sign)) :
                   ExtendedFloat(finalRoundHelper(mantissa << 1), exponent + 64, sign);
    }
    static constexpr ExtendedFloat addHelper2(std::uint64_t aMantissa,
                                              std::uint16_t aExponent,
                                              std::uint64_t bMantissa,
                                              std::uint16_t bExponent,
                                              std::uint16_t maxExponent,
                                              bool sign) noexcept
    {
        return addHelper3(shiftHelper(aMantissa, maxExponent - aExponent + 1)
                              + shiftHelper(bMantissa, maxExponent - bExponent + 1),
                          maxExponent,
                          sign);
    }
    static constexpr ExtendedFloat addHelper(std::uint64_t aMantissa,
                                             std::uint16_t aExponent,
                                             std::uint64_t bMantissa,
                                             std::uint16_t bExponent,
                                             bool sign) noexcept
    {
        return addHelper2(aMantissa,
                          aExponent,
                          bMantissa,
                          bExponent,
                          aExponent < bExponent ? bExponent : aExponent,
                          sign);
    }
    constexpr friend ExtendedFloat operator+(const ExtendedFloat &a,
                                             const ExtendedFloat &b) noexcept
    {
        return a.isNaN() ? a : b.isNaN() ?
                           b :
                           a.isInfinite() ?
                           (b.isInfinite() ? (a.sign == b.sign ? a : NaN()) : a) :
                           b.isInfinite() ?
                           b :
                           a.isZero() ?
                           (b.isZero() ? Zero(a.sign && b.sign) : b) :
                           b.isZero() ?
                           a :
                           a.sign == b.sign ?
                           addHelper(a.mantissa, a.exponent, b.mantissa, b.exponent, a.sign) :
                           a.sign ? subtractHelper(b.mantissa, b.exponent, a.mantissa, a.exponent) :
                                    subtractHelper(a.mantissa, a.exponent, b.mantissa, b.exponent);
    }
    constexpr friend ExtendedFloat operator-(const ExtendedFloat &a,
                                             const ExtendedFloat &b) noexcept
    {
        return a + b.operator-();
    }
    constexpr ExtendedFloat &operator+=(const ExtendedFloat &v) noexcept
    {
        return *this = *this + v;
    }
    constexpr ExtendedFloat &operator-=(const ExtendedFloat &v) noexcept
    {
        return *this = *this - v;
    }
    friend constexpr bool operator==(const ExtendedFloat &a, const ExtendedFloat &b) noexcept
    {
        return a.isNaN() ? false : b.isNaN() ? false : a.isZero() ?
                                               b.isZero() :
                                               a.exponent == b.exponent && a.mantissa == b.mantissa;
    }
    friend constexpr bool operator!=(const ExtendedFloat &a, const ExtendedFloat &b) noexcept
    {
        return !(a == b);
    }
    static constexpr int compareHelper(const ExtendedFloat &a, const ExtendedFloat &b) noexcept
    {
        return a.isZero() ? (b.isZero() ? 0 : (b.sign ? 1 : -1)) : a.sign != b.sign ?
                            (a.sign ? -1 : 1) :
                            a.exponent != b.exponent ?
                            ((a.exponent < b.exponent) != a.sign ? -1 : 1) :
                            a.mantissa == b.mantissa ? 0 :
                                                       (a.mantissa < b.mantissa) != a.sign ? -1 : 1;
    }
    friend constexpr bool operator<(const ExtendedFloat &a, const ExtendedFloat &b) noexcept
    {
        return a.isNaN() ? false : b.isNaN() ? false : compareHelper(a, b) < 0;
    }
    friend constexpr bool operator<=(const ExtendedFloat &a, const ExtendedFloat &b) noexcept
    {
        return a.isNaN() ? false : b.isNaN() ? false : compareHelper(a, b) <= 0;
    }
    friend constexpr bool operator>(const ExtendedFloat &a, const ExtendedFloat &b) noexcept
    {
        return a.isNaN() ? false : b.isNaN() ? false : compareHelper(a, b) > 0;
    }
    friend constexpr bool operator>=(const ExtendedFloat &a, const ExtendedFloat &b) noexcept
    {
        return a.isNaN() ? false : b.isNaN() ? false : compareHelper(a, b) >= 0;
    }
    static constexpr ExtendedFloat mulHelper4(UInt128 mantissa,
                                              std::int32_t exponent,
                                              bool sign) noexcept
    {
        return exponent >= infinityNaNExponent() ?
                   Infinity(sign) :
                   exponent <= -128 ?
                   Zero(sign) :
                   exponent < 0 ? ExtendedFloat(finalRoundHelper(mantissa >> -exponent), 64, sign) :
                                  ExtendedFloat(finalRoundHelper(mantissa), exponent + 64, sign);
    }
    static constexpr ExtendedFloat mulHelper3(UInt128 mantissa,
                                              std::int32_t exponent,
                                              bool sign,
                                              unsigned shift) noexcept
    {
        return mulHelper4(mantissa << shift, exponent - shift, sign);
    }
    static constexpr ExtendedFloat mulHelper2(UInt128 mantissa,
                                              std::int32_t exponent,
                                              bool sign) noexcept
    {
        return mantissa == UInt128(0) ? Zero(sign) :
                                        mulHelper3(mantissa, exponent, sign, clz128(mantissa));
    }
    static constexpr ExtendedFloat mulHelper(std::uint64_t aMantissa,
                                             std::int32_t aExponent,
                                             std::uint64_t bMantissa,
                                             std::int32_t bExponent,
                                             bool sign) noexcept
    {
        return mulHelper2(UInt128(aMantissa) * UInt128(bMantissa),
                          aExponent + bExponent - exponentBias() + 1,
                          sign);
    }
    constexpr friend ExtendedFloat operator*(const ExtendedFloat &a,
                                             const ExtendedFloat &b) noexcept
    {
        return a.isNaN() ? a : b.isNaN() ?
                           b :
                           a.isInfinite() ?
                           (b.isZero() ? NaN() : Infinity(a.sign != b.sign)) :
                           b.isInfinite() ?
                           (a.isZero() ? NaN() : Infinity(a.sign != b.sign)) :
                           mulHelper(
                               a.mantissa, a.exponent, b.mantissa, b.exponent, a.sign != b.sign);
    }
    constexpr ExtendedFloat &operator*=(const ExtendedFloat &v) noexcept
    {
        return *this = *this * v;
    }
    static constexpr int compareU128(UInt128 a, UInt128 b) noexcept
    {
        return a == b ? 0 : a < b ? -1 : 1;
    }
    static constexpr ExtendedFloat divHelper6(UInt128 mantissa,
                                              std::int32_t exponent,
                                              bool sign) noexcept
    {
        return exponent >= infinityNaNExponent() ?
                   Infinity(sign) :
                   exponent <= -128 ?
                   Zero(sign) :
                   exponent < 0 ? ExtendedFloat(finalRoundHelper(mantissa >> -exponent), 64, sign) :
                                  ExtendedFloat(finalRoundHelper(mantissa), exponent + 64, sign);
    }
    static constexpr ExtendedFloat divHelper5(UInt128 quotient,
                                              unsigned shift,
                                              int roundExtraBitsCompareValue,
                                              std::int32_t exponent,
                                              bool sign) noexcept
    {
        return divHelper6(
            ((quotient << 2) | UInt128(static_cast<std::uint64_t>(2 - roundExtraBitsCompareValue)))
                << (shift - 2),
            exponent - shift + 64,
            sign);
    }
    static constexpr ExtendedFloat divHelper4(UInt128::DivModResult mantissa,
                                              std::uint64_t bMantissa,
                                              std::int32_t exponent,
                                              bool sign) noexcept
    {
        return divHelper5(mantissa.divResult,
                          clz128(mantissa.divResult),
                          compareU128(UInt128(bMantissa), mantissa.modResult << 1),
                          exponent,
                          sign);
    }
    static constexpr ExtendedFloat divHelper3(std::uint64_t aMantissa,
                                              std::uint64_t bMantissa,
                                              std::int32_t exponent,
                                              bool sign) noexcept
    {
        return divHelper4(
            UInt128::divmod(UInt128(aMantissa, 0), UInt128(bMantissa)), bMantissa, exponent, sign);
    }
    static constexpr ExtendedFloat divHelper2(std::uint64_t aMantissa,
                                              std::int32_t aExponent,
                                              unsigned aShift,
                                              std::uint64_t bMantissa,
                                              std::int32_t bExponent,
                                              unsigned bShift,
                                              bool sign) noexcept
    {
        return divHelper3(aMantissa << aShift,
                          bMantissa << bShift,
                          aExponent - aShift - (bExponent - bShift) + exponentBias() - 1,
                          sign);
    }
    static constexpr ExtendedFloat divHelper(std::uint64_t aMantissa,
                                             std::int32_t aExponent,
                                             std::uint64_t bMantissa,
                                             std::int32_t bExponent,
                                             bool sign) noexcept
    {
        return divHelper2(
            aMantissa, aExponent, clz64(aMantissa), bMantissa, bExponent, clz64(bMantissa), sign);
    }
    friend constexpr ExtendedFloat operator/(const ExtendedFloat &a,
                                             const ExtendedFloat &b) noexcept
    {
        return a.isNaN() ? a : b.isNaN() ?
                           b :
                           a.isInfinite() ?
                           (b.isInfinite() ? NaN() : Infinity(a.sign != b.sign)) :
                           b.isZero() ?
                           (a.isZero() ? NaN() : Infinity(a.sign != b.sign)) :
                           b.isInfinite() || a.isZero() ?
                           Zero(a.sign != b.sign) :
                           divHelper(
                               a.mantissa, a.exponent, b.mantissa, b.exponent, a.sign != b.sign);
    }
    constexpr ExtendedFloat &operator/=(const ExtendedFloat &v) noexcept
    {
        return *this = *this / v;
    }
    static constexpr ExtendedFloat floorCeilHelper2(std::uint64_t mantissa,
                                                    std::int32_t exponent) noexcept
    {
        return exponent >= infinityNaNExponent() ?
                   Infinity() :
                   exponent <= -128 ?
                   Zero() :
                   exponent < 0 ?
                   ExtendedFloat(finalRoundHelper(UInt128(mantissa, 0) >> -exponent), 64) :
                   ExtendedFloat(finalRoundHelper(UInt128(mantissa, 0)), exponent + 64);
    }
    static constexpr ExtendedFloat floorCeilHelper(UInt128 mantissa, std::int32_t exponent) noexcept
    {
        return mantissa.high != 0 ? floorCeilHelper2((mantissa >> 1).low, exponent + 1) :
                                    floorCeilHelper2(mantissa.low, exponent);
    }
    static constexpr ExtendedFloat ceilHelper2(UInt128 mantissa) noexcept
    {
        return mantissa.low != 0 ? (mantissa.high == ~static_cast<std::uint64_t>(0) ?
                                        TwoToThe64() :
                                        ExtendedFloat(mantissa.high + 1)) :
                                   ExtendedFloat(mantissa.high);
    }
    static constexpr ExtendedFloat ceilHelper(std::uint64_t mantissa,
                                              std::int32_t exponent) noexcept
    {
        return exponent < exponentBias() ?
                   One() :
                   exponent >= exponentBias() + 63 ?
                   ExtendedFloat(NormalizedTag{}, mantissa, exponent) :
                   ceilHelper2(UInt128(mantissa, 0) >> (exponentBias() - exponent + 63));
    }
    static constexpr ExtendedFloat floorHelper2(UInt128 mantissa) noexcept
    {
        return ExtendedFloat(mantissa.high);
    }
    static constexpr ExtendedFloat floorHelper(std::uint64_t mantissa,
                                               std::int32_t exponent) noexcept
    {
        return exponent < exponentBias() ?
                   Zero() :
                   exponent >= exponentBias() + 63 ?
                   ExtendedFloat(NormalizedTag{}, mantissa, exponent) :
                   floorHelper2(UInt128(mantissa, 0) >> (exponentBias() - exponent + 63));
    }
    constexpr friend ExtendedFloat floor(const ExtendedFloat &v) noexcept
    {
        return !v.isFinite() || v.isZero() ? v : v.sign ? -ceilHelper(v.mantissa, v.exponent) :
                                                          floorHelper(v.mantissa, v.exponent);
    }
    constexpr friend ExtendedFloat trunc(const ExtendedFloat &v) noexcept
    {
        return !v.isFinite() || v.isZero() ? v : v.sign ? -floorHelper(v.mantissa, v.exponent) :
                                                          floorHelper(v.mantissa, v.exponent);
    }
    constexpr friend ExtendedFloat ceil(const ExtendedFloat &v) noexcept
    {
        return !v.isFinite() || v.isZero() ? v : v.sign ? -floorHelper(v.mantissa, v.exponent) :
                                                          ceilHelper(v.mantissa, v.exponent);
    }
    static constexpr ExtendedFloat roundHelper(std::uint64_t mantissa,
                                               std::int32_t exponent) noexcept
    {
        return exponent < exponentBias() - 2 ?
                   Zero() :
                   exponent >= exponentBias() + 63 ?
                   ExtendedFloat(NormalizedTag{}, mantissa, exponent) :
                   ExtendedFloat(((UInt128(mantissa, 0) >> (exponentBias() - exponent + 64))
                                  + UInt128(0x4000000000000000ULL))
                                 >> 63);
    }
    constexpr friend ExtendedFloat round(const ExtendedFloat &v) noexcept
    {
        return !v.isFinite() || v.isZero() ? v : v.sign ? -roundHelper(v.mantissa, v.exponent) :
                                                          roundHelper(v.mantissa, v.exponent);
    }
    explicit constexpr operator std::uint64_t() const noexcept
    {
        return isNaN() ? 0 : isInfinite() ?
                         (sign ? 0 : ~static_cast<std::uint64_t>(0)) :
                         exponent < exponentBias() || sign ?
                         0 :
                         *this >= TwoToThe64() ?
                         ~static_cast<std::uint64_t>(0) :
                         (UInt128(mantissa, 0) >> (exponentBias() - exponent + 63)).high;
    }
    static constexpr std::int64_t toInt64Helper(bool sign, std::uint64_t uint64Value) noexcept
    {
        return sign ? (uint64Value > 0x8000000000000000ULL ?
                           -static_cast<std::int64_t>(0x7FFFFFFFFFFFFFFFULL) - 1 :
                           -static_cast<std::int64_t>(uint64Value)) :
                      uint64Value >= 0x8000000000000000ULL ?
                      static_cast<std::int64_t>(0x7FFFFFFFFFFFFFFFULL) :
                      static_cast<std::int64_t>(uint64Value);
    }
    explicit constexpr operator std::int64_t() const noexcept
    {
        return isNaN() ? 0 : sign ? toInt64Helper(true, static_cast<std::uint64_t>(operator-())) :
                                    toInt64Helper(false, static_cast<std::uint64_t>(*this));
    }
    static constexpr ExtendedFloat powHelper(const ExtendedFloat &base,
                                             const ExtendedFloat &currentValue,
                                             std::uint64_t exponent) noexcept
    {
        return exponent == 0 ? currentValue : exponent == 1 ?
                               currentValue * base :
                               exponent == 2 ?
                               currentValue * (base * base) :
                               exponent & 1 ?
                               powHelper(base * base, currentValue * base, exponent >> 1) :
                               powHelper(base * base, currentValue, exponent >> 1);
    }
    constexpr friend ExtendedFloat pow(const ExtendedFloat &base, std::uint64_t exponent) noexcept
    {
        return powHelper(base, One(), exponent);
    }
    friend ExtendedFloat pow(const ExtendedFloat &base, std::int64_t exponent) noexcept
    {
        return exponent < 0 ? powHelper(One() / base, One(), -exponent) :
                              powHelper(base, One(), exponent);
    }
    constexpr friend int ilogb(const ExtendedFloat &v) noexcept
    {
        return v.isNaN() ? FP_ILOGBNAN : v.isZero() ? FP_ILOGB0 : v.isInfinite() ?
                                                      std::numeric_limits<int>::max() :
                                                      static_cast<std::int32_t>(v.exponent)
                                                                          - exponentBias()
                                                                          - clz64(v.mantissa);
    }
    static constexpr ExtendedFloat scalbnHelper(std::uint64_t mantissa,
                                                std::int64_t exponent,
                                                bool sign) noexcept
    {
        return exponent >= infinityNaNExponent() ?
                   Infinity(sign) :
                   exponent <= -128 ?
                   Zero(sign) :
                   exponent < 0 ?
                   ExtendedFloat(finalRoundHelper(UInt128(mantissa, 0) >> -exponent), 64, sign) :
                   ExtendedFloat(finalRoundHelper(UInt128(mantissa, 0)), exponent + 64, sign);
    }
    constexpr friend ExtendedFloat scalbn(const ExtendedFloat &v, std::int64_t exponent) noexcept
    {
        return !v.isFinite() || v.isZero() ? v : scalbnHelper(
                                                     v.mantissa, v.exponent + exponent, v.sign);
    }
    static constexpr std::uint64_t log2Helper4(UInt128 mantissa) noexcept
    {
        return ~mantissa.high == 0
                       || ((mantissa.high & 1) == 0 && mantissa.low == 0x8000000000000000ULL) ?
                   mantissa.high :
                   (mantissa + UInt128(0x8000000000000000ULL)).high;
    }
    static constexpr UInt128 log2Helper3(UInt128 mantissa, unsigned bitsLeft) noexcept
    {
        return (bitsLeft > 0 ?
                    log2Helper2(
                        log2Helper4(mantissa << (mantissa.high & 0x8000000000000000ULL ? 0 : 1)),
                        bitsLeft - 1)
                        >> 1 :
                    UInt128(0))
               | UInt128(mantissa.high & 0x8000000000000000ULL, 0);
    }
    static constexpr UInt128 log2Helper2(std::uint64_t mantissa, unsigned bitsLeft) noexcept
    {
        return log2Helper3(UInt128(mantissa) * UInt128(mantissa), bitsLeft);
    }
    static constexpr ExtendedFloat log2Helper(const ExtendedFloat &v, unsigned shift) noexcept
    {
        return ExtendedFloat(finalRoundHelper(log2Helper2(v.mantissa << shift, 67)),
                             exponentBias() - 1 + 64,
                             0)
               + ExtendedFloat(static_cast<std::int64_t>(v.exponent) - exponentBias() - shift);
    }
    constexpr friend ExtendedFloat log2(const ExtendedFloat &v) noexcept
    {
        return v.isNaN() ? v : v.isZero() ? Infinity(true) : v.sign ?
                                            NaN() :
                                            v.isInfinite() ? v : log2Helper(v, clz64(v.mantissa));
    }
    static constexpr ExtendedFloat Log10Of2() noexcept
    {
        return ExtendedFloat(NormalizedTag{}, 0x9A209A84FBCFF799ULL, exponentBias() - 2);
    }
    static constexpr ExtendedFloat LogOf2() noexcept
    {
        return ExtendedFloat(NormalizedTag{}, 0xB17217F7D1CF79ACULL, exponentBias() - 1);
    }
    constexpr friend ExtendedFloat log10(const ExtendedFloat &v) noexcept
    {
        return log2(v) * Log10Of2();
    }
    constexpr friend ExtendedFloat log(const ExtendedFloat &v) noexcept
    {
        return log2(v) * LogOf2();
    }
};
}
}
}

#endif /* UTIL_SOFT_FLOAT_H_ */
