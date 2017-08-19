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
#include "util/constexpr_array.h"

namespace vulkan_cpu
{
namespace image
{
struct Image
{
    const VkImageType type;
    const VkImageTiling tiling;
    const VkFormat format;
    const util::Constexpr_array<std::uint32_t, 3> dimensions;
    const std::size_t memory_size;
    std::unique_ptr<unsigned char[]> memory;
    Image(VkImageType type,
          VkImageTiling tiling,
          VkFormat format,
          const util::Constexpr_array<std::uint32_t, 3> &dimensions,
          std::unique_ptr<unsigned char[]> memory = nullptr) noexcept
        : type(type),
          tiling(tiling),
          format(format),
          dimensions(dimensions),
          memory_size(get_memory_size(type, tiling, format, dimensions)),
          memory(std::move(memory))
    {
    }
    static constexpr std::size_t get_memory_size(
        VkImageType type,
        VkImageTiling tiling,
        VkFormat format,
        const util::Constexpr_array<std::uint32_t, 3> &dimensions) noexcept
    {
#warning finish implementing Image
        assert(type == VK_IMAGE_TYPE_2D);
        assert(tiling == VK_IMAGE_TILING_LINEAR);
        assert(format == VK_FORMAT_R8G8B8A8_SRGB);
        return sizeof(std::uint32_t) * dimensions[0] * dimensions[1];
    }
#warning finish implementing Image
};
}
}

#endif // IMAGE_IMAGE_H_
