// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
use crate::api;
use crate::handle::SharedHandle;

#[derive(Debug)]
pub struct BufferMemory {
    pub device_memory: SharedHandle<api::VkDeviceMemory>,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Buffer {
    pub size: usize,
    pub memory: Option<BufferMemory>,
}

#[derive(Debug)]
pub struct BufferSlice {
    pub buffer: SharedHandle<api::VkBuffer>,
    pub offset: usize,
    pub size: usize,
}

impl BufferSlice {
    pub unsafe fn from(v: &api::VkDescriptorBufferInfo) -> Self {
        let buffer = SharedHandle::from(v.buffer).unwrap();
        assert!(v.offset < buffer.size as u64);
        let offset = v.offset as usize;
        let size = if v.range == api::VK_WHOLE_SIZE as u64 {
            buffer.size - offset
        } else {
            assert!(v.range != 0);
            assert!(v.range.checked_add(v.offset).unwrap() <= buffer.size as u64);
            v.range as usize
        };
        Self {
            buffer,
            offset,
            size,
        }
    }
}

#[derive(Debug)]
pub struct BufferView {}
