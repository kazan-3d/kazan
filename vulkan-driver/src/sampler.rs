// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2019 Jacob Lifshay
use crate::api;
use crate::handle::SharedHandle;

#[derive(Debug)]
pub struct AnisotropySettings {
    pub max: f32,
}

#[derive(Debug)]
pub struct Sampler {
    pub mag_filter: api::VkFilter,
    pub min_filter: api::VkFilter,
    pub mipmap_mode: api::VkSamplerMipmapMode,
    pub address_modes: [api::VkSamplerAddressMode; 3],
    pub mip_lod_bias: f32,
    pub anisotropy: Option<AnisotropySettings>,
    pub compare_op: Option<api::VkCompareOp>,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: api::VkBorderColor,
    pub unnormalized_coordinates: bool,
    pub sampler_ycbcr_conversion: Option<SharedHandle<api::VkSamplerYcbcrConversion>>,
}

#[derive(Debug)]
pub struct SamplerYcbcrConversion {}
