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
include(TestBigEndian)
TEST_BIG_ENDIAN(is_big_endian)
if(is_big_endian)
set(UTIL_ENDIAN Big)
else()
set(UTIL_ENDIAN Little)
endif()
configure_file(endian_config.h.in endian_config.h ESCAPE_QUOTES)
set(sources bit_intrinsics.cpp
            bitset.cpp
            copy_cv_ref.cpp
            filesystem.cpp
            in_place.cpp
            invoke.cpp
            is_referenceable.cpp
            is_swappable.cpp
            optional.cpp
            soft_float.cpp
            string_view.cpp
            variant.cpp
            void_t.cpp)
add_library(util STATIC ${sources})
target_include_directories(util PUBLIC ${CMAKE_CURRENT_BINARY_DIR}/..)
add_executable(util_test EXCLUDE_FROM_ALL ${sources} util_test.cpp)
target_include_directories(util_test PUBLIC ${CMAKE_CURRENT_BINARY_DIR}/..)
