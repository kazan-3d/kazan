#!/bin/bash
# SPDX-License-Identifier: LGPL-2.1-or-later
# Copyright 2018 Jacob Lifshay

set -e

do_update=1
if [[ "$*" == '--no-update' ]]; then
    do_update=0
elif [[ "$*" != '' ]]; then
    printf "unknown arguments\nusage: %s [--no-update]\n" "$0" >&2
    exit 1
fi

cts_source="$(realpath VK-GL-CTS)"

if [[ ! -d "$cts_source" ]]; then
    if ((do_update)); then
        git clone "https://github.com/KhronosGroup/VK-GL-CTS"
    else
        echo "need to run without --no-update" >&2
        exit 1
    fi
fi
cts_build="$(realpath VK-GL-CTS/build)"
if ((do_update)); then
    (
        cd "$cts_source"
        git pull
        python2 external/fetch_sources.py
    )
fi
if [[ ! -d "$cts_build" ]]; then
    if ((do_update)); then
        (
            mkdir "$cts_build"
            cd "$cts_build"
            cmake -G Ninja -DCMAKE_BUILD_TYPE=Debug ..
        )
    else
        echo "need to run without --no-update" >&2
        exit 1
    fi
fi
(
    cd "$cts_build"
    ninja
)
exec ./run.sh bash -c "cd '$cts_build'/external/vulkancts/modules/vulkan; exec ./deqp-vk --deqp-caselist-file='$cts_source'/external/vulkancts/mustpass/1.1.3/vk-default.txt --deqp-log-images=disable --deqp-log-shader-sources=disable"
