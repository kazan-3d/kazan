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
#ifndef GENERATE_SPIRV_PARSER_INSTRUCTION_PROPERTIES_H_
#define GENERATE_SPIRV_PARSER_INSTRUCTION_PROPERTIES_H_

#include "util/string_view.h"
#include <initializer_list>
#include <cstdint>
#include <cassert>

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
struct Instruction_properties_descriptor
{
    struct Operand_descriptor
    {
        enum class Integer_literal_size : std::uint_least8_t // to save space
        {
            not_implemented,
            always_32bits,
            always_64bits,
            matches_type_of_operand_0,
        };
        Integer_literal_size integer_literal_size;
        constexpr Operand_descriptor() noexcept
            : integer_literal_size(Integer_literal_size::not_implemented)
        {
        }
        constexpr Operand_descriptor(Integer_literal_size integer_literal_size) noexcept
            : integer_literal_size(integer_literal_size)
        {
        }
    };
    struct Operand_descriptors
    {
        static constexpr std::size_t allocated_size = 10; // increase if we run out of room
    private:
        std::size_t used_size;
        Operand_descriptor operands[allocated_size];

    public:
        constexpr Operand_descriptors(
            std::initializer_list<Operand_descriptor> initializer) noexcept
            : used_size(initializer.size()),
              operands{}
        {
            assert(initializer.size() <= allocated_size);
            for(std::size_t i = 0; i < initializer.size(); i++)
                operands[i] = initializer.begin()[i];
        }
        constexpr Operand_descriptors() noexcept : used_size(0), operands{}
        {
        }
        typedef Operand_descriptor *iterator;
        typedef const Operand_descriptor *const_iterator;
        constexpr iterator begin() noexcept
        {
            return operands;
        }
        constexpr const_iterator begin() const noexcept
        {
            return operands;
        }
        constexpr iterator end() noexcept
        {
            return operands + used_size;
        }
        constexpr const_iterator end() const noexcept
        {
            return operands + used_size;
        }
        constexpr std::size_t size() const noexcept
        {
            return used_size;
        }
        constexpr Operand_descriptor *data() noexcept
        {
            return operands;
        }
        constexpr const Operand_descriptor *data() const noexcept
        {
            return operands;
        }
    };
    util::string_view extension_instruction_set_import_name;
    util::string_view instruction_name;
    Operand_descriptors operand_descriptors;
    constexpr Instruction_properties_descriptor(
        util::string_view extension_instruction_set_import_name,
        util::string_view instruction_name,
        Operand_descriptors operand_descriptors) noexcept
        : extension_instruction_set_import_name(extension_instruction_set_import_name),
          instruction_name(instruction_name),
          operand_descriptors(operand_descriptors)
    {
    }
};

struct Instruction_properties_descriptors
{
    const Instruction_properties_descriptor *descriptors;
    std::size_t descriptor_count;
    constexpr explicit Instruction_properties_descriptors(
        const Instruction_properties_descriptor *descriptors, std::size_t descriptor_count) noexcept
        : descriptors(descriptors),
          descriptor_count(descriptor_count)
    {
    }
    typedef const Instruction_properties_descriptor *iterator;
    typedef const Instruction_properties_descriptor *const_iterator;
    constexpr const_iterator begin() const noexcept
    {
        return descriptors;
    }
    constexpr const_iterator end() const noexcept
    {
        return descriptors + descriptor_count;
    }
    static Instruction_properties_descriptors get() noexcept;
};
}
}

#endif /* GENERATE_SPIRV_PARSER_INSTRUCTION_PROPERTIES_H_ */
