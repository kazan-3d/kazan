// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
pub use handle::{
    VkBuffer, VkBufferView, VkCommandBuffer, VkCommandPool, VkDebugReportCallbackEXT,
    VkDebugUtilsMessengerEXT, VkDescriptorPool, VkDescriptorSet, VkDescriptorSetLayout,
    VkDescriptorUpdateTemplate, VkDevice, VkDeviceMemory, VkDisplayKHR, VkDisplayModeKHR, VkEvent,
    VkFence, VkFramebuffer, VkImage, VkImageView, VkInstance, VkPhysicalDevice, VkPipeline,
    VkPipelineCache, VkPipelineLayout, VkQueryPool, VkQueue, VkRenderPass, VkSampler,
    VkSamplerYcbcrConversion, VkSemaphore, VkShaderModule, VkSurfaceKHR, VkSwapchainKHR,
    VkValidationCacheEXT,
};
#[cfg(unix)]
use xcb::ffi::{xcb_connection_t, xcb_visualid_t, xcb_window_t};
include!(concat!(env!("OUT_DIR"), "/vulkan-types.rs"));
