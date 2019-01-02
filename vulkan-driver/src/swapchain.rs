// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2019 Jacob Lifshay
use crate::api;
#[cfg(target_os = "linux")]
use crate::xcb_swapchain::XcbSurfaceImplementation;
use enum_map::Enum;
use std::any::Any;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{self, Debug};
use std::ptr::NonNull;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Enum)]
#[allow(non_camel_case_types)]
pub enum SurfacePlatform {
    VK_ICD_WSI_PLATFORM_WAYLAND,
    VK_ICD_WSI_PLATFORM_WIN32,
    VK_ICD_WSI_PLATFORM_XCB,
    VK_ICD_WSI_PLATFORM_XLIB,
    VK_ICD_WSI_PLATFORM_ANDROID,
    VK_ICD_WSI_PLATFORM_MACOS,
    VK_ICD_WSI_PLATFORM_IOS,
    VK_ICD_WSI_PLATFORM_DISPLAY,
}

#[derive(Debug)]
pub struct UnknownSurfacePlatform(pub api::VkIcdWsiPlatform);

impl fmt::Display for UnknownSurfacePlatform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown surface platform {:?}", self.0)
    }
}

impl Error for UnknownSurfacePlatform {}

impl SurfacePlatform {
    pub fn from(platform: api::VkIcdWsiPlatform) -> Result<Self, UnknownSurfacePlatform> {
        match platform {
            api::VK_ICD_WSI_PLATFORM_WAYLAND => Ok(SurfacePlatform::VK_ICD_WSI_PLATFORM_WAYLAND),
            api::VK_ICD_WSI_PLATFORM_WIN32 => Ok(SurfacePlatform::VK_ICD_WSI_PLATFORM_WIN32),
            api::VK_ICD_WSI_PLATFORM_XCB => Ok(SurfacePlatform::VK_ICD_WSI_PLATFORM_XCB),
            api::VK_ICD_WSI_PLATFORM_XLIB => Ok(SurfacePlatform::VK_ICD_WSI_PLATFORM_XLIB),
            api::VK_ICD_WSI_PLATFORM_ANDROID => Ok(SurfacePlatform::VK_ICD_WSI_PLATFORM_ANDROID),
            api::VK_ICD_WSI_PLATFORM_MACOS => Ok(SurfacePlatform::VK_ICD_WSI_PLATFORM_MACOS),
            api::VK_ICD_WSI_PLATFORM_IOS => Ok(SurfacePlatform::VK_ICD_WSI_PLATFORM_IOS),
            api::VK_ICD_WSI_PLATFORM_DISPLAY => Ok(SurfacePlatform::VK_ICD_WSI_PLATFORM_DISPLAY),
            platform => Err(UnknownSurfacePlatform(platform)),
        }
    }
    pub fn get_surface_implementation(self) -> Cow<'static, dyn SurfaceImplementation> {
        #[cfg(target_os = "linux")]
        const XCB_SURFACE_IMPLEMENTATION: XcbSurfaceImplementation = XcbSurfaceImplementation;
        match self {
            #[cfg(target_os = "linux")]
            SurfacePlatform::VK_ICD_WSI_PLATFORM_XCB => Cow::Borrowed(&XCB_SURFACE_IMPLEMENTATION),
            _ => Cow::Owned(FallbackSurfaceImplementation(self).duplicate()),
        }
    }
}

impl From<SurfacePlatform> for api::VkIcdWsiPlatform {
    fn from(platform: SurfacePlatform) -> api::VkIcdWsiPlatform {
        match platform {
            SurfacePlatform::VK_ICD_WSI_PLATFORM_WAYLAND => api::VK_ICD_WSI_PLATFORM_WAYLAND,
            SurfacePlatform::VK_ICD_WSI_PLATFORM_WIN32 => api::VK_ICD_WSI_PLATFORM_WIN32,
            SurfacePlatform::VK_ICD_WSI_PLATFORM_XCB => api::VK_ICD_WSI_PLATFORM_XCB,
            SurfacePlatform::VK_ICD_WSI_PLATFORM_XLIB => api::VK_ICD_WSI_PLATFORM_XLIB,
            SurfacePlatform::VK_ICD_WSI_PLATFORM_ANDROID => api::VK_ICD_WSI_PLATFORM_ANDROID,
            SurfacePlatform::VK_ICD_WSI_PLATFORM_MACOS => api::VK_ICD_WSI_PLATFORM_MACOS,
            SurfacePlatform::VK_ICD_WSI_PLATFORM_IOS => api::VK_ICD_WSI_PLATFORM_IOS,
            SurfacePlatform::VK_ICD_WSI_PLATFORM_DISPLAY => api::VK_ICD_WSI_PLATFORM_DISPLAY,
        }
    }
}

pub trait Swapchain: Any + Sync + Send + Debug {}

pub trait SurfaceImplementation: Any + Sync + Send + Debug {
    fn get_platform(&self) -> SurfacePlatform;
    unsafe fn get_surface_formats(
        &self,
        surface: api::VkSurfaceKHR,
    ) -> Result<Cow<'static, [api::VkSurfaceFormatKHR]>, api::VkResult>;
    unsafe fn get_present_modes(
        &self,
        surface: api::VkSurfaceKHR,
    ) -> Result<Cow<'static, [api::VkPresentModeKHR]>, api::VkResult>;
    unsafe fn get_capabilities(
        &self,
        surface: api::VkSurfaceKHR,
    ) -> Result<api::VkSurfaceCapabilitiesKHR, api::VkResult>;
    unsafe fn build(
        &self,
        create_info: &api::VkSwapchainCreateInfoKHR,
        device_group_create_info: Option<&api::VkDeviceGroupSwapchainCreateInfoKHR>,
    ) -> Result<Box<Swapchain>, api::VkResult>;
    unsafe fn destroy_surface(&self, surface: NonNull<api::VkIcdSurfaceBase>);
    fn duplicate(&self) -> Box<dyn SurfaceImplementation>;
}

impl ToOwned for dyn SurfaceImplementation {
    type Owned = Box<dyn SurfaceImplementation>;
    fn to_owned(&self) -> Box<dyn SurfaceImplementation> {
        self.duplicate()
    }
}

#[derive(Debug)]
pub struct FallbackSurfaceImplementation(SurfacePlatform);

impl FallbackSurfaceImplementation {
    pub fn report_error(&self) -> ! {
        panic!(
            "there is no surface implementation for {:?}",
            self.get_platform()
        )
    }
}

impl SurfaceImplementation for FallbackSurfaceImplementation {
    fn get_platform(&self) -> SurfacePlatform {
        self.0
    }
    unsafe fn get_surface_formats(
        &self,
        _surface: api::VkSurfaceKHR,
    ) -> Result<Cow<'static, [api::VkSurfaceFormatKHR]>, api::VkResult> {
        self.report_error()
    }
    unsafe fn get_present_modes(
        &self,
        _surface: api::VkSurfaceKHR,
    ) -> Result<Cow<'static, [api::VkPresentModeKHR]>, api::VkResult> {
        self.report_error()
    }
    unsafe fn get_capabilities(
        &self,
        _surface: api::VkSurfaceKHR,
    ) -> Result<api::VkSurfaceCapabilitiesKHR, api::VkResult> {
        self.report_error()
    }
    unsafe fn build(
        &self,
        _create_info: &api::VkSwapchainCreateInfoKHR,
        _device_group_create_info: Option<&api::VkDeviceGroupSwapchainCreateInfoKHR>,
    ) -> Result<Box<Swapchain>, api::VkResult> {
        self.report_error()
    }
    unsafe fn destroy_surface(&self, _surface: NonNull<api::VkIcdSurfaceBase>) {
        self.report_error()
    }
    fn duplicate(&self) -> Box<dyn SurfaceImplementation> {
        Box::new(FallbackSurfaceImplementation(self.0))
    }
}
