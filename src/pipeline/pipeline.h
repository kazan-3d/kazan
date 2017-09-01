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
#include "spirv/spirv.h"
#include "image/image.h"

namespace vulkan_cpu
{
namespace pipeline
{
template <typename T>
struct Api_object_deleter
{
    void operator()(T *) const noexcept = delete;
    typedef void Vulkan_handle;
    typedef void Create_info;
};

template <typename Object_>
struct Api_object_handle : public std::unique_ptr<Object_, Api_object_deleter<Object_>>
{
    typedef typename Api_object_deleter<Object_>::Vulkan_handle Vulkan_handle;
    typedef typename Api_object_deleter<Object_>::Create_info Create_info;
    typedef Object_ Object;
    using std::unique_ptr<Object_, Api_object_deleter<Object_>>::unique_ptr;
    static Object *from_handle(Vulkan_handle vulkan_handle) noexcept
    {
        return reinterpret_cast<Object *>(vulkan_handle);
    }
    static Api_object_handle move_from_handle(Vulkan_handle vulkan_handle) noexcept
    {
        return Api_object_handle(from_handle(vulkan_handle));
    }
    static Api_object_handle make(const Create_info &create_info);
};

template <typename Object>
inline typename Api_object_deleter<Object>::Vulkan_handle to_handle(Object *object) noexcept
{
    static_assert(!std::is_void<typename Api_object_deleter<Object>::Vulkan_handle>::value, "");
    return reinterpret_cast<typename Api_object_deleter<Object>::Vulkan_handle>(object);
}

template <typename Object>
inline typename Api_object_deleter<Object>::Vulkan_handle move_to_handle(
    Api_object_handle<Object> object) noexcept
{
    return to_handle(object.release());
}

class Pipeline_cache;

template <>
struct Api_object_deleter<Pipeline_cache>
{
    void operator()(Pipeline_cache *pipeline_cache) const noexcept;
    typedef VkPipelineCache Vulkan_handle;
    typedef VkPipelineCacheCreateInfo Create_info;
};

typedef Api_object_handle<Pipeline_cache> Pipeline_cache_handle;

class Render_pass;

template <>
struct Api_object_deleter<Render_pass>
{
    void operator()(Render_pass *render_pass) const noexcept;
    typedef VkRenderPass Vulkan_handle;
    typedef VkRenderPassCreateInfo Create_info;
};

typedef Api_object_handle<Render_pass> Render_pass_handle;

class Pipeline_layout;

template <>
struct Api_object_deleter<Pipeline_layout>
{
    void operator()(Pipeline_layout *pipeline_layout) const noexcept;
    typedef VkPipelineLayout Vulkan_handle;
    typedef VkPipelineLayoutCreateInfo Create_info;
};

typedef Api_object_handle<Pipeline_layout> Pipeline_layout_handle;

struct Shader_module
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
};

template <>
struct Api_object_deleter<Shader_module>
{
    void operator()(Shader_module *shader_module) const noexcept
    {
        delete shader_module;
    }
    typedef VkShaderModule Vulkan_handle;
    typedef VkShaderModuleCreateInfo Create_info;
};

typedef Api_object_handle<Shader_module> Shader_module_handle;

template <>
inline Shader_module_handle Shader_module_handle::make(const VkShaderModuleCreateInfo &create_info)
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
    return Shader_module_handle(new Shader_module(std::move(bytes), create_info.codeSize));
}

class Pipeline
{
    Pipeline(const Pipeline &) = delete;
    Pipeline &operator=(const Pipeline &) = delete;

public:
    constexpr Pipeline() noexcept
    {
    }
    virtual ~Pipeline() = default;
    static std::unique_ptr<Pipeline> move_from_handle(VkPipeline pipeline) noexcept
    {
        return std::unique_ptr<Pipeline>(from_handle(pipeline));
    }
    static Pipeline *from_handle(VkPipeline pipeline) noexcept
    {
        return reinterpret_cast<Pipeline *>(pipeline);
    }

protected:
    static llvm_wrapper::Module optimize_module(llvm_wrapper::Module module,
                                                ::LLVMTargetMachineRef target_machine);
};

inline VkPipeline to_handle(Pipeline *pipeline) noexcept
{
    return reinterpret_cast<VkPipeline>(pipeline);
}

inline VkPipeline move_to_handle(std::unique_ptr<Pipeline> pipeline) noexcept
{
    return to_handle(pipeline.release());
}

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
             const image::Image &color_attachment,
             void *const *bindings);
    static std::unique_ptr<Graphics_pipeline> make(Pipeline_cache *pipeline_cache,
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

inline VkPipeline to_handle(Graphics_pipeline *pipeline) noexcept
{
    return to_handle(static_cast<Pipeline *>(pipeline));
}

inline VkPipeline move_to_handle(std::unique_ptr<Graphics_pipeline> pipeline) noexcept
{
    return to_handle(pipeline.release());
}
}
}

#endif // PIPELINE_PIPELINE_H_
