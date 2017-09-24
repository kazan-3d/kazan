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
#ifndef PIPELINE_PIPELINE_H_
#define PIPELINE_PIPELINE_H_

#include <memory>
#include <cstdint>
#include <utility>
#include <cassert>
#include <cstring>
#include "llvm_wrapper/llvm_wrapper.h"
#include "vulkan/vulkan.h"
#include "vulkan/remove_xlib_macros.h"
#include "spirv/spirv.h"
#include "vulkan/api_objects.h"
#include "spirv_to_llvm/spirv_to_llvm.h"

namespace kazan
{
namespace pipeline
{
class Pipeline_cache : public vulkan::Vulkan_nondispatchable_object<Pipeline_cache, VkPipelineCache>
{
#warning finish implementing Pipeline_cache
public:
    static std::unique_ptr<Pipeline_cache> create(vulkan::Vulkan_device &,
                                                  const VkPipelineCacheCreateInfo &create_info)
    {
        assert(create_info.sType == VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO);
        assert(create_info.initialDataSize == 0 || create_info.pInitialData);
#warning finish implementing Pipeline_cache::create
        return std::make_unique<Pipeline_cache>();
    }
};

struct Instantiated_pipeline_layout
{
    vulkan::Vulkan_pipeline_layout &base;
    struct Binding
    {
        vulkan::Vulkan_descriptor_set_layout::Binding *base;
        std::shared_ptr<spirv_to_llvm::Array_type_descriptor> type;
        std::size_t member_index;
        constexpr Binding() noexcept : base(), type(), member_index(-1)
        {
        }
        Binding(vulkan::Vulkan_descriptor_set_layout::Binding &base,
                std::shared_ptr<spirv_to_llvm::Array_type_descriptor> type,
                std::size_t member_index) noexcept : base(&base),
                                                     type(std::move(type)),
                                                     member_index(member_index)
        {
        }
        explicit operator bool() const noexcept
        {
            return base != nullptr;
        }
    };
    struct Descriptor_set
    {
        vulkan::Vulkan_descriptor_set_layout *base;
        std::vector<Binding> bindings;
        Descriptor_set() noexcept : base(nullptr), bindings()
        {
        }
        explicit Descriptor_set(vulkan::Vulkan_descriptor_set_layout &base)
            : base(&base), bindings()
        {
        }
        explicit operator bool() const noexcept
        {
            return base != nullptr;
        }
    };
    std::vector<Descriptor_set> descriptor_sets;
    std::shared_ptr<spirv_to_llvm::Struct_type_descriptor> type;
    Instantiated_pipeline_layout(vulkan::Vulkan_pipeline_layout &base,
                                 ::LLVMContextRef llvm_context,
                                 ::LLVMTargetDataRef target_data);
};

struct Shader_module : public vulkan::Vulkan_nondispatchable_object<Shader_module, VkShaderModule>
{
    std::shared_ptr<unsigned char> bytes;
    std::size_t byte_count;
    Shader_module(std::shared_ptr<unsigned char> bytes, std::size_t byte_count) noexcept
        : bytes(std::move(bytes)),
          byte_count(byte_count)
    {
    }
    const spirv::Word *words() const noexcept
    {
        return reinterpret_cast<const spirv::Word *>(bytes.get());
    }
    std::size_t word_count() const noexcept
    {
        assert(byte_count % sizeof(spirv::Word) == 0);
        return byte_count / sizeof(spirv::Word);
    }
    static std::unique_ptr<Shader_module> create(vulkan::Vulkan_device &,
                                                 const VkShaderModuleCreateInfo &create_info)
    {
        struct Code_deleter
        {
            void operator()(unsigned char *bytes) const noexcept
            {
                delete[] bytes;
            }
        };
        auto bytes =
            std::shared_ptr<unsigned char>(new unsigned char[create_info.codeSize], Code_deleter{});
        std::memcpy(bytes.get(), create_info.pCode, create_info.codeSize);
        return std::make_unique<Shader_module>(std::move(bytes), create_info.codeSize);
    }
};

class Pipeline : public vulkan::Vulkan_nondispatchable_object<Pipeline, VkPipeline>
{
    Pipeline(const Pipeline &) = delete;
    Pipeline &operator=(const Pipeline &) = delete;

public:
    constexpr Pipeline() noexcept
    {
    }
    virtual ~Pipeline() = default;

protected:
    static llvm_wrapper::Module optimize_module(llvm_wrapper::Module module,
                                                ::LLVMTargetMachineRef target_machine);
};

class Graphics_pipeline final : public Pipeline
{
private:
    struct Implementation;

public:
#warning finish adding draw function parameters
    typedef void (*Vertex_shader_function)(std::uint32_t vertex_start_index,
                                           std::uint32_t vertex_end_index,
                                           std::uint32_t instance_id,
                                           void *output_buffer,
                                           void *const *bindings);
    typedef void (*Fragment_shader_function)(std::uint32_t *color_attachment_pixel);

public:
    void run_vertex_shader(std::uint32_t vertex_start_index,
                           std::uint32_t vertex_end_index,
                           std::uint32_t instance_id,
                           void *output_buffer,
                           void *const *input_bindings) const noexcept
    {
        vertex_shader_function(
            vertex_start_index, vertex_end_index, instance_id, output_buffer, input_bindings);
    }
    std::size_t get_vertex_shader_output_struct_size() const noexcept
    {
        return vertex_shader_output_struct_size;
    }
    void dump_vertex_shader_output_struct(const void *output_struct) const;
    void run_fragment_shader(std::uint32_t *color_attachment_pixel) const noexcept
    {
        fragment_shader_function(color_attachment_pixel);
    }
    void run(std::uint32_t vertex_start_index,
             std::uint32_t vertex_end_index,
             std::uint32_t instance_id,
             const vulkan::Vulkan_image &color_attachment,
             void *const *bindings);
    static std::unique_ptr<Graphics_pipeline> create(
        vulkan::Vulkan_device &,
        Pipeline_cache *pipeline_cache,
        const VkGraphicsPipelineCreateInfo &create_info);
    static std::unique_ptr<Graphics_pipeline> move_from_handle(VkPipeline pipeline) noexcept
    {
        return std::unique_ptr<Graphics_pipeline>(from_handle(pipeline));
    }
    static Graphics_pipeline *from_handle(VkPipeline pipeline) noexcept
    {
        auto *retval = Pipeline::from_handle(pipeline);
        assert(!retval || dynamic_cast<Graphics_pipeline *>(retval));
        return static_cast<Graphics_pipeline *>(retval);
    }

private:
    Graphics_pipeline(std::shared_ptr<Implementation> implementation,
                      Vertex_shader_function vertex_shader_function,
                      std::size_t vertex_shader_output_struct_size,
                      std::size_t vertex_shader_position_output_offset,
                      Fragment_shader_function fragment_shader_function,
                      VkViewport viewport,
                      VkRect2D scissor_rect) noexcept
        : implementation(std::move(implementation)),
          vertex_shader_function(vertex_shader_function),
          vertex_shader_output_struct_size(vertex_shader_output_struct_size),
          vertex_shader_position_output_offset(vertex_shader_position_output_offset),
          fragment_shader_function(fragment_shader_function),
          viewport(viewport),
          scissor_rect(scissor_rect)
    {
    }

private:
    std::shared_ptr<Implementation> implementation;
    Vertex_shader_function vertex_shader_function;
    std::size_t vertex_shader_output_struct_size;
    std::size_t vertex_shader_position_output_offset;
    Fragment_shader_function fragment_shader_function;
    VkViewport viewport;
    VkRect2D scissor_rect;
};

using vulkan::move_to_handle;
using vulkan::to_handle;
}
}

#endif // PIPELINE_PIPELINE_H_
