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
set(sources vulkan.cpp
            api_objects.cpp)
add_library(kazan_vulkan STATIC ${sources})
target_link_libraries(kazan_vulkan
                      kazan_spirv
                      Threads::Threads)
target_compile_definitions(kazan_vulkan PUBLIC VK_NO_PROTOTYPES)
if(UNIX AND NOT CYGWIN AND NOT CMAKE_SYSTEM_NAME STREQUAL "Android")
    set(USE_X11 1)
    # set(USE_WAYLAND 1) # wayland support is not implemented yet
elseif(WIN32)
    set(USE_WIN32 1)
endif()

if(USE_X11)
    target_compile_definitions(kazan_vulkan PUBLIC VK_USE_PLATFORM_XCB_KHR)
    target_compile_definitions(kazan_vulkan PUBLIC VK_USE_PLATFORM_XLIB_KHR)

    message("CMAKE_MODULE_PATH: " ${CMAKE_MODULE_PATH})

    find_package(XCB REQUIRED COMPONENTS xcb xcb-shm)
    target_include_directories(kazan_vulkan PUBLIC ${XCB_INCLUDE_DIRS})
    target_link_libraries(kazan_vulkan ${XCB_LIBRARIES})

    find_package(X11_XCB REQUIRED)
    target_include_directories(kazan_vulkan PUBLIC ${X11_XCB_INCLUDE_DIRS})
    target_link_libraries(kazan_vulkan ${X11_XCB_LIBRARIES})

    find_package(X11 REQUIRED)
    target_include_directories(kazan_vulkan PUBLIC ${X11_INCLUDE_DIRS})
    target_link_libraries(kazan_vulkan ${X11_LIBRARIES})
endif()

if(USE_WAYLAND)
    target_compile_definitions(kazan_vulkan PUBLIC VK_USE_PLATFORM_WAYLAND_KHR)
    find_package(Wayland REQUIRED)
    target_include_directories(kazan_vulkan PUBLIC ${WAYLAND_INCLUDE_DIR})
    target_link_libraries(kazan_vulkan ${WAYLAND_LIBRARIES})
    target_compile_definitions(kazan_vulkan ${WAYLAND_DEFINITIONS})
endif()

if(USE_WIN32)
    target_compile_definitions(kazan_vulkan PUBLIC VK_USE_PLATFORM_WIN32_KHR)
endif()
