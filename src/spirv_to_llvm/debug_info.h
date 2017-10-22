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
#ifndef SPIRV_TO_LLVM_DEBUG_INFO_H_
#define SPIRV_TO_LLVM_DEBUG_INFO_H_

#include <cstddef>
#include "spirv/spirv.h"
#include "vulkan/api_objects.h"
#include <string>
#include "translator.h"

namespace kazan
{
namespace spirv_to_llvm
{
struct Spirv_string final : public Spirv_id
{
    const std::string value;
    explicit Spirv_string(std::string value) noexcept : value(std::move(value))
    {
    }
};

struct Spirv_location_without_instruction_start_index
{
    const vulkan::Vulkan_shader_module *shader_module;
    const Spirv_string *filename;
    util::string_view get_filename_string() const noexcept
    {
        if(filename)
            return filename->value;
        return {};
    }
    struct Line_info
    {
        spirv::Word line = 0;
        spirv::Word column = 0;
        constexpr Line_info() noexcept
        {
        }
        constexpr Line_info(spirv::Word line, spirv::Word column) noexcept : line(line),
                                                                             column(column)
        {
        }
    };
    util::optional<Line_info> line_info;
    Spirv_location_without_instruction_start_index() noexcept : shader_module(nullptr),
                                                                filename(nullptr),
                                                                line_info()
    {
    }
    Spirv_location_without_instruction_start_index(
        const vulkan::Vulkan_shader_module *shader_module,
        const Spirv_string *filename,
        util::optional<Line_info> line_info) noexcept : shader_module(shader_module),
                                                        filename(filename),
                                                        line_info(line_info)
    {
    }
};

struct Spirv_location : public Spirv_location_without_instruction_start_index
{
    std::size_t instruction_start_index;
    Spirv_location() noexcept : Spirv_location_without_instruction_start_index(),
                                instruction_start_index(0)
    {
    }
    Spirv_location(Spirv_location_without_instruction_start_index location,
                   std::size_t instruction_start_index) noexcept
        : Spirv_location_without_instruction_start_index(location),
          instruction_start_index(instruction_start_index)
    {
    }
};

class Parser_debug_callbacks : public virtual Parser_callbacks_base
{
private:
    const Spirv_string *source_filename = nullptr;
    Spirv_location_without_instruction_start_index current_location;

protected:
    virtual void clear_line_info_because_end_of_block() override final;
    virtual Spirv_location get_location(std::size_t instruction_start_index) const
        noexcept override final;

public:
    virtual void handle_instruction_op_source_continued(
        spirv::Op_source_continued instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_source(spirv::Op_source instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_source_extension(
        spirv::Op_source_extension instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_name(spirv::Op_name instruction,
                                            std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_member_name(spirv::Op_member_name instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_string(spirv::Op_string instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_line(spirv::Op_line instruction,
                                            std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_no_line(spirv::Op_no_line instruction,
                                               std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_module_processed(
        spirv::Op_module_processed instruction, std::size_t instruction_start_index) override;
};
}
}

#endif // SPIRV_TO_LLVM_DEBUG_INFO_H_
