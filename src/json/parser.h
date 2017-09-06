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

#ifndef JSON_PARSER_H_
#define JSON_PARSER_H_

#include <string>
#include <memory>
#include <stdexcept>
#include <vector>
#include <iosfwd>
#include "json.h"
#include "source.h"
#include "location.h"
#include "util/optional.h"

namespace kazan
{
namespace json
{
class Parse_error : public std::runtime_error
{
public:
    Location location;
    Parse_error(json::Location location, const std::string &message)
        : runtime_error(location.to_string() + ": error: " + message)
    {
    }
    Parse_error(json::Location location, const char *message)
        : runtime_error(location.to_string() + ": error: " + message)
    {
    }
};

struct Parse_options
{
    bool allow_infinity_and_nan;
    bool allow_explicit_plus_sign_in_mantissa;
    bool allow_single_quote_strings;
    bool allow_number_to_start_with_dot;
    constexpr Parse_options() noexcept : allow_infinity_and_nan(false),
                                         allow_explicit_plus_sign_in_mantissa(false),
                                         allow_single_quote_strings(false),
                                         allow_number_to_start_with_dot(false)
    {
    }
    constexpr Parse_options(bool allow_infinity_and_nan,
                            bool allow_explicit_plus_sign_in_mantissa,
                            bool allow_single_quote_strings,
                            bool allow_number_to_start_with_dot) noexcept
        : allow_infinity_and_nan(allow_infinity_and_nan),
          allow_explicit_plus_sign_in_mantissa(allow_explicit_plus_sign_in_mantissa),
          allow_single_quote_strings(allow_single_quote_strings),
          allow_number_to_start_with_dot(allow_number_to_start_with_dot)
    {
    }
    static constexpr Parse_options default_options() noexcept
    {
        return Parse_options();
    }
    static constexpr Parse_options relaxed_options() noexcept
    {
        return Parse_options(true, true, true, true);
    }
};

ast::Value parse(const Source *source, Parse_options options = Parse_options::default_options());
}
}

#endif /* JSON_PARSER_H_ */
