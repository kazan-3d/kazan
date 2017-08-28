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
#include "pipeline.h"
#include "spirv_to_llvm/spirv_to_llvm.h"
#include "llvm_wrapper/llvm_wrapper.h"
#include "llvm_wrapper/orc_compile_stack.h"
#include "vulkan/util.h"
#include "util/soft_float.h"
#include "json/json.h"
#include <stdexcept>
#include <cassert>
#include <vector>
#include <iostream>

namespace vulkan_cpu
{
namespace pipeline
{
class Pipeline_cache
{
};

void Api_object_deleter<Pipeline_cache>::operator()(Pipeline_cache *pipeline_cache) const noexcept
{
    delete pipeline_cache;
}

class Render_pass
{
};

void Api_object_deleter<Render_pass>::operator()(Render_pass *render_pass) const noexcept
{
    delete render_pass;
}

template <>
Render_pass_handle Render_pass_handle::make(const VkRenderPassCreateInfo &render_pass_create_info)
{
#warning finish implementing Render_pass_handle::make
    return Render_pass_handle(new Render_pass());
}

class Pipeline_layout
{
};

void Api_object_deleter<Pipeline_layout>::operator()(Pipeline_layout *pipeline_layout) const
    noexcept
{
    delete pipeline_layout;
}

template <>
Pipeline_layout_handle Pipeline_layout_handle::make(
    const VkPipelineLayoutCreateInfo &pipeline_layout_create_info)
{
#warning finish implementing Pipeline_layout_handle::make
    return Pipeline_layout_handle(new Pipeline_layout());
}

llvm_wrapper::Module Pipeline::optimize_module(llvm_wrapper::Module module,
                                               ::LLVMTargetMachineRef target_machine)
{
    switch(llvm_wrapper::Target_machine::get_code_gen_opt_level(target_machine))
    {
    case ::LLVMCodeGenLevelNone:
    case ::LLVMCodeGenLevelLess:
        break;
    case ::LLVMCodeGenLevelDefault:
    case ::LLVMCodeGenLevelAggressive:
    {
#warning finish implementing module optimizations
        {
            auto manager = llvm_wrapper::Pass_manager::create_function_pass_manager(module.get());
            ::LLVMAddAnalysisPasses(target_machine, manager.get());
            ::LLVMAddPromoteMemoryToRegisterPass(manager.get());
            ::LLVMAddScalarReplAggregatesPass(manager.get());
            ::LLVMAddScalarizerPass(manager.get());
            ::LLVMAddEarlyCSEMemSSAPass(manager.get());
            ::LLVMAddSCCPPass(manager.get());
            ::LLVMAddAggressiveDCEPass(manager.get());
            ::LLVMAddLICMPass(manager.get());
            ::LLVMAddCFGSimplificationPass(manager.get());
            ::LLVMAddReassociatePass(manager.get());
            ::LLVMAddInstructionCombiningPass(manager.get());
            ::LLVMAddNewGVNPass(manager.get());
            ::LLVMAddCorrelatedValuePropagationPass(manager.get());
            ::LLVMInitializeFunctionPassManager(manager.get());
            for(auto fn = ::LLVMGetFirstFunction(module.get()); fn; fn = ::LLVMGetNextFunction(fn))
                ::LLVMRunFunctionPassManager(manager.get(), fn);
            ::LLVMFinalizeFunctionPassManager(manager.get());
        }
        {
            auto manager = llvm_wrapper::Pass_manager::create_module_pass_manager();
            ::LLVMAddAnalysisPasses(target_machine, manager.get());
            ::LLVMAddIPSCCPPass(manager.get());
            ::LLVMAddFunctionInliningPass(manager.get());
            ::LLVMAddDeadArgEliminationPass(manager.get());
            ::LLVMAddGlobalDCEPass(manager.get());
            ::LLVMRunPassManager(manager.get(), module.get());
        }
        {
            auto manager = llvm_wrapper::Pass_manager::create_function_pass_manager(module.get());
            ::LLVMAddAnalysisPasses(target_machine, manager.get());
            ::LLVMAddCFGSimplificationPass(manager.get());
            ::LLVMAddPromoteMemoryToRegisterPass(manager.get());
            ::LLVMAddScalarReplAggregatesPass(manager.get());
            ::LLVMAddReassociatePass(manager.get());
            ::LLVMAddInstructionCombiningPass(manager.get());
            ::LLVMAddLoopUnrollPass(manager.get());
            ::LLVMAddSLPVectorizePass(manager.get());
            ::LLVMAddAggressiveDCEPass(manager.get());
            ::LLVMInitializeFunctionPassManager(manager.get());
            for(auto fn = ::LLVMGetFirstFunction(module.get()); fn; fn = ::LLVMGetNextFunction(fn))
                ::LLVMRunFunctionPassManager(manager.get(), fn);
            ::LLVMFinalizeFunctionPassManager(manager.get());
        }
        std::cerr << "optimized module:" << std::endl;
        ::LLVMDumpModule(module.get());
        break;
    }
    }
    return module;
}

struct Graphics_pipeline::Implementation
{
    llvm_wrapper::Context llvm_context = llvm_wrapper::Context::create();
    spirv_to_llvm::Jit_symbol_resolver jit_symbol_resolver;
    llvm_wrapper::Orc_compile_stack jit_stack;
    llvm_wrapper::Target_data data_layout;
    std::vector<spirv_to_llvm::Converted_module> compiled_shaders;
    std::shared_ptr<spirv_to_llvm::Struct_type_descriptor> vertex_shader_output_struct;
    std::string append_value_to_string(std::string str,
                                       spirv_to_llvm::Type_descriptor &type,
                                       const void *value) const
    {
        struct Visitor : public spirv_to_llvm::Type_descriptor::Type_visitor
        {
            const Implementation *this_;
            std::string &str;
            const void *value;
            Visitor(const Implementation *this_, std::string &str, const void *value) noexcept
                : this_(this_),
                  str(str),
                  value(value)
            {
            }
            virtual void visit(spirv_to_llvm::Simple_type_descriptor &type) override
            {
                auto llvm_type = type.get_or_make_type().type;
                switch(::LLVMGetTypeKind(llvm_type))
                {
                case ::LLVMVoidTypeKind:
                case ::LLVMX86_FP80TypeKind:
                case ::LLVMFP128TypeKind:
                case ::LLVMPPC_FP128TypeKind:
                case ::LLVMLabelTypeKind:
                case ::LLVMFunctionTypeKind:
                case ::LLVMStructTypeKind:
                case ::LLVMArrayTypeKind:
                case ::LLVMPointerTypeKind:
                case ::LLVMVectorTypeKind:
                case ::LLVMMetadataTypeKind:
                case ::LLVMX86_MMXTypeKind:
                case ::LLVMTokenTypeKind:
                    break;
                case ::LLVMHalfTypeKind:
                {
                    auto integer_value = *static_cast<const std::uint16_t *>(value);
                    auto float_value =
                        util::soft_float::ExtendedFloat::fromHalfPrecision(integer_value);
                    str = json::ast::Number_value::append_double_to_string(
                        static_cast<double>(float_value), std::move(str));
                    if(float_value.isNaN())
                    {
                        str += " (0x";
                        str = json::ast::Number_value::append_unsigned_integer_to_string(
                            integer_value, std::move(str), 0x10);
                        str += ")";
                    }
                    return;
                }
                case ::LLVMFloatTypeKind:
                {
                    static_assert(sizeof(std::uint32_t) == sizeof(float)
                                      && alignof(std::uint32_t) == alignof(float),
                                  "");
                    union
                    {
                        std::uint32_t integer_value;
                        float float_value;
                    };
                    integer_value = *static_cast<const std::uint32_t *>(value);
                    str = json::ast::Number_value::append_double_to_string(float_value,
                                                                           std::move(str));
                    if(std::isnan(float_value))
                    {
                        str += " (0x";
                        str = json::ast::Number_value::append_unsigned_integer_to_string(
                            integer_value, std::move(str), 0x10);
                        str += ")";
                    }
                    return;
                }
                case ::LLVMDoubleTypeKind:
                {
                    static_assert(sizeof(std::uint64_t) == sizeof(double)
                                      && alignof(std::uint64_t) == alignof(double),
                                  "");
                    union
                    {
                        std::uint64_t integer_value;
                        double float_value;
                    };
                    integer_value = *static_cast<const std::uint64_t *>(value);
                    str = json::ast::Number_value::append_double_to_string(float_value,
                                                                           std::move(str));
                    if(std::isnan(float_value))
                    {
                        str += " (0x";
                        str = json::ast::Number_value::append_unsigned_integer_to_string(
                            integer_value, std::move(str), 0x10);
                        str += ")";
                    }
                    return;
                }
                case ::LLVMIntegerTypeKind:
                {
                    switch(::LLVMGetIntTypeWidth(llvm_type))
                    {
                    case 8:
                    {
                        auto integer_value = *static_cast<const std::uint8_t *>(value);
                        str += "0x";
                        str = json::ast::Number_value::append_unsigned_integer_to_string(
                            integer_value, std::move(str), 0x10);
                        str += " ";
                        str = json::ast::Number_value::append_unsigned_integer_to_string(
                            integer_value, std::move(str));
                        str += " ";
                        str = json::ast::Number_value::append_signed_integer_to_string(
                            static_cast<std::int8_t>(integer_value), std::move(str));
                        return;
                    }
                    case 16:
                    {
                        auto integer_value = *static_cast<const std::uint16_t *>(value);
                        str += "0x";
                        str = json::ast::Number_value::append_unsigned_integer_to_string(
                            integer_value, std::move(str), 0x10);
                        str += " ";
                        str = json::ast::Number_value::append_unsigned_integer_to_string(
                            integer_value, std::move(str));
                        str += " ";
                        str = json::ast::Number_value::append_signed_integer_to_string(
                            static_cast<std::int16_t>(integer_value), std::move(str));
                        return;
                    }
                    case 32:
                    {
                        auto integer_value = *static_cast<const std::uint32_t *>(value);
                        str += "0x";
                        str = json::ast::Number_value::append_unsigned_integer_to_string(
                            integer_value, std::move(str), 0x10);
                        str += " ";
                        str = json::ast::Number_value::append_unsigned_integer_to_string(
                            integer_value, std::move(str));
                        str += " ";
                        str = json::ast::Number_value::append_signed_integer_to_string(
                            static_cast<std::int32_t>(integer_value), std::move(str));
                        return;
                    }
                    case 64:
                    {
                        auto integer_value = *static_cast<const std::uint64_t *>(value);
                        str += "0x";
                        str = json::ast::Number_value::append_unsigned_integer_to_string(
                            integer_value, std::move(str), 0x10);
                        str += " ";
                        str = json::ast::Number_value::append_unsigned_integer_to_string(
                            integer_value, std::move(str));
                        str += " ";
                        str = json::ast::Number_value::append_signed_integer_to_string(
                            static_cast<std::int64_t>(integer_value), std::move(str));
                        return;
                    }
                    }
                    break;
                }
                }
                assert(!"unhandled type");
                throw std::runtime_error("unhandled type");
            }
            virtual void visit(spirv_to_llvm::Vector_type_descriptor &type) override
            {
                auto llvm_element_type = type.get_element_type()->get_or_make_type().type;
                std::size_t element_size =
                    ::LLVMABISizeOfType(this_->data_layout.get(), llvm_element_type);
                std::size_t element_count = type.get_element_count();
                str += "<";
                auto separator = "";
                for(std::size_t i = 0; i < element_count; i++)
                {
                    str += separator;
                    separator = ", ";
                    str = this_->append_value_to_string(
                        std::move(str),
                        *type.get_element_type(),
                        static_cast<const char *>(value) + i * element_size);
                }
                str += ">";
            }
            virtual void visit(spirv_to_llvm::Matrix_type_descriptor &type) override
            {
                assert(!"dumping matrix not implemented");
                throw std::runtime_error("dumping matrix not implemented");
#warning dumping matrix not implemented
            }
            virtual void visit(spirv_to_llvm::Array_type_descriptor &type) override
            {
                auto llvm_element_type = type.get_element_type()->get_or_make_type().type;
                std::size_t element_size =
                    ::LLVMABISizeOfType(this_->data_layout.get(), llvm_element_type);
                std::size_t element_count = type.get_element_count();
                str += "[";
                auto separator = "";
                for(std::size_t i = 0; i < element_count; i++)
                {
                    str += separator;
                    separator = ", ";
                    str = this_->append_value_to_string(
                        std::move(str),
                        *type.get_element_type(),
                        static_cast<const char *>(value) + i * element_size);
                }
                str += "]";
            }
            virtual void visit(spirv_to_llvm::Pointer_type_descriptor &type) override
            {
                str += "pointer:0x";
                str = json::ast::Number_value::append_unsigned_integer_to_string(
                    reinterpret_cast<std::uint64_t>(*static_cast<const void *const *>(value)),
                    std::move(str),
                    0x10);
            }
            virtual void visit(spirv_to_llvm::Function_type_descriptor &type) override
            {
                str += "function:0x";
                str = json::ast::Number_value::append_unsigned_integer_to_string(
                    reinterpret_cast<std::uint64_t>(*static_cast<const void *const *>(value)),
                    std::move(str),
                    0x10);
            }
            virtual void visit(spirv_to_llvm::Struct_type_descriptor &type) override
            {
                auto &&members = type.get_members(true);
                auto llvm_type = type.get_or_make_type().type;
                str += "{";
                auto separator = "";
                for(auto &member : members)
                {
                    str += separator;
                    separator = ", ";
                    str = this_->append_value_to_string(
                        std::move(str),
                        *member.type,
                        static_cast<const char *>(value)
                            + ::LLVMOffsetOfElement(
                                  this_->data_layout.get(), llvm_type, member.llvm_member_index));
                }
                str += "}";
            }
        };
        type.visit(Visitor(this, str, value));
        return str;
    }
};

void Graphics_pipeline::dump_vertex_shader_output_struct(const void *output_struct) const
{
    std::cerr << "output: "
              << implementation->append_value_to_string(
                     {}, *implementation->vertex_shader_output_struct, output_struct)
              << std::endl;
}

void Graphics_pipeline::run(std::uint32_t vertex_start_index,
                            std::uint32_t vertex_end_index,
                            std::uint32_t instance_id,
                            const image::Image &color_attachment)
{
    typedef std::uint32_t Pixel_type;
    assert(color_attachment.descriptor.tiling == VK_IMAGE_TILING_LINEAR);
    std::size_t color_attachment_stride = color_attachment.descriptor.get_memory_stride();
    std::size_t color_attachment_pixel_size = color_attachment.descriptor.get_memory_pixel_size();
    unsigned char *color_attachment_memory = color_attachment.memory.get();
    float viewport_x_scale, viewport_x_offset, viewport_y_scale, viewport_y_offset,
        viewport_z_scale, viewport_z_offset;
    {
        float px = viewport.width;
        float ox = viewport.x + 0.5f * viewport.width;
        float py = viewport.height;
        float oy = viewport.y + 0.5f * viewport.height;
        float pz = viewport.maxDepth - viewport.minDepth;
        float oz = viewport.minDepth;
        viewport_x_scale = px * 0.5f;
        viewport_x_offset = ox;
        viewport_y_scale = py * 0.5f;
        viewport_y_offset = oy;
        viewport_z_scale = pz;
        viewport_z_offset = oz;
    }
    constexpr std::size_t vec4_native_alignment = alignof(float) * 4;
    constexpr std::size_t max_alignment = alignof(std::max_align_t);
    constexpr std::size_t vec4_alignment =
        vec4_native_alignment > max_alignment ? max_alignment : vec4_native_alignment;
    constexpr std::size_t ivec4_native_alignment = alignof(std::int32_t) * 4;
    constexpr std::size_t ivec4_alignment =
        ivec4_native_alignment > max_alignment ? max_alignment : ivec4_native_alignment;
    struct alignas(vec4_alignment) Vec4
    {
        float x;
        float y;
        float z;
        float w;
        constexpr Vec4() noexcept : x(), y(), z(), w()
        {
        }
        constexpr explicit Vec4(float x, float y, float z, float w) noexcept : x(x),
                                                                               y(y),
                                                                               z(z),
                                                                               w(w)
        {
        }
    };
    struct alignas(ivec4_alignment) Ivec4
    {
        std::int32_t x;
        std::int32_t y;
        std::int32_t z;
        std::int32_t w;
        constexpr Ivec4() noexcept : x(), y(), z(), w()
        {
        }
        constexpr explicit Ivec4(std::int32_t x,
                                 std::int32_t y,
                                 std::int32_t z,
                                 std::int32_t w) noexcept : x(x),
                                                            y(y),
                                                            z(z),
                                                            w(w)
        {
        }
    };
    auto interpolate_float = [](float t, float v0, float v1) noexcept->float
    {
        return t * v1 + (1.0f - t) * v0;
    };
    auto interpolate_vec4 = [interpolate_float](
                                float t, const Vec4 &v0, const Vec4 &v1) noexcept->Vec4
    {
        return Vec4(interpolate_float(t, v0.x, v1.x),
                    interpolate_float(t, v0.y, v1.y),
                    interpolate_float(t, v0.z, v1.z),
                    interpolate_float(t, v0.w, v1.w));
    };
    static constexpr std::size_t triangle_vertex_count = 3;
    struct Triangle
    {
        Vec4 vertexes[triangle_vertex_count];
        constexpr Triangle() noexcept : vertexes{}
        {
        }
        constexpr Triangle(const Vec4 &v0, const Vec4 &v1, const Vec4 &v2) noexcept
            : vertexes{v0, v1, v2}
        {
        }
    };
    auto solve_for_t = [](float v0, float v1) noexcept->float
    {
        // solves interpolate_float(t, v0, v1) == 0
        return v0 / (v0 - v1);
    };
    auto clip_edge = [solve_for_t, interpolate_vec4](const Vec4 &start_vertex,
                                                     const Vec4 &end_vertex,
                                                     Vec4 *output_vertexes,
                                                     std::size_t &output_vertex_count,
                                                     auto eval_vertex) -> bool
    {
        // eval_vertex returns a non-negative number if the vertex is inside the clip volume
        float start_vertex_signed_distance = eval_vertex(start_vertex);
        float end_vertex_signed_distance = eval_vertex(end_vertex);
        if(start_vertex_signed_distance != start_vertex_signed_distance)
            return false; // triangle has a NaN coordinate; skip it
        if(start_vertex_signed_distance < 0)
        {
            // start_vertex is outside
            if(end_vertex_signed_distance < 0)
            {
                // end_vertex is outside; do nothing
            }
            else
            {
                // end_vertex is inside
                output_vertexes[output_vertex_count++] = interpolate_vec4(
                    solve_for_t(start_vertex_signed_distance, end_vertex_signed_distance),
                    start_vertex,
                    end_vertex);
                output_vertexes[output_vertex_count++] = end_vertex;
            }
        }
        else
        {
            // start_vertex is inside
            if(end_vertex_signed_distance < 0)
            {
                // end_vertex is outside
                output_vertexes[output_vertex_count++] = interpolate_vec4(
                    solve_for_t(start_vertex_signed_distance, end_vertex_signed_distance),
                    start_vertex,
                    end_vertex);
            }
            else
            {
                // end_vertex is inside
                output_vertexes[output_vertex_count++] = end_vertex;
            }
        }
        return true;
    };
    auto clip_triangles = [clip_edge](
        std::vector<Triangle> &triangles, std::vector<Triangle> &temp_triangles, auto eval_vertex)
    {
        temp_triangles.clear();
        for(auto &input_ref : triangles)
        {
            Triangle input = input_ref; // copy to enable compiler optimizations
            constexpr std::size_t max_clipped_output_vertex_count = 4;
            Vec4 output_vertexes[max_clipped_output_vertex_count];
            std::size_t output_vertex_count = 0;
            bool skip_triangle = false;
            std::size_t end_vertex_index = 1;
            for(std::size_t start_vertex_index = 0; start_vertex_index < triangle_vertex_count;
                start_vertex_index++)
            {
                if(!clip_edge(input.vertexes[start_vertex_index],
                              input.vertexes[end_vertex_index],
                              output_vertexes,
                              output_vertex_count,
                              eval_vertex))
                {
                    skip_triangle = true;
                    break;
                }
                if(++end_vertex_index >= triangle_vertex_count)
                    end_vertex_index = 0;
            }
            if(skip_triangle)
                continue;
            switch(output_vertex_count)
            {
            case 0:
            case 1:
            case 2:
                continue;
            case 3:
                temp_triangles.push_back(
                    Triangle(output_vertexes[0], output_vertexes[1], output_vertexes[2]));
                continue;
            case 4:
                temp_triangles.push_back(
                    Triangle(output_vertexes[0], output_vertexes[1], output_vertexes[2]));
                temp_triangles.push_back(
                    Triangle(output_vertexes[0], output_vertexes[2], output_vertexes[3]));
                continue;
            }
            assert(!"clipping algorithm failed");
        }
        temp_triangles.swap(triangles);
    };
    std::vector<Triangle> triangles;
    std::vector<Triangle> temp_triangles;
    constexpr std::size_t chunk_max_size = 96;
    static_assert(chunk_max_size % triangle_vertex_count == 0, "");
    std::unique_ptr<unsigned char[]> chunk_vertex_buffer(
        new unsigned char[get_vertex_shader_output_struct_size() * chunk_max_size]);
    while(vertex_start_index < vertex_end_index)
    {
        std::uint32_t chunk_size = vertex_end_index - vertex_start_index;
        if(chunk_size > chunk_max_size)
            chunk_size = chunk_max_size;
        auto current_vertex_start_index = vertex_start_index;
        vertex_start_index += chunk_size;
        run_vertex_shader(current_vertex_start_index,
                          current_vertex_start_index + chunk_size,
                          instance_id,
                          chunk_vertex_buffer.get());
        const unsigned char *current_vertex =
            chunk_vertex_buffer.get() + vertex_shader_position_output_offset;
        triangles.clear();
        for(std::uint32_t i = 0; i + triangle_vertex_count <= chunk_size;
            i += triangle_vertex_count)
        {
            Triangle triangle;
            for(std::size_t j = 0; j < triangle_vertex_count; j++)
            {
                triangle.vertexes[j] = *reinterpret_cast<const Vec4 *>(current_vertex);
                current_vertex += vertex_shader_output_struct_size;
            }
            triangles.push_back(triangle);
        }
        // clip to 0 <= vertex.z
        clip_triangles(triangles,
                       temp_triangles,
                       [](const Vec4 &vertex) noexcept->float
                       {
                           return vertex.z;
                       });
        // clip to vertex.z <= vertex.w
        clip_triangles(triangles,
                       temp_triangles,
                       [](const Vec4 &vertex) noexcept->float
                       {
                           return vertex.w - vertex.z;
                       });
        // clip to -vertex.w <= vertex.x
        clip_triangles(triangles,
                       temp_triangles,
                       [](const Vec4 &vertex) noexcept->float
                       {
                           return vertex.x + vertex.w;
                       });
        // clip to vertex.x <= vertex.w
        clip_triangles(triangles,
                       temp_triangles,
                       [](const Vec4 &vertex) noexcept->float
                       {
                           return vertex.w - vertex.x;
                       });
        // clip to -vertex.w <= vertex.y
        clip_triangles(triangles,
                       temp_triangles,
                       [](const Vec4 &vertex) noexcept->float
                       {
                           return vertex.y + vertex.w;
                       });
        // clip to vertex.y <= vertex.w
        clip_triangles(triangles,
                       temp_triangles,
                       [](const Vec4 &vertex) noexcept->float
                       {
                           return vertex.w - vertex.y;
                       });
        VkOffset2D clipped_scissor_rect_min = scissor_rect.offset;
        VkOffset2D clipped_scissor_rect_end = {
            .x = scissor_rect.offset.x + static_cast<std::int32_t>(scissor_rect.extent.width),
            .y = scissor_rect.offset.y + static_cast<std::int32_t>(scissor_rect.extent.height),
        };
        if(clipped_scissor_rect_min.x < 0)
            clipped_scissor_rect_min.x = 0;
        if(clipped_scissor_rect_min.y < 0)
            clipped_scissor_rect_min.y = 0;
        if(clipped_scissor_rect_end.x > color_attachment.descriptor.extent.width)
            clipped_scissor_rect_end.x = color_attachment.descriptor.extent.width;
        if(clipped_scissor_rect_end.y < color_attachment.descriptor.extent.height)
            clipped_scissor_rect_end.y = color_attachment.descriptor.extent.height;
        if(clipped_scissor_rect_end.x <= clipped_scissor_rect_min.x)
            continue;
        if(clipped_scissor_rect_end.y <= clipped_scissor_rect_min.y)
            continue;
        for(std::size_t triangle_index = 0; triangle_index < triangles.size(); triangle_index++)
        {
            Triangle triangle = triangles[triangle_index];
            Vec4 projected_triangle_and_inv_w[triangle_vertex_count];
            Vec4 framebuffer_coordinates[triangle_vertex_count];
            for(std::size_t i = 0; i < triangle_vertex_count; i++)
            {
                projected_triangle_and_inv_w[i].w = 1.0f / triangle.vertexes[i].w;
                projected_triangle_and_inv_w[i].x =
                    triangle.vertexes[i].x * projected_triangle_and_inv_w[i].w;
                projected_triangle_and_inv_w[i].y =
                    triangle.vertexes[i].y * projected_triangle_and_inv_w[i].w;
                projected_triangle_and_inv_w[i].z =
                    triangle.vertexes[i].z * projected_triangle_and_inv_w[i].w;
                framebuffer_coordinates[i] =
                    Vec4(projected_triangle_and_inv_w[i].x * viewport_x_scale + viewport_x_offset,
                         projected_triangle_and_inv_w[i].y * viewport_y_scale + viewport_y_offset,
                         projected_triangle_and_inv_w[i].z * viewport_z_scale + viewport_z_offset,
                         0);
            }
            float orientation = 0;
            for(std::size_t start_vertex_index = 0, end_vertex_index = 1;
                start_vertex_index < triangle_vertex_count;
                start_vertex_index++)
            {
                float x1 = framebuffer_coordinates[start_vertex_index].x;
                float y1 = framebuffer_coordinates[start_vertex_index].y;
                float x2 = framebuffer_coordinates[end_vertex_index].x;
                float y2 = framebuffer_coordinates[end_vertex_index].y;
                orientation += x2 * y1 - x1 * y2;
                if(++end_vertex_index >= triangle_vertex_count)
                    end_vertex_index = 0;
            }
            if(!(orientation < 0)
               && !(orientation > 0)) // zero area triangle or triangle coordinate is NaN
                continue;
            // orientation > 0 for counter-clockwise triangle
            // orientation < 0 for clockwise triangle
            std::int32_t min_x, end_x, min_y, end_y;
            bool first = true;
            for(std::size_t i = 0; i < triangle_vertex_count; i++)
            {
                // x and y will be >= 0 so we can use truncate instead of floor for speed
                auto current_min_x = static_cast<std::int32_t>(framebuffer_coordinates[i].x);
                auto current_min_y = static_cast<std::int32_t>(framebuffer_coordinates[i].y);
                std::int32_t current_end_x = current_min_x + 1;
                std::int32_t current_end_y = current_min_y + 1;
                if(first || current_min_x < min_x)
                    min_x = current_min_x;
                if(first || current_end_x > end_x)
                    end_x = current_end_x;
                if(first || current_min_y < min_y)
                    min_y = current_min_y;
                if(first || current_end_y > end_y)
                    end_y = current_end_y;
                first = false;
            }
            if(min_x < clipped_scissor_rect_min.x)
                min_x = clipped_scissor_rect_min.x;
            if(end_x > clipped_scissor_rect_end.x)
                end_x = clipped_scissor_rect_end.x;
            if(min_y < clipped_scissor_rect_min.y)
                min_y = clipped_scissor_rect_min.y;
            if(end_y > clipped_scissor_rect_end.y)
                end_y = clipped_scissor_rect_end.y;
            constexpr int log2_scale = 16;
            constexpr auto scale = 1LL << log2_scale;
            typedef std::int64_t Edge_equation_integer_type;
            struct Edge_equation
            {
                Edge_equation_integer_type a;
                Edge_equation_integer_type b;
                Edge_equation_integer_type c;
                Edge_equation_integer_type padding;
                constexpr Edge_equation() noexcept : a(), b(), c(), padding()
                {
                }
                constexpr Edge_equation(Edge_equation_integer_type a,
                                        Edge_equation_integer_type b,
                                        Edge_equation_integer_type c) noexcept : a(a),
                                                                                 b(b),
                                                                                 c(c),
                                                                                 padding()
                {
                }
                constexpr bool inside(std::int32_t x, std::int32_t y) const noexcept
                {
                    return a * x + b * y + c >= 0;
                }
            };
            Edge_equation edge_equations[triangle_vertex_count];
            bool skip_triangle = false;
            for(std::size_t start_vertex_index = 0, end_vertex_index = 1, other_vertex_index = 2;
                start_vertex_index < triangle_vertex_count;
                start_vertex_index++)
            {
                float x1_float = framebuffer_coordinates[start_vertex_index].x;
                float y1_float = framebuffer_coordinates[start_vertex_index].y;
                float x2_float = framebuffer_coordinates[end_vertex_index].x;
                float y2_float = framebuffer_coordinates[end_vertex_index].y;
                [[gnu::unused]] float x3_float = framebuffer_coordinates[other_vertex_index].x;
                [[gnu::unused]] float y3_float = framebuffer_coordinates[other_vertex_index].y;
                auto x1_fixed = static_cast<Edge_equation_integer_type>(x1_float * scale);
                auto y1_fixed = static_cast<Edge_equation_integer_type>(y1_float * scale);
                auto x2_fixed = static_cast<Edge_equation_integer_type>(x2_float * scale);
                auto y2_fixed = static_cast<Edge_equation_integer_type>(y2_float * scale);
                [[gnu::unused]] auto x3_fixed =
                    static_cast<Edge_equation_integer_type>(x3_float * scale);
                [[gnu::unused]] auto y3_fixed =
                    static_cast<Edge_equation_integer_type>(y3_float * scale);
                Edge_equation_integer_type a;
                Edge_equation_integer_type b;
                Edge_equation_integer_type c;
                {
                    // solve a * x1 + b * y1 + c == 0 &&
                    // a * x2 + b * y2 + c == 0 &&
                    // a * x3 + b * y3 + c >= 0
                    if(x1_fixed == x2_fixed && y1_fixed == y2_fixed)
                    {
                        // rounded to a zero-area triangle
                        skip_triangle = true;
                        break;
                    }
                    Edge_equation_integer_type a_fixed = (y1_fixed - y2_fixed) * scale;
                    Edge_equation_integer_type b_fixed = (x2_fixed - x1_fixed) * scale;
                    Edge_equation_integer_type c_fixed =
                        (x1_fixed * y2_fixed - x2_fixed * y1_fixed);

                    // offset to end up checking at pixel center instead of top-left pixel corner
                    c_fixed += (a_fixed + b_fixed) / 2;

                    a = a_fixed;
                    b = b_fixed;
                    c = c_fixed;
                    if(orientation > 0)
                    {
                        // fix sign
                        a = -a;
                        b = -b;
                        c = -c;
                    }
                }
                // handle top-left fill rule
                if(a < 0 || (a == 0 && b < 0))
                {
                    // not a top-left edge, fixup c
                    // effectively changes the '>=' to '>' in Edge_equation::inside
                    c--;
                }

                edge_equations[start_vertex_index] = Edge_equation(a, b, c);
                if(++end_vertex_index >= triangle_vertex_count)
                    end_vertex_index = 0;
                if(++other_vertex_index >= triangle_vertex_count)
                    other_vertex_index = 0;
            }
            if(skip_triangle)
                continue;
            auto fs = this->fragment_shader_function;
            for(std::int32_t y = min_y; y < end_y; y++)
            {
                for(std::int32_t x = min_x; x < end_x; x++)
                {
                    bool inside = true;
                    for(auto &edge_equation : edge_equations)
                    {
                        inside &= edge_equation.inside(x, y);
                    }
                    if(inside)
                    {
                        auto *pixel = reinterpret_cast<Pixel_type *>(
                            color_attachment_memory
                            + (static_cast<std::size_t>(x) * color_attachment_pixel_size
                               + static_cast<std::size_t>(y) * color_attachment_stride));
                        fs(pixel);
                    }
                }
            }
        };
    }
}

std::unique_ptr<Graphics_pipeline> Graphics_pipeline::make(
    Pipeline_cache *pipeline_cache, const VkGraphicsPipelineCreateInfo &create_info)
{
    assert(create_info.sType == VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO);
    auto *render_pass = Render_pass_handle::from_handle(create_info.renderPass);
    assert(render_pass);
    auto *pipeline_layout = Pipeline_layout_handle::from_handle(create_info.layout);
    assert(pipeline_layout);
    if(create_info.flags & VK_PIPELINE_CREATE_DERIVATIVE_BIT)
    {
#warning implement creating derived pipelines
        throw std::runtime_error("creating derived pipelines is not implemented");
    }
    auto implementation = std::make_shared<Implementation>();
    auto optimization_level = ::LLVMCodeGenLevelDefault;
    if(create_info.flags & VK_PIPELINE_CREATE_DISABLE_OPTIMIZATION_BIT)
        optimization_level = ::LLVMCodeGenLevelNone;
    auto llvm_target_machine =
        llvm_wrapper::Target_machine::create_native_target_machine(optimization_level);
    implementation->compiled_shaders.reserve(create_info.stageCount);
    util::Enum_set<spirv::Execution_model> found_shader_stages;
    for(std::size_t i = 0; i < create_info.stageCount; i++)
    {
        auto &stage_info = create_info.pStages[i];
        auto execution_models =
            vulkan::get_execution_models_from_shader_stage_flags(stage_info.stage);
        assert(execution_models.size() == 1);
        auto execution_model = *execution_models.begin();
        bool added_to_found_shader_stages =
            std::get<1>(found_shader_stages.insert(execution_model));
        if(!added_to_found_shader_stages)
            throw std::runtime_error("duplicate shader stage");
        auto *shader_module = Shader_module_handle::from_handle(stage_info.module);
        assert(shader_module);
        {
            spirv::Dump_callbacks dump_callbacks;
            try
            {
                spirv::parse(dump_callbacks, shader_module->words(), shader_module->word_count());
            }
            catch(spirv::Parser_error &e)
            {
                std::cerr << dump_callbacks.ss.str() << std::endl;
                throw;
            }
            std::cerr << dump_callbacks.ss.str() << std::endl;
        }
        auto compiled_shader = spirv_to_llvm::spirv_to_llvm(implementation->llvm_context.get(),
                                                            llvm_target_machine.get(),
                                                            shader_module->words(),
                                                            shader_module->word_count(),
                                                            implementation->compiled_shaders.size(),
                                                            execution_model,
                                                            stage_info.pName);
        std::cerr << "Translation to LLVM succeeded." << std::endl;
        ::LLVMDumpModule(compiled_shader.module.get());
        bool failed =
            ::LLVMVerifyModule(compiled_shader.module.get(), ::LLVMPrintMessageAction, nullptr);
        if(failed)
            throw std::runtime_error("LLVM module verification failed");
        implementation->compiled_shaders.push_back(std::move(compiled_shader));
    }
    implementation->data_layout = llvm_target_machine.create_target_data_layout();
    implementation->jit_stack =
        llvm_wrapper::Orc_compile_stack::create(std::move(llvm_target_machine), optimize_module);
    Vertex_shader_function vertex_shader_function = nullptr;
    std::size_t vertex_shader_output_struct_size = 0;
    util::optional<std::size_t> vertex_shader_position_output_offset;
    Fragment_shader_function fragment_shader_function = nullptr;
    for(auto &compiled_shader : implementation->compiled_shaders)
    {
        implementation->jit_stack.add_eagerly_compiled_ir(
            std::move(compiled_shader.module),
            &spirv_to_llvm::Jit_symbol_resolver::resolve,
            static_cast<void *>(&implementation->jit_symbol_resolver));
        auto shader_entry_point_address = implementation->jit_stack.get_symbol_address(
            compiled_shader.entry_function_name.c_str());
        std::cerr << "shader entry: " << compiled_shader.entry_function_name << ": "
                  << reinterpret_cast<void *>(shader_entry_point_address) << std::endl;
        assert(shader_entry_point_address);
        switch(compiled_shader.execution_model)
        {
        case spirv::Execution_model::fragment:
            fragment_shader_function =
                reinterpret_cast<Fragment_shader_function>(shader_entry_point_address);
#warning finish implementing Graphics_pipeline::make
            continue;
#warning finish implementing Graphics_pipeline::make
            throw std::runtime_error("creating fragment shaders is not implemented");
        case spirv::Execution_model::geometry:
#warning finish implementing Graphics_pipeline::make
            throw std::runtime_error("creating geometry shaders is not implemented");
        case spirv::Execution_model::gl_compute:
        case spirv::Execution_model::kernel:
            throw std::runtime_error("can't create compute shaders from Graphics_pipeline::make");
        case spirv::Execution_model::tessellation_control:
        case spirv::Execution_model::tessellation_evaluation:
#warning finish implementing Graphics_pipeline::make
            throw std::runtime_error("creating tessellation shaders is not implemented");
        case spirv::Execution_model::vertex:
        {
            vertex_shader_function =
                reinterpret_cast<Vertex_shader_function>(shader_entry_point_address);
            implementation->vertex_shader_output_struct = compiled_shader.outputs_struct;
            auto llvm_vertex_shader_output_struct =
                implementation->vertex_shader_output_struct->get_or_make_type().type;
            vertex_shader_output_struct_size = ::LLVMABISizeOfType(
                implementation->data_layout.get(), llvm_vertex_shader_output_struct);
            for(auto &member : implementation->vertex_shader_output_struct->get_members(true))
            {
                for(auto &decoration : member.decorations)
                {
                    if(decoration.value == spirv::Decoration::built_in)
                    {
                        auto &builtin =
                            util::get<spirv::Decoration_built_in_parameters>(decoration.parameters);
                        if(builtin.built_in == spirv::Built_in::position)
                        {
                            vertex_shader_position_output_offset =
                                ::LLVMOffsetOfElement(implementation->data_layout.get(),
                                                      llvm_vertex_shader_output_struct,
                                                      member.llvm_member_index);
                            break;
                        }
                    }
                }
                if(vertex_shader_position_output_offset)
                    break;
                if(auto *struct_type =
                       dynamic_cast<spirv_to_llvm::Struct_type_descriptor *>(member.type.get()))
                {
                    std::size_t struct_offset =
                        ::LLVMOffsetOfElement(implementation->data_layout.get(),
                                              llvm_vertex_shader_output_struct,
                                              member.llvm_member_index);
                    auto llvm_struct_type = struct_type->get_or_make_type().type;
                    for(auto &submember : struct_type->get_members(true))
                    {
                        for(auto &decoration : submember.decorations)
                        {
                            if(decoration.value == spirv::Decoration::built_in)
                            {
                                auto &builtin = util::get<spirv::Decoration_built_in_parameters>(
                                    decoration.parameters);
                                if(builtin.built_in == spirv::Built_in::position)
                                {
                                    vertex_shader_position_output_offset =
                                        struct_offset
                                        + ::LLVMOffsetOfElement(implementation->data_layout.get(),
                                                                llvm_struct_type,
                                                                submember.llvm_member_index);
                                    break;
                                }
                            }
                        }
                        if(vertex_shader_position_output_offset)
                            break;
                    }
                }
                if(vertex_shader_position_output_offset)
                    break;
            }
            if(!vertex_shader_position_output_offset)
                throw std::runtime_error("can't find vertex shader Position output");
#warning finish implementing Graphics_pipeline::make
            continue;
        }
        }
        throw std::runtime_error("unknown shader kind");
    }
#warning finish implementing Graphics_pipeline::make
    if(!vertex_shader_function)
        throw std::runtime_error("graphics pipeline doesn't have vertex shader");
    if(!create_info.pViewportState)
        throw std::runtime_error("missing viewport state");
    if(create_info.pViewportState->viewportCount != 1)
        throw std::runtime_error("unimplemented viewport count");
    if(!create_info.pViewportState->pViewports)
        throw std::runtime_error("missing viewport list");
    if(!create_info.pViewportState->pScissors)
        throw std::runtime_error("missing scissor rectangle list");
    assert(vertex_shader_position_output_offset);
    return std::unique_ptr<Graphics_pipeline>(
        new Graphics_pipeline(std::move(implementation),
                              vertex_shader_function,
                              vertex_shader_output_struct_size,
                              *vertex_shader_position_output_offset,
                              fragment_shader_function,
                              create_info.pViewportState->pViewports[0],
                              create_info.pViewportState->pScissors[0]));
}
}
}
