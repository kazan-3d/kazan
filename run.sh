#!/bin/sh
set -e
cargo build
export VK_ICD_FILENAMES="$(realpath "$(ls --sort=time target/debug/build/vulkan-driver-*/out/kazan_driver.json | head -n 1)")"
exec "$@"
