# SPDX-License-Identifier: LGPL-2.1-or-later
# Copyright 2018 Jacob Lifshay
FROM rust:stretch
# Note that APT_KEY_DONT_WARN_ON_DANGEROUS_USAGE makes apt-key ignore the output not being a terminal
RUN set -e; \
    printf "deb http://apt.llvm.org/stretch/ llvm-toolchain-stretch-7 main\ndeb-src http://apt.llvm.org/stretch/ llvm-toolchain-stretch-7 main" > /etc/apt/sources.list.d/llvm.list; \
    (wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | APT_KEY_DONT_WARN_ON_DANGEROUS_USAGE=1 apt-key add -) 2>&1; \
    apt-get update; \
    apt-get install -y \
        clang-7 \
        libclang-7-dev \
        cmake \
        ninja-build \
        libgl1-mesa-dev \
        libxcb-shm0 \
        ; \
    rm -rf /var/lib/apt/lists/*
WORKDIR /build
RUN version=1.1.85.0; wget -O vulkansdk.tar.gz -nv "https://sdk.lunarg.com/sdk/download/1.1.85.0/linux/vulkansdk-linux-x86_64-$version.tar.gz" && tar -xaf vulkansdk.tar.gz && rm vulkansdk.tar.gz && mv "$version" vulkansdk
ENV VULKAN_SDK=/build/vulkansdk/x86_64
ENV PATH="$VULKAN_SDK/bin:$PATH" LD_LIBRARY_PATH="$VULKAN_SDK/lib:" VK_LAYER_PATH="$VULKAN_SDK/etc/explicit_layer.d"
WORKDIR /build/kazan
COPY run-cts.sh run-cts.sh
RUN ./run-cts.sh --update-only
COPY external/ external/
COPY Cargo.toml Cargo.toml
COPY vulkan-driver/Cargo.toml vulkan-driver/build.rs vulkan-driver/vulkan-wrapper.h vulkan-driver/
RUN set -e; \
    mkdir -p vulkan-driver/src; \
    echo "// empty" > vulkan-driver/src/lib.rs; \
    cargo build; \
    rm vulkan-driver/src/lib.rs
COPY . .
RUN touch -c vulkan-driver/src/lib.rs && cargo build
CMD ["./run-cts.sh", "--no-update"]
