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
#include "parser.h"
#include <fstream>
#include <iostream>
#include <cassert>
#include <algorithm>
#include <limits>
#include "../util/soft_float.h"

namespace vulkan_cpu
{
namespace json
{
namespace
{
constexpr bool is_new_line(char ch) noexcept
{
    return ch == '\r' || ch == '\n';
}

constexpr bool is_new_line_pair(char ch1, char ch2) noexcept
{
    return ch1 == '\r' && ch2 == '\n';
}

template <typename Add_Index>
void find_line_start_indexes_helper(Add_Index &&add_index,
                                    const char *contents,
                                    std::size_t contents_size)
{
    for(std::size_t i = 0; i < contents_size; i++)
    {
        char ch = contents[i];
        if(i + 1 < contents_size)
        {
            char ch2 = contents[i + 1];
            if(is_new_line_pair(ch, ch2))
            {
                add_index(i + 2);
                i++;
                continue;
            }
        }
        if(is_new_line(ch))
            add_index(i + 1);
    }
}
}

std::vector<std::size_t> Source::find_line_start_indexes(const char *contents,
                                                         std::size_t contents_size)
{
    std::size_t retval_size = 0;
    find_line_start_indexes_helper(
        [&](std::size_t)
        {
            retval_size++;
        },
        contents,
        contents_size);
    std::vector<std::size_t> retval;
    retval.reserve(retval_size);
    find_line_start_indexes_helper(
        [&](std::size_t index)
        {
            retval.push_back(index);
        },
        contents,
        contents_size);
    return retval;
}

Source Source::load_file(std::string file_name)
{
    // TODO: add code to use mmap
    std::ifstream is;
    is.exceptions(std::ios::badbit | std::ios::failbit);
    is.open(file_name);
    std::vector<char> buffer;
    while(is.peek() != std::char_traits<char>::eof())
    {
        if(buffer.size() == buffer.capacity())
            buffer.reserve(buffer.size() * 2);
        buffer.push_back(is.get());
    }
    is.close();
    buffer.shrink_to_fit();
    std::size_t contents_size = buffer.size();
    auto buffer_ptr = std::make_shared<std::vector<char>>(std::move(buffer));
    std::shared_ptr<const char> contents(buffer_ptr, buffer_ptr->data());
    return Source(std::move(file_name), std::move(contents), contents_size);
}

Source Source::load_stdin()
{
    auto &is = std::cin;
    is.clear();
    auto previous_exceptions = is.exceptions();
    std::vector<char> buffer;
    try
    {
        is.exceptions(std::ios::badbit | std::ios::failbit);
        while(is.peek() != std::char_traits<char>::eof())
        {
            if(buffer.size() == buffer.capacity())
                buffer.reserve(buffer.size() * 2);
            buffer.push_back(is.get());
        }
    }
    catch(...)
    {
        is.clear();
        is.exceptions(previous_exceptions);
    }
    is.clear();
    is.exceptions(previous_exceptions);
    buffer.shrink_to_fit();
    std::size_t contents_size = buffer.size();
    auto buffer_ptr = std::make_shared<std::vector<char>>(std::move(buffer));
    std::shared_ptr<const char> contents(buffer_ptr, buffer_ptr->data());
    return Source("stdin", std::move(contents), contents_size);
}

std::ostream &operator<<(std::ostream &os, const Source::Line_and_column &v)
{
    os << v.to_string();
    return os;
}

Source::Line_and_index Source::get_line_and_start_index(std::size_t char_index) const noexcept
{
    std::size_t line =
        1 + line_start_indexes.size()
        + (line_start_indexes.rbegin() - std::lower_bound(line_start_indexes.rbegin(),
                                                          line_start_indexes.rend(),
                                                          char_index,
                                                          std::greater<std::size_t>()));
    return Line_and_index(line, line <= 1 ? 0 : line_start_indexes[line - 2]);
}

namespace
{
constexpr std::size_t get_column_after_tab(std::size_t column, std::size_t tab_size) noexcept
{
    return tab_size == 0 || column == 0 ? column + 1 :
                                          column + (tab_size - (column - 1) % tab_size);
}
}

Source::Line_and_column Source::get_line_and_column(std::size_t char_index,
                                                    std::size_t tab_size) const noexcept
{
    auto line_and_start_index = get_line_and_start_index(char_index);
    std::size_t column = 1;
    for(std::size_t i = line_and_start_index.index; i < char_index; i++)
    {
        int ch = contents.get()[i];
        if(ch == '\t')
            column = get_column_after_tab(column, tab_size);
        else
            column++;
    }
    return Line_and_column(line_and_start_index.line, column);
}

std::ostream &operator<<(std::ostream &os, const Location &v)
{
    os << v.to_string();
    return os;
}

namespace
{
enum class Token_type
{
    eof,
    l_bracket,
    r_bracket,
    l_brace,
    r_brace,
    colon,
    comma,
    true_literal,
    false_literal,
    null_literal,
    string,
    number,
};

class Tokenizer final
{
private:
    std::size_t input_char_index;
    static constexpr int eof = std::char_traits<char>::eof();

public:
    const Source *const source;
    const Parse_options options;
    Location token_location;
    ast::Value token_value;
    Token_type token_type;

private:
    int peekc() const noexcept
    {
        if(input_char_index >= source->contents_size)
            return eof;
        return static_cast<unsigned char>(source->contents.get()[input_char_index]);
    }
    int getc() noexcept
    {
        int retval = peekc();
        input_char_index++;
        return retval;
    }
    static constexpr bool is_digit(int ch, unsigned base = 10) noexcept
    {
        return get_digit_value(ch) >= 0;
    }
    static constexpr int get_digit_value(int ch, unsigned base = 10) noexcept
    {
        unsigned retval{};
        if(ch >= '0' && ch <= '9')
            retval = ch - '0';
        else if(ch >= 'a' && ch <= 'z')
            retval = ch - 'a' + 0xA;
        else if(ch >= 'A' && ch <= 'Z')
            retval = ch - 'A' + 0xA;
        else
            return -1;
        if(retval >= base)
            return -1;
        return retval;
    }
    static constexpr bool is_letter(int ch) noexcept
    {
        return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z');
    }
    static constexpr bool is_control_character(int ch) noexcept
    {
        return ch >= 0 && ch < 0x20U;
    }
    static constexpr bool is_whitespace(int ch) noexcept
    {
        return ch == '\t' || ch == '\n' || ch == '\r' || ch == ' ';
    }
    static bool match_buffer_with_string(const char *buffer,
                                         std::size_t buffer_size,
                                         const char *string) noexcept
    {
        for(; buffer_size != 0 && *string; string++, buffer++, buffer_size--)
            if(*string != *buffer)
                return false;
        if(*string || buffer_size != 0)
            return false;
        return true;
    }
    std::uint32_t parse_4_hex_digits()
    {
        std::uint32_t retval = 0;
        for(std::size_t i = 0; i < 4; i++)
        {
            int digit_char = peekc();
            int digit_value = get_digit_value(digit_char, 0x10);
            if(digit_value < 0)
                throw Parse_error(Location(source, input_char_index), "missing hex digit");
            getc();
            retval <<= 4;
            retval |= digit_value;
        }
        return retval;
    }
    static std::string append_utf8(std::string buffer, std::uint32_t ch)
    {
        if(ch < 0x80U)
        {
            buffer += static_cast<unsigned char>(ch);
        }
        else if(ch < 0x800U)
        {
            buffer += static_cast<unsigned char>((ch >> 6) | 0xC0U);
            buffer += static_cast<unsigned char>((ch & 0x3FU) | 0x80U);
        }
        else if(ch < 0x10000UL)
        {
            buffer += static_cast<unsigned char>((ch >> 12) | 0xE0U);
            buffer += static_cast<unsigned char>(((ch >> 6) & 0x3FU) | 0x80U);
            buffer += static_cast<unsigned char>((ch & 0x3FU) | 0x80U);
        }
        else
        {
            buffer += static_cast<unsigned char>(((ch >> 18) & 0x7U) | 0xF0U);
            buffer += static_cast<unsigned char>(((ch >> 12) & 0x3FU) | 0xE0U);
            buffer += static_cast<unsigned char>(((ch >> 6) & 0x3FU) | 0x80U);
            buffer += static_cast<unsigned char>((ch & 0x3FU) | 0x80U);
        }
        return buffer;
    }

public:
    Tokenizer(const json::Source *source, Parse_options options)
        : input_char_index(0), source(source), options(options), token_location(), token_value()
    {
        next();
    }
    ast::Value get()
    {
        auto retval = token_value;
        next();
        return retval;
    }
    void next()
    {
        while(is_whitespace(peekc()))
            getc();
        token_location = Location(source, input_char_index);
        token_value = nullptr;
        bool got_minus = false, got_plus = false;
        if(peekc() == '-')
        {
            getc();
            got_minus = true;
        }
        else if(options.allow_explicit_plus_sign_in_mantissa && peekc() == '+')
        {
            getc();
            got_plus = true;
        }
        if(is_letter(peekc()))
        {
            const char *name = source->contents.get() + input_char_index;
            std::size_t name_size = 0;
            while(is_letter(peekc()) || is_digit(peekc()))
            {
                getc();
                name_size++;
            }
            if(!got_minus && !got_plus)
            {
                if(match_buffer_with_string(name, name_size, "null"))
                {
                    token_value = nullptr;
                    token_type = json::Token_type::null_literal;
                    return;
                }
                if(match_buffer_with_string(name, name_size, "false"))
                {
                    token_value = false;
                    token_type = json::Token_type::false_literal;
                    return;
                }
                if(match_buffer_with_string(name, name_size, "true"))
                {
                    token_value = true;
                    token_type = json::Token_type::true_literal;
                    return;
                }
            }
            if(options.allow_infinity_and_nan)
            {
                if(match_buffer_with_string(name, name_size, "NaN")
                   || match_buffer_with_string(name, name_size, "nan")
                   || match_buffer_with_string(name, name_size, "NAN"))
                {
                    token_value = std::numeric_limits<double>::quiet_NaN();
                    token_type = json::Token_type::number;
                    return;
                }
                if(match_buffer_with_string(name, name_size, "Infinity")
                   || match_buffer_with_string(name, name_size, "INFINITY")
                   || match_buffer_with_string(name, name_size, "infinity")
                   || match_buffer_with_string(name, name_size, "inf")
                   || match_buffer_with_string(name, name_size, "INF"))
                {
                    token_value = got_minus ? -std::numeric_limits<double>::infinity() :
                                              std::numeric_limits<double>::infinity();
                    token_type = json::Token_type::number;
                    return;
                }
            }
            throw Parse_error(token_location,
                              (got_minus || got_plus ? "invalid number: " : "invalid identifier: ")
                                  + std::string(name, name_size));
        }
        if(got_minus || got_plus || is_digit(peekc())
           || (options.allow_number_to_start_with_dot && peekc() == '.'))
        {
            auto mantissa = util::soft_float::ExtendedFloat::Zero();
            bool got_any_digit = false;
            if(is_digit(peekc()))
            {
                if(peekc() == '0')
                {
                    getc();
                    got_any_digit = true;
                    if(is_digit(peekc()))
                        throw Parse_error(Location(source, input_char_index),
                                          "extra leading zero not allowed in numbers");
                }
                else
                {
                    while(is_digit(peekc()))
                    {
                        std::int64_t digit = get_digit_value(getc());
                        got_any_digit = true;
                        mantissa *= util::soft_float::ExtendedFloat(static_cast<std::uint64_t>(10));
                        mantissa += util::soft_float::ExtendedFloat(digit);
                    }
                }
            }
            std::int64_t exponent_offset = 0;
            if(peekc() == '.')
            {
                getc();
                while(is_digit(peekc()))
                {
                    std::int64_t digit = get_digit_value(getc());
                    got_any_digit = true;
                    mantissa *= util::soft_float::ExtendedFloat(static_cast<std::uint64_t>(10));
                    exponent_offset--;
                    mantissa += util::soft_float::ExtendedFloat(digit);
                }
            }
            if(!got_any_digit)
                throw Parse_error(Location(source, input_char_index), "missing digit");
            std::int64_t exponent = 0;
            if(peekc() == 'e' || peekc() == 'E')
            {
                getc();
                bool exponent_is_negative = false;
                if(peekc() == '-')
                {
                    exponent_is_negative = true;
                    getc();
                }
                else if(peekc() == '+')
                {
                    getc();
                }
                if(!is_digit(peekc()))
                    throw Parse_error(Location(source, input_char_index), "missing digit");
                while(is_digit(peekc()))
                {
                    exponent *= 10;
                    exponent += get_digit_value(getc());
                }
                if(exponent_is_negative)
                    exponent = -exponent;
            }
            exponent += exponent_offset;
            auto value =
                mantissa
                * pow(util::soft_float::ExtendedFloat(static_cast<std::uint64_t>(10)), exponent);
            token_type = json::Token_type::number;
            token_value = static_cast<double>(value);
            return;
        }
        if(peekc() == '\"' || (options.allow_single_quote_strings && peekc() == '\''))
        {
            int quote = getc();
            std::string value;
            while(true)
            {
                if(peekc() == eof || is_control_character(peekc()))
                    throw Parse_error(token_location, "string missing closing quote");
                if(peekc() == quote)
                {
                    getc();
                    break;
                }
                if(peekc() == '\\')
                {
                    auto escape_location = Location(source, input_char_index);
                    getc();
                    switch(peekc())
                    {
                    case '\"':
                    case '\\':
                    case '/':
                        value += getc();
                        break;
                    case 'b':
                        value += '\b';
                        getc();
                        break;
                    case 'f':
                        value += '\f';
                        getc();
                        break;
                    case 'n':
                        value += '\n';
                        getc();
                        break;
                    case 'r':
                        value += '\r';
                        getc();
                        break;
                    case 't':
                        value += '\t';
                        getc();
                        break;
                    case 'u':
                    {
                        getc();
                        std::uint32_t ch = parse_4_hex_digits();
                        if(ch >= 0xD800U && ch < 0xDC00U && peekc() == '\\')
                        {
                            escape_location = Location(source, input_char_index);
                            getc();
                            if(peekc() == 'u')
                            {
                                getc();
                                std::uint32_t ch2 = parse_4_hex_digits();
                                if(ch2 >= 0xDC00U && ch2 < 0xE000U)
                                {
                                    // got surrogate pair
                                    ch = ((ch & 0x3FFU) >> 10) + (ch2 & 0x3FFU) + 0x10000UL;
                                }
                                else
                                {
                                    input_char_index = escape_location.char_index;
                                }
                            }
                            else
                            {
                                input_char_index = escape_location.char_index;
                            }
                        }
                        value = append_utf8(std::move(value), ch);
                        break;
                    }
                    default:
                        if(options.allow_single_quote_strings && peekc() == '\'')
                        {
                            value += getc();
                            break;
                        }
                        throw Parse_error(escape_location, "invalid escape sequence");
                    }
                }
                else
                {
                    value += getc();
                }
            }
            token_type = json::Token_type::string;
            token_value = std::move(value);
            return;
        }
        switch(peekc())
        {
        case eof:
            token_type = json::Token_type::eof;
            token_value = nullptr;
            return;
        case '[':
            getc();
            token_type = json::Token_type::l_bracket;
            token_value = nullptr;
            return;
        case ']':
            getc();
            token_type = json::Token_type::r_bracket;
            token_value = nullptr;
            return;
        case '{':
            getc();
            token_type = json::Token_type::l_brace;
            token_value = nullptr;
            return;
        case '}':
            getc();
            token_type = json::Token_type::r_brace;
            token_value = nullptr;
            return;
        case ':':
            getc();
            token_type = json::Token_type::colon;
            token_value = nullptr;
            return;
        case ',':
            getc();
            token_type = json::Token_type::comma;
            token_value = nullptr;
            return;
        }
        throw Parse_error(token_location, "invalid character");
    }
};

ast::Value parse_value(Tokenizer &tokenizer)
{
    switch(tokenizer.token_type)
    {
    case Token_type::eof:
        throw Parse_error(tokenizer.token_location, "missing value");
    case Token_type::number:
    case Token_type::string:
    case Token_type::true_literal:
    case Token_type::false_literal:
    case Token_type::null_literal:
        return tokenizer.get();
    case Token_type::l_bracket:
    {
        std::vector<ast::Value> values;
        tokenizer.next();
        if(tokenizer.token_type == Token_type::r_bracket)
        {
            tokenizer.next();
        }
        else
        {
            while(true)
            {
                values.push_back(parse_value(tokenizer));
                if(tokenizer.token_type == Token_type::comma)
                {
                    tokenizer.next();
                    continue;
                }
                if(tokenizer.token_type == Token_type::r_bracket)
                {
                    tokenizer.next();
                    break;
                }
                throw Parse_error(tokenizer.token_location, "missing , or ]");
            }
        }
        return ast::Array(std::move(values));
    }
    case Token_type::l_brace:
    {
        std::unordered_map<std::string, ast::Value> values;
        tokenizer.next();
        if(tokenizer.token_type == Token_type::r_brace)
        {
            tokenizer.next();
        }
        else
        {
            while(true)
            {
                if(tokenizer.token_type != Token_type::string)
                    throw Parse_error(tokenizer.token_location, "missing string");
                auto string_value = std::move(util::get<ast::String_value>(tokenizer.get()).value);
                if(tokenizer.token_type != Token_type::colon)
                    throw Parse_error(tokenizer.token_location, "missing ':'");
                tokenizer.next();
                values.emplace(std::move(string_value), parse_value(tokenizer));
                if(tokenizer.token_type == Token_type::comma)
                {
                    tokenizer.next();
                    continue;
                }
                if(tokenizer.token_type == Token_type::r_brace)
                {
                    tokenizer.next();
                    break;
                }
                throw Parse_error(tokenizer.token_location, "missing ',' or '}'");
            }
        }
        return ast::Object(std::move(values));
    }
    default:
        break;
    }
    throw Parse_error(tokenizer.token_location, "token not allowed here");
}
}

ast::Value parse(const Source *source, Parse_options options)
{
    Tokenizer tokenizer(source, options);
    auto retval = parse_value(tokenizer);
    if(tokenizer.token_type != Token_type::eof)
        throw Parse_error(tokenizer.token_location, "unexpected token");
    return retval;
}
}
}
