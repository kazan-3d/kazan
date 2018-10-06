// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
use api;
use api_impl::{Device, Instance, PhysicalDevice, Queue};
use buffer::Buffer;
use device_memory::DeviceMemory;
use sampler::Sampler;
use sampler::SamplerYcbcrConversion;
use shader_module::ShaderModule;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr::null_mut;
use std::ptr::NonNull;
use swapchain::Swapchain;

#[repr(C)]
pub struct DispatchableType<T> {
    loader_dispatch_ptr: usize,
    value: T,
}

impl<T> From<T> for DispatchableType<T> {
    fn from(v: T) -> Self {
        Self {
            loader_dispatch_ptr: api::ICD_LOADER_MAGIC as usize,
            value: v,
        }
    }
}

impl<T> Deref for DispatchableType<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for DispatchableType<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

pub trait HandleAllocFree: Handle {
    unsafe fn allocate<T: Into<Self::Value>>(v: T) -> Self {
        Self::new(Some(NonNull::new_unchecked(Box::into_raw(Box::new(
            v.into(),
        )))))
    }
    unsafe fn free(self) {
        Box::from_raw(self.get().unwrap().as_ptr());
    }
}

pub trait Handle: Copy + Eq + fmt::Debug {
    type Value;
    fn get(&self) -> Option<NonNull<Self::Value>>;
    fn new(v: Option<NonNull<Self::Value>>) -> Self;
    fn null() -> Self {
        Self::new(None)
    }
    fn is_null(&self) -> bool {
        self.get().is_none()
    }
    fn take(&mut self) -> Self {
        let retval = self.clone();
        *self = Self::null();
        retval
    }
}

#[repr(transparent)]
pub struct DispatchableHandle<T>(Option<NonNull<()>>, PhantomData<*mut DispatchableType<T>>);

impl<T> Clone for DispatchableHandle<T> {
    fn clone(&self) -> Self {
        DispatchableHandle(self.0, PhantomData)
    }
}

impl<T> Copy for DispatchableHandle<T> {}

impl<T> Eq for DispatchableHandle<T> {}

impl<T> PartialEq for DispatchableHandle<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl<T> fmt::Debug for DispatchableHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("DispatchableHandle")
            .field(
                &self
                    .get()
                    .map(|v| v.as_ptr())
                    .unwrap_or(null_mut::<*mut ()>() as *mut _),
            )
            .finish()
    }
}

impl<T> Handle for DispatchableHandle<T> {
    type Value = DispatchableType<T>;
    fn get(&self) -> Option<NonNull<DispatchableType<T>>> {
        unsafe { mem::transmute(self.0) }
    }
    fn new(v: Option<NonNull<DispatchableType<T>>>) -> Self {
        unsafe { DispatchableHandle(mem::transmute(v), PhantomData) }
    }
}

#[repr(transparent)]
pub struct NondispatchableHandle<T>(u64, PhantomData<Option<NonNull<T>>>);

impl<T> Clone for NondispatchableHandle<T> {
    fn clone(&self) -> Self {
        NondispatchableHandle(self.0, PhantomData)
    }
}

impl<T> Copy for NondispatchableHandle<T> {}

impl<T> Eq for NondispatchableHandle<T> {}

impl<T> PartialEq for NondispatchableHandle<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl<T> fmt::Debug for NondispatchableHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("NondispatchableHandle")
            .field(&self.get().map(|v| v.as_ptr()).unwrap_or(null_mut()))
            .finish()
    }
}

impl<T> Handle for NondispatchableHandle<T> {
    type Value = T;
    fn get(&self) -> Option<NonNull<T>> {
        NonNull::new(self.0 as *mut T)
    }
    fn new(v: Option<NonNull<T>>) -> Self {
        NondispatchableHandle(
            v.map(|v| v.as_ptr()).unwrap_or(null_mut()) as u64,
            PhantomData,
        )
    }
}

pub struct OwnedHandle<T: HandleAllocFree>(NonNull<T::Value>);

impl<T: HandleAllocFree> OwnedHandle<T> {
    pub fn new<I: Into<T::Value>>(v: I) -> Self {
        unsafe { OwnedHandle(T::allocate(v).get().unwrap()) }
    }
    pub unsafe fn from(v: T) -> Option<Self> {
        v.get().map(OwnedHandle)
    }
    pub unsafe fn take(self) -> T {
        let retval = self.0;
        mem::forget(self);
        T::new(Some(retval))
    }
    pub unsafe fn get_handle(&self) -> T {
        T::new(Some(self.0))
    }
}

impl<T: HandleAllocFree> Deref for OwnedHandle<T> {
    type Target = T::Value;
    fn deref(&self) -> &T::Value {
        unsafe { &*self.0.as_ptr() }
    }
}

impl<T: HandleAllocFree> DerefMut for OwnedHandle<T> {
    fn deref_mut(&mut self) -> &mut T::Value {
        unsafe { &mut *self.0.as_ptr() }
    }
}

impl<T: HandleAllocFree> Drop for OwnedHandle<T> {
    fn drop(&mut self) {
        unsafe {
            T::new(Some(self.0)).free();
        }
    }
}

impl<T: HandleAllocFree> fmt::Debug for OwnedHandle<T>
where
    T::Value: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("OwnedHandle").field((*self).deref()).finish()
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct SharedHandle<T: Handle>(NonNull<T::Value>);

impl<T: Handle> SharedHandle<T> {
    pub unsafe fn from(v: T) -> Option<Self> {
        v.get().map(SharedHandle)
    }
    pub unsafe fn take(self) -> T {
        T::new(Some(self.0))
    }
    pub unsafe fn get_handle(&self) -> T {
        T::new(Some(self.0))
    }
    pub fn into_nonnull(self) -> NonNull<T::Value> {
        self.0
    }
}

impl<T: Handle> Deref for SharedHandle<T> {
    type Target = T::Value;
    fn deref(&self) -> &T::Value {
        unsafe { &*self.0.as_ptr() }
    }
}

impl<T: Handle> fmt::Debug for SharedHandle<T>
where
    T::Value: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("SharedHandle")
            .field((*self).deref())
            .finish()
    }
}

pub type VkInstance = DispatchableHandle<Instance>;

impl HandleAllocFree for VkInstance {}

pub type VkPhysicalDevice = DispatchableHandle<PhysicalDevice>;

impl HandleAllocFree for VkPhysicalDevice {}

pub type VkDevice = DispatchableHandle<Device>;

impl HandleAllocFree for VkDevice {}

pub type VkQueue = DispatchableHandle<Queue>;

impl HandleAllocFree for VkQueue {}

pub struct CommandBuffer {}

pub type VkCommandBuffer = DispatchableHandle<CommandBuffer>;

impl HandleAllocFree for VkCommandBuffer {}

pub struct Semaphore {}

pub type VkSemaphore = NondispatchableHandle<Semaphore>;

impl HandleAllocFree for VkSemaphore {}

pub struct Fence {}

pub type VkFence = NondispatchableHandle<Fence>;

impl HandleAllocFree for VkFence {}

pub type VkDeviceMemory = NondispatchableHandle<DeviceMemory>;

impl HandleAllocFree for VkDeviceMemory {}

pub type VkBuffer = NondispatchableHandle<Buffer>;

impl HandleAllocFree for VkBuffer {}

pub struct Image {}

pub type VkImage = NondispatchableHandle<Image>;

impl HandleAllocFree for VkImage {}

pub struct Event {}

pub type VkEvent = NondispatchableHandle<Event>;

impl HandleAllocFree for VkEvent {}

pub struct QueryPool {}

pub type VkQueryPool = NondispatchableHandle<QueryPool>;

impl HandleAllocFree for VkQueryPool {}

pub struct BufferView {}

pub type VkBufferView = NondispatchableHandle<BufferView>;

impl HandleAllocFree for VkBufferView {}

pub struct ImageView {}

pub type VkImageView = NondispatchableHandle<ImageView>;

impl HandleAllocFree for VkImageView {}

pub type VkShaderModule = NondispatchableHandle<ShaderModule>;

impl HandleAllocFree for VkShaderModule {}

pub struct PipelineCache {}

pub type VkPipelineCache = NondispatchableHandle<PipelineCache>;

impl HandleAllocFree for VkPipelineCache {}

pub struct PipelineLayout {}

pub type VkPipelineLayout = NondispatchableHandle<PipelineLayout>;

impl HandleAllocFree for VkPipelineLayout {}

pub struct RenderPass {}

pub type VkRenderPass = NondispatchableHandle<RenderPass>;

impl HandleAllocFree for VkRenderPass {}

pub struct Pipeline {}

pub type VkPipeline = NondispatchableHandle<Pipeline>;

impl HandleAllocFree for VkPipeline {}

pub struct DescriptorSetLayout {}

pub type VkDescriptorSetLayout = NondispatchableHandle<DescriptorSetLayout>;

impl HandleAllocFree for VkDescriptorSetLayout {}

pub type VkSampler = NondispatchableHandle<Sampler>;

impl HandleAllocFree for VkSampler {}

pub struct DescriptorPool {}

pub type VkDescriptorPool = NondispatchableHandle<DescriptorPool>;

impl HandleAllocFree for VkDescriptorPool {}

pub struct DescriptorSet {}

pub type VkDescriptorSet = NondispatchableHandle<DescriptorSet>;

impl HandleAllocFree for VkDescriptorSet {}

pub struct Framebuffer {}

pub type VkFramebuffer = NondispatchableHandle<Framebuffer>;

impl HandleAllocFree for VkFramebuffer {}

pub struct CommandPool {}

pub type VkCommandPool = NondispatchableHandle<CommandPool>;

impl HandleAllocFree for VkCommandPool {}

pub type VkSamplerYcbcrConversion = NondispatchableHandle<SamplerYcbcrConversion>;

impl HandleAllocFree for VkSamplerYcbcrConversion {}

pub struct DescriptorUpdateTemplate {}

pub type VkDescriptorUpdateTemplate = NondispatchableHandle<DescriptorUpdateTemplate>;

impl HandleAllocFree for VkDescriptorUpdateTemplate {}

pub type VkSurfaceKHR = NondispatchableHandle<api::VkIcdSurfaceBase>;

// HandleAllocFree specifically not implemented for VkSurfaceKHR

pub type VkSwapchainKHR = NondispatchableHandle<Box<Swapchain>>;

impl HandleAllocFree for VkSwapchainKHR {}

pub struct DisplayKHR {}

pub type VkDisplayKHR = NondispatchableHandle<DisplayKHR>;

impl HandleAllocFree for VkDisplayKHR {}

pub struct DisplayModeKHR {}

pub type VkDisplayModeKHR = NondispatchableHandle<DisplayModeKHR>;

impl HandleAllocFree for VkDisplayModeKHR {}

pub struct DebugReportCallbackEXT {}

pub type VkDebugReportCallbackEXT = NondispatchableHandle<DebugReportCallbackEXT>;

impl HandleAllocFree for VkDebugReportCallbackEXT {}

pub struct DebugUtilsMessengerEXT {}

pub type VkDebugUtilsMessengerEXT = NondispatchableHandle<DebugUtilsMessengerEXT>;

impl HandleAllocFree for VkDebugUtilsMessengerEXT {}

pub struct ValidationCacheEXT {}

pub type VkValidationCacheEXT = NondispatchableHandle<ValidationCacheEXT>;

impl HandleAllocFree for VkValidationCacheEXT {}
