// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2019 Jacob Lifshay
#include <stdint.h>
#ifdef __ANDROID__
#error not supported on Android; need to fix ABI
#endif
#ifdef __unix
#define VK_USE_PLATFORM_XCB_KHR
#endif
#define VK_NO_PROTOTYPES
#include <vulkan/vulkan.h>
#include <vulkan/vk_icd.h>
#undef VK_NO_PROTOTYPES
#ifdef VK_USE_PLATFORM_XCB_KHR
#undef VK_USE_PLATFORM_XCB_KHR
#endif
