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

#include "source.h"
#include "json.h"
#include <iostream>
#include <fstream>
#include <algorithm>

namespace vulkan_cpu
{
namespace json
{
std::string Source::Line_and_column::append_to_string(std::string buffer) const
{
    buffer = ast::Number_value::append_unsigned_integer_to_string(line, std::move(buffer));
    buffer += ':';
    buffer = ast::Number_value::append_unsigned_integer_to_string(column, std::move(buffer));
    return buffer;
}

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

Source Source::load_file(const util::filesystem::path &file_path)
{
    // TODO: add code to use mmap
    std::ifstream is;
    is.exceptions(std::ios::badbit);
    is.open(file_path);
    if(!is)
        throw util::filesystem::filesystem_error(
            "open failed", file_path, std::make_error_code(std::io_errc::stream));
    is.exceptions(std::ios::badbit | std::ios::failbit);
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
    return Source(file_path.string(), std::move(contents), contents_size);
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
}
}
