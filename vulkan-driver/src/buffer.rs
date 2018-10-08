// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
use api;
use handle::SharedHandle;

pub struct BufferMemory {
    pub device_memory: SharedHandle<api::VkDeviceMemory>,
    pub offset: usize,
}

pub struct Buffer {
    pub size: usize,
    pub memory: Option<BufferMemory>,
}
