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

#ifndef JSON_LOCATION_H_
#define JSON_LOCATION_H_

#include "source.h"
#include <string>
#include <iosfwd>

namespace kazan
{
namespace json
{
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
}
}

#endif /* JSON_LOCATION_H_ */
