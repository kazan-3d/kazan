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
#ifndef IMAGE_IMAGE_H_
#define IMAGE_IMAGE_H_

#include "vulkan/vulkan.h"
#include <memory>
#include <cassert>
#include <cstdint>
#include "util/enum.h"

namespace vulkan_cpu
{
namespace image
{
struct Image_descriptor
{
    static constexpr VkImageCreateFlags supported_flags =
        VK_IMAGE_CREATE_MUTABLE_FORMAT_BIT | VK_IMAGE_CREATE_CUBE_COMPATIBLE_BIT;
    VkImageCreateFlags flags;
    VkImageType type;
    VkFormat format;
    VkExtent3D extent;
    std::uint32_t mip_levels;
    std::uint32_t array_layers;
    static constexpr VkSampleCountFlags supported_samples = VK_SAMPLE_COUNT_1_BIT;
    VkSampleCountFlagBits samples;
    VkImageTiling tiling;
    constexpr explicit Image_descriptor(const VkImageCreateInfo &image_create_info) noexcept
        : flags(image_create_info.flags),
          type(image_create_info.imageType),
          format(image_create_info.format),
          extent(image_create_info.extent),
          mip_levels(image_create_info.mipLevels),
          array_layers(image_create_info.arrayLayers),
          samples(image_create_info.samples),
          tiling(image_create_info.tiling)
    {
        assert(image_create_info.sType == VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO);
        assert((flags & ~supported_flags) == 0);
        assert((samples & ~supported_samples) == 0);
        assert(extent.width > 0);
        assert(extent.height > 0);
        assert(extent.depth > 0);

#warning finish implementing Image
        assert(type == VK_IMAGE_TYPE_2D && "unimplemented image type");
        assert(extent.depth == 1);

        assert(format == VK_FORMAT_B8G8R8A8_UNORM && "unimplemented image format");
        assert(mip_levels == 1 && "mipmapping is unimplemented");
        assert(array_layers == 1 && "array images are unimplemented");
        assert(tiling == VK_IMAGE_TILING_LINEAR && "non-linear image tiling is unimplemented");
        assert(image_create_info.initialLayout == VK_IMAGE_LAYOUT_UNDEFINED
               && "preinitialized images are unimplemented");
    }
    constexpr Image_descriptor(VkImageCreateFlags flags,
                               VkImageType type,
                               VkFormat format,
                               VkExtent3D extent,
                               std::uint32_t mip_levels,
                               std::uint32_t array_layers,
                               VkSampleCountFlagBits samples,
                               VkImageTiling tiling) noexcept : flags(flags),
                                                                type(type),
                                                                format(format),
                                                                extent(extent),
                                                                mip_levels(mip_levels),
                                                                array_layers(array_layers),
                                                                samples(samples),
                                                                tiling(tiling)
    {
    }
    constexpr std::size_t get_memory_size() const noexcept
    {
#warning finish implementing Image
        assert(samples == VK_SAMPLE_COUNT_1_BIT && "multisample images are unimplemented");
        assert(extent.width > 0);
        assert(extent.height > 0);
        assert(extent.depth > 0);

        assert(type == VK_IMAGE_TYPE_2D && "unimplemented image type");
        assert(extent.depth == 1);

        assert(format == VK_FORMAT_B8G8R8A8_UNORM && "unimplemented image format");
        assert(mip_levels == 1 && "mipmapping is unimplemented");
        assert(array_layers == 1 && "array images are unimplemented");
        assert(tiling == VK_IMAGE_TILING_LINEAR && "non-linear image tiling is unimplemented");
        std::size_t retval = sizeof(std::uint32_t);
        retval *= extent.width;
        retval *= extent.height;
        return retval;
    }
};

struct Allocate_memory_tag
{
    explicit constexpr Allocate_memory_tag(int) noexcept
    {
    }
};

constexpr Allocate_memory_tag allocate_memory_tag{0};

struct Image
{
    const Image_descriptor descriptor;
    std::unique_ptr<unsigned char[]> memory;
    Image(const Image_descriptor &descriptor,
          std::unique_ptr<unsigned char[]> memory = nullptr) noexcept : descriptor(descriptor),
                                                                        memory(std::move(memory))
    {
    }
    Image(const Image_descriptor &descriptor, Allocate_memory_tag)
        : descriptor(descriptor), memory(new unsigned char[descriptor.get_memory_size()])
    {
    }
#warning finish implementing Image
};
}
}

#endif // IMAGE_IMAGE_H_
