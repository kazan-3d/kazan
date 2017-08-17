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
#include "vulkan/util.h"
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

struct Graphics_pipeline::Implementation
{
    spirv_to_llvm::Jit_symbol_resolver jit_symbol_resolver;
    llvm_wrapper::Orc_jit_stack jit_stack;
    llvm_wrapper::Context llvm_context = llvm_wrapper::Context::create();
    std::uint64_t next_module_id = 1;
    std::uint64_t make_module_id() noexcept
    {
        return next_module_id++;
    }
};

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
    auto llvm_target_machine = llvm_wrapper::Target_machine::create_native_target_machine();
    std::vector<spirv_to_llvm::Converted_module> compiled_shaders;
    compiled_shaders.reserve(create_info.stageCount);
    for(std::size_t i = 0; i < create_info.stageCount; i++)
    {
        auto &stage_info = create_info.pStages[i];
        auto execution_models =
            vulkan::get_execution_models_from_shader_stage_flags(stage_info.stage);
        assert(execution_models.size() == 1);
        auto execution_model = *execution_models.begin();
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
                                                            implementation->make_module_id(),
                                                            execution_model,
                                                            stage_info.pName);
        std::cerr << "Translation to LLVM succeeded." << std::endl;
        ::LLVMDumpModule(compiled_shader.module.get());
        bool failed =
            ::LLVMVerifyModule(compiled_shader.module.get(), ::LLVMPrintMessageAction, nullptr);
        if(failed)
            throw std::runtime_error("LLVM module verification failed");
        compiled_shaders.push_back(std::move(compiled_shader));
    }
    auto data_layout = llvm_target_machine.create_target_data_layout();
    implementation->jit_stack = llvm_wrapper::Orc_jit_stack::create(std::move(llvm_target_machine));
    Vertex_shader_function vertex_shader_function = nullptr;
    std::size_t vertex_shader_output_struct_size = 0;
    for(auto &compiled_shader : compiled_shaders)
    {
        vertex_shader_output_struct_size = implementation->jit_stack.add_eagerly_compiled_ir(
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
            vertex_shader_output_struct_size = ::LLVMABISizeOfType(
                data_layout.get(), compiled_shader.outputs_struct->get_or_make_type().type);
#warning finish implementing Graphics_pipeline::make
            continue;
        }
        }
        throw std::runtime_error("unknown shader kind");
    }
#warning finish implementing Graphics_pipeline::make
    if(!vertex_shader_function)
        throw std::runtime_error("graphics pipeline doesn't have vertex shader");
    return std::unique_ptr<Graphics_pipeline>(new Graphics_pipeline(
        std::move(implementation), vertex_shader_function, vertex_shader_output_struct_size));
}
}
}
