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
#include "llvm_wrapper/llvm_wrapper.h"

namespace vulkan_cpu
{
namespace pipeline
{
class Pipeline
{
    Pipeline(const Pipeline &) = delete;
    Pipeline &operator=(const Pipeline &) = delete;

public:
    typedef std::uintptr_t Handle;

public:
    constexpr Pipeline() noexcept
    {
    }
    virtual ~Pipeline() = default;
    static std::unique_ptr<Pipeline> move_from_handle(Handle pipeline) noexcept
    {
        return std::unique_ptr<Pipeline>(from_handle(pipeline));
    }
    static Pipeline *from_handle(Handle pipeline) noexcept
    {
        return reinterpret_cast<Pipeline *>(pipeline);
    }
};

inline Pipeline::Handle to_handle(Pipeline *pipeline) noexcept
{
    return reinterpret_cast<Pipeline::Handle>(pipeline);
}

inline Pipeline::Handle move_to_handle(std::unique_ptr<Pipeline> pipeline) noexcept
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
#warning finish implementing Graphics_pipeline::make
    static std::unique_ptr<Graphics_pipeline> make();
    static std::unique_ptr<Graphics_pipeline> move_from_handle(Handle pipeline) noexcept
    {
        return std::unique_ptr<Graphics_pipeline>(from_handle(pipeline));
    }
    static Graphics_pipeline *from_handle(Handle pipeline) noexcept
    {
        auto *retval = reinterpret_cast<Pipeline *>(pipeline);
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

inline Pipeline::Handle to_handle(Graphics_pipeline *pipeline) noexcept
{
    return to_handle(static_cast<Pipeline *>(pipeline));
}

inline Pipeline::Handle move_to_handle(std::unique_ptr<Graphics_pipeline> pipeline) noexcept
{
    return to_handle(pipeline.release());
}
}
}

#endif // PIPELINE_PIPELINE_H_
