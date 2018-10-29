# Kazan

Kazan is an in-progress Vulkan driver that supports cross-platform software rendering, and (eventually) is a driver for [libre-riscv.org's RISC-V based GPU](https://libre-riscv.org/3d_gpu/).

## License

Kazan is licensed under the LGPL, v2.1 or later. See [LICENSE.md](https://salsa.debian.org/Kazan-team/kazan/blob/master/LICENSE.md) for details.

Kazan uses third-party software, which has their own licenses.

## Code of Conduct

As part of Kazan being hosted by [Debian](https://www.debian.org/), it is important to follow the [Debian Code of Conduct](https://www.debian.org/code_of_conduct) when participating in Kazan's development.

## Branches

The [`master` branch](https://salsa.debian.org/Kazan-team/kazan/tree/master) is the new Rust version. The previous C++ version is in the [`kazan-old` branch](https://salsa.debian.org/Kazan-team/kazan/tree/kazan-old). The version as of the end of GSOC 2017 is in the [`gsoc-2017` tag](https://salsa.debian.org/Kazan-team/kazan/tree/gsoc-2017).

## Building using Docker

* Clone Git repo:
      git clone --recursive https://salsa.debian.org/Kazan-team/kazan.git

* Build:
      cd kazan
      docker build -t kazan-cts .
  If the build fails due to Out-of-Memory, reduce the number of processes run simultaneously by passing `--cpuset-cpus=<cpu-numbers>` to only use the specified CPUs:
      # only use CPUs 0, 1, and 2
      docker build --cpuset-cpus=0,1,2 -t kazan-cts .

## Building on Ubuntu 18.04

* Install Rust via [rustup](https://rustup.rs/):
      curl https://sh.rustup.rs -sSf | sh
  You need to restart your shell after installing Rust so `PATH` gets set correctly.

* Install required packages:
      sudo apt-get install cmake ninja-build libgl1-mesa-dev libxcb-shm0 libclang-dev clang build-essential git

* Clone Git repo:
      git clone --recursive https://salsa.debian.org/Kazan-team/kazan.git

* Build using Cargo:
      cd kazan
      cargo build -vv
  Building using `-vv` is recommended because the build process builds LLVM and it doesn't show LLVM's build progress unless using `-vv`.

* Run unit tests:
      cargo test

* Run a program using the built driver:
      ./run.sh <program-name> [<args> ...]
  For example, to run `vulkaninfo`:
      ./run.sh vulkaninfo

* Run the Vulkan Conformance Test Suite (CTS):
  * Build and run the CTS:
        ./run-cts.sh
  * Only build the CTS:
        ./run-cts.sh --update-only
  * Run the CTS without trying to rebuild:
        ./run-cts.sh --no-update
  The CTS is known to fail the `dEQP-VK.api.version_check.entry_points` test due to a bug in the `libvulkan1.so` that comes packaged in Ubuntu 18.04. That test should pass when using the `libvulkan1.so` from the Vulkan SDK.

## News

### We've moved! - 2018-10-23

Kazan's new canonical location is [salsa.debian.org/Kazan-team/kazan](https://salsa.debian.org/Kazan-team/kazan).

The [Kazan GitHub repository](https://github.com/kazan-3d/kazan) is now used as a read-only mirror.

Kazan's domain names, [kazan-3d.org](http://kazan-3d.org/) and [kazan.graphics](http://kazan.graphics), now redirect to Kazan's canonical location on Debian Salsa.
