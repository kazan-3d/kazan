# Copyright 2017 Jacob Lifshay
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#
cmake_minimum_required(VERSION 3.1 FATAL_ERROR)

set(sources literal_string.cpp)

set(spirv_parser_generated_include_dir ${CMAKE_CURRENT_BINARY_DIR}/generated)
set(spirv_parser_generated_dir ${spirv_parser_generated_include_dir}/spirv)
set(spirv_parser_sources ${spirv_parser_generated_dir}/parser.cpp ${spirv_parser_generated_dir}/spirv.cpp)
set(spirv_parser_headers ${spirv_parser_generated_dir}/parser.h ${spirv_parser_generated_dir}/spirv.h)
set(spirv_grammar_json_dir ${CMAKE_CURRENT_SOURCE_DIR}/../khronos-spirv)
set(spirv_core_grammar_json ${spirv_grammar_json_dir}/spirv.core.grammar.json)
file(GLOB spirv_extension_instruction_set_json_files ${spirv_grammar_json_dir}/extinst.*.grammar.json)
message(STATUS "spirv_parser_generated_dir: " ${spirv_parser_generated_dir})
foreach(i ${spirv_extension_instruction_set_json_files})
    message(STATUS "spirv_extension_instruction_set_json_files: " ${i})
endforeach(i)

add_custom_command(OUTPUT ${spirv_parser_sources} ${spirv_parser_headers}
                   COMMAND ${CMAKE_COMMAND} -E make_directory ${spirv_parser_generated_dir}
                   COMMAND ${CMAKE_COMMAND} -E chdir ${spirv_parser_generated_include_dir} $<TARGET_FILE:generate_spirv_parser> ${spirv_grammar_json_dir} spirv
                   MAIN_DEPENDENCY ${spirv_core_grammar_json}
                   DEPENDS $<TARGET_FILE:generate_spirv_parser> ${spirv_extension_instruction_set_json_files}
                   VERBATIM
                   COMMENT "Generating SPIR-V Parser")
set(sources ${sources} ${spirv_parser_sources})
add_library(vulkan_cpu_spirv STATIC ${sources})
target_link_libraries(vulkan_cpu_spirv vulkan_cpu_util vulkan_cpu_json)
target_include_directories(vulkan_cpu_spirv PUBLIC ${spirv_parser_generated_include_dir})
