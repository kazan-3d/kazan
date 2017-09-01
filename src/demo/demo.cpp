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
#include <cctype>
#include <SDL.h>
#include "spirv/spirv.h"
#include "spirv/parser.h"
#include "util/optional.h"
#include "util/string_view.h"
#include "pipeline/pipeline.h"
#include "vulkan/vulkan.h"
#include "util/void_t.h"

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

template <typename Integer_type>
util::optional<Integer_type> parse_unsigned_integer(
    util::string_view str,
    Integer_type max_value = std::numeric_limits<Integer_type>::max()) noexcept
{
    static_assert(std::is_unsigned<Integer_type>::value, "");
    if(str.empty())
        return {};
    Integer_type retval = 0;
    for(char ch : str)
    {
        if(ch < '0' || ch > '9')
            return {};
        unsigned ch_value = ch - '0';
        if(retval > max_value / 10 || (retval == max_value / 10 && ch_value > max_value % 10))
            return {};
        retval *= 10;
        retval += ch_value;
    }
    return retval;
}

template <typename Integer_type>
util::optional<Integer_type> parse_signed_integer(
    util::string_view str,
    Integer_type max_value = std::numeric_limits<Integer_type>::max(),
    Integer_type min_value = std::numeric_limits<Integer_type>::min()) noexcept
{
    assert(max_value >= min_value);
    static_assert(std::is_signed<Integer_type>::value, "");
    typedef typename std::make_unsigned<Integer_type>::type Unsigned_type;
    bool is_negative = false;
    if(str.empty())
        return {};
    if(str.front() == '+')
    {
        str.remove_prefix(1);
    }
    else if(str.front() == '-')
    {
        is_negative = true;
        str.remove_prefix(1);
    }
    if(str.empty())
        return {};
    Unsigned_type unsigned_max;
    if(is_negative)
    {
        if(min_value > 0)
            return {};
        unsigned_max = -static_cast<Unsigned_type>(min_value);
    }
    else
    {
        if(max_value < 0)
            return {};
        unsigned_max = static_cast<Unsigned_type>(max_value);
    }
    auto unsigned_retval = parse_unsigned_integer(str, unsigned_max);
    if(!unsigned_retval)
        return {};
    Integer_type retval;
    if(is_negative)
    {
        retval = static_cast<Integer_type>(-*unsigned_retval);
        if(retval > max_value)
            return {};
    }
    else
    {
        retval = static_cast<Integer_type>(*unsigned_retval);
        if(retval < min_value)
            return {};
    }
    return retval;
}

struct Vertex_input_struct
{
    struct alignas(4 * alignof(float)) Vec4
    {
        float x, y, z, w;
        static constexpr VkFormat format = VK_FORMAT_R32G32B32A32_SFLOAT;
    };
    Vec4 position;
    static constexpr VkFormat position_format = decltype(position)::format;
    static constexpr std::size_t position_location = 0; // must match tri.vert
    constexpr Vertex_input_struct() noexcept : position{}
    {
    }
    constexpr explicit Vertex_input_struct(const Vec4 &position) noexcept : position(position)
    {
    }
};

struct Wavefront_obj_parse_error : public std::runtime_error
{
    const char *filename;
    std::size_t line_number;
    static std::string make_what(const char *filename,
                                 std::size_t line_number,
                                 util::string_view message)
    {
        std::ostringstream ss;
        ss << filename << ":" << line_number << ": error: " << message;
        return ss.str();
    }
    Wavefront_obj_parse_error(const char *filename,
                              std::size_t line_number,
                              util::string_view message)
        : runtime_error(make_what(filename, line_number, message)),
          filename(filename),
          line_number(line_number)
    {
    }
};

std::vector<Vertex_input_struct> load_wavefront_obj_file(const char *filename)
{
    std::vector<Vertex_input_struct> retval;
    std::ifstream is(filename);
    if(!is)
        throw Wavefront_obj_parse_error(filename, 0, "failed to open file");
    std::size_t line_number = 0;
    struct Vertex
    {
        float x = 0, y = 0, z = 0;
    };
    std::vector<Vertex> vertexes;
    struct Texture_vertex
    {
        float u = 0, v = 0;
    };
    std::vector<Texture_vertex> texture_vertexes;
    struct Normal_vertex
    {
        float x = 0, y = 0, z = 0;
    };
    std::vector<Normal_vertex> normal_vertexes;
    while(is)
    {
        line_number++;
        std::string line_str;
        std::getline(is, line_str);
        for(char &ch : line_str)
        {
            // replace all white space with a space character
            if(std::isspace(static_cast<unsigned char>(ch)))
                ch = ' ';
        }
        util::string_view line(line_str);
        // skip leading whitespace
        while(!line.empty() && line.front() == ' ')
            line.remove_prefix(1);
        // skip trailing whitespace
        while(!line.empty() && line.back() == ' ')
            line.remove_suffix(1);
        // skip blank lines and comments
        if(line.empty() || line.front() == '#')
            continue;
        auto command = line.substr(0, line.find_first_of(' '));
        line.remove_prefix(command.size());
        // skip leading whitespace
        while(!line.empty() && line.front() == ' ')
            line.remove_prefix(1);
        if(command == "v") // vertex
        {
            std::istringstream is;
            is.str(std::string(line));
            float x = 0, y = 0, z = 0;
            is >> x >> y >> z;
            if(!is || is.peek() != std::char_traits<char>::eof())
                throw Wavefront_obj_parse_error(
                    filename, line_number, "parsing vertex command failed");
            vertexes.push_back({
                .x = x, .y = y, .z = z,
            });
        }
        else if(command == "vn") // vertex normal
        {
            std::istringstream is;
            is.str(std::string(line));
            float x = 0, y = 0, z = 0;
            is >> x >> y >> z;
            if(!is || is.peek() != std::char_traits<char>::eof())
                throw Wavefront_obj_parse_error(
                    filename, line_number, "parsing vertex normal command failed");
            normal_vertexes.push_back({
                .x = x, .y = y, .z = z,
            });
        }
        else if(command == "vt") // vertex texture
        {
            std::istringstream is;
            is.str(std::string(line));
            float u = 0, v = 0;
            is >> u >> v;
            if(!is || is.peek() != std::char_traits<char>::eof())
                throw Wavefront_obj_parse_error(
                    filename, line_number, "parsing vertex texture command failed");
            texture_vertexes.push_back({
                .u = u, .v = v,
            });
        }
        else if(command == "s" && line == "off")
        {
            // smoothing groups are not implemented, so turning smoothing off has no effect
        }
        else if(command == "f")
        {
            struct Face_vertex
            {
                Vertex vertex;
                util::optional<Texture_vertex> texture_vertex;
                util::optional<Normal_vertex> normal_vertex;
            };
            std::vector<Face_vertex> face_vertexes;
            while(!line.empty())
            {
                auto vertex_str = line.substr(0, line.find_first_of(' '));
                line.remove_prefix(vertex_str.size());
                // skip leading whitespace
                while(!line.empty() && line.front() == ' ')
                    line.remove_prefix(1);
                assert(!vertex_str.empty());
                auto vertex_index_str = vertex_str.substr(0, vertex_str.find_first_of('/'));
                vertex_str.remove_prefix(vertex_index_str.size());
                util::string_view vertex_texture_index_str;
                util::string_view vertex_normal_index_str;
                if(!vertex_str.empty())
                {
                    vertex_str.remove_prefix(1); // remove slash
                    vertex_texture_index_str = vertex_str.substr(0, vertex_str.find_first_of('/'));
                    vertex_str.remove_prefix(vertex_texture_index_str.size());
                    if(!vertex_str.empty())
                    {
                        vertex_str.remove_prefix(1); // remove slash
                        vertex_normal_index_str = vertex_str;
                    }
                }
                auto parse_vertex_index = [filename, line_number](
                    auto &vertexes,
                    util::string_view vertex_index_str,
                    const char *vertex_index_name)
                {
                    std::size_t vertex_count = vertexes.size();
                    std::int64_t max_index = vertex_count;
                    std::int64_t min_index = -vertex_count;
                    auto vertex_index =
                        parse_signed_integer(vertex_index_str, max_index, min_index);
                    if(!vertex_index || *vertex_index == 0)
                        throw Wavefront_obj_parse_error(filename,
                                                        line_number,
                                                        std::string("invalid ") + vertex_index_name
                                                            + ": "
                                                            + std::string(vertex_index_str));
                    if(*vertex_index < 0)
                        *vertex_index += vertex_count;
                    else
                        (*vertex_index)--;
                    assert(*vertex_index >= 0 && *vertex_index < vertex_count);
                    return vertexes[*vertex_index];
                };
                auto vertex = parse_vertex_index(vertexes, vertex_index_str, "vertex index");
                util::optional<Texture_vertex> texture_vertex;
                if(!vertex_texture_index_str.empty())
                    texture_vertex = parse_vertex_index(
                        texture_vertexes, vertex_texture_index_str, "vertex texture index");
                util::optional<Normal_vertex> normal_vertex;
                if(!vertex_normal_index_str.empty())
                    normal_vertex = parse_vertex_index(
                        normal_vertexes, vertex_normal_index_str, "vertex normal index");
                face_vertexes.push_back({
                    .vertex = vertex,
                    .texture_vertex = texture_vertex,
                    .normal_vertex = normal_vertex,
                });
            }
            if(face_vertexes.size() < 3)
                throw Wavefront_obj_parse_error(
                    filename, line_number, "faces must have at least 3 vertexes");
            std::vector<Vertex_input_struct> transformed_vertexes;
            transformed_vertexes.reserve(face_vertexes.size());
            for(auto &face_vertex : face_vertexes)
            {
                // convert from obj coordinate system to OpenGL coordinate system
                float global_x = face_vertex.vertex.x;
                float global_y = -face_vertex.vertex.z;
                float global_z = face_vertex.vertex.y;
                // camera transformation
                float camera_x = global_x;
                float camera_y = global_y;
                float camera_z = global_z - 1;
                // perspective project
                constexpr float far_plane = 10;
                constexpr float factor = 1 / far_plane;
                float projected_x = factor * camera_x;
                float projected_y = -factor * camera_y;
                float projected_z = -factor * camera_z;
                float projected_w = -factor * camera_z;
                // fix aspect ratio
                constexpr float x_aspect_ratio_correction = 3.0 / 4;
                constexpr float y_aspect_ratio_correction = 1;
                float final_x = projected_x * x_aspect_ratio_correction;
                float final_y = projected_y * y_aspect_ratio_correction;
                float final_z = projected_z;
                float final_w = projected_w;
                transformed_vertexes.push_back(Vertex_input_struct({
                    .x = final_x, .y = final_y, .z = final_z, .w = final_w,
                }));
            }
            // triangulate face
            for(std::size_t leading_vertex_index = 2, trailing_vertex_index = 1;
                leading_vertex_index < transformed_vertexes.size();
                leading_vertex_index++, trailing_vertex_index++)
            {
                retval.push_back(transformed_vertexes[0]);
                retval.push_back(transformed_vertexes[trailing_vertex_index]);
                retval.push_back(transformed_vertexes[leading_vertex_index]);
            }
        }
#warning finish implementing load_wavefront_obj_file
        else
            throw Wavefront_obj_parse_error(
                filename, line_number, "unimplemented command: " + std::string(command));
    }
    return retval;
}

int test_main(int argc, char **argv)
{
    if(SDL_Init(0) < 0)
    {
        std::cerr << "SDL_Init failed: " << SDL_GetError() << std::endl;
        return 1;
    }
    struct Shutdown_sdl
    {
        ~Shutdown_sdl()
        {
            SDL_Quit();
        }
    } shutdown_sdl;
    const char *vertex_shader_filename = "test-files/tri.vert.spv";
    const char *fragment_shader_filename = "test-files/tri.frag.spv";
    const char *vertexes_filename = "test-files/demo-text.obj";
    if(argc > 1)
    {
        if(argc != 4 || argv[1][0] == '-' || argv[2][0] == '-' || argv[3][0] == '-')
        {
            std::cerr << "usage: demo [<file.vert.spv> <file.frag.spv> <vertexes.obj>]\n";
            return 1;
        }
        vertex_shader_filename = argv[1];
        fragment_shader_filename = argv[2];
        vertexes_filename = argv[3];
    }
    try
    {
        auto vertex_shader = load_shader(vertex_shader_filename);
        auto fragment_shader = load_shader(fragment_shader_filename);
        auto vertexes = load_wavefront_obj_file(vertexes_filename);
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
        constexpr std::size_t vertex_input_buffer_binding_index = 0;
        constexpr std::size_t binding_count = vertex_input_buffer_binding_index + 1;
        constexpr std::size_t vertex_input_binding_description_count = 1;
        VkVertexInputBindingDescription
            vertex_input_binding_descriptions[vertex_input_binding_description_count] = {
                {
                    .binding = vertex_input_buffer_binding_index,
                    .stride = sizeof(Vertex_input_struct),
                    .inputRate = VK_VERTEX_INPUT_RATE_VERTEX,
                },
            };
        constexpr std::size_t vertex_input_attribute_description_count = 1;
        VkVertexInputAttributeDescription
            vertex_input_attribute_descriptions[vertex_input_attribute_description_count] = {
                {
                    .location = Vertex_input_struct::position_location,
                    .binding = vertex_input_buffer_binding_index,
                    .format = Vertex_input_struct::position_format,
                    .offset = offsetof(Vertex_input_struct, position),
                },
            };
        VkPipelineVertexInputStateCreateInfo pipeline_vertex_input_state_create_info = {
            .sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            .pNext = nullptr,
            .flags = 0,
            .vertexBindingDescriptionCount = vertex_input_binding_description_count,
            .pVertexBindingDescriptions = vertex_input_binding_descriptions,
            .vertexAttributeDescriptionCount = vertex_input_attribute_description_count,
            .pVertexAttributeDescriptions = vertex_input_attribute_descriptions,
        };
        VkPipelineInputAssemblyStateCreateInfo pipeline_input_assembly_state_create_info = {
            .sType = VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            .pNext = nullptr,
            .flags = 0,
            .topology = VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
            .primitiveRestartEnable = false,
        };
        static constexpr std::size_t window_width = 1024;
        static_assert(window_width % 4 == 0, "");
        static constexpr std::size_t window_height = window_width * 3ULL / 4;
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
        VkImageCreateInfo image_create_info = {
            .sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
            .pNext = nullptr,
            .flags = 0,
            .imageType = VK_IMAGE_TYPE_2D,
            .format = VK_FORMAT_B8G8R8A8_UNORM,
            .extent =
                {
                    .width = window_width, .height = window_height, .depth = 1,
                },
            .mipLevels = 1,
            .arrayLayers = 1,
            .samples = VK_SAMPLE_COUNT_1_BIT,
            .tiling = VK_IMAGE_TILING_LINEAR,
            .usage = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT | VK_IMAGE_USAGE_TRANSFER_SRC_BIT,
            .sharingMode = VK_SHARING_MODE_EXCLUSIVE,
            .queueFamilyIndexCount = 0,
            .pQueueFamilyIndices = nullptr,
            .initialLayout = VK_IMAGE_LAYOUT_UNDEFINED,
        };
        image::Image color_attachment(image::Image_descriptor(image_create_info),
                                      image::allocate_memory_tag);
        VkClearColorValue clear_color;
        // set clear_color to opaque gray
        clear_color.float32[0] = 0.25;
        clear_color.float32[1] = 0.25;
        clear_color.float32[2] = 0.25;
        clear_color.float32[3] = 1;
        color_attachment.clear(clear_color);
        constexpr std::uint32_t vertex_start_index = 0;
        std::uint32_t vertex_end_index = vertexes.size();
        constexpr std::uint32_t instance_id = 0;
        void *bindings[binding_count] = {
            vertexes.data(),
        };
        graphics_pipeline->run(
            vertex_start_index, vertex_end_index, instance_id, color_attachment, bindings);
        typedef std::uint32_t Pixel_type;
        // check Pixel_type
        static_assert(std::is_void<util::void_t<decltype(graphics_pipeline->run_fragment_shader(
                          static_cast<Pixel_type *>(nullptr)))>>::value,
                      "");
        auto rgba = [](std::uint8_t r,
                       std::uint8_t g,
                       std::uint8_t b,
                       std::uint8_t a) noexcept->Pixel_type
        {
            union
            {
                Pixel_type retval;
                std::uint8_t bytes[4];
            };
            static_assert(sizeof(retval) == sizeof(bytes), "");
            bytes[0] = b;
            bytes[1] = g;
            bytes[2] = r;
            bytes[3] = a;
            return retval;
        };
        constexpr std::size_t bits_per_pixel = 32;
        struct Surface_deleter
        {
            void operator()(SDL_Surface *v) const noexcept
            {
                ::SDL_FreeSurface(v);
            }
        };
        std::unique_ptr<SDL_Surface, Surface_deleter> surface(
            SDL_CreateRGBSurfaceFrom(color_attachment.memory.get(),
                                     window_width,
                                     window_height,
                                     bits_per_pixel,
                                     color_attachment.descriptor.get_memory_stride(),
                                     rgba(0xFF, 0, 0, 0),
                                     rgba(0, 0xFF, 0, 0),
                                     rgba(0, 0, 0xFF, 0),
                                     rgba(0, 0, 0, 0xFF)));
        if(!surface)
            throw std::runtime_error(std::string("SDL_CreateRGBSurfaceFrom failed: ")
                                     + SDL_GetError());
        const char *output_file = "output.bmp";
        if(SDL_SaveBMP(surface.get(), output_file) < 0)
            throw std::runtime_error(std::string("SDL_SaveBMP failed: ") + SDL_GetError());
        std::cerr << "saved output image to " << output_file << std::endl;
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
