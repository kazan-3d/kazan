use api;
use handle::Handle;
use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Copy, Clone)]
enum ProcAddressScope {
    Device,
    Instance,
    Global,
}

impl ProcAddressScope {
    fn global(self) -> bool {
        match self {
            ProcAddressScope::Device | ProcAddressScope::Instance | ProcAddressScope::Global => {
                true
            }
        }
    }
    fn instance(self) -> bool {
        match self {
            ProcAddressScope::Device | ProcAddressScope::Instance => true,
            _ => false,
        }
    }
    fn device(self) -> bool {
        match self {
            ProcAddressScope::Device => true,
            _ => false,
        }
    }
}

struct Extensions {}

impl Default for Extensions {
    fn default() -> Self {
        Self {}
    }
}

fn get_proc_address(
    name: *const c_char,
    scope: ProcAddressScope,
    extensions: &Extensions,
) -> api::PFN_vkVoidFunction {
    let name = unsafe { CStr::from_ptr(name) }.to_str().ok()?;
    use api::*;
    use std::mem::transmute;
    macro_rules! proc_address {
        ($name:ident, $pfn_name:ident, $required_scope:ident, $required_extension:expr) => {
            if scope.$required_scope() && $required_extension && stringify!($name) == name {
                let f: $pfn_name = Some($name);
                return unsafe { transmute(f) };
            }
        };
    }
    #[cfg_attr(rustfmt, rustfmt_skip)]
    {
        proc_address!(vkAcquireNextImage2KHR, PFN_vkAcquireNextImage2KHR, unknown, unknown);
        proc_address!(vkAcquireNextImageKHR, PFN_vkAcquireNextImageKHR, unknown, unknown);
        proc_address!(vkAllocateCommandBuffers, PFN_vkAllocateCommandBuffers, unknown, unknown);
        proc_address!(vkAllocateDescriptorSets, PFN_vkAllocateDescriptorSets, unknown, unknown);
        proc_address!(vkAllocateMemory, PFN_vkAllocateMemory, unknown, unknown);
        proc_address!(vkBeginCommandBuffer, PFN_vkBeginCommandBuffer, unknown, unknown);
        proc_address!(vkBindBufferMemory, PFN_vkBindBufferMemory, unknown, unknown);
        proc_address!(vkBindBufferMemory2, PFN_vkBindBufferMemory2, unknown, unknown);
        proc_address!(vkBindBufferMemory2KHR, PFN_vkBindBufferMemory2KHR, unknown, unknown);
        proc_address!(vkBindImageMemory, PFN_vkBindImageMemory, unknown, unknown);
        proc_address!(vkBindImageMemory2, PFN_vkBindImageMemory2, unknown, unknown);
        proc_address!(vkBindImageMemory2KHR, PFN_vkBindImageMemory2KHR, unknown, unknown);
        proc_address!(vkCmdBeginConditionalRenderingEXT, PFN_vkCmdBeginConditionalRenderingEXT, unknown, unknown);
        proc_address!(vkCmdBeginDebugUtilsLabelEXT, PFN_vkCmdBeginDebugUtilsLabelEXT, unknown, unknown);
        proc_address!(vkCmdBeginQuery, PFN_vkCmdBeginQuery, unknown, unknown);
        proc_address!(vkCmdBeginRenderPass, PFN_vkCmdBeginRenderPass, unknown, unknown);
        proc_address!(vkCmdBeginRenderPass2KHR, PFN_vkCmdBeginRenderPass2KHR, unknown, unknown);
        proc_address!(vkCmdBindDescriptorSets, PFN_vkCmdBindDescriptorSets, unknown, unknown);
        proc_address!(vkCmdBindIndexBuffer, PFN_vkCmdBindIndexBuffer, unknown, unknown);
        proc_address!(vkCmdBindPipeline, PFN_vkCmdBindPipeline, unknown, unknown);
        proc_address!(vkCmdBindShadingRateImageNV, PFN_vkCmdBindShadingRateImageNV, unknown, unknown);
        proc_address!(vkCmdBindVertexBuffers, PFN_vkCmdBindVertexBuffers, unknown, unknown);
        proc_address!(vkCmdBlitImage, PFN_vkCmdBlitImage, unknown, unknown);
        proc_address!(vkCmdClearAttachments, PFN_vkCmdClearAttachments, unknown, unknown);
        proc_address!(vkCmdClearColorImage, PFN_vkCmdClearColorImage, unknown, unknown);
        proc_address!(vkCmdClearDepthStencilImage, PFN_vkCmdClearDepthStencilImage, unknown, unknown);
        proc_address!(vkCmdCopyBuffer, PFN_vkCmdCopyBuffer, unknown, unknown);
        proc_address!(vkCmdCopyBufferToImage, PFN_vkCmdCopyBufferToImage, unknown, unknown);
        proc_address!(vkCmdCopyImage, PFN_vkCmdCopyImage, unknown, unknown);
        proc_address!(vkCmdCopyImageToBuffer, PFN_vkCmdCopyImageToBuffer, unknown, unknown);
        proc_address!(vkCmdCopyQueryPoolResults, PFN_vkCmdCopyQueryPoolResults, unknown, unknown);
        proc_address!(vkCmdDebugMarkerBeginEXT, PFN_vkCmdDebugMarkerBeginEXT, unknown, unknown);
        proc_address!(vkCmdDebugMarkerEndEXT, PFN_vkCmdDebugMarkerEndEXT, unknown, unknown);
        proc_address!(vkCmdDebugMarkerInsertEXT, PFN_vkCmdDebugMarkerInsertEXT, unknown, unknown);
        proc_address!(vkCmdDispatch, PFN_vkCmdDispatch, unknown, unknown);
        proc_address!(vkCmdDispatchBase, PFN_vkCmdDispatchBase, unknown, unknown);
        proc_address!(vkCmdDispatchBaseKHR, PFN_vkCmdDispatchBaseKHR, unknown, unknown);
        proc_address!(vkCmdDispatchIndirect, PFN_vkCmdDispatchIndirect, unknown, unknown);
        proc_address!(vkCmdDraw, PFN_vkCmdDraw, unknown, unknown);
        proc_address!(vkCmdDrawIndexed, PFN_vkCmdDrawIndexed, unknown, unknown);
        proc_address!(vkCmdDrawIndexedIndirect, PFN_vkCmdDrawIndexedIndirect, unknown, unknown);
        proc_address!(vkCmdDrawIndexedIndirectCountAMD, PFN_vkCmdDrawIndexedIndirectCountAMD, unknown, unknown);
        proc_address!(vkCmdDrawIndexedIndirectCountKHR, PFN_vkCmdDrawIndexedIndirectCountKHR, unknown, unknown);
        proc_address!(vkCmdDrawIndirect, PFN_vkCmdDrawIndirect, unknown, unknown);
        proc_address!(vkCmdDrawIndirectCountAMD, PFN_vkCmdDrawIndirectCountAMD, unknown, unknown);
        proc_address!(vkCmdDrawIndirectCountKHR, PFN_vkCmdDrawIndirectCountKHR, unknown, unknown);
        proc_address!(vkCmdDrawMeshTasksIndirectCountNV, PFN_vkCmdDrawMeshTasksIndirectCountNV, unknown, unknown);
        proc_address!(vkCmdDrawMeshTasksIndirectNV, PFN_vkCmdDrawMeshTasksIndirectNV, unknown, unknown);
        proc_address!(vkCmdDrawMeshTasksNV, PFN_vkCmdDrawMeshTasksNV, unknown, unknown);
        proc_address!(vkCmdEndConditionalRenderingEXT, PFN_vkCmdEndConditionalRenderingEXT, unknown, unknown);
        proc_address!(vkCmdEndDebugUtilsLabelEXT, PFN_vkCmdEndDebugUtilsLabelEXT, unknown, unknown);
        proc_address!(vkCmdEndQuery, PFN_vkCmdEndQuery, unknown, unknown);
        proc_address!(vkCmdEndRenderPass, PFN_vkCmdEndRenderPass, unknown, unknown);
        proc_address!(vkCmdEndRenderPass2KHR, PFN_vkCmdEndRenderPass2KHR, unknown, unknown);
        proc_address!(vkCmdExecuteCommands, PFN_vkCmdExecuteCommands, unknown, unknown);
        proc_address!(vkCmdFillBuffer, PFN_vkCmdFillBuffer, unknown, unknown);
        proc_address!(vkCmdInsertDebugUtilsLabelEXT, PFN_vkCmdInsertDebugUtilsLabelEXT, unknown, unknown);
        proc_address!(vkCmdNextSubpass, PFN_vkCmdNextSubpass, unknown, unknown);
        proc_address!(vkCmdNextSubpass2KHR, PFN_vkCmdNextSubpass2KHR, unknown, unknown);
        proc_address!(vkCmdPipelineBarrier, PFN_vkCmdPipelineBarrier, unknown, unknown);
        proc_address!(vkCmdPushConstants, PFN_vkCmdPushConstants, unknown, unknown);
        proc_address!(vkCmdPushDescriptorSetKHR, PFN_vkCmdPushDescriptorSetKHR, unknown, unknown);
        proc_address!(vkCmdPushDescriptorSetWithTemplateKHR, PFN_vkCmdPushDescriptorSetWithTemplateKHR, unknown, unknown);
        proc_address!(vkCmdResetEvent, PFN_vkCmdResetEvent, unknown, unknown);
        proc_address!(vkCmdResetQueryPool, PFN_vkCmdResetQueryPool, unknown, unknown);
        proc_address!(vkCmdResolveImage, PFN_vkCmdResolveImage, unknown, unknown);
        proc_address!(vkCmdSetBlendConstants, PFN_vkCmdSetBlendConstants, unknown, unknown);
        proc_address!(vkCmdSetCheckpointNV, PFN_vkCmdSetCheckpointNV, unknown, unknown);
        proc_address!(vkCmdSetCoarseSampleOrderNV, PFN_vkCmdSetCoarseSampleOrderNV, unknown, unknown);
        proc_address!(vkCmdSetDepthBias, PFN_vkCmdSetDepthBias, unknown, unknown);
        proc_address!(vkCmdSetDepthBounds, PFN_vkCmdSetDepthBounds, unknown, unknown);
        proc_address!(vkCmdSetDeviceMask, PFN_vkCmdSetDeviceMask, unknown, unknown);
        proc_address!(vkCmdSetDeviceMaskKHR, PFN_vkCmdSetDeviceMaskKHR, unknown, unknown);
        proc_address!(vkCmdSetDiscardRectangleEXT, PFN_vkCmdSetDiscardRectangleEXT, unknown, unknown);
        proc_address!(vkCmdSetEvent, PFN_vkCmdSetEvent, unknown, unknown);
        proc_address!(vkCmdSetExclusiveScissorNV, PFN_vkCmdSetExclusiveScissorNV, unknown, unknown);
        proc_address!(vkCmdSetLineWidth, PFN_vkCmdSetLineWidth, unknown, unknown);
        proc_address!(vkCmdSetSampleLocationsEXT, PFN_vkCmdSetSampleLocationsEXT, unknown, unknown);
        proc_address!(vkCmdSetScissor, PFN_vkCmdSetScissor, unknown, unknown);
        proc_address!(vkCmdSetStencilCompareMask, PFN_vkCmdSetStencilCompareMask, unknown, unknown);
        proc_address!(vkCmdSetStencilReference, PFN_vkCmdSetStencilReference, unknown, unknown);
        proc_address!(vkCmdSetStencilWriteMask, PFN_vkCmdSetStencilWriteMask, unknown, unknown);
        proc_address!(vkCmdSetViewport, PFN_vkCmdSetViewport, unknown, unknown);
        proc_address!(vkCmdSetViewportShadingRatePaletteNV, PFN_vkCmdSetViewportShadingRatePaletteNV, unknown, unknown);
        proc_address!(vkCmdSetViewportWScalingNV, PFN_vkCmdSetViewportWScalingNV, unknown, unknown);
        proc_address!(vkCmdUpdateBuffer, PFN_vkCmdUpdateBuffer, unknown, unknown);
        proc_address!(vkCmdWaitEvents, PFN_vkCmdWaitEvents, unknown, unknown);
        proc_address!(vkCmdWriteBufferMarkerAMD, PFN_vkCmdWriteBufferMarkerAMD, unknown, unknown);
        proc_address!(vkCmdWriteTimestamp, PFN_vkCmdWriteTimestamp, unknown, unknown);
        proc_address!(vkCreateBuffer, PFN_vkCreateBuffer, unknown, unknown);
        proc_address!(vkCreateBufferView, PFN_vkCreateBufferView, unknown, unknown);
        proc_address!(vkCreateCommandPool, PFN_vkCreateCommandPool, unknown, unknown);
        proc_address!(vkCreateComputePipelines, PFN_vkCreateComputePipelines, unknown, unknown);
        proc_address!(vkCreateDebugReportCallbackEXT, PFN_vkCreateDebugReportCallbackEXT, unknown, unknown);
        proc_address!(vkCreateDebugUtilsMessengerEXT, PFN_vkCreateDebugUtilsMessengerEXT, unknown, unknown);
        proc_address!(vkCreateDescriptorPool, PFN_vkCreateDescriptorPool, unknown, unknown);
        proc_address!(vkCreateDescriptorSetLayout, PFN_vkCreateDescriptorSetLayout, unknown, unknown);
        proc_address!(vkCreateDescriptorUpdateTemplate, PFN_vkCreateDescriptorUpdateTemplate, unknown, unknown);
        proc_address!(vkCreateDescriptorUpdateTemplateKHR, PFN_vkCreateDescriptorUpdateTemplateKHR, unknown, unknown);
        proc_address!(vkCreateDevice, PFN_vkCreateDevice, unknown, unknown);
        proc_address!(vkCreateDisplayModeKHR, PFN_vkCreateDisplayModeKHR, unknown, unknown);
        proc_address!(vkCreateDisplayPlaneSurfaceKHR, PFN_vkCreateDisplayPlaneSurfaceKHR, unknown, unknown);
        proc_address!(vkCreateEvent, PFN_vkCreateEvent, unknown, unknown);
        proc_address!(vkCreateFence, PFN_vkCreateFence, unknown, unknown);
        proc_address!(vkCreateFramebuffer, PFN_vkCreateFramebuffer, unknown, unknown);
        proc_address!(vkCreateGraphicsPipelines, PFN_vkCreateGraphicsPipelines, unknown, unknown);
        proc_address!(vkCreateImage, PFN_vkCreateImage, unknown, unknown);
        proc_address!(vkCreateImageView, PFN_vkCreateImageView, unknown, unknown);
        proc_address!(vkCreateInstance, PFN_vkCreateInstance, unknown, unknown);
        proc_address!(vkCreatePipelineCache, PFN_vkCreatePipelineCache, unknown, unknown);
        proc_address!(vkCreatePipelineLayout, PFN_vkCreatePipelineLayout, unknown, unknown);
        proc_address!(vkCreateQueryPool, PFN_vkCreateQueryPool, unknown, unknown);
        proc_address!(vkCreateRenderPass, PFN_vkCreateRenderPass, unknown, unknown);
        proc_address!(vkCreateRenderPass2KHR, PFN_vkCreateRenderPass2KHR, unknown, unknown);
        proc_address!(vkCreateSampler, PFN_vkCreateSampler, unknown, unknown);
        proc_address!(vkCreateSamplerYcbcrConversion, PFN_vkCreateSamplerYcbcrConversion, unknown, unknown);
        proc_address!(vkCreateSamplerYcbcrConversionKHR, PFN_vkCreateSamplerYcbcrConversionKHR, unknown, unknown);
        proc_address!(vkCreateSemaphore, PFN_vkCreateSemaphore, unknown, unknown);
        proc_address!(vkCreateShaderModule, PFN_vkCreateShaderModule, unknown, unknown);
        proc_address!(vkCreateSharedSwapchainsKHR, PFN_vkCreateSharedSwapchainsKHR, unknown, unknown);
        proc_address!(vkCreateSwapchainKHR, PFN_vkCreateSwapchainKHR, unknown, unknown);
        proc_address!(vkCreateValidationCacheEXT, PFN_vkCreateValidationCacheEXT, unknown, unknown);
        proc_address!(vkCreateXcbSurfaceKHR, PFN_vkCreateXcbSurfaceKHR, unknown, unknown);
        proc_address!(vkDebugMarkerSetObjectNameEXT, PFN_vkDebugMarkerSetObjectNameEXT, unknown, unknown);
        proc_address!(vkDebugMarkerSetObjectTagEXT, PFN_vkDebugMarkerSetObjectTagEXT, unknown, unknown);
        proc_address!(vkDebugReportCallbackEXT, PFN_vkDebugReportCallbackEXT, unknown, unknown);
        proc_address!(vkDebugReportMessageEXT, PFN_vkDebugReportMessageEXT, unknown, unknown);
        proc_address!(vkDebugUtilsMessengerCallbackEXT, PFN_vkDebugUtilsMessengerCallbackEXT, unknown, unknown);
        proc_address!(vkDestroyBuffer, PFN_vkDestroyBuffer, unknown, unknown);
        proc_address!(vkDestroyBufferView, PFN_vkDestroyBufferView, unknown, unknown);
        proc_address!(vkDestroyCommandPool, PFN_vkDestroyCommandPool, unknown, unknown);
        proc_address!(vkDestroyDebugReportCallbackEXT, PFN_vkDestroyDebugReportCallbackEXT, unknown, unknown);
        proc_address!(vkDestroyDebugUtilsMessengerEXT, PFN_vkDestroyDebugUtilsMessengerEXT, unknown, unknown);
        proc_address!(vkDestroyDescriptorPool, PFN_vkDestroyDescriptorPool, unknown, unknown);
        proc_address!(vkDestroyDescriptorSetLayout, PFN_vkDestroyDescriptorSetLayout, unknown, unknown);
        proc_address!(vkDestroyDescriptorUpdateTemplate, PFN_vkDestroyDescriptorUpdateTemplate, unknown, unknown);
        proc_address!(vkDestroyDescriptorUpdateTemplateKHR, PFN_vkDestroyDescriptorUpdateTemplateKHR, unknown, unknown);
        proc_address!(vkDestroyDevice, PFN_vkDestroyDevice, unknown, unknown);
        proc_address!(vkDestroyEvent, PFN_vkDestroyEvent, unknown, unknown);
        proc_address!(vkDestroyFence, PFN_vkDestroyFence, unknown, unknown);
        proc_address!(vkDestroyFramebuffer, PFN_vkDestroyFramebuffer, unknown, unknown);
        proc_address!(vkDestroyImage, PFN_vkDestroyImage, unknown, unknown);
        proc_address!(vkDestroyImageView, PFN_vkDestroyImageView, unknown, unknown);
        proc_address!(vkDestroyInstance, PFN_vkDestroyInstance, unknown, unknown);
        proc_address!(vkDestroyPipeline, PFN_vkDestroyPipeline, unknown, unknown);
        proc_address!(vkDestroyPipelineCache, PFN_vkDestroyPipelineCache, unknown, unknown);
        proc_address!(vkDestroyPipelineLayout, PFN_vkDestroyPipelineLayout, unknown, unknown);
        proc_address!(vkDestroyQueryPool, PFN_vkDestroyQueryPool, unknown, unknown);
        proc_address!(vkDestroyRenderPass, PFN_vkDestroyRenderPass, unknown, unknown);
        proc_address!(vkDestroySampler, PFN_vkDestroySampler, unknown, unknown);
        proc_address!(vkDestroySamplerYcbcrConversion, PFN_vkDestroySamplerYcbcrConversion, unknown, unknown);
        proc_address!(vkDestroySamplerYcbcrConversionKHR, PFN_vkDestroySamplerYcbcrConversionKHR, unknown, unknown);
        proc_address!(vkDestroySemaphore, PFN_vkDestroySemaphore, unknown, unknown);
        proc_address!(vkDestroyShaderModule, PFN_vkDestroyShaderModule, unknown, unknown);
        proc_address!(vkDestroySurfaceKHR, PFN_vkDestroySurfaceKHR, unknown, unknown);
        proc_address!(vkDestroySwapchainKHR, PFN_vkDestroySwapchainKHR, unknown, unknown);
        proc_address!(vkDestroyValidationCacheEXT, PFN_vkDestroyValidationCacheEXT, unknown, unknown);
        proc_address!(vkDeviceWaitIdle, PFN_vkDeviceWaitIdle, unknown, unknown);
        proc_address!(vkDisplayPowerControlEXT, PFN_vkDisplayPowerControlEXT, unknown, unknown);
        proc_address!(vkEndCommandBuffer, PFN_vkEndCommandBuffer, unknown, unknown);
        proc_address!(vkEnumerateDeviceExtensionProperties, PFN_vkEnumerateDeviceExtensionProperties, unknown, unknown);
        proc_address!(vkEnumerateDeviceLayerProperties, PFN_vkEnumerateDeviceLayerProperties, unknown, unknown);
        proc_address!(vkEnumerateInstanceExtensionProperties, PFN_vkEnumerateInstanceExtensionProperties, unknown, unknown);
        proc_address!(vkEnumerateInstanceLayerProperties, PFN_vkEnumerateInstanceLayerProperties, unknown, unknown);
        proc_address!(vkEnumerateInstanceVersion, PFN_vkEnumerateInstanceVersion, unknown, unknown);
        proc_address!(vkEnumeratePhysicalDeviceGroups, PFN_vkEnumeratePhysicalDeviceGroups, unknown, unknown);
        proc_address!(vkEnumeratePhysicalDeviceGroupsKHR, PFN_vkEnumeratePhysicalDeviceGroupsKHR, unknown, unknown);
        proc_address!(vkEnumeratePhysicalDevices, PFN_vkEnumeratePhysicalDevices, unknown, unknown);
        proc_address!(vkFlushMappedMemoryRanges, PFN_vkFlushMappedMemoryRanges, unknown, unknown);
        proc_address!(vkFreeCommandBuffers, PFN_vkFreeCommandBuffers, unknown, unknown);
        proc_address!(vkFreeDescriptorSets, PFN_vkFreeDescriptorSets, unknown, unknown);
        proc_address!(vkFreeFunction, PFN_vkFreeFunction, unknown, unknown);
        proc_address!(vkFreeMemory, PFN_vkFreeMemory, unknown, unknown);
        proc_address!(vkGetBufferMemoryRequirements, PFN_vkGetBufferMemoryRequirements, unknown, unknown);
        proc_address!(vkGetBufferMemoryRequirements2, PFN_vkGetBufferMemoryRequirements2, unknown, unknown);
        proc_address!(vkGetBufferMemoryRequirements2KHR, PFN_vkGetBufferMemoryRequirements2KHR, unknown, unknown);
        proc_address!(vkGetDescriptorSetLayoutSupport, PFN_vkGetDescriptorSetLayoutSupport, unknown, unknown);
        proc_address!(vkGetDescriptorSetLayoutSupportKHR, PFN_vkGetDescriptorSetLayoutSupportKHR, unknown, unknown);
        proc_address!(vkGetDeviceGroupPeerMemoryFeatures, PFN_vkGetDeviceGroupPeerMemoryFeatures, unknown, unknown);
        proc_address!(vkGetDeviceGroupPeerMemoryFeaturesKHR, PFN_vkGetDeviceGroupPeerMemoryFeaturesKHR, unknown, unknown);
        proc_address!(vkGetDeviceGroupPresentCapabilitiesKHR, PFN_vkGetDeviceGroupPresentCapabilitiesKHR, unknown, unknown);
        proc_address!(vkGetDeviceGroupSurfacePresentModesKHR, PFN_vkGetDeviceGroupSurfacePresentModesKHR, unknown, unknown);
        proc_address!(vkGetDeviceMemoryCommitment, PFN_vkGetDeviceMemoryCommitment, unknown, unknown);
        proc_address!(vkGetDeviceProcAddr, PFN_vkGetDeviceProcAddr, unknown, unknown);
        proc_address!(vkGetDeviceQueue, PFN_vkGetDeviceQueue, unknown, unknown);
        proc_address!(vkGetDeviceQueue2, PFN_vkGetDeviceQueue2, unknown, unknown);
        proc_address!(vkGetDisplayModeProperties2KHR, PFN_vkGetDisplayModeProperties2KHR, unknown, unknown);
        proc_address!(vkGetDisplayModePropertiesKHR, PFN_vkGetDisplayModePropertiesKHR, unknown, unknown);
        proc_address!(vkGetDisplayPlaneCapabilities2KHR, PFN_vkGetDisplayPlaneCapabilities2KHR, unknown, unknown);
        proc_address!(vkGetDisplayPlaneCapabilitiesKHR, PFN_vkGetDisplayPlaneCapabilitiesKHR, unknown, unknown);
        proc_address!(vkGetDisplayPlaneSupportedDisplaysKHR, PFN_vkGetDisplayPlaneSupportedDisplaysKHR, unknown, unknown);
        proc_address!(vkGetEventStatus, PFN_vkGetEventStatus, unknown, unknown);
        proc_address!(vkGetFenceFdKHR, PFN_vkGetFenceFdKHR, unknown, unknown);
        proc_address!(vkGetFenceStatus, PFN_vkGetFenceStatus, unknown, unknown);
        proc_address!(vkGetImageMemoryRequirements, PFN_vkGetImageMemoryRequirements, unknown, unknown);
        proc_address!(vkGetImageMemoryRequirements2, PFN_vkGetImageMemoryRequirements2, unknown, unknown);
        proc_address!(vkGetImageMemoryRequirements2KHR, PFN_vkGetImageMemoryRequirements2KHR, unknown, unknown);
        proc_address!(vkGetImageSparseMemoryRequirements, PFN_vkGetImageSparseMemoryRequirements, unknown, unknown);
        proc_address!(vkGetImageSparseMemoryRequirements2, PFN_vkGetImageSparseMemoryRequirements2, unknown, unknown);
        proc_address!(vkGetImageSparseMemoryRequirements2KHR, PFN_vkGetImageSparseMemoryRequirements2KHR, unknown, unknown);
        proc_address!(vkGetImageSubresourceLayout, PFN_vkGetImageSubresourceLayout, unknown, unknown);
        proc_address!(vkGetInstanceProcAddr, PFN_vkGetInstanceProcAddr, unknown, unknown);
        proc_address!(vkGetMemoryFdKHR, PFN_vkGetMemoryFdKHR, unknown, unknown);
        proc_address!(vkGetMemoryFdPropertiesKHR, PFN_vkGetMemoryFdPropertiesKHR, unknown, unknown);
        proc_address!(vkGetMemoryHostPointerPropertiesEXT, PFN_vkGetMemoryHostPointerPropertiesEXT, unknown, unknown);
        proc_address!(vkGetPastPresentationTimingGOOGLE, PFN_vkGetPastPresentationTimingGOOGLE, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceDisplayPlaneProperties2KHR, PFN_vkGetPhysicalDeviceDisplayPlaneProperties2KHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceDisplayPlanePropertiesKHR, PFN_vkGetPhysicalDeviceDisplayPlanePropertiesKHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceDisplayProperties2KHR, PFN_vkGetPhysicalDeviceDisplayProperties2KHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceDisplayPropertiesKHR, PFN_vkGetPhysicalDeviceDisplayPropertiesKHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceExternalBufferProperties, PFN_vkGetPhysicalDeviceExternalBufferProperties, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceExternalBufferPropertiesKHR, PFN_vkGetPhysicalDeviceExternalBufferPropertiesKHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceExternalFenceProperties, PFN_vkGetPhysicalDeviceExternalFenceProperties, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceExternalFencePropertiesKHR, PFN_vkGetPhysicalDeviceExternalFencePropertiesKHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceExternalImageFormatPropertiesNV, PFN_vkGetPhysicalDeviceExternalImageFormatPropertiesNV, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceExternalSemaphoreProperties, PFN_vkGetPhysicalDeviceExternalSemaphoreProperties, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceExternalSemaphorePropertiesKHR, PFN_vkGetPhysicalDeviceExternalSemaphorePropertiesKHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceFeatures, PFN_vkGetPhysicalDeviceFeatures, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceFeatures2, PFN_vkGetPhysicalDeviceFeatures2, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceFeatures2KHR, PFN_vkGetPhysicalDeviceFeatures2KHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceFormatProperties, PFN_vkGetPhysicalDeviceFormatProperties, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceFormatProperties2, PFN_vkGetPhysicalDeviceFormatProperties2, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceFormatProperties2KHR, PFN_vkGetPhysicalDeviceFormatProperties2KHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceImageFormatProperties, PFN_vkGetPhysicalDeviceImageFormatProperties, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceImageFormatProperties2, PFN_vkGetPhysicalDeviceImageFormatProperties2, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceImageFormatProperties2KHR, PFN_vkGetPhysicalDeviceImageFormatProperties2KHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceMemoryProperties, PFN_vkGetPhysicalDeviceMemoryProperties, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceMemoryProperties2, PFN_vkGetPhysicalDeviceMemoryProperties2, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceMemoryProperties2KHR, PFN_vkGetPhysicalDeviceMemoryProperties2KHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceMultisamplePropertiesEXT, PFN_vkGetPhysicalDeviceMultisamplePropertiesEXT, unknown, unknown);
        proc_address!(vkGetPhysicalDevicePresentRectanglesKHR, PFN_vkGetPhysicalDevicePresentRectanglesKHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceProperties, PFN_vkGetPhysicalDeviceProperties, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceProperties2, PFN_vkGetPhysicalDeviceProperties2, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceProperties2KHR, PFN_vkGetPhysicalDeviceProperties2KHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceQueueFamilyProperties, PFN_vkGetPhysicalDeviceQueueFamilyProperties, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceQueueFamilyProperties2, PFN_vkGetPhysicalDeviceQueueFamilyProperties2, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceQueueFamilyProperties2KHR, PFN_vkGetPhysicalDeviceQueueFamilyProperties2KHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceSparseImageFormatProperties, PFN_vkGetPhysicalDeviceSparseImageFormatProperties, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceSparseImageFormatProperties2, PFN_vkGetPhysicalDeviceSparseImageFormatProperties2, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceSparseImageFormatProperties2KHR, PFN_vkGetPhysicalDeviceSparseImageFormatProperties2KHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceSurfaceCapabilities2EXT, PFN_vkGetPhysicalDeviceSurfaceCapabilities2EXT, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceSurfaceCapabilities2KHR, PFN_vkGetPhysicalDeviceSurfaceCapabilities2KHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR, PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceSurfaceFormats2KHR, PFN_vkGetPhysicalDeviceSurfaceFormats2KHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceSurfaceFormatsKHR, PFN_vkGetPhysicalDeviceSurfaceFormatsKHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceSurfacePresentModesKHR, PFN_vkGetPhysicalDeviceSurfacePresentModesKHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceSurfaceSupportKHR, PFN_vkGetPhysicalDeviceSurfaceSupportKHR, unknown, unknown);
        proc_address!(vkGetPhysicalDeviceXcbPresentationSupportKHR, PFN_vkGetPhysicalDeviceXcbPresentationSupportKHR, unknown, unknown);
        proc_address!(vkGetPipelineCacheData, PFN_vkGetPipelineCacheData, unknown, unknown);
        proc_address!(vkGetQueryPoolResults, PFN_vkGetQueryPoolResults, unknown, unknown);
        proc_address!(vkGetQueueCheckpointDataNV, PFN_vkGetQueueCheckpointDataNV, unknown, unknown);
        proc_address!(vkGetRefreshCycleDurationGOOGLE, PFN_vkGetRefreshCycleDurationGOOGLE, unknown, unknown);
        proc_address!(vkGetRenderAreaGranularity, PFN_vkGetRenderAreaGranularity, unknown, unknown);
        proc_address!(vkGetSemaphoreFdKHR, PFN_vkGetSemaphoreFdKHR, unknown, unknown);
        proc_address!(vkGetShaderInfoAMD, PFN_vkGetShaderInfoAMD, unknown, unknown);
        proc_address!(vkGetSwapchainCounterEXT, PFN_vkGetSwapchainCounterEXT, unknown, unknown);
        proc_address!(vkGetSwapchainImagesKHR, PFN_vkGetSwapchainImagesKHR, unknown, unknown);
        proc_address!(vkGetSwapchainStatusKHR, PFN_vkGetSwapchainStatusKHR, unknown, unknown);
        proc_address!(vkGetValidationCacheDataEXT, PFN_vkGetValidationCacheDataEXT, unknown, unknown);
        proc_address!(vkImportFenceFdKHR, PFN_vkImportFenceFdKHR, unknown, unknown);
        proc_address!(vkImportSemaphoreFdKHR, PFN_vkImportSemaphoreFdKHR, unknown, unknown);
        proc_address!(vkInternalAllocationNotification, PFN_vkInternalAllocationNotification, unknown, unknown);
        proc_address!(vkInternalFreeNotification, PFN_vkInternalFreeNotification, unknown, unknown);
        proc_address!(vkInvalidateMappedMemoryRanges, PFN_vkInvalidateMappedMemoryRanges, unknown, unknown);
        proc_address!(vkMapMemory, PFN_vkMapMemory, unknown, unknown);
        proc_address!(vkMergePipelineCaches, PFN_vkMergePipelineCaches, unknown, unknown);
        proc_address!(vkMergeValidationCachesEXT, PFN_vkMergeValidationCachesEXT, unknown, unknown);
        proc_address!(vkNegotiateLoaderICDInterfaceVersion, PFN_vkNegotiateLoaderICDInterfaceVersion, unknown, unknown);
        proc_address!(vkQueueBeginDebugUtilsLabelEXT, PFN_vkQueueBeginDebugUtilsLabelEXT, unknown, unknown);
        proc_address!(vkQueueBindSparse, PFN_vkQueueBindSparse, unknown, unknown);
        proc_address!(vkQueueEndDebugUtilsLabelEXT, PFN_vkQueueEndDebugUtilsLabelEXT, unknown, unknown);
        proc_address!(vkQueueInsertDebugUtilsLabelEXT, PFN_vkQueueInsertDebugUtilsLabelEXT, unknown, unknown);
        proc_address!(vkQueuePresentKHR, PFN_vkQueuePresentKHR, unknown, unknown);
        proc_address!(vkQueueSubmit, PFN_vkQueueSubmit, unknown, unknown);
        proc_address!(vkQueueWaitIdle, PFN_vkQueueWaitIdle, unknown, unknown);
        proc_address!(vkReallocationFunction, PFN_vkReallocationFunction, unknown, unknown);
        proc_address!(vkRegisterDeviceEventEXT, PFN_vkRegisterDeviceEventEXT, unknown, unknown);
        proc_address!(vkRegisterDisplayEventEXT, PFN_vkRegisterDisplayEventEXT, unknown, unknown);
        proc_address!(vkReleaseDisplayEXT, PFN_vkReleaseDisplayEXT, unknown, unknown);
        proc_address!(vkResetCommandBuffer, PFN_vkResetCommandBuffer, unknown, unknown);
        proc_address!(vkResetCommandPool, PFN_vkResetCommandPool, unknown, unknown);
        proc_address!(vkResetDescriptorPool, PFN_vkResetDescriptorPool, unknown, unknown);
        proc_address!(vkResetEvent, PFN_vkResetEvent, unknown, unknown);
        proc_address!(vkResetFences, PFN_vkResetFences, unknown, unknown);
        proc_address!(vkSetDebugUtilsObjectNameEXT, PFN_vkSetDebugUtilsObjectNameEXT, unknown, unknown);
        proc_address!(vkSetDebugUtilsObjectTagEXT, PFN_vkSetDebugUtilsObjectTagEXT, unknown, unknown);
        proc_address!(vkSetEvent, PFN_vkSetEvent, unknown, unknown);
        proc_address!(vkSetHdrMetadataEXT, PFN_vkSetHdrMetadataEXT, unknown, unknown);
        proc_address!(vkSubmitDebugUtilsMessageEXT, PFN_vkSubmitDebugUtilsMessageEXT, unknown, unknown);
        proc_address!(vkTrimCommandPool, PFN_vkTrimCommandPool, unknown, unknown);
        proc_address!(vkTrimCommandPoolKHR, PFN_vkTrimCommandPoolKHR, unknown, unknown);
        proc_address!(vkUnmapMemory, PFN_vkUnmapMemory, unknown, unknown);
        proc_address!(vkUpdateDescriptorSetWithTemplate, PFN_vkUpdateDescriptorSetWithTemplate, unknown, unknown);
        proc_address!(vkUpdateDescriptorSetWithTemplateKHR, PFN_vkUpdateDescriptorSetWithTemplateKHR, unknown, unknown);
        proc_address!(vkUpdateDescriptorSets, PFN_vkUpdateDescriptorSets, unknown, unknown);
        proc_address!(vkVoidFunction, PFN_vkVoidFunction, unknown, unknown);
        proc_address!(vkWaitForFences, PFN_vkWaitForFences, unknown, unknown);
    }
    None
}

pub struct Instance {
    enabled_extensions: Extensions,
}

#[allow(non_snake_case)]
pub extern "system" fn vkGetInstanceProcAddr(
    instance: api::VkInstance,
    name: *const c_char,
) -> api::PFN_vkVoidFunction {
    match instance.get() {
        Some(instance) => get_proc_address(name, ProcAddressScope::Instance, unsafe {
            &instance.as_ref().enabled_extensions
        }),
        None => get_proc_address(name, ProcAddressScope::Global, &Extensions::default()),
    }
}
