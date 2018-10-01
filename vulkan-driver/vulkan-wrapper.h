// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
#include <stdint.h>
#ifdef __ANDROID__
#error not supported on Android; need to fix ABI
#endif
#define VK_NO_PROTOTYPES
#include <vulkan/vulkan.h>
#include <vulkan/vk_icd.h>
#ifdef __unix
typedef struct xcb_connection_t xcb_connection_t;
typedef uint32_t xcb_visualid_t;
typedef uint32_t xcb_window_t;
#include <vulkan/vulkan_xcb.h>
#endif
#undef VK_NO_PROTOTYPES
