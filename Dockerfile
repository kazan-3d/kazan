# SPDX-License-Identifier: LGPL-2.1-or-later
# Copyright 2018 Jacob Lifshay
FROM rust:stretch
RUN set -e; \
    apt-get update; \
    apt-get install -y \
        cmake \
        ninja-build \
        libgl1-mesa-dev \
        libxcb-shm0 \
        libclang-dev \
        clang \
        ; \
    rm -rf /var/lib/apt/lists/*
WORKDIR /build
RUN version=1.1.85.0; wget -O vulkansdk.tar.gz -nv "https://sdk.lunarg.com/sdk/download/1.1.85.0/linux/vulkansdk-linux-x86_64-$version.tar.gz" && tar -xaf vulkansdk.tar.gz && rm vulkansdk.tar.gz && mv "$version" vulkansdk
ENV VULKAN_SDK=/build/vulkansdk/x86_64
ENV PATH="$VULKAN_SDK/bin:$PATH" LD_LIBRARY_PATH="$VULKAN_SDK/lib:" VK_LAYER_PATH="$VULKAN_SDK/etc/explicit_layer.d"
WORKDIR /build/kazan
COPY run-cts.sh run-cts.sh
RUN ./run-cts.sh --update-only
COPY . .
RUN cargo build -vv
CMD ["./run-cts.sh", "--no-update"]
