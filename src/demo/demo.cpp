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
#include <SDL.h>
#include "spirv/spirv.h"
#include "spirv/parser.h"
#include "util/optional.h"
#include "util/string_view.h"
#include "pipeline/pipeline.h"
#include "vulkan/vulkan.h"

#if SDL_MAJOR_VERSION != 2
#error wrong SDL varsion
#endif

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

pipeline::Shader_module_handle load_shader(const char *filename)
{
    std::cerr << "loading " << filename << std::endl;
    auto file = load_file(filename);
    if(!file)
        throw std::runtime_error("loading shader failed: " + std::string(filename));
    dump_words(*file);
    std::cerr << std::endl;
    VkShaderModuleCreateInfo shader_module_create_info = {
        .sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
        .pNext = nullptr,
        .flags = 0,
        .codeSize = file->size() * sizeof(spirv::Word),
        .pCode = file->data(),
    };
    return pipeline::Shader_module_handle::make(shader_module_create_info);
}

pipeline::Pipeline_layout_handle make_pipeline_layout()
{
    VkPipelineLayoutCreateInfo pipeline_layout_create_info = {
        .sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
        .pNext = nullptr,
        .flags = 0,
        .setLayoutCount = 0,
        .pSetLayouts = nullptr,
        .pushConstantRangeCount = 0,
        .pPushConstantRanges = nullptr,
    };
    return pipeline::Pipeline_layout_handle::make(pipeline_layout_create_info);
}

int test_main(int argc, char **argv)
{
    const char *vertex_shader_filename = "test-files/tri.vert.spv";
    const char *fragment_shader_filename = "test-files/tri.frag.spv";
    if(argc > 1)
    {
        if(argc != 3 || argv[1][0] == '-' || argv[2][0] == '-')
        {
            std::cerr << "usage: demo [<file.vert.spv> <file.frag.spv>]\n";
            return 1;
        }
        vertex_shader_filename = argv[1];
        fragment_shader_filename = argv[2];
    }
    try
    {
        auto vertex_shader = load_shader(vertex_shader_filename);
        auto fragment_shader = load_shader(fragment_shader_filename);
        auto pipeline_layout = make_pipeline_layout();
        constexpr std::size_t main_color_attachment_index = 0;
        constexpr std::size_t attachment_count = main_color_attachment_index + 1;
        VkAttachmentDescription attachments[attachment_count] = {};
        attachments[main_color_attachment_index] = VkAttachmentDescription{
            .flags = 0,
            .format = VK_FORMAT_B8G8R8A8_UNORM,
            .samples = VK_SAMPLE_COUNT_1_BIT,
            .loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR,
            .storeOp = VK_ATTACHMENT_STORE_OP_STORE,
            .stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE,
            .stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE,
            .initialLayout = VK_IMAGE_LAYOUT_UNDEFINED,
            .finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
        };
        constexpr std::size_t color_attachment_count = 1;
        VkAttachmentReference color_attachment_references[color_attachment_count] = {
            {
                .attachment = main_color_attachment_index,
                .layout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
            },
        };
        constexpr std::size_t subpass_count = 1;
        VkSubpassDescription subpass_descriptions[subpass_count] = {
            {
                .flags = 0,
                .pipelineBindPoint = VK_PIPELINE_BIND_POINT_GRAPHICS,
                .inputAttachmentCount = 0,
                .pInputAttachments = nullptr,
                .colorAttachmentCount = color_attachment_count,
                .pColorAttachments = color_attachment_references,
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
            .attachmentCount = attachment_count,
            .pAttachments = attachments,
            .subpassCount = subpass_count,
            .pSubpasses = subpass_descriptions,
            .dependencyCount = 0,
            .pDependencies = nullptr,
        };
        auto render_pass = pipeline::Render_pass_handle::make(render_pass_create_info);
        constexpr std::size_t stage_index_vertex = 0;
        constexpr std::size_t stage_index_fragment = stage_index_vertex + 1;
        constexpr std::size_t stage_count = stage_index_fragment + 1;
        VkPipelineShaderStageCreateInfo stages[stage_count] = {};
        stages[stage_index_vertex] = VkPipelineShaderStageCreateInfo{
            .sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            .pNext = nullptr,
            .flags = 0,
            .stage = VK_SHADER_STAGE_VERTEX_BIT,
            .module = pipeline::to_handle(vertex_shader.get()),
            .pName = "main",
            .pSpecializationInfo = nullptr,
        };
        stages[stage_index_fragment] = VkPipelineShaderStageCreateInfo{
            .sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            .pNext = nullptr,
            .flags = 0,
            .stage = VK_SHADER_STAGE_FRAGMENT_BIT,
            .module = pipeline::to_handle(fragment_shader.get()),
            .pName = "main",
            .pSpecializationInfo = nullptr,
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
            .topology = VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
            .primitiveRestartEnable = false,
        };
        static constexpr std::size_t window_width = 640;
        static constexpr std::size_t window_height = 480;
        static constexpr std::size_t viewport_count = 1;
        VkViewport viewports[viewport_count] = {
            {
                .x = 0,
                .y = 0,
                .width = window_width,
                .height = window_height,
                .minDepth = 0,
                .maxDepth = 1,
            },
        };
        VkRect2D scissors[viewport_count] = {
            {
                .offset =
                    {
                        .x = 0, .y = 0,
                    },
                .extent =
                    {
                        .width = window_width, .height = window_height,
                    },
            },
        };
        VkPipelineViewportStateCreateInfo pipeline_viewport_state_create_info = {
            .sType = VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            .pNext = nullptr,
            .flags = 0,
            .viewportCount = viewport_count,
            .pViewports = viewports,
            .scissorCount = viewport_count,
            .pScissors = scissors,
        };
        VkPipelineRasterizationStateCreateInfo pipeline_rasterization_state_create_info = {
            .sType = VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            .pNext = nullptr,
            .flags = 0,
            .depthClampEnable = false,
            .rasterizerDiscardEnable = false,
            .polygonMode = VK_POLYGON_MODE_FILL,
            .cullMode = VK_CULL_MODE_NONE,
            .frontFace = VK_FRONT_FACE_COUNTER_CLOCKWISE,
            .depthBiasEnable = false,
            .depthBiasConstantFactor = 0,
            .depthBiasClamp = 0,
            .depthBiasSlopeFactor = 0,
            .lineWidth = 1,
        };
        VkPipelineMultisampleStateCreateInfo pipeline_multisample_state_create_info = {
            .sType = VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            .pNext = nullptr,
            .flags = 0,
            .rasterizationSamples = VK_SAMPLE_COUNT_1_BIT,
            .sampleShadingEnable = false,
            .minSampleShading = 1,
            .pSampleMask = nullptr,
            .alphaToCoverageEnable = false,
            .alphaToOneEnable = false,
        };
        VkPipelineColorBlendAttachmentState color_blend_attachment_states[color_attachment_count] =
            {
                {
                    .blendEnable = false,
                    .srcColorBlendFactor = VK_BLEND_FACTOR_SRC_COLOR,
                    .dstColorBlendFactor = VK_BLEND_FACTOR_ZERO,
                    .colorBlendOp = VK_BLEND_OP_ADD,
                    .srcAlphaBlendFactor = VK_BLEND_FACTOR_SRC_ALPHA,
                    .dstAlphaBlendFactor = VK_BLEND_FACTOR_ZERO,
                    .alphaBlendOp = VK_BLEND_OP_ADD,
                    .colorWriteMask = VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT
                                      | VK_COLOR_COMPONENT_B_BIT
                                      | VK_COLOR_COMPONENT_A_BIT,
                },
            };
        VkPipelineColorBlendStateCreateInfo pipeline_color_blend_state_create_info = {
            .sType = VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            .pNext = nullptr,
            .flags = 0,
            .logicOpEnable = false,
            .logicOp = VK_LOGIC_OP_COPY,
            .attachmentCount = color_attachment_count,
            .pAttachments = color_blend_attachment_states,
            .blendConstants =
                {
                    0, 0, 0, 0,
                },
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
            .pViewportState = &pipeline_viewport_state_create_info,
            .pRasterizationState = &pipeline_rasterization_state_create_info,
            .pMultisampleState = &pipeline_multisample_state_create_info,
            .pDepthStencilState = nullptr,
            .pColorBlendState = &pipeline_color_blend_state_create_info,
            .pDynamicState = nullptr,
            .layout = pipeline::to_handle(pipeline_layout.get()),
            .renderPass = pipeline::to_handle(render_pass.get()),
            .subpass = 0,
            .basePipelineHandle = VK_NULL_HANDLE,
            .basePipelineIndex = -1,
        };
        auto graphics_pipeline =
            pipeline::Graphics_pipeline::make(nullptr, graphics_pipeline_create_info);
    }
    catch(std::runtime_error &e)
    {
        std::cerr << "error: " << e.what() << std::endl;
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
