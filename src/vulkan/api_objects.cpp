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

namespace kazan
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
        {
            std::cerr << "Error: unsupported extension passed to vkCreateInstance: "
                      << create_info.ppEnabledExtensionNames[i] << std::endl;
            return VK_ERROR_EXTENSION_NOT_PRESENT;
        }
        if(get_extension_scope(extension) != Extension_scope::Instance)
        {
            std::cerr << "Error: device extension passed to vkCreateInstance: "
                      << create_info.ppEnabledExtensionNames[i] << std::endl;
            return VK_ERROR_EXTENSION_NOT_PRESENT;
        }
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
                          << get_extension_properties(extension).extensionName
                          << " depends on extension "
                          << get_extension_properties(dependency).extensionName << ", however "
                          << get_extension_properties(dependency).extensionName << " is not enabled"
                          << std::endl;
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

util::variant<std::unique_ptr<Vulkan_device>, VkResult> Vulkan_device::create(
    Vulkan_physical_device &physical_device, const VkDeviceCreateInfo &create_info)
{
    assert(create_info.sType == VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO);
    Supported_extensions extensions;
    Supported_extensions all_extensions = physical_device.instance.extensions;
    for(std::size_t i = 0; i < create_info.enabledExtensionCount; i++)
    {
        auto extension = parse_extension_name(create_info.ppEnabledExtensionNames[i]);
        if(extension == Supported_extension::Not_supported)
        {
            std::cerr << "Error: unsupported extension passed to vkCreateDevice: "
                      << create_info.ppEnabledExtensionNames[i] << std::endl;
            return VK_ERROR_EXTENSION_NOT_PRESENT;
        }
        if(get_extension_scope(extension) != Extension_scope::Device)
        {
            std::cerr << "Error: instance extension passed to vkCreateDevice: "
                      << create_info.ppEnabledExtensionNames[i] << std::endl;
            return VK_ERROR_EXTENSION_NOT_PRESENT;
        }
        if(!std::get<1>(extensions.insert(extension)))
        {
            std::cerr << "Warning: duplicate extension passed to vkCreateDevice: "
                      << create_info.ppEnabledExtensionNames[i] << std::endl;
        }
        all_extensions.insert(extension);
    }
    for(auto extension : extensions)
    {
        for(auto dependency : get_extension_dependencies(extension))
        {
            if(all_extensions.count(dependency) == 0)
            {
                std::cerr << "Error: vkCreateDevice: enabled extension "
                          << get_extension_properties(extension).extensionName
                          << " depends on extension "
                          << get_extension_properties(dependency).extensionName << ", however "
                          << get_extension_properties(dependency).extensionName << " is not enabled"
                          << std::endl;
                return VK_ERROR_INITIALIZATION_FAILED;
            }
        }
    }
    // add enabled instance extensions
    for(auto extension : physical_device.instance.extensions)
        extensions.insert(extension);
    VkPhysicalDeviceFeatures enabled_features = {};
    if(create_info.pEnabledFeatures)
        enabled_features = *create_info.pEnabledFeatures;
    struct Feature_descriptor
    {
        VkBool32 VkPhysicalDeviceFeatures::*member;
        const char *name;
    };
    static constexpr std::initializer_list<Feature_descriptor> features = {
        {
            .member = &VkPhysicalDeviceFeatures::robustBufferAccess, .name = "robustBufferAccess",
        },
        {
            .member = &VkPhysicalDeviceFeatures::fullDrawIndexUint32, .name = "fullDrawIndexUint32",
        },
        {
            .member = &VkPhysicalDeviceFeatures::imageCubeArray, .name = "imageCubeArray",
        },
        {
            .member = &VkPhysicalDeviceFeatures::independentBlend, .name = "independentBlend",
        },
        {
            .member = &VkPhysicalDeviceFeatures::geometryShader, .name = "geometryShader",
        },
        {
            .member = &VkPhysicalDeviceFeatures::tessellationShader, .name = "tessellationShader",
        },
        {
            .member = &VkPhysicalDeviceFeatures::sampleRateShading, .name = "sampleRateShading",
        },
        {
            .member = &VkPhysicalDeviceFeatures::dualSrcBlend, .name = "dualSrcBlend",
        },
        {
            .member = &VkPhysicalDeviceFeatures::logicOp, .name = "logicOp",
        },
        {
            .member = &VkPhysicalDeviceFeatures::multiDrawIndirect, .name = "multiDrawIndirect",
        },
        {
            .member = &VkPhysicalDeviceFeatures::drawIndirectFirstInstance,
            .name = "drawIndirectFirstInstance",
        },
        {
            .member = &VkPhysicalDeviceFeatures::depthClamp, .name = "depthClamp",
        },
        {
            .member = &VkPhysicalDeviceFeatures::depthBiasClamp, .name = "depthBiasClamp",
        },
        {
            .member = &VkPhysicalDeviceFeatures::fillModeNonSolid, .name = "fillModeNonSolid",
        },
        {
            .member = &VkPhysicalDeviceFeatures::depthBounds, .name = "depthBounds",
        },
        {
            .member = &VkPhysicalDeviceFeatures::wideLines, .name = "wideLines",
        },
        {
            .member = &VkPhysicalDeviceFeatures::largePoints, .name = "largePoints",
        },
        {
            .member = &VkPhysicalDeviceFeatures::alphaToOne, .name = "alphaToOne",
        },
        {
            .member = &VkPhysicalDeviceFeatures::multiViewport, .name = "multiViewport",
        },
        {
            .member = &VkPhysicalDeviceFeatures::samplerAnisotropy, .name = "samplerAnisotropy",
        },
        {
            .member = &VkPhysicalDeviceFeatures::textureCompressionETC2,
            .name = "textureCompressionETC2",
        },
        {
            .member = &VkPhysicalDeviceFeatures::textureCompressionASTC_LDR,
            .name = "textureCompressionASTC_LDR",
        },
        {
            .member = &VkPhysicalDeviceFeatures::textureCompressionBC,
            .name = "textureCompressionBC",
        },
        {
            .member = &VkPhysicalDeviceFeatures::occlusionQueryPrecise,
            .name = "occlusionQueryPrecise",
        },
        {
            .member = &VkPhysicalDeviceFeatures::pipelineStatisticsQuery,
            .name = "pipelineStatisticsQuery",
        },
        {
            .member = &VkPhysicalDeviceFeatures::vertexPipelineStoresAndAtomics,
            .name = "vertexPipelineStoresAndAtomics",
        },
        {
            .member = &VkPhysicalDeviceFeatures::fragmentStoresAndAtomics,
            .name = "fragmentStoresAndAtomics",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderTessellationAndGeometryPointSize,
            .name = "shaderTessellationAndGeometryPointSize",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderImageGatherExtended,
            .name = "shaderImageGatherExtended",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderStorageImageExtendedFormats,
            .name = "shaderStorageImageExtendedFormats",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderStorageImageMultisample,
            .name = "shaderStorageImageMultisample",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderStorageImageReadWithoutFormat,
            .name = "shaderStorageImageReadWithoutFormat",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderStorageImageWriteWithoutFormat,
            .name = "shaderStorageImageWriteWithoutFormat",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderUniformBufferArrayDynamicIndexing,
            .name = "shaderUniformBufferArrayDynamicIndexing",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderSampledImageArrayDynamicIndexing,
            .name = "shaderSampledImageArrayDynamicIndexing",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderStorageBufferArrayDynamicIndexing,
            .name = "shaderStorageBufferArrayDynamicIndexing",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderStorageImageArrayDynamicIndexing,
            .name = "shaderStorageImageArrayDynamicIndexing",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderClipDistance, .name = "shaderClipDistance",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderCullDistance, .name = "shaderCullDistance",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderFloat64, .name = "shaderFloat64",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderInt64, .name = "shaderInt64",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderInt16, .name = "shaderInt16",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderResourceResidency,
            .name = "shaderResourceResidency",
        },
        {
            .member = &VkPhysicalDeviceFeatures::shaderResourceMinLod,
            .name = "shaderResourceMinLod",
        },
        {
            .member = &VkPhysicalDeviceFeatures::sparseBinding, .name = "sparseBinding",
        },
        {
            .member = &VkPhysicalDeviceFeatures::sparseResidencyBuffer,
            .name = "sparseResidencyBuffer",
        },
        {
            .member = &VkPhysicalDeviceFeatures::sparseResidencyImage2D,
            .name = "sparseResidencyImage2D",
        },
        {
            .member = &VkPhysicalDeviceFeatures::sparseResidencyImage3D,
            .name = "sparseResidencyImage3D",
        },
        {
            .member = &VkPhysicalDeviceFeatures::sparseResidency2Samples,
            .name = "sparseResidency2Samples",
        },
        {
            .member = &VkPhysicalDeviceFeatures::sparseResidency4Samples,
            .name = "sparseResidency4Samples",
        },
        {
            .member = &VkPhysicalDeviceFeatures::sparseResidency8Samples,
            .name = "sparseResidency8Samples",
        },
        {
            .member = &VkPhysicalDeviceFeatures::sparseResidency16Samples,
            .name = "sparseResidency16Samples",
        },
        {
            .member = &VkPhysicalDeviceFeatures::sparseResidencyAliased,
            .name = "sparseResidencyAliased",
        },
        {
            .member = &VkPhysicalDeviceFeatures::variableMultisampleRate,
            .name = "variableMultisampleRate",
        },
        {
            .member = &VkPhysicalDeviceFeatures::inheritedQueries, .name = "inheritedQueries",
        },
    };
    for(auto &feature : features)
    {
        if(enabled_features.*feature.member && !(physical_device.features.*feature.member))
        {
            std::cerr << "Error: vkCreateDevice: feature not supported: " << feature.name
                      << std::endl;
            return VK_ERROR_FEATURE_NOT_PRESENT;
        }
    }
    assert(create_info.queueCreateInfoCount == 1);
    assert(create_info.pQueueCreateInfos);
    assert(create_info.pQueueCreateInfos[0].sType == VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO);
    assert(create_info.pQueueCreateInfos[0].queueFamilyIndex == 0);
    assert(create_info.pQueueCreateInfos[0].queueCount == 1);
    return std::make_unique<Vulkan_device>(physical_device, enabled_features, extensions);
}
}
}
