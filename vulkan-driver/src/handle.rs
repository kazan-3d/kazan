use api;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr::null_mut;
use std::ptr::NonNull;

#[repr(C)]
pub struct DispatchableType<T> {
    loader_dispatch_ptr: usize,
    value: T,
}

impl<T> DispatchableType<T> {}

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

pub trait Handle: Clone {
    type Value;
    fn get(&self) -> Option<NonNull<Self::Value>>;
    fn new(v: Option<NonNull<Self::Value>>) -> Self;
    fn null() -> Self {
        Self::new(None)
    }
}

#[repr(transparent)]
pub struct DispatchableHandle<T>(Option<NonNull<DispatchableType<T>>>);

impl<T> Clone for DispatchableHandle<T> {
    fn clone(&self) -> Self {
        DispatchableHandle(self.0)
    }
}

impl<T> Handle for DispatchableHandle<T> {
    type Value = DispatchableType<T>;
    fn get(&self) -> Option<NonNull<DispatchableType<T>>> {
        self.0
    }
    fn new(v: Option<NonNull<DispatchableType<T>>>) -> Self {
        DispatchableHandle(v)
    }
}

#[repr(transparent)]
pub struct NondispatchableHandle<T>(u64, PhantomData<Option<NonNull<T>>>);

impl<T> Clone for NondispatchableHandle<T> {
    fn clone(&self) -> Self {
        NondispatchableHandle(self.0, PhantomData)
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

pub type VkInstance = DispatchableHandle<::api_impl::Instance>;

pub struct PhysicalDevice {}

pub type VkPhysicalDevice = DispatchableHandle<PhysicalDevice>;

pub struct Device {}

pub type VkDevice = DispatchableHandle<Device>;

pub struct Queue {}

pub type VkQueue = DispatchableHandle<Queue>;

pub struct CommandBuffer {}

pub type VkCommandBuffer = DispatchableHandle<CommandBuffer>;

pub struct Semaphore {}

pub type VkSemaphore = NondispatchableHandle<Semaphore>;

pub struct Fence {}

pub type VkFence = NondispatchableHandle<Fence>;

pub struct DeviceMemory {}

pub type VkDeviceMemory = NondispatchableHandle<DeviceMemory>;

pub struct Buffer {}

pub type VkBuffer = NondispatchableHandle<Buffer>;

pub struct Image {}

pub type VkImage = NondispatchableHandle<Image>;

pub struct Event {}

pub type VkEvent = NondispatchableHandle<Event>;

pub struct QueryPool {}

pub type VkQueryPool = NondispatchableHandle<QueryPool>;

pub struct BufferView {}

pub type VkBufferView = NondispatchableHandle<BufferView>;

pub struct ImageView {}

pub type VkImageView = NondispatchableHandle<ImageView>;

pub struct ShaderModule {}

pub type VkShaderModule = NondispatchableHandle<ShaderModule>;

pub struct PipelineCache {}

pub type VkPipelineCache = NondispatchableHandle<PipelineCache>;

pub struct PipelineLayout {}

pub type VkPipelineLayout = NondispatchableHandle<PipelineLayout>;

pub struct RenderPass {}

pub type VkRenderPass = NondispatchableHandle<RenderPass>;

pub struct Pipeline {}

pub type VkPipeline = NondispatchableHandle<Pipeline>;

pub struct DescriptorSetLayout {}

pub type VkDescriptorSetLayout = NondispatchableHandle<DescriptorSetLayout>;

pub struct Sampler {}

pub type VkSampler = NondispatchableHandle<Sampler>;

pub struct DescriptorPool {}

pub type VkDescriptorPool = NondispatchableHandle<DescriptorPool>;

pub struct DescriptorSet {}

pub type VkDescriptorSet = NondispatchableHandle<DescriptorSet>;

pub struct Framebuffer {}

pub type VkFramebuffer = NondispatchableHandle<Framebuffer>;

pub struct CommandPool {}

pub type VkCommandPool = NondispatchableHandle<CommandPool>;

pub struct SamplerYcbcrConversion {}

pub type VkSamplerYcbcrConversion = NondispatchableHandle<SamplerYcbcrConversion>;

pub struct DescriptorUpdateTemplate {}

pub type VkDescriptorUpdateTemplate = NondispatchableHandle<DescriptorUpdateTemplate>;

pub struct SurfaceKHR {}

pub type VkSurfaceKHR = NondispatchableHandle<SurfaceKHR>;

pub struct SwapchainKHR {}

pub type VkSwapchainKHR = NondispatchableHandle<SwapchainKHR>;

pub struct DisplayKHR {}

pub type VkDisplayKHR = NondispatchableHandle<DisplayKHR>;

pub struct DisplayModeKHR {}

pub type VkDisplayModeKHR = NondispatchableHandle<DisplayModeKHR>;

pub struct DebugReportCallbackEXT {}

pub type VkDebugReportCallbackEXT = NondispatchableHandle<DebugReportCallbackEXT>;

pub struct DebugUtilsMessengerEXT {}

pub type VkDebugUtilsMessengerEXT = NondispatchableHandle<DebugUtilsMessengerEXT>;

pub struct ValidationCacheEXT {}

pub type VkValidationCacheEXT = NondispatchableHandle<ValidationCacheEXT>;
