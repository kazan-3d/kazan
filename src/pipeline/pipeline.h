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
#include "llvm_wrapper/llvm_wrapper.h"
#include "vulkan/vulkan.h"

namespace vulkan_cpu
{
namespace pipeline
{
class Pipeline_cache;

class Pipeline_cache_handle
{
private:
    static void destroy(Pipeline_cache *value) noexcept;

public:
    explicit Pipeline_cache_handle(Pipeline_cache *value) noexcept : value(value)
    {
    }
    constexpr Pipeline_cache_handle() noexcept : value(nullptr)
    {
    }
    Pipeline_cache_handle(Pipeline_cache_handle &&rt) noexcept : value(rt.value)
    {
        rt.value = nullptr;
    }
    Pipeline_cache_handle &operator=(Pipeline_cache_handle rt) noexcept
    {
        swap(rt);
        return *this;
    }
    ~Pipeline_cache_handle() noexcept
    {
        if(value)
            destroy(value);
    }
    void swap(Pipeline_cache_handle &rt) noexcept
    {
        using std::swap;
        swap(value, rt.value);
    }
    static Pipeline_cache *from_handle(VkPipelineCache pipeline_cache) noexcept
    {
        return reinterpret_cast<Pipeline_cache *>(pipeline_cache);
    }
    static Pipeline_cache_handle move_from_handle(VkPipelineCache pipeline_cache) noexcept
    {
        return Pipeline_cache_handle(from_handle(pipeline_cache));
    }
    Pipeline_cache *get() const noexcept
    {
        return value;
    }
    Pipeline_cache *release() noexcept
    {
        auto retval = value;
        value = nullptr;
        return retval;
    }
    Pipeline_cache *operator->() const noexcept
    {
        assert(value);
        return value;
    }
    Pipeline_cache &operator*() const noexcept
    {
        return *operator->();
    }

private:
    Pipeline_cache *value;
};

inline VkPipelineCache to_handle(Pipeline_cache *pipeline_cache) noexcept
{
    return reinterpret_cast<VkPipelineCache>(pipeline_cache);
}

inline VkPipelineCache move_to_handle(Pipeline_cache_handle pipeline_cache) noexcept
{
    return to_handle(pipeline_cache.release());
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
public:
#warning finish adding draw function parameters
    typedef void (*Vertex_shader_function)(std::uint32_t vertex_start_index,
                                           std::uint32_t vertex_end_index,
                                           std::uint32_t instance_id,
                                           void *output_buffer);

public:
    const Vertex_shader_function get_vertex_shader_function() const noexcept
    {
        return vertex_shader_function;
    }
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
    Graphics_pipeline(std::shared_ptr<void> state,
                      Vertex_shader_function vertex_shader_function) noexcept
        : state(std::move(state)),
          vertex_shader_function(vertex_shader_function)
    {
    }

private:
    std::shared_ptr<void> state;
    Vertex_shader_function vertex_shader_function;
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
