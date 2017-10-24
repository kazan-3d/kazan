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
#include "parser_callbacks_capabilities.h"

namespace kazan
{
namespace spirv_to_llvm
{
namespace parser_callbacks
{
void Capabilities_callbacks::handle_instruction_op_capability(spirv::Op_capability instruction,
                                                              std::size_t instruction_start_index)
{
    using spirv::Capability;
    util::Enum_set<Capability> work_list{instruction.capability};
    while(!work_list.empty())
    {
        auto capability = *work_list.begin();
        work_list.erase(capability);
        if(std::get<1>(enabled_capabilities.insert(capability)))
        {
            auto additional_capabilities = get_directly_required_capabilities(capability);
            work_list.insert(additional_capabilities.begin(), additional_capabilities.end());
        }
    }
    constexpr util::Enum_set<Capability> implemented_capabilities{
        Capability::matrix,
        Capability::shader,
        Capability::float64,
        Capability::int64,
        Capability::int16,
        Capability::input_attachment,
        Capability::sampled1d,
        Capability::image1d,
        Capability::sampled_buffer,
        Capability::image_buffer,
        Capability::image_query,
        Capability::derivative_control,
    };
    for(auto capability : enabled_capabilities)
    {
        if(implemented_capabilities.count(capability) == 0)
            throw spirv::Parser_error(
                instruction_start_index,
                instruction_start_index,
                "capability not implemented: " + std::string(get_enumerant_name(capability)));
    }
}
}
}
}
