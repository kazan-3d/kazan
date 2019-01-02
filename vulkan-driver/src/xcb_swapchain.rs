// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
use crate::api;
use crate::handle::Handle;
use crate::image::{ImageMultisampleCount, ImageProperties, SupportedTilings, Tiling};
use crate::swapchain::{SurfaceImplementation, SurfacePlatform, Swapchain};
use libc;
use std::borrow::Cow;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_char;
use std::ptr::null_mut;
use std::ptr::NonNull;
use xcb;

#[derive(Debug)]
pub struct XcbSwapchain {}

impl Swapchain for XcbSwapchain {}

struct ReplyObject<T>(NonNull<T>);

impl<T> ReplyObject<T> {
    unsafe fn from(v: *mut T) -> Option<Self> {
        NonNull::new(v).map(ReplyObject)
    }
}

impl<T> Deref for ReplyObject<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for ReplyObject<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}

impl<T> Drop for ReplyObject<T> {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.0.as_ptr() as *mut libc::c_void);
        }
    }
}

struct ServerObject<Id: 'static + Copy> {
    id: Id,
    connection: *mut xcb::ffi::xcb_connection_t,
    free_fn: unsafe extern "C" fn(
        connection: *mut xcb::ffi::xcb_connection_t,
        id: Id,
    ) -> xcb::ffi::xcb_void_cookie_t,
}

impl<Id: 'static + Copy> ServerObject<Id> {
    #[allow(dead_code)]
    fn get(&self) -> Id {
        self.id
    }
}

impl<Id: 'static + Copy> Drop for ServerObject<Id> {
    fn drop(&mut self) {
        unsafe {
            (self.free_fn)(self.connection, self.id);
        }
    }
}

type Gc = ServerObject<xcb::ffi::xcb_gcontext_t>;

unsafe fn create_gc(
    id: xcb::ffi::xcb_gcontext_t,
    connection: *mut xcb::ffi::xcb_connection_t,
) -> Gc {
    ServerObject {
        id,
        connection,
        free_fn: xcb::ffi::xcb_free_gc,
    }
}

type Pixmap = ServerObject<xcb::ffi::xcb_pixmap_t>;

#[allow(dead_code)]
unsafe fn create_pixmap(
    id: xcb::ffi::xcb_pixmap_t,
    connection: *mut xcb::ffi::xcb_connection_t,
) -> Pixmap {
    ServerObject {
        id,
        connection,
        free_fn: xcb::ffi::xcb_free_pixmap,
    }
}

type ShmSeg = ServerObject<xcb::ffi::shm::xcb_shm_seg_t>;

#[allow(dead_code)]
unsafe fn create_shm_seg(
    id: xcb::ffi::shm::xcb_shm_seg_t,
    connection: *mut xcb::ffi::xcb_connection_t,
) -> ShmSeg {
    ServerObject {
        id,
        connection,
        free_fn: xcb::ffi::shm::xcb_shm_detach,
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum SurfaceFormatGroup {
    R8G8B8A8,
    B8G8R8A8,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum SwapchainSetupError {
    BadSurface,
    NoSupport,
}

unsafe fn query_extension(
    connection: *mut xcb::ffi::xcb_connection_t,
    extension_name: &str,
) -> xcb::ffi::xcb_query_extension_cookie_t {
    let len = extension_name.len() as u16;
    assert_eq!(len as usize, extension_name.len());
    xcb::ffi::xcb_query_extension(connection, len, extension_name.as_ptr() as *const c_char)
}

pub const MAX_SWAPCHAIN_IMAGE_COUNT: u32 = 16;

#[allow(dead_code)]
struct SwapchainSetupFirstStage {
    gc: Gc,
    shm_supported: bool,
    window_depth: u8,
    surface_format_group: SurfaceFormatGroup,
    present_modes: &'static [api::VkPresentModeKHR],
    capabilities: api::VkSurfaceCapabilitiesKHR,
    shared_present_capabilities: Option<api::VkSharedPresentSurfaceCapabilitiesKHR>,
    image_pixel_size: usize,
    scanline_alignment: usize,
    shm_version: Option<xcb::ffi::shm::xcb_shm_query_version_cookie_t>,
    image_properties: ImageProperties,
}

impl SwapchainSetupFirstStage {
    unsafe fn new(
        connection: *mut xcb::ffi::xcb_connection_t,
        window: xcb::ffi::xcb_window_t,
        is_full_setup: bool,
    ) -> Result<Self, SwapchainSetupError> {
        #![allow(clippy::cast_lossless)]
        let has_mit_shm = query_extension(connection, "MIT-SHM");
        let geometry = xcb::ffi::xcb_get_geometry(connection, window);
        let window_attributes = xcb::ffi::xcb_get_window_attributes(connection, window);
        let tree = xcb::ffi::xcb_query_tree(connection, window);
        let gc = xcb::ffi::xcb_generate_id(connection);
        xcb::ffi::xcb_create_gc(
            connection,
            gc,
            window,
            xcb::ffi::XCB_GC_GRAPHICS_EXPOSURES,
            [0].as_ptr(),
        );
        let gc = create_gc(gc, connection);
        let has_mit_shm = ReplyObject::from(xcb::ffi::xcb_query_extension_reply(
            connection,
            has_mit_shm,
            null_mut(),
        ));
        let shm_supported = has_mit_shm.map(|v| v.present != 0).unwrap_or(false);
        let shm_version = if is_full_setup && shm_supported {
            Some(xcb::ffi::shm::xcb_shm_query_version(connection))
        } else {
            None
        };
        let geometry = ReplyObject::from(xcb::ffi::xcb_get_geometry_reply(
            connection,
            geometry,
            null_mut(),
        ))
        .ok_or(SwapchainSetupError::BadSurface)?;
        let image_extent = api::VkExtent2D {
            width: geometry.width as u32,
            height: geometry.height as u32,
        };
        mem::drop(geometry);
        let window_attributes = ReplyObject::from(xcb::ffi::xcb_get_window_attributes_reply(
            connection,
            window_attributes,
            null_mut(),
        ))
        .ok_or(SwapchainSetupError::BadSurface)?;
        let window_visual_id = window_attributes.visual;
        mem::drop(window_attributes);
        let tree = ReplyObject::from(xcb::ffi::xcb_query_tree_reply(connection, tree, null_mut()))
            .ok_or(SwapchainSetupError::BadSurface)?;
        let root_window = tree.root;
        mem::drop(tree);
        let mut screen = None;
        let mut roots_iter =
            xcb::ffi::xcb_setup_roots_iterator(xcb::ffi::xcb_get_setup(connection));
        while roots_iter.rem != 0 {
            if (*roots_iter.data).root == root_window {
                screen = Some(roots_iter.data);
                break;
            }
            xcb::ffi::xcb_screen_next(&mut roots_iter);
        }
        let screen = screen.ok_or(SwapchainSetupError::BadSurface)?;
        let mut window_visual_type_and_depth = None;
        let mut depth_iter = xcb::ffi::xcb_screen_allowed_depths_iterator(screen);
        while depth_iter.rem != 0 {
            let mut visual_iter = xcb::ffi::xcb_depth_visuals_iterator(depth_iter.data);
            while visual_iter.rem != 0 {
                if (*visual_iter.data).visual_id == window_visual_id {
                    window_visual_type_and_depth =
                        Some((visual_iter.data, (*depth_iter.data).depth));
                    break;
                }
                xcb::ffi::xcb_visualtype_next(&mut visual_iter);
            }
            if window_visual_type_and_depth.is_some() {
                break;
            }
            xcb::ffi::xcb_depth_next(&mut depth_iter);
        }
        let (window_visual_type, window_depth) =
            window_visual_type_and_depth.ok_or(SwapchainSetupError::BadSurface)?;
        let window_visual_type = &*window_visual_type;
        let red_mask = window_visual_type.red_mask;
        let green_mask = window_visual_type.green_mask;
        let blue_mask = window_visual_type.blue_mask;
        let alpha_mask = match window_depth {
            24 => 0,
            32 => !(red_mask | green_mask | blue_mask),
            _ => return Err(SwapchainSetupError::NoSupport),
        };
        let mut window_pixmap_format = None;
        let mut formats_iter =
            xcb::ffi::xcb_setup_pixmap_formats_iterator(xcb::ffi::xcb_get_setup(connection));
        while formats_iter.rem != 0 {
            if (*formats_iter.data).depth == window_depth {
                window_pixmap_format = Some(formats_iter.data);
                break;
            }
            xcb::ffi::xcb_format_next(&mut formats_iter);
        }
        let window_pixmap_format =
            &*(window_pixmap_format.ok_or(SwapchainSetupError::BadSurface)?);
        let image_pixel_size = match window_pixmap_format.bits_per_pixel {
            24 => 3,
            32 => 4,
            _ => return Err(SwapchainSetupError::NoSupport),
        };
        fn u32_from_bytes(v: [u8; 4]) -> u32 {
            unsafe { mem::transmute(v) }
        }
        let surface_format_group = match (
            u32_from_bytes([0xFF, 0, 0, 0]),
            u32_from_bytes([0, 0xFF, 0, 0]),
            u32_from_bytes([0, 0, 0xFF, 0]),
            u32_from_bytes([0, 0, 0, 0xFF]),
        ) {
            (r, g, b, a)
                if r == red_mask
                    && g == green_mask
                    && b == blue_mask
                    && (alpha_mask == 0 || a == alpha_mask) =>
            {
                SurfaceFormatGroup::R8G8B8A8
            }
            (b, g, r, a)
                if r == red_mask
                    && g == green_mask
                    && b == blue_mask
                    && (alpha_mask == 0 || a == alpha_mask) =>
            {
                SurfaceFormatGroup::B8G8R8A8
            }
            _ => return Err(SwapchainSetupError::NoSupport),
        };
        let scanline_alignment = match window_pixmap_format.scanline_pad {
            8 => 1,
            16 => 2,
            32 => 4,
            _ => unreachable!("invalid pixmap format scanline_pad"),
        };
        const PRESENT_MODES: &[api::VkPresentModeKHR] = &[
            api::VK_PRESENT_MODE_FIFO_KHR, // FIXME: properly implement FIFO present mode using X11 Present extension
            api::VK_PRESENT_MODE_IMMEDIATE_KHR,
        ];
        Ok(Self {
            gc,
            shm_supported,
            window_depth,
            surface_format_group,
            present_modes: PRESENT_MODES,
            capabilities: api::VkSurfaceCapabilitiesKHR {
                minImageCount: 2,
                maxImageCount: MAX_SWAPCHAIN_IMAGE_COUNT,
                currentExtent: image_extent,
                minImageExtent: image_extent,
                maxImageExtent: image_extent,
                maxImageArrayLayers: 1,
                supportedTransforms: api::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
                currentTransform: api::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
                supportedCompositeAlpha: api::VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
                supportedUsageFlags: api::VK_IMAGE_USAGE_TRANSFER_SRC_BIT
                    | api::VK_IMAGE_USAGE_TRANSFER_DST_BIT
                    | api::VK_IMAGE_USAGE_SAMPLED_BIT
                    | api::VK_IMAGE_USAGE_STORAGE_BIT
                    | api::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT
                    | api::VK_IMAGE_USAGE_INPUT_ATTACHMENT_BIT,
            },
            shared_present_capabilities: None,
            image_pixel_size,
            scanline_alignment,
            shm_version,
            image_properties: ImageProperties {
                supported_tilings: SupportedTilings::Any,
                format: api::VK_FORMAT_UNDEFINED,
                extents: api::VkExtent3D {
                    width: image_extent.width,
                    height: image_extent.height,
                    depth: 1,
                },
                array_layers: 1,
                mip_levels: 1,
                multisample_count: ImageMultisampleCount::Count1,
                swapchain_present_tiling: Some(Tiling::Linear),
            },
        })
    }
}

impl XcbSwapchain {
    unsafe fn new(
        _create_info: &api::VkSwapchainCreateInfoKHR,
        _device_group_create_info: Option<&api::VkDeviceGroupSwapchainCreateInfoKHR>,
    ) -> Result<Self, api::VkResult> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct XcbSurfaceImplementation;

impl XcbSurfaceImplementation {
    unsafe fn get_surface(&self, surface: api::VkSurfaceKHR) -> &api::VkIcdSurfaceXcb {
        let surface = surface.get().unwrap().as_ptr();
        assert_eq!((*surface).platform, api::VK_ICD_WSI_PLATFORM_XCB);
        #[allow(clippy::cast_ptr_alignment)]
        &*(surface as *const api::VkIcdSurfaceXcb)
    }
}

impl SurfaceImplementation for XcbSurfaceImplementation {
    fn get_platform(&self) -> SurfacePlatform {
        SurfacePlatform::VK_ICD_WSI_PLATFORM_XCB
    }
    unsafe fn get_surface_formats(
        &self,
        surface: api::VkSurfaceKHR,
    ) -> Result<Cow<'static, [api::VkSurfaceFormatKHR]>, api::VkResult> {
        let surface = &self.get_surface(surface);
        let first_stage = SwapchainSetupFirstStage::new(surface.connection, surface.window, false)
            .map_err(|v| match v {
                SwapchainSetupError::BadSurface | SwapchainSetupError::NoSupport => {
                    api::VK_ERROR_SURFACE_LOST_KHR
                }
            })?;
        match first_stage.surface_format_group {
            SurfaceFormatGroup::B8G8R8A8 => {
                const SURFACE_FORMATS: &[api::VkSurfaceFormatKHR] = &[
                    api::VkSurfaceFormatKHR {
                        format: api::VK_FORMAT_B8G8R8A8_SRGB,
                        colorSpace: api::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
                    },
                    api::VkSurfaceFormatKHR {
                        format: api::VK_FORMAT_B8G8R8A8_UNORM,
                        colorSpace: api::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
                    },
                ];
                Ok(Cow::Borrowed(SURFACE_FORMATS))
            }
            SurfaceFormatGroup::R8G8B8A8 => {
                const SURFACE_FORMATS: &[api::VkSurfaceFormatKHR] = &[
                    api::VkSurfaceFormatKHR {
                        format: api::VK_FORMAT_R8G8B8A8_SRGB,
                        colorSpace: api::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
                    },
                    api::VkSurfaceFormatKHR {
                        format: api::VK_FORMAT_R8G8B8A8_UNORM,
                        colorSpace: api::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
                    },
                ];
                Ok(Cow::Borrowed(SURFACE_FORMATS))
            }
        }
    }
    unsafe fn get_present_modes(
        &self,
        surface: api::VkSurfaceKHR,
    ) -> Result<Cow<'static, [api::VkPresentModeKHR]>, api::VkResult> {
        let surface = &self.get_surface(surface);
        let first_stage = SwapchainSetupFirstStage::new(surface.connection, surface.window, false)
            .map_err(|v| match v {
                SwapchainSetupError::BadSurface | SwapchainSetupError::NoSupport => {
                    api::VK_ERROR_SURFACE_LOST_KHR
                }
            })?;
        Ok(Cow::Borrowed(first_stage.present_modes))
    }
    unsafe fn get_capabilities(
        &self,
        surface: api::VkSurfaceKHR,
    ) -> Result<api::VkSurfaceCapabilitiesKHR, api::VkResult> {
        let surface = &self.get_surface(surface);
        let first_stage = SwapchainSetupFirstStage::new(surface.connection, surface.window, false)
            .map_err(|v| match v {
                SwapchainSetupError::BadSurface | SwapchainSetupError::NoSupport => {
                    api::VK_ERROR_SURFACE_LOST_KHR
                }
            })?;
        Ok(first_stage.capabilities)
    }
    unsafe fn build(
        &self,
        create_info: &api::VkSwapchainCreateInfoKHR,
        device_group_create_info: Option<&api::VkDeviceGroupSwapchainCreateInfoKHR>,
    ) -> Result<Box<Swapchain>, api::VkResult> {
        Ok(Box::new(XcbSwapchain::new(
            create_info,
            device_group_create_info,
        )?))
    }
    unsafe fn destroy_surface(&self, surface: NonNull<api::VkIcdSurfaceBase>) {
        #[allow(clippy::cast_ptr_alignment)]
        Box::from_raw(surface.as_ptr() as *mut api::VkIcdSurfaceXcb);
    }
    fn duplicate(&self) -> Box<dyn SurfaceImplementation> {
        Box::new(Self {})
    }
}
