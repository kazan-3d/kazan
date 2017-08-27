#!/bin/sh
set -e
mkdir -p build
cd build
cmake .. -G Ninja -DCMAKE_C_COMPILER=clang-4.0 -DCMAKE_CXX_COMPILER=clang++-4.0 -DCMAKE_BUILD_TYPE=Debug
ninja

