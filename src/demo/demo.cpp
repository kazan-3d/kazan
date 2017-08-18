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
#include <fstream>
#include <iostream>
#include <sstream>
#include <vector>
#include <array>
#include <type_traits>
#include <string>
#include "spirv/spirv.h"
#include "spirv/parser.h"
#include "util/optional.h"
#include "util/string_view.h"
#include "pipeline/pipeline.h"
#include "vulkan/vulkan.h"

namespace vulkan_cpu
{
namespace test
{
util::optional<std::vector<spirv::Word>> load_file(const char *filename)
{
    using spirv::Word;
    constexpr int eof = std::char_traits<char>::eof();
    std::ifstream is;
    is.open(filename, std::ios::in | std::ios::binary);
    if(!is)
        return {};
    constexpr std::size_t block_size = 0x1000;
    std::vector<std::array<Word, block_size>> blocks;
    std::array<unsigned char, sizeof(Word)> word_bytes{};
    static_assert(sizeof(Word) == 4, "");
    static_assert(std::is_same<std::uint8_t, unsigned char>::value, "");
    auto read_little_endian = [](const unsigned char *bytes) -> Word
    {
        Word retval = bytes[3];
        retval <<= 8;
        retval |= bytes[2];
        retval <<= 8;
        retval |= bytes[1];
        retval <<= 8;
        retval |= bytes[0];
        return retval;
    };
    auto read_big_endian = [](const unsigned char *bytes) -> Word
    {
        Word retval = bytes[0];
        retval <<= 8;
        retval |= bytes[1];
        retval <<= 8;
        retval |= bytes[2];
        retval <<= 8;
        retval |= bytes[3];
        return retval;
    };
    for(unsigned char &byte : word_bytes)
    {
        auto v = is.get();
        if(v == eof)
            return {};
        byte = v;
    }
    Word (*read_word_fn)(const unsigned char *) = nullptr;
    if(read_little_endian(word_bytes.data()) == spirv::magic_number)
        read_word_fn = read_little_endian;
    else if(read_big_endian(word_bytes.data()) == spirv::magic_number)
        read_word_fn = read_big_endian;
    else
        return {};
    std::size_t word_count = 1;
    blocks.emplace_back();
    blocks[0][0] = read_word_fn(word_bytes.data());
    std::size_t word_in_block_index = 1;
    while(is.peek() != eof)
    {
        for(unsigned char &byte : word_bytes)
        {
            auto v = is.get();
            if(v == eof)
                return {};
            byte = v;
        }
        blocks.back()[word_in_block_index++] = read_word_fn(word_bytes.data());
        word_count++;
        if(word_in_block_index >= block_size)
        {
            word_in_block_index = 0;
            blocks.emplace_back();
        }
    }
    std::vector<Word> retval;
    retval.reserve(word_count);
    word_in_block_index = 0;
    for(std::size_t word_index = 0, block_index = 0; word_index < word_count; word_index++)
    {
        retval.push_back(blocks[block_index][word_in_block_index++]);
        if(word_in_block_index >= block_size)
        {
            word_in_block_index = 0;
            block_index++;
        }
    }
    return std::move(retval);
}

void dump_words(const spirv::Word *words, std::size_t word_count)
{
    constexpr std::size_t max_words_per_line = 4;
    auto old_fill = std::cerr.fill('0');
    auto old_flags = std::cerr.flags(std::ios::uppercase | std::ios::hex | std::ios::right);
    auto old_width = std::cerr.width();
    bool wrote_line_beginning = false;
    bool wrote_line_ending = true;
    std::cerr << "Words:\n";
    std::size_t current_words_per_line = 0;
    std::size_t index;
    auto seperator = "";
    auto write_line_beginning = [&]()
    {
        std::cerr.width(8);
        std::cerr << index << ":";
        seperator = " ";
        wrote_line_beginning = true;
        wrote_line_ending = false;
    };
    std::string chars = "";
    auto write_line_ending = [&]()
    {
        while(current_words_per_line < max_words_per_line)
        {
            std::cerr << seperator;
            seperator = " ";
            std::cerr.width(8);
            std::cerr.fill(' ');
            std::cerr << "";
            std::cerr.fill('0');
            current_words_per_line++;
        }
        std::cerr << seperator << " |" << chars << "|\n";
        seperator = "";
        wrote_line_ending = true;
        wrote_line_beginning = false;
        current_words_per_line = 0;
        chars.clear();
    };
    auto append_char = [&](unsigned ch)
    {
        if(ch >= 0x20U && ch < 0x7FU)
            chars += static_cast<char>(ch);
        else
            chars += '.';
    };
    auto write_word = [&](spirv::Word w)
    {
        std::cerr << seperator;
        seperator = " ";
        std::cerr.width(8);
        std::cerr << w;
        current_words_per_line++;
        append_char(w & 0xFFU);
        append_char((w >> 8) & 0xFFU);
        append_char((w >> 16) & 0xFFU);
        append_char((w >> 24) & 0xFFU);
    };
    for(index = 0; index < word_count; index++)
    {
        if(current_words_per_line >= max_words_per_line)
            write_line_ending();
        if(!wrote_line_beginning)
            write_line_beginning();
        write_word(words[index]);
    }
    if(!wrote_line_ending)
        write_line_ending();
    std::cerr.flush();
    std::cerr.width(old_width);
    std::cerr.fill(old_fill);
    std::cerr.flags(old_flags);
}

void dump_words(const std::vector<spirv::Word> &words)
{
    dump_words(words.data(), words.size());
}

int test_main(int argc, char **argv)
{
    const char *filename = "test-files/test.spv";
    if(argc > 1)
    {
        if(argv[1][0] == '-')
        {
            std::cerr << "usage: demo [<file.spv>]\n";
            return 1;
        }
        filename = argv[1];
    }
    std::cerr << "loading " << filename << std::endl;
    auto file = load_file(filename);
    if(file)
    {
        dump_words(*file);
        std::cerr << std::endl;
        try
        {
            VkShaderModuleCreateInfo shader_module_create_info = {
                .sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
                .pNext = nullptr,
                .flags = 0,
                .codeSize = file->size() * sizeof(spirv::Word),
                .pCode = file->data(),
            };
            auto shader_module = pipeline::Shader_module_handle::make(shader_module_create_info);
            VkPipelineLayoutCreateInfo pipeline_layout_create_info = {
                .sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
                .pNext = nullptr,
                .flags = 0,
                .setLayoutCount = 0,
                .pSetLayouts = nullptr,
                .pushConstantRangeCount = 0,
                .pPushConstantRanges = nullptr,
            };
            auto pipeline_layout =
                pipeline::Pipeline_layout_handle::make(pipeline_layout_create_info);
            constexpr std::size_t subpass_count = 1;
            VkSubpassDescription subpass_descriptions[subpass_count] = {
                {
                    .flags = 0,
                    .pipelineBindPoint = VK_PIPELINE_BIND_POINT_GRAPHICS,
                    .inputAttachmentCount = 0,
                    .pInputAttachments = nullptr,
                    .colorAttachmentCount = 0,
                    .pColorAttachments = nullptr,
                    .pResolveAttachments = nullptr,
                    .pDepthStencilAttachment = nullptr,
                    .preserveAttachmentCount = 0,
                    .pPreserveAttachments = nullptr,
                },
            };
            VkRenderPassCreateInfo render_pass_create_info = {
                .sType = VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
                .pNext = nullptr,
                .flags = 0,
                .attachmentCount = 0,
                .pAttachments = nullptr,
                .subpassCount = subpass_count,
                .pSubpasses = subpass_descriptions,
                .dependencyCount = 0,
                .pDependencies = nullptr,
            };
            auto render_pass = pipeline::Render_pass_handle::make(render_pass_create_info);
            constexpr std::size_t stage_count = 1;
            VkPipelineShaderStageCreateInfo stages[stage_count] = {
                {
                    .sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                    .pNext = nullptr,
                    .flags = 0,
                    .stage = VK_SHADER_STAGE_VERTEX_BIT,
                    .module = pipeline::to_handle(shader_module.get()),
                    .pName = "main",
                    .pSpecializationInfo = nullptr,
                },
            };
            VkPipelineVertexInputStateCreateInfo pipeline_vertex_input_state_create_info = {
                .sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
                .pNext = nullptr,
                .flags = 0,
                .vertexBindingDescriptionCount = 0,
                .pVertexBindingDescriptions = nullptr,
                .vertexAttributeDescriptionCount = 0,
                .pVertexAttributeDescriptions = nullptr,
            };
            VkPipelineInputAssemblyStateCreateInfo pipeline_input_assembly_state_create_info = {
                .sType = VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
                .pNext = nullptr,
                .flags = 0,
                .topology = VK_PRIMITIVE_TOPOLOGY_POINT_LIST,
                .primitiveRestartEnable = false,
            };
            VkPipelineRasterizationStateCreateInfo pipeline_rasterization_state_create_info = {
                .sType = VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
                .pNext = nullptr,
                .flags = 0,
                .depthClampEnable = false,
                .rasterizerDiscardEnable = true,
                .polygonMode = VK_POLYGON_MODE_FILL,
                .cullMode = VK_CULL_MODE_BACK_BIT,
                .frontFace = VK_FRONT_FACE_COUNTER_CLOCKWISE,
                .depthBiasEnable = false,
                .depthBiasConstantFactor = 0,
                .depthBiasClamp = 0,
                .depthBiasSlopeFactor = 0,
                .lineWidth = 1,
            };
            VkGraphicsPipelineCreateInfo graphics_pipeline_create_info = {
                .sType = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
                .pNext = nullptr,
                .flags = 0,
                .stageCount = stage_count,
                .pStages = stages,
                .pVertexInputState = &pipeline_vertex_input_state_create_info,
                .pInputAssemblyState = &pipeline_input_assembly_state_create_info,
                .pTessellationState = nullptr,
                .pViewportState = nullptr,
                .pRasterizationState = &pipeline_rasterization_state_create_info,
                .pMultisampleState = nullptr,
                .pDepthStencilState = nullptr,
                .pColorBlendState = nullptr,
                .pDynamicState = nullptr,
                .layout = pipeline::to_handle(pipeline_layout.get()),
                .renderPass = pipeline::to_handle(render_pass.get()),
                .subpass = 0,
                .basePipelineHandle = VK_NULL_HANDLE,
                .basePipelineIndex = -1,
            };
            auto graphics_pipeline =
                pipeline::Graphics_pipeline::make(nullptr, graphics_pipeline_create_info);
            std::cerr << "vertex_shader_output_struct_size: "
                      << graphics_pipeline->get_vertex_shader_output_struct_size() << std::endl;
            constexpr std::uint32_t vertex_start_index = 0;
            constexpr std::uint32_t vertex_end_index = 3;
            constexpr std::uint32_t instance_id = 0;
            constexpr std::size_t vertex_count = vertex_end_index - vertex_start_index;
            std::size_t output_buffer_size =
                graphics_pipeline->get_vertex_shader_output_struct_size() * vertex_count;
            std::unique_ptr<unsigned char[]> output_buffer(new unsigned char[output_buffer_size]);
            for(std::size_t i = 0; i < output_buffer_size; i++)
                output_buffer[i] = 0;
            graphics_pipeline->run_vertex_shader(
                vertex_start_index, vertex_end_index, instance_id, output_buffer.get());
            std::cerr << "shader completed" << std::endl;
            for(std::size_t i = 0; i < vertex_count; i++)
            {
                graphics_pipeline->dump_vertex_shader_output_struct(output_buffer.get()
                                                                    + graphics_pipeline->get_vertex_shader_output_struct_size() * i);
            }
        }
        catch(std::runtime_error &e)
        {
            std::cerr << "error: " << e.what() << std::endl;
            return 1;
        }
    }
    else
    {
        std::cerr << "error: can't load file" << std::endl;
        return 1;
    }
    return 0;
}
}
}

int main(int argc, char **argv)
{
    return vulkan_cpu::test::test_main(argc, argv);
}
