// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
use crate::api;
use crate::handle::Handle;
use crate::swapchain::{SurfaceImplementation, SurfacePlatform, Swapchain};
use std::borrow::Cow;
use std::ptr::NonNull;
use crate::xcb_swapchain::{XcbSurfaceImplementation, XcbSwapchain};
use xcb;

const XCB_SURFACE_IMPLEMENTATION: XcbSurfaceImplementation = XcbSurfaceImplementation;

#[derive(Debug)]
pub struct XlibSurfaceImplementation;

impl XlibSurfaceImplementation {
    unsafe fn get_surface(&self, surface: api::VkSurfaceKHR) -> &api::VkIcdSurfaceXlib {
        let surface = surface.get().unwrap().as_ptr();
        assert_eq!((*surface).platform, api::VK_ICD_WSI_PLATFORM_XLIB);
        #[allow(clippy::cast_ptr_alignment)]
        &*(surface as *const api::VkIcdSurfaceXlib)
    }

    unsafe fn surface_to_xcb(&self, xlib_surface: &api::VkIcdSurfaceXlib) -> api::VkIcdSurfaceXcb {
        api::VkIcdSurfaceXcb {
            base: api::VkIcdSurfaceBase {
                platform: api::VK_ICD_WSI_PLATFORM_XCB,
            },
            connection: xcb::ffi::xlib_xcb::XGetXCBConnection(xlib_surface.dpy as *mut _),
            window: xlib_surface.window as u32,
        }
    }
}

impl SurfaceImplementation for XlibSurfaceImplementation {
    fn get_platform(&self) -> SurfacePlatform {
        SurfacePlatform::VK_ICD_WSI_PLATFORM_XLIB
    }

    unsafe fn get_surface_formats(
        &self,
        surface: api::VkSurfaceKHR,
    ) -> Result<Cow<'static, [api::VkSurfaceFormatKHR]>, api::VkResult> {
        let xlib_surface = self.get_surface(surface);
        let mut xcb_surface = self.surface_to_xcb(xlib_surface);
        let surface = api::VkSurfaceKHR::new(Some(NonNull::new_unchecked(&mut xcb_surface as *mut api::VkIcdSurfaceXcb as *mut _)));
        XCB_SURFACE_IMPLEMENTATION.get_surface_formats(surface)
    }

    unsafe fn get_present_modes(
        &self,
        surface: api::VkSurfaceKHR,
    ) -> Result<Cow<'static, [api::VkPresentModeKHR]>, api::VkResult> {
        let xlib_surface = self.get_surface(surface);
        let mut xcb_surface = self.surface_to_xcb(xlib_surface);
        let surface = api::VkSurfaceKHR::new(Some(NonNull::new_unchecked(&mut xcb_surface as *mut api::VkIcdSurfaceXcb as *mut _)));
        XCB_SURFACE_IMPLEMENTATION.get_present_modes(surface)
    }

    unsafe fn get_capabilities(
        &self,
        surface: api::VkSurfaceKHR,
    ) -> Result<api::VkSurfaceCapabilitiesKHR, api::VkResult> {
        let xlib_surface = self.get_surface(surface);
        let mut xcb_surface = self.surface_to_xcb(xlib_surface);
        let surface = api::VkSurfaceKHR::new(Some(NonNull::new_unchecked(&mut xcb_surface as *mut api::VkIcdSurfaceXcb as *mut _)));
        XCB_SURFACE_IMPLEMENTATION.get_capabilities(surface)
    }
    unsafe fn build(
        &self,
        create_info: &api::VkSwapchainCreateInfoKHR,
        device_group_create_info: Option<&api::VkDeviceGroupSwapchainCreateInfoKHR>,
    ) -> Result<Box<Swapchain>, api::VkResult> {
        // We can just create an XCB swapchain, since the xlib surface is build on top of XCB.
        Ok(Box::new(XcbSwapchain::new(
            create_info,
            device_group_create_info,
        )?))
    }
    unsafe fn destroy_surface(&self, surface: NonNull<api::VkIcdSurfaceBase>) {
        #[allow(clippy::cast_ptr_alignment)]
        Box::from_raw(surface.as_ptr() as *mut api::VkIcdSurfaceXlib);
    }

    fn duplicate(&self) -> Box<dyn SurfaceImplementation> {
        Box::new(Self {})
    }
}
