// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
#![cfg_attr(
    feature = "cargo-clippy",
    allow(clippy::unneeded_field_pattern)
)]
use api;
use constants::IMAGE_ALIGNMENT;
use device_memory::DeviceMemoryLayout;
use handle::SharedHandle;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SupportedTilings {
    Any,
    LinearOnly,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Tiling {
    Linear,
    #[allow(dead_code)]
    Tiled,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ImageMultisampleCount {
    Count1,
    Count4,
}

#[derive(Copy, Clone, Debug)]
pub struct ImageProperties {
    pub supported_tilings: SupportedTilings,
    pub format: api::VkFormat,
    pub extents: api::VkExtent3D,
    pub array_layers: u32,
    pub mip_levels: u32,
    pub multisample_count: ImageMultisampleCount,
    pub swapchain_present_tiling: Option<Tiling>,
}

#[derive(Copy, Clone, Debug)]
pub struct ImageComputedProperties {
    pub pixel_size_in_bytes: usize,
    pub memory_layout: DeviceMemoryLayout,
}

impl ImageProperties {
    pub fn computed_properties(&self) -> ImageComputedProperties {
        match *self {
            Self {
                supported_tilings: SupportedTilings::Any,
                format: api::VK_FORMAT_R8G8B8A8_UNORM,
                extents,
                array_layers,
                mip_levels: 1,
                multisample_count: ImageMultisampleCount::Count1,
                swapchain_present_tiling: _,
            } => {
                let pixel_size_in_bytes = 4;
                ImageComputedProperties {
                    pixel_size_in_bytes,
                    memory_layout: DeviceMemoryLayout::calculate(
                        pixel_size_in_bytes
                            .checked_mul(extents.width as usize)
                            .unwrap()
                            .checked_mul(extents.height as usize)
                            .unwrap()
                            .checked_mul(extents.depth as usize)
                            .unwrap()
                            .checked_mul(array_layers as usize)
                            .unwrap(),
                        IMAGE_ALIGNMENT,
                    ),
                }
            }
            _ => unimplemented!("ImageProperties::computed_properties({:?})", self),
        }
    }
}

#[derive(Debug)]
pub struct ImageMemory {
    pub device_memory: SharedHandle<api::VkDeviceMemory>,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Image {
    pub properties: ImageProperties,
    pub memory: Option<ImageMemory>,
}
