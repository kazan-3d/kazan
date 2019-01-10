// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![allow(clippy::unneeded_field_pattern)]
use crate::api;
use crate::constants::IMAGE_ALIGNMENT;
use crate::device_memory::DeviceMemoryLayout;
use crate::handle::SharedHandle;
use std::error;
use std::fmt;

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
    pub fn get_tiling(&self, image_layout: api::VkImageLayout) -> Tiling {
        if image_layout == api::VK_IMAGE_LAYOUT_PRESENT_SRC_KHR {
            self.swapchain_present_tiling.unwrap()
        } else {
            match self.supported_tilings {
                SupportedTilings::LinearOnly => Tiling::Linear,
                SupportedTilings::Any => Tiling::Tiled,
            }
        }
    }
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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ImageViewType {
    Type1D,
    Type2D,
    Type3D,
    Cube,
    Array1D,
    Array2D,
    CubeArray,
}

impl ImageViewType {
    pub fn from(v: api::VkImageViewType) -> Self {
        match v {
            api::VK_IMAGE_VIEW_TYPE_1D => ImageViewType::Type1D,
            api::VK_IMAGE_VIEW_TYPE_2D => ImageViewType::Type2D,
            api::VK_IMAGE_VIEW_TYPE_3D => ImageViewType::Type3D,
            api::VK_IMAGE_VIEW_TYPE_CUBE => ImageViewType::Cube,
            api::VK_IMAGE_VIEW_TYPE_1D_ARRAY => ImageViewType::Array1D,
            api::VK_IMAGE_VIEW_TYPE_2D_ARRAY => ImageViewType::Array2D,
            api::VK_IMAGE_VIEW_TYPE_CUBE_ARRAY => ImageViewType::CubeArray,
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ComponentSwizzle {
    Zero,
    One,
    X,
    Y,
    Z,
    W,
}

#[derive(Debug)]
pub struct InvalidVkComponentSwizzle;

impl fmt::Display for InvalidVkComponentSwizzle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid VkComponentSwizzle")
    }
}

impl error::Error for InvalidVkComponentSwizzle {}

impl ComponentSwizzle {
    pub fn from(
        v: api::VkComponentSwizzle,
        identity: Self,
    ) -> Result<Self, InvalidVkComponentSwizzle> {
        match v {
            api::VK_COMPONENT_SWIZZLE_IDENTITY => Ok(identity),
            api::VK_COMPONENT_SWIZZLE_ZERO => Ok(ComponentSwizzle::Zero),
            api::VK_COMPONENT_SWIZZLE_ONE => Ok(ComponentSwizzle::One),
            api::VK_COMPONENT_SWIZZLE_R => Ok(ComponentSwizzle::X),
            api::VK_COMPONENT_SWIZZLE_G => Ok(ComponentSwizzle::Y),
            api::VK_COMPONENT_SWIZZLE_B => Ok(ComponentSwizzle::Z),
            api::VK_COMPONENT_SWIZZLE_A => Ok(ComponentSwizzle::W),
            _ => Err(InvalidVkComponentSwizzle),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ComponentMapping {
    pub x: ComponentSwizzle,
    pub y: ComponentSwizzle,
    pub z: ComponentSwizzle,
    pub w: ComponentSwizzle,
}

impl ComponentMapping {
    pub const IDENTITY: ComponentMapping = ComponentMapping {
        x: ComponentSwizzle::X,
        y: ComponentSwizzle::Y,
        z: ComponentSwizzle::Z,
        w: ComponentSwizzle::W,
    };
    pub fn from(v: api::VkComponentMapping) -> Result<Self, InvalidVkComponentSwizzle> {
        Ok(Self {
            x: ComponentSwizzle::from(v.r, ComponentSwizzle::X)?,
            y: ComponentSwizzle::from(v.g, ComponentSwizzle::Y)?,
            z: ComponentSwizzle::from(v.b, ComponentSwizzle::Z)?,
            w: ComponentSwizzle::from(v.a, ComponentSwizzle::W)?,
        })
    }
}

impl Default for ComponentMapping {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[derive(Debug)]
pub struct ImageView {
    pub image: SharedHandle<api::VkImage>,
    pub view_type: ImageViewType,
    pub format: api::VkFormat,
    pub component_mapping: ComponentMapping,
    pub subresource_range: api::VkImageSubresourceRange,
}
