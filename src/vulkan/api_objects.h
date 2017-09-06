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
#ifndef VULKAN_API_OBJECTS_H_
#define VULKAN_API_OBJECTS_H_

#include "vulkan/vulkan.h"
#include "vulkan/vk_icd.h"
#include "util/enum.h"
#include "util/string_view.h"
#include "util/variant.h"
#include <memory>
#include <cassert>
#include <chrono>

namespace kazan
{
namespace vulkan
{
enum class Supported_extension
{
    Not_supported,
};

kazan_util_generate_enum_traits(Supported_extension, Supported_extension::Not_supported);

typedef util::Enum_set<Supported_extension> Supported_extensions;

enum class Extension_scope
{
    Not_supported,
    Instance,
    Device,
};

constexpr Extension_scope get_extension_scope(Supported_extension extension) noexcept
{
    switch(extension)
    {
    case Supported_extension::Not_supported:
        return Extension_scope::Not_supported;
    }
    assert(!"unknown extension");
    return Extension_scope::Not_supported;
}

constexpr util::string_view get_extension_name(Supported_extension extension) noexcept
{
    using namespace util::string_view_literals;
    switch(extension)
    {
    case Supported_extension::Not_supported:
        return ""_sv;
    }
    assert(!"unknown extension");
    return ""_sv;
}

constexpr Supported_extensions get_extension_dependencies(Supported_extension extension) noexcept
{
    switch(extension)
    {
    case Supported_extension::Not_supported:
        return {};
    }
    assert(!"unknown extension");
    return {};
}

inline Supported_extension parse_extension_name(util::string_view name) noexcept
{
    if(name.empty())
        return Supported_extension::Not_supported;
    for(auto extension : util::Enum_traits<Supported_extension>::values)
    {
        if(extension == Supported_extension::Not_supported)
            continue;
        if(get_extension_name(extension) == name)
            return extension;
    }
    return Supported_extension::Not_supported;
}

template <typename Object_type, typename Vulkan_handle_type>
struct Vulkan_dispatchable_object
{
    VK_LOADER_DATA vulkan_loader_data{.loaderMagic = ICD_LOADER_MAGIC};
    typedef Vulkan_handle_type Vulkan_handle;
    typedef Object_type Object;
    static Object_type *from_handle(Vulkan_handle_type v) noexcept
    {
        return static_cast<Object_type *>(reinterpret_cast<Vulkan_dispatchable_object *>(v));
    }
    static std::unique_ptr<Object_type> move_from_handle(Vulkan_handle_type v) noexcept
    {
        return std::unique_ptr<Object_type>(from_handle(v));
    }
};

template <typename Object_type>
typename std::
    enable_if<std::is_base_of<Vulkan_dispatchable_object<Object_type,
                                                         typename Object_type::Vulkan_handle>,
                              Object_type>::value,
              typename Object_type::Vulkan_handle>::type
    to_handle(Object_type *object) noexcept
{
    return reinterpret_cast<typename Object_type::Vulkan_handle>(
        static_cast<Vulkan_dispatchable_object<Object_type, typename Object_type::Vulkan_handle> *>(
            object));
}

template <typename Object_type>
decltype(to_handle(static_cast<Object_type *>(nullptr))) move_to_handle(
    std::unique_ptr<Object_type> v) noexcept
{
    return to_handle(v.release());
}

struct Vulkan_instance;

struct Vulkan_physical_device
    : public Vulkan_dispatchable_object<Vulkan_physical_device, VkPhysicalDevice>
{
    Vulkan_instance &instance;
    VkPhysicalDeviceProperties properties;
    Vulkan_physical_device(Vulkan_instance &instance) noexcept
        : instance(instance),
          properties{
              .apiVersion = VK_MAKE_VERSION(1, 0, VK_HEADER_VERSION),
              .driverVersion = 0,
#warning change vendorID to the correct value
              .vendorID = 0x12345678UL,
              .deviceID = 0,
              .deviceType = VK_PHYSICAL_DEVICE_TYPE_CPU,
              .deviceName = "Kazan Software Renderer",
#warning calculate the correct value of pipelineCacheUUID as the hash of the target cpu info and the hashed source code
              .pipelineCacheUUID = {},
              .limits =
                  {
                      .maxImageDimension1D = 1UL << 23,
                      .maxImageDimension2D = 1UL << 23,
                      .maxImageDimension3D = 1UL << 23,
                      .maxImageDimensionCube = 1UL << 23,
                      .maxImageArrayLayers = static_cast<std::uint32_t>(-1),
                      .maxTexelBufferElements = static_cast<std::uint32_t>(-1),
                      .maxUniformBufferRange = static_cast<std::uint32_t>(-1),
                      .maxStorageBufferRange = static_cast<std::uint32_t>(-1),
                      .maxPushConstantsSize = static_cast<std::uint32_t>(-1),
                      .maxMemoryAllocationCount = static_cast<std::uint32_t>(-1),
                      .maxSamplerAllocationCount = static_cast<std::uint32_t>(-1),
                      .bufferImageGranularity = 1,
                      .sparseAddressSpaceSize = 0,
                      .maxBoundDescriptorSets = static_cast<std::uint32_t>(-1),
                      .maxPerStageDescriptorSamplers = static_cast<std::uint32_t>(-1),
                      .maxPerStageDescriptorUniformBuffers = static_cast<std::uint32_t>(-1),
                      .maxPerStageDescriptorStorageBuffers = static_cast<std::uint32_t>(-1),
                      .maxPerStageDescriptorSampledImages = static_cast<std::uint32_t>(-1),
                      .maxPerStageDescriptorStorageImages = static_cast<std::uint32_t>(-1),
                      .maxPerStageDescriptorInputAttachments = static_cast<std::uint32_t>(-1),
                      .maxPerStageResources = static_cast<std::uint32_t>(-1),
                      .maxDescriptorSetSamplers = static_cast<std::uint32_t>(-1),
                      .maxDescriptorSetUniformBuffers = static_cast<std::uint32_t>(-1),
                      .maxDescriptorSetUniformBuffersDynamic = static_cast<std::uint32_t>(-1),
                      .maxDescriptorSetStorageBuffers = static_cast<std::uint32_t>(-1),
                      .maxDescriptorSetStorageBuffersDynamic = static_cast<std::uint32_t>(-1),
                      .maxDescriptorSetSampledImages = static_cast<std::uint32_t>(-1),
                      .maxDescriptorSetStorageImages = static_cast<std::uint32_t>(-1),
                      .maxDescriptorSetInputAttachments = static_cast<std::uint32_t>(-1),
                      .maxVertexInputAttributes = static_cast<std::uint32_t>(-1),
                      .maxVertexInputBindings = static_cast<std::uint32_t>(-1),
                      .maxVertexInputAttributeOffset = static_cast<std::uint32_t>(-1),
                      .maxVertexInputBindingStride = static_cast<std::uint32_t>(-1),
                      .maxVertexOutputComponents = static_cast<std::uint32_t>(-1),
                      .maxTessellationGenerationLevel = 0,
                      .maxTessellationPatchSize = 0,
                      .maxTessellationControlPerVertexInputComponents = 0,
                      .maxTessellationControlPerVertexOutputComponents = 0,
                      .maxTessellationControlPerPatchOutputComponents = 0,
                      .maxTessellationControlTotalOutputComponents = 0,
                      .maxTessellationEvaluationInputComponents = 0,
                      .maxTessellationEvaluationOutputComponents = 0,
                      .maxGeometryShaderInvocations = 0,
                      .maxGeometryInputComponents = 0,
                      .maxGeometryOutputComponents = 0,
                      .maxGeometryOutputVertices = 0,
                      .maxGeometryTotalOutputComponents = 0,
                      .maxFragmentInputComponents = static_cast<std::uint32_t>(-1),
                      .maxFragmentOutputAttachments = static_cast<std::uint32_t>(-1),
                      .maxFragmentDualSrcAttachments = 0,
                      .maxFragmentCombinedOutputResources = static_cast<std::uint32_t>(-1),
                      .maxComputeSharedMemorySize = static_cast<std::uint32_t>(-1),
                      .maxComputeWorkGroupCount =
                          {
                              static_cast<std::uint32_t>(-1),
                              static_cast<std::uint32_t>(-1),
                              static_cast<std::uint32_t>(-1),
                          },
                      .maxComputeWorkGroupInvocations = static_cast<std::uint32_t>(-1),
                      .maxComputeWorkGroupSize =
                          {
                              static_cast<std::uint32_t>(-1),
                              static_cast<std::uint32_t>(-1),
                              static_cast<std::uint32_t>(-1),
                          },
                      .subPixelPrecisionBits = 16,
                      .subTexelPrecisionBits = 8,
                      .mipmapPrecisionBits = 8,
                      .maxDrawIndexedIndexValue = static_cast<std::uint32_t>(-1),
                      .maxDrawIndirectCount = static_cast<std::uint32_t>(-1),
                      .maxSamplerLodBias = 65536.0f,
                      .maxSamplerAnisotropy = 1,
                      .maxViewports = 1,
                      .maxViewportDimensions =
                          {
                              1UL << 23, 1UL << 23,
                          },
                      .viewportBoundsRange = {-1.0f * (1UL << 23), 1UL << 23},
                      .viewportSubPixelBits = 16,
                      .minMemoryMapAlignment = 64,
                      .minTexelBufferOffsetAlignment = alignof(std::max_align_t),
                      .minUniformBufferOffsetAlignment = alignof(std::max_align_t),
                      .minStorageBufferOffsetAlignment = alignof(std::max_align_t),
                      .minTexelOffset = std::numeric_limits<std::int32_t>::min(),
                      .maxTexelOffset = std::numeric_limits<std::int32_t>::max(),
                      .minTexelGatherOffset = 0,
                      .maxTexelGatherOffset = 0,
                      .minInterpolationOffset = 0,
                      .maxInterpolationOffset = 0,
                      .subPixelInterpolationOffsetBits = 0,
                      .maxFramebufferWidth = 1UL << 23,
                      .maxFramebufferHeight = 1UL << 23,
                      .maxFramebufferLayers = static_cast<std::uint32_t>(-1),
#warning fix up sample counts after adding multisampling
                      .framebufferColorSampleCounts = VK_SAMPLE_COUNT_1_BIT,
                      .framebufferDepthSampleCounts = VK_SAMPLE_COUNT_1_BIT,
                      .framebufferStencilSampleCounts = VK_SAMPLE_COUNT_1_BIT,
                      .framebufferNoAttachmentsSampleCounts = VK_SAMPLE_COUNT_1_BIT,
                      .maxColorAttachments = static_cast<std::uint32_t>(-1),
                      .sampledImageColorSampleCounts = VK_SAMPLE_COUNT_1_BIT,
                      .sampledImageIntegerSampleCounts = VK_SAMPLE_COUNT_1_BIT,
                      .sampledImageDepthSampleCounts = VK_SAMPLE_COUNT_1_BIT,
                      .sampledImageStencilSampleCounts = VK_SAMPLE_COUNT_1_BIT,
                      .storageImageSampleCounts = VK_SAMPLE_COUNT_1_BIT,
                      .maxSampleMaskWords = 1,
                      .timestampComputeAndGraphics = true,
                      .timestampPeriod =
                          std::chrono::duration_cast<std::chrono::duration<double, std::nano>>(
                              std::chrono::steady_clock::duration(1))
                              .count(),
                      .maxClipDistances = 0,
                      .maxCullDistances = 0,
                      .maxCombinedClipAndCullDistances = 0,
                      .discreteQueuePriorities = 2,
                      .pointSizeRange =
                          {
                              1, 1,
                          },
                      .lineWidthRange =
                          {
                              1, 1,
                          },
                      .pointSizeGranularity = 0,
                      .lineWidthGranularity = 0,
#warning update strictLines when the line rasterizer is implemented
                      .strictLines = true,
                      .standardSampleLocations = true,
                      .optimalBufferCopyOffsetAlignment = 16,
                      .optimalBufferCopyRowPitchAlignment = 16,
                      .nonCoherentAtomSize = 1,
                  },
              .sparseProperties =
                  {
#warning update upon implementation of sparse memory
                      .residencyStandard2DBlockShape = false,
                      .residencyStandard2DMultisampleBlockShape = false,
                      .residencyStandard3DBlockShape = false,
                      .residencyAlignedMipSize = false,
                      .residencyNonResidentStrict = false,
                  },
          }
    {
    }
};

struct Vulkan_instance : public Vulkan_dispatchable_object<Vulkan_instance, VkInstance>
{
    Vulkan_instance(const Vulkan_instance &) = delete;
    Vulkan_instance &operator=(const Vulkan_instance &) = delete;

    struct App_info
    {
        std::string application_name;
        std::uint32_t application_version;
        std::string engine_name;
        std::uint32_t engine_version;
        std::uint32_t api_version;
        App_info(std::string application_name,
                 std::uint32_t application_version,
                 std::string engine_name,
                 std::uint32_t engine_version,
                 std::uint32_t api_version) noexcept
            : application_name(std::move(application_name)),
              application_version(application_version),
              engine_name(std::move(engine_name)),
              engine_version(engine_version),
              api_version(api_version)
        {
        }
        explicit App_info(const VkApplicationInfo &application_info)
            : application_name(
                  application_info.pApplicationName ? application_info.pApplicationName : ""),
              application_version(application_info.applicationVersion),
              engine_name(application_info.pEngineName ? application_info.pEngineName : ""),
              engine_version(application_info.engineVersion),
              api_version(application_info.apiVersion)
        {
            assert(application_info.sType == VK_STRUCTURE_TYPE_APPLICATION_INFO);
        }
        App_info() noexcept : application_name(),
                              application_version(),
                              engine_name(),
                              engine_version(),
                              api_version()
        {
        }
    };
    App_info app_info;
    Supported_extensions extensions;
    Vulkan_physical_device physical_device;
    Vulkan_instance(App_info app_info, Supported_extensions extensions) noexcept
        : app_info(std::move(app_info)),
          extensions(std::move(extensions)),
          physical_device(*this)
    {
    }
    static util::variant<std::unique_ptr<Vulkan_instance>, VkResult> create(
        const VkInstanceCreateInfo &create_info);
#warning finish implementing Vulkan_instance
};
}
}

#endif // VULKAN_API_OBJECTS_H_
