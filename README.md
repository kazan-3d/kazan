# Kazan

[![Build Status](https://travis-ci.org/kazan-3d/kazan.svg?branch=master)](https://travis-ci.org/kazan-3d/kazan)

Work-in-progress for Vulkan implementation on cpu

[TODO list](docs/todo.md)

[Documentation](docs)

## Build in Docker

    docker build -t kazan .

## Build under Ubuntu 16.04 (xenial)

    sudo apt install git clang-4.0 build-essential cmake ninja-build llvm-4.0-dev libsdl2-dev curl imagemagick libxcb-shm0-dev libxcb1-dev libx11-dev libx11-xcb-dev
    git clone https://github.com/kazan-3d/kazan.git
    cd kazan
    mkdir build
    cd build
    cmake .. -G Ninja -DCMAKE_CXX_COMPILER=clang++-4.0 -DCMAKE_C_COMPILER=clang-4.0 -DCMAKE_BUILD_TYPE=Debug
    ninja

## Naming

Kazan used to be named vulkan-cpu. Kazan is a Japanese word that means "volcano".
