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
cmake_minimum_required(VERSION 3.3 FATAL_ERROR)

set(sources core_instructions.cpp
            fragment_entry_point.cpp
            glsl_instructions.cpp
            opencl_instructions.cpp
            parser_callbacks_annotations.cpp
            parser_callbacks_capabilities.cpp
            parser_callbacks_debug.cpp
            parser_callbacks_extensions.cpp
            spirv_to_llvm.cpp
            translator.cpp
            vertex_entry_point.cpp)
add_library(kazan_spirv_to_llvm STATIC ${sources})
target_link_libraries(kazan_spirv_to_llvm
                      kazan_util
                      kazan_spirv
                      kazan_llvm_wrapper
                      kazan_pipeline
                      kazan_vulkan)
