# Vulkan-cpu

[![Build Status](https://travis-ci.org/programmerjake/vulkan-cpu.svg?branch=master)](https://travis-ci.org/programmerjake/vulkan-cpu)

Work-in-progress for Vulkan implementation on cpu

## Build in Docker

    docker build -t vulkan-cpu .

## Build under Ubuntu 16.04 (xenial)

    sudo apt install build-essential git clang-4.0 llvm-4.0-dev cmake zlib1g-dev libsdl2-dev
    git clone https://github.com/programmerjake/vulkan-cpu.git
    cd vulkan-cpu
    mkdir build
    cd build
    cmake .. -DCMAKE_CXX_COMPILER="`which clang++-4.0`" -DCMAKE_C_COMPILER="`which clang-4.0`" -DCMAKE_BUILD_TYPE=Debug
    make
