// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
use api;

pub enum SupportedTilings {
    Any,
    LinearOnly,
}

pub struct ImageDescriptor {
    pub supported_tilings: SupportedTilings,
    pub format: api::VkFormat,
    pub extents: api::VkExtent3D,
    pub mip_levels: u32,
}
