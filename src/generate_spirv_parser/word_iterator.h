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

#ifndef GENERATE_SPIRV_PARSER_WORD_ITERATOR_H_
#define GENERATE_SPIRV_PARSER_WORD_ITERATOR_H_

#include "util/string_view.h"
#include "util/optional.h"
#include <iterator>

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
namespace generate
{
class Word_iterator
{
public:
    typedef std::ptrdiff_t difference_type;
    typedef util::string_view value_type;
    typedef const util::string_view &reference;
    typedef const util::string_view *pointer;
    typedef std::input_iterator_tag iterator_category;

private:
    enum class Char_class
    {
        uppercase,
        number,
        other_identifier,
        word_separator
    };
    static constexpr Char_class get_char_class(char ch) noexcept
    {
        if(ch >= 'A' && ch <= 'Z')
            return Char_class::uppercase;
        if(ch >= 'a' && ch <= 'z')
            return Char_class::other_identifier;
        if(ch >= '0' && ch <= '9')
            return Char_class::number;
        return Char_class::word_separator;
    }

private:
    util::string_view word;
    util::string_view words;

private:
    constexpr void next() noexcept
    {
        util::optional<std::size_t> word_start;
        Char_class last_char_class = Char_class::word_separator;
        for(std::size_t i = 0; i < words.size(); i++)
        {
            auto current_char_class = get_char_class(words[i]);
            if(word_start)
            {
                switch(current_char_class)
                {
                case Char_class::word_separator:
                    word = util::string_view(words.data() + *word_start, i - *word_start);
                    words.remove_prefix(i);
                    return;
                case Char_class::uppercase:
                    if(last_char_class != Char_class::uppercase
                       && last_char_class != Char_class::number)
                    {
                        word = util::string_view(words.data() + *word_start, i - *word_start);
                        words.remove_prefix(i);
                        return;
                    }
                    if(i + 1 < words.size()
                       && get_char_class(words[i + 1]) == Char_class::other_identifier)
                    {
                        word = util::string_view(words.data() + *word_start, i - *word_start);
                        words.remove_prefix(i);
                        return;
                    }
                    break;
                case Char_class::other_identifier:
                case Char_class::number:
                    break;
                }
            }
            else if(current_char_class != Char_class::word_separator)
            {
                word_start = i;
            }
            last_char_class = current_char_class;
        }
        if(word_start)
            word = util::string_view(words.data() + *word_start, words.size() - *word_start);
        else
            word = {};
        words = {};
    }
    constexpr bool at_end() const noexcept
    {
        return word.empty();
    }

public:
    constexpr Word_iterator() noexcept : word(), words()
    {
    }
    constexpr explicit Word_iterator(util::string_view words) noexcept : word(), words(words)
    {
        next();
    }
    constexpr const util::string_view &operator*() const noexcept
    {
        return word;
    }
    constexpr const util::string_view *operator->() const noexcept
    {
        return &word;
    }
    constexpr Word_iterator &operator++() noexcept
    {
        next();
        return *this;
    }
    constexpr Word_iterator operator++(int) noexcept
    {
        auto retval = *this;
        next();
        return retval;
    }
    constexpr bool operator==(const Word_iterator &rt) const noexcept
    {
        return word.empty() == rt.word.empty();
    }
    constexpr bool operator!=(const Word_iterator &rt) const noexcept
    {
        return word.empty() != rt.word.empty();
    }
    constexpr Word_iterator begin() const noexcept
    {
        return *this;
    }
    constexpr Word_iterator end() const noexcept
    {
        return {};
    }
};

template <std::size_t Iterator_count>
class Chained_word_iterator
{
public:
    typedef std::ptrdiff_t difference_type;
    typedef util::string_view value_type;
    typedef const util::string_view &reference;
    typedef const util::string_view *pointer;
    typedef std::input_iterator_tag iterator_category;

private:
    Word_iterator iterators[Iterator_count];
    std::size_t current_iterator_index;

private:
    template <bool... Values>
    static constexpr bool variadic_and() noexcept
    {
        for(bool v : {Values...})
            if(!v)
                return false;
        return true;
    }
    static constexpr util::string_view to_string_view_helper(util::string_view v) noexcept
    {
        return v;
    }
    constexpr bool at_end() const noexcept
    {
        return current_iterator_index == Iterator_count;
    }
    constexpr void next() noexcept
    {
        assert(current_iterator_index < Iterator_count);
        ++iterators[current_iterator_index];
        while(iterators[current_iterator_index] == Word_iterator())
            if(++current_iterator_index == Iterator_count)
                return;
    }

public:
    constexpr Chained_word_iterator() noexcept : iterators{}, current_iterator_index(Iterator_count)
    {
    }
    template <typename... Args>
    constexpr explicit Chained_word_iterator(Args &&... args) noexcept(
        variadic_and<noexcept(to_string_view_helper(std::forward<Args>(args)))...>())
        : iterators{Word_iterator(to_string_view_helper(std::forward<Args>(args)))...},
          current_iterator_index(0)
    {
    }
    constexpr const util::string_view &operator*() const noexcept
    {
        assert(current_iterator_index < Iterator_count);
        return *iterators[current_iterator_index];
    }
    constexpr const util::string_view *operator->() const noexcept
    {
        return &operator*();
    }
    constexpr Chained_word_iterator &operator++() noexcept
    {
        next();
        return *this;
    }
    constexpr Chained_word_iterator operator++(int) noexcept
    {
        auto retval = *this;
        next();
        return retval;
    }
    constexpr bool operator==(const Chained_word_iterator &rt) const noexcept
    {
        return at_end() == rt.at_end();
    }
    constexpr bool operator!=(const Chained_word_iterator &rt) const noexcept
    {
        return at_end() != rt.at_end();
    }
    constexpr Chained_word_iterator begin() const noexcept
    {
        return *this;
    }
    constexpr Chained_word_iterator end() const noexcept
    {
        return {};
    }
};

template <typename... Args>
constexpr Chained_word_iterator<sizeof...(Args)>
    make_chained_word_iterator(Args &&... args) noexcept(
        noexcept(Chained_word_iterator<sizeof...(Args)>(std::forward<Args>(args)...)))
{
    return Chained_word_iterator<sizeof...(Args)>(std::forward<Args>(args)...);
}
}
}
}

#endif /* GENERATE_SPIRV_PARSER_WORD_ITERATOR_H_ */
