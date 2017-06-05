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
#include "../util/optional.h"

namespace vulkan_cpu
{
namespace json
{
struct Source
{
    std::string file_name;
    std::shared_ptr<const char> contents; // use a shared_ptr so you can use mmap-ed memory
    std::size_t contents_size;
    /** doesn't have first line to save memory */
    std::vector<std::size_t> line_start_indexes;
    static std::vector<std::size_t> find_line_start_indexes(const char *contents,
                                                            std::size_t contents_size);
    Source(Source &&) = default;
    Source(const Source &) = delete;
    Source &operator=(Source &&) = default;
    Source &operator=(const Source &) = delete;
    Source() : file_name(), contents(), contents_size(0), line_start_indexes()
    {
    }
    explicit Source(std::string file_name) noexcept : file_name(std::move(file_name)),
                                                      contents(),
                                                      contents_size(0),
                                                      line_start_indexes()
    {
    }
    Source(std::string file_name,
           std::shared_ptr<const char> contents,
           std::size_t contents_size) noexcept
        : file_name(std::move(file_name)),
          contents(std::move(contents)),
          contents_size(contents_size),
          line_start_indexes(find_line_start_indexes(this->contents.get(), contents_size))
    {
    }
    Source(std::string file_name, std::string contents_in)
        : file_name(file_name),
          contents(),
          contents_size(contents_in.size()),
          line_start_indexes(find_line_start_indexes(contents_in.data(), contents_size))
    {
        auto str = std::make_shared<std::string>(std::move(contents_in));
        contents = std::shared_ptr<const char>(str, str->data());
    }
    Source(std::string file_name, std::vector<char> contents_in)
        : file_name(file_name),
          contents(),
          contents_size(contents_in.size()),
          line_start_indexes(find_line_start_indexes(contents_in.data(), contents_size))
    {
        auto str = std::make_shared<std::vector<char>>(std::move(contents_in));
        contents = std::shared_ptr<const char>(str, str->data());
    }
    Source(std::string file_name, std::vector<unsigned char> contents_in)
        : file_name(file_name),
          contents(),
          contents_size(contents_in.size()),
          line_start_indexes(find_line_start_indexes(
              reinterpret_cast<const char *>(contents_in.data()), contents_size))
    {
        auto str = std::make_shared<std::vector<unsigned char>>(std::move(contents_in));
        contents = std::shared_ptr<const char>(str, reinterpret_cast<const char *>(str->data()));
    }
    explicit operator bool() const noexcept
    {
        return contents != nullptr;
    }
    static Source load_file(std::string file_name);
    static Source load_stdin();
    struct Line_and_index
    {
        std::size_t line;
        std::size_t index;
        constexpr Line_and_index() noexcept : line(), index()
        {
        }
        constexpr Line_and_index(std::size_t line, std::size_t index) noexcept : line(line),
                                                                                 index(index)
        {
        }
    };
    struct Line_and_column
    {
        std::size_t line;
        std::size_t column;
        constexpr Line_and_column() noexcept : line(), column()
        {
        }
        constexpr Line_and_column(std::size_t line, std::size_t column) noexcept : line(line),
                                                                                   column(column)
        {
        }
        std::string append_to_string(std::string buffer) const
        {
            buffer = ast::Number_value::append_unsigned_integer_to_string(line, std::move(buffer));
            buffer += ':';
            buffer =
                ast::Number_value::append_unsigned_integer_to_string(column, std::move(buffer));
            return buffer;
        }
        std::string to_string(std::string buffer = {}) const
        {
            buffer.clear();
            return append_to_string(std::move(buffer));
        }
        friend std::ostream &operator<<(std::ostream &os, const Line_and_column &v);
    };
    static constexpr std::size_t default_tab_size = 8;
    Line_and_index get_line_and_start_index(std::size_t char_index) const noexcept;
    Line_and_column get_line_and_column(std::size_t char_index,
                                        std::size_t tab_size = default_tab_size) const noexcept;
};

struct Location
{
    const Source *source;
    std::size_t char_index;
    constexpr Location() noexcept : source(nullptr), char_index(0)
    {
    }
    constexpr Location(const json::Source *source, std::size_t char_index) noexcept
        : source(source),
          char_index(char_index)
    {
    }
    json::Source::Line_and_index get_line_and_start_index() const noexcept
    {
        if(!source)
            return {};
        return source->get_line_and_start_index(char_index);
    }
    json::Source::Line_and_column get_line_and_column(
        std::size_t tab_size = json::Source::default_tab_size) const noexcept
    {
        if(!source)
            return {};
        return source->get_line_and_column(char_index, tab_size);
    }
    std::string to_string(std::string buffer = {},
                          std::size_t tab_size = json::Source::default_tab_size) const
    {
        buffer.clear();
        return append_to_string(std::move(buffer));
    }
    std::string append_to_string(std::string buffer,
                                 std::size_t tab_size = json::Source::default_tab_size) const
    {
        if(!source || source->file_name.empty())
            buffer += "<unknown>";
        else
            buffer += source->file_name;
        buffer += ':';
        buffer = get_line_and_column(tab_size).append_to_string(std::move(buffer));
        return buffer;
    }
    friend std::ostream &operator<<(std::ostream &os, const Location &v);
};

class Parse_error : public std::runtime_error
{
public:
    Location location;
    Parse_error(json::Location location, const std::string &message)
        : runtime_error(location.to_string() + ": " + message)
    {
    }
    Parse_error(json::Location location, const char *message)
        : runtime_error(location.to_string() + ": " + message)
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