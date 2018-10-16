// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
use api;
use enum_map::EnumMap;
use std::alloc;
use std::fmt::{self, Debug, Display};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Enum)]
#[repr(u32)]
pub enum DeviceMemoryType {
    Main = 0,
}

impl DeviceMemoryType {
    pub fn from_index(index: u32) -> Option<Self> {
        for (enumerant, _) in EnumMap::<Self, ()>::from(|_| {}).iter() {
            if enumerant as u32 == index {
                return Some(enumerant);
            }
        }
        None
    }
    pub fn heap(self) -> DeviceMemoryHeap {
        match self {
            DeviceMemoryType::Main => DeviceMemoryHeap::Main,
        }
    }
    pub fn flags(self) -> api::VkMemoryPropertyFlags {
        match self {
            DeviceMemoryType::Main => {
                api::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT
                    | api::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
                    | api::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT
                    | api::VK_MEMORY_PROPERTY_HOST_CACHED_BIT
            }
        }
    }
    pub fn to_bits(self) -> u32 {
        1 << (self as u32)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct DeviceMemoryTypes(EnumMap<DeviceMemoryType, bool>);

impl Default for DeviceMemoryTypes {
    fn default() -> Self {
        DeviceMemoryTypes(enum_map!{_ => false})
    }
}

impl DeviceMemoryTypes {
    #[allow(dead_code)]
    pub fn to_bits(self) -> u32 {
        let mut retval = 0;
        for (enumerant, value) in self.iter() {
            if *value {
                retval |= enumerant.to_bits();
            }
        }
        retval
    }
}

impl From<EnumMap<DeviceMemoryType, bool>> for DeviceMemoryTypes {
    fn from(v: EnumMap<DeviceMemoryType, bool>) -> Self {
        DeviceMemoryTypes(v)
    }
}

impl From<DeviceMemoryType> for DeviceMemoryTypes {
    fn from(v: DeviceMemoryType) -> Self {
        DeviceMemoryTypes(EnumMap::from(|i| i == v))
    }
}

impl Deref for DeviceMemoryTypes {
    type Target = EnumMap<DeviceMemoryType, bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DeviceMemoryTypes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Enum)]
#[repr(u32)]
pub enum DeviceMemoryHeap {
    Main = 0,
}

impl DeviceMemoryHeap {
    pub fn flags(self) -> api::VkMemoryHeapFlags {
        match self {
            DeviceMemoryHeap::Main => api::VK_MEMORY_HEAP_DEVICE_LOCAL_BIT,
        }
    }
    #[allow(dead_code)]
    pub fn to_bits(self) -> u32 {
        1 << (self as u32)
    }
    #[allow(dead_code)]
    pub fn from_index(index: u32) -> Option<Self> {
        for (enumerant, _) in EnumMap::<Self, ()>::from(|_| {}).iter() {
            if enumerant as u32 == index {
                return Some(enumerant);
            }
        }
        None
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct DeviceMemoryHeaps(EnumMap<DeviceMemoryHeap, bool>);

impl Default for DeviceMemoryHeaps {
    fn default() -> Self {
        DeviceMemoryHeaps(enum_map!{_ => false})
    }
}

impl DeviceMemoryHeaps {
    #[allow(dead_code)]
    pub fn to_bits(self) -> u32 {
        let mut retval = 0;
        for (enumerant, value) in self.iter() {
            if *value {
                retval |= enumerant.to_bits();
            }
        }
        retval
    }
}

impl From<EnumMap<DeviceMemoryHeap, bool>> for DeviceMemoryHeaps {
    fn from(v: EnumMap<DeviceMemoryHeap, bool>) -> Self {
        DeviceMemoryHeaps(v)
    }
}

impl From<DeviceMemoryHeap> for DeviceMemoryHeaps {
    fn from(v: DeviceMemoryHeap) -> Self {
        DeviceMemoryHeaps(EnumMap::from(|i| i == v))
    }
}

impl Deref for DeviceMemoryHeaps {
    type Target = EnumMap<DeviceMemoryHeap, bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DeviceMemoryHeaps {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DeviceMemoryLayout {
    pub size: usize,
    pub alignment: usize,
}

impl DeviceMemoryLayout {
    pub fn calculate(required_size: usize, required_alignment: usize) -> Self {
        assert!(required_alignment.is_power_of_two());
        assert_ne!(required_size, 0);
        Self {
            size: (required_size + required_alignment - 1) & !(required_alignment - 1),
            alignment: required_alignment,
        }
    }
}

pub trait DeviceMemoryAllocation: 'static + Send + Sync + Debug {
    unsafe fn get(&self) -> NonNull<u8>;
    fn layout(&self) -> DeviceMemoryLayout;
    fn size(&self) -> usize {
        self.layout().size
    }
}

#[derive(Debug)]
pub struct DefaultDeviceMemoryAllocation {
    memory: NonNull<u8>,
    layout: alloc::Layout,
}

#[derive(Debug)]
pub struct DefaultDeviceMemoryAllocationFailure;

impl Display for DefaultDeviceMemoryAllocationFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("device memory allocation failed with default allocator")
    }
}

impl DefaultDeviceMemoryAllocation {
    pub fn new(layout: DeviceMemoryLayout) -> Result<Self, DefaultDeviceMemoryAllocationFailure> {
        unsafe {
            let layout = alloc::Layout::from_size_align(layout.size, layout.alignment).unwrap();
            Ok(Self {
                memory: NonNull::new(alloc::alloc(layout))
                    .ok_or(DefaultDeviceMemoryAllocationFailure)?,
                layout: layout,
            })
        }
    }
}

unsafe impl Send for DefaultDeviceMemoryAllocation {}

unsafe impl Sync for DefaultDeviceMemoryAllocation {}

impl DeviceMemoryAllocation for DefaultDeviceMemoryAllocation {
    unsafe fn get(&self) -> NonNull<u8> {
        self.memory
    }
    fn layout(&self) -> DeviceMemoryLayout {
        DeviceMemoryLayout {
            size: self.layout.size(),
            alignment: self.layout.align(),
        }
    }
}

impl Drop for DefaultDeviceMemoryAllocation {
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(self.memory.as_ptr(), self.layout);
        }
    }
}

#[derive(Debug)]
pub enum DeviceMemory {
    Default(DefaultDeviceMemoryAllocation),
    #[allow(dead_code)]
    Special(Box<dyn DeviceMemoryAllocation>),
}

impl DeviceMemory {
    pub fn allocate_from_default_heap(
        layout: DeviceMemoryLayout,
    ) -> Result<Self, DefaultDeviceMemoryAllocationFailure> {
        Ok(DeviceMemory::Default(DefaultDeviceMemoryAllocation::new(
            layout,
        )?))
    }
}

impl DeviceMemoryAllocation for DeviceMemory {
    unsafe fn get(&self) -> NonNull<u8> {
        match self {
            DeviceMemory::Default(memory) => memory.get(),
            DeviceMemory::Special(memory) => memory.as_ref().get(),
        }
    }
    fn layout(&self) -> DeviceMemoryLayout {
        match self {
            DeviceMemory::Default(memory) => memory.layout(),
            DeviceMemory::Special(memory) => memory.as_ref().layout(),
        }
    }
}
