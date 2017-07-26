# Build under Ubuntu

```sudo apt install software-properties-common wget
wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add -
sudo apt-add-repository 'deb http://apt.llvm.org/xenial/ llvm-toolchain-xenial-4.0 main'
sudo apt update
sudo apt install build-essential git clang-4.0 llvm-4.0-dev cmake zlib1g-dev
git clone https://github.com/programmerjake/vulkan-cpu.git
cd vulkan-cpu
mkdir build
cd build
cmake .. -DCMAKE_CXX_COMPILER="`which clang++-4.0`" -DCMAKE_C_COMPILER="`which clang-4.0`" -DCMAKE_BUILD_TYPE=Debug
make
```