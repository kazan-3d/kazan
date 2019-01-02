// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::const_static_lifetime)]
#![allow(clippy::unreadable_literal)]
pub use crate::handle::{
    VkBuffer, VkBufferView, VkCommandBuffer, VkCommandPool, VkDebugReportCallbackEXT,
    VkDebugUtilsMessengerEXT, VkDescriptorPool, VkDescriptorSet, VkDescriptorSetLayout,
    VkDescriptorUpdateTemplate, VkDevice, VkDeviceMemory, VkDisplayKHR, VkDisplayModeKHR, VkEvent,
    VkFence, VkFramebuffer, VkImage, VkImageView, VkInstance, VkPhysicalDevice, VkPipeline,
    VkPipelineCache, VkPipelineLayout, VkQueryPool, VkQueue, VkRenderPass, VkSampler,
    VkSamplerYcbcrConversion, VkSemaphore, VkShaderModule, VkSurfaceKHR, VkSwapchainKHR,
    VkValidationCacheEXT,
};
#[cfg(target_os = "linux")]
use xcb::ffi::{xcb_connection_t, xcb_visualid_t, xcb_window_t};
include!(concat!(env!("OUT_DIR"), "/vulkan-types.rs"));

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VkDebugReportCallbackCreateInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkDebugReportFlagsEXT,
    pub pfnCallback: PFN_vkDebugReportCallbackEXT,
    pub pUserData: *mut ::std::os::raw::c_void,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VkDebugUtilsMessengerCreateInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkDebugUtilsMessengerCreateFlagsEXT,
    pub messageSeverity: VkDebugUtilsMessageSeverityFlagsEXT,
    pub messageType: VkDebugUtilsMessageTypeFlagsEXT,
    pub pfnUserCallback: PFN_vkDebugUtilsMessengerCallbackEXT,
    pub pUserData: *mut ::std::os::raw::c_void,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VkAllocationCallbacks {
    pub pUserData: *mut ::std::os::raw::c_void,
    pub pfnAllocation: PFN_vkAllocationFunction,
    pub pfnReallocation: PFN_vkReallocationFunction,
    pub pfnFree: PFN_vkFreeFunction,
    pub pfnInternalAllocation: PFN_vkInternalAllocationNotification,
    pub pfnInternalFree: PFN_vkInternalFreeNotification,
}
