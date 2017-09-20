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
#include "vulkan/remove_xlib_macros.h"
#include "spirv/spirv.h"
#include "util/enum.h"

namespace kazan
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

constexpr VkComponentMapping normalize_component_mapping(
    VkComponentMapping component_mapping) noexcept
{
    if(component_mapping.r == VK_COMPONENT_SWIZZLE_IDENTITY)
        component_mapping.r = VK_COMPONENT_SWIZZLE_R;
    if(component_mapping.g == VK_COMPONENT_SWIZZLE_IDENTITY)
        component_mapping.g = VK_COMPONENT_SWIZZLE_G;
    if(component_mapping.b == VK_COMPONENT_SWIZZLE_IDENTITY)
        component_mapping.b = VK_COMPONENT_SWIZZLE_B;
    if(component_mapping.a == VK_COMPONENT_SWIZZLE_IDENTITY)
        component_mapping.a = VK_COMPONENT_SWIZZLE_A;
    return component_mapping;
}

constexpr bool is_identity_component_mapping(const VkComponentMapping &component_mapping) noexcept
{
    auto normalized = normalize_component_mapping(component_mapping);
    if(normalized.r != VK_COMPONENT_SWIZZLE_R)
        return false;
    if(normalized.g != VK_COMPONENT_SWIZZLE_G)
        return false;
    if(normalized.b != VK_COMPONENT_SWIZZLE_B)
        return false;
    if(normalized.a != VK_COMPONENT_SWIZZLE_A)
        return false;
    return true;
}
}
}

#endif // VULKAN_UTIL_H_
