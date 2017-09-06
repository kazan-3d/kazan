# Kazan

[![Build Status](https://travis-ci.org/kazan-3d/kazan.svg?branch=master)](https://travis-ci.org/kazan-3d/kazan)

Work-in-progress for Vulkan implementation on cpu

[TODO list](docs/todo.md)

[Documentation](docs)

## Build in Docker

    docker build -t kazan .

## Build under Ubuntu 16.04 (xenial)

    sudo apt install build-essential git clang-4.0 llvm-4.0-dev cmake zlib1g-dev libsdl2-dev
    git clone https://github.com/kazan-3d/kazan.git
    cd kazan
    mkdir build
    cd build
    cmake .. -DCMAKE_CXX_COMPILER="`which clang++-4.0`" -DCMAKE_C_COMPILER="`which clang-4.0`" -DCMAKE_BUILD_TYPE=Debug
    make

## Naming

Kazan used to be named vulkan-cpu. Kazan is a Japanese word that means "volcano".
