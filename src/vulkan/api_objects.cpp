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
#include "api_objects.h"
#include "util/optional.h"
#include <iostream>

namespace vulkan_cpu
{
namespace vulkan
{
util::variant<std::unique_ptr<Vulkan_instance>, VkResult> Vulkan_instance::create(
    const VkInstanceCreateInfo &create_info)
{
    assert(create_info.sType == VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO);
    assert(create_info.enabledLayerCount == 0); // we don't support layers here
    Supported_extensions extensions;
    for(std::size_t i = 0; i < create_info.enabledExtensionCount; i++)
    {
        auto extension = parse_extension_name(create_info.ppEnabledExtensionNames[i]);
        if(extension == Supported_extension::Not_supported)
            return VK_ERROR_EXTENSION_NOT_PRESENT;
        if(!std::get<1>(extensions.insert(extension)))
        {
            std::cerr << "Warning: duplicate extension passed to vkCreateInstance: "
                      << create_info.ppEnabledExtensionNames[i] << std::endl;
        }
    }
    for(auto extension : extensions)
    {
        for(auto dependency : get_extension_dependencies(extension))
        {
            if(extensions.count(dependency) == 0)
            {
                std::cerr << "Error: vkCreateInstance: enabled extension "
                          << get_extension_name(extension) << " depends on extension "
                          << get_extension_name(dependency) << ", however "
                          << get_extension_name(dependency) << " is not enabled" << std::endl;
                return VK_ERROR_INITIALIZATION_FAILED;
            }
        }
    }
    util::optional<App_info> app_info;
    if(create_info.pApplicationInfo)
    {
        assert(create_info.pApplicationInfo->sType == VK_STRUCTURE_TYPE_APPLICATION_INFO);
        if(create_info.pApplicationInfo->apiVersion != 0
           && (VK_VERSION_MAJOR(create_info.pApplicationInfo->apiVersion) != 1
               || VK_VERSION_MINOR(create_info.pApplicationInfo->apiVersion) != 0))
        {
            return VK_ERROR_INCOMPATIBLE_DRIVER;
        }
        app_info.emplace(*create_info.pApplicationInfo);
    }
    else
    {
        app_info.emplace();
    }
    return std::make_unique<Vulkan_instance>(std::move(*app_info), std::move(extensions));
}
}
}
