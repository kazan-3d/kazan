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
#ifndef VULKAN_UTIL_H_
#define VULKAN_UTIL_H_

#include "vulkan/vulkan.h"
#include "spirv/spirv.h"
#include "util/enum.h"

namespace vulkan_cpu
{
namespace vulkan
{
constexpr util::Enum_set<spirv::Execution_model> get_execution_models_from_shader_stage_flags(
    VkShaderStageFlags stages) noexcept
{
    util::Enum_set<spirv::Execution_model> retval;
    if(stages & VK_SHADER_STAGE_COMPUTE_BIT)
        retval.insert(spirv::Execution_model::gl_compute);
    if(stages & VK_SHADER_STAGE_FRAGMENT_BIT)
        retval.insert(spirv::Execution_model::fragment);
    if(stages & VK_SHADER_STAGE_GEOMETRY_BIT)
        retval.insert(spirv::Execution_model::geometry);
    if(stages & VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT)
        retval.insert(spirv::Execution_model::tessellation_control);
    if(stages & VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT)
        retval.insert(spirv::Execution_model::tessellation_evaluation);
    if(stages & VK_SHADER_STAGE_VERTEX_BIT)
        retval.insert(spirv::Execution_model::vertex);
    return retval;
}
}
}

#endif // VULKAN_UTIL_H_
