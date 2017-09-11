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

set(CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} "${CMAKE_SOURCE_DIR}/cmake")

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

project(xcb-present-test CXX C)
add_compile_options(-Wall -ftemplate-depth=1024)
add_executable(main main.cpp)

find_package(SDL2 REQUIRED)
string(STRIP ${SDL2_LIBRARIES} libraries) # workaround SDL2 automake bug
target_include_directories(main PRIVATE ${SDL2_INCLUDE_DIRS})
target_link_libraries(main ${libraries})

find_package(XCB REQUIRED)
target_include_directories(main PRIVATE ${XCB_INCLUDE_DIRS})
target_link_libraries(main ${XCB_LIBRARIES})

find_package(X11_XCB REQUIRED)
target_include_directories(main PRIVATE ${X11_XCB_INCLUDE_DIRS})
target_link_libraries(main ${X11_XCB_LIBRARIES})


