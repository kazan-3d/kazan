// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

// allow unneeded_field_pattern to ensure fields aren't accidently missed
#![allow(clippy::unneeded_field_pattern)]

use crate::api;
use crate::buffer::{Buffer, BufferMemory};
use crate::constants::*;
use crate::descriptor_set::{
    Descriptor, DescriptorLayout, DescriptorPool, DescriptorSet, DescriptorSetLayout,
    DescriptorWriteArg,
};
use crate::device_memory::{
    DeviceMemory, DeviceMemoryAllocation, DeviceMemoryHeap, DeviceMemoryHeaps, DeviceMemoryLayout,
    DeviceMemoryType, DeviceMemoryTypes,
};
use crate::handle::{Handle, MutHandle, OwnedHandle, SharedHandle};
use crate::image::{
    ComponentMapping, Image, ImageMemory, ImageMultisampleCount, ImageProperties, ImageView,
    ImageViewType, SupportedTilings,
};
use crate::pipeline::{self, PipelineLayout};
use crate::render_pass::RenderPass;
use crate::sampler;
use crate::sampler::Sampler;
use crate::shader_module::ShaderModule;
use crate::swapchain::SurfacePlatform;
use crate::util;
use enum_map::{enum_map, Enum, EnumMap};
use std::ffi::CStr;
use std::iter;
use std::iter::FromIterator;
use std::mem;
use std::ops::*;
use std::os::raw::{c_char, c_void};
use std::ptr::null;
use std::ptr::null_mut;
#[cfg(target_os = "linux")]
use std::ptr::NonNull;
use std::str::FromStr;
use sys_info;
use uuid;
#[cfg(target_os = "linux")]
use xcb;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Enum)]
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum Extension {
    VK_KHR_surface,
    VK_KHR_bind_memory2,
    VK_KHR_device_group_creation,
    VK_KHR_device_group,
    VK_KHR_descriptor_update_template,
    VK_KHR_maintenance1,
    VK_KHR_get_memory_requirements2,
    VK_KHR_get_physical_device_properties2,
    VK_KHR_sampler_ycbcr_conversion,
    VK_KHR_maintenance2,
    VK_KHR_maintenance3,
    VK_KHR_external_memory_capabilities,
    VK_KHR_external_fence_capabilities,
    VK_KHR_external_semaphore_capabilities,
    VK_KHR_16bit_storage,
    VK_KHR_storage_buffer_storage_class,
    VK_KHR_dedicated_allocation,
    VK_KHR_external_fence,
    VK_KHR_external_memory,
    VK_KHR_external_semaphore,
    VK_KHR_multiview,
    VK_KHR_relaxed_block_layout,
    VK_KHR_shader_draw_parameters,
    VK_KHR_variable_pointers,
    VK_KHR_swapchain,
    #[cfg(target_os = "linux")]
    VK_KHR_xcb_surface,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ExtensionScope {
    Device,
    Instance,
}

macro_rules! extensions {
    [$($extension:expr),*] => {
        {
            let extensions: Extensions = [$($extension),*].iter().map(|v|*v).collect();
            extensions
        }
    };
}

impl Extension {
    pub fn get_required_extensions(self) -> Extensions {
        match self {
            Extension::VK_KHR_surface
            | Extension::VK_KHR_bind_memory2
            | Extension::VK_KHR_device_group_creation
            | Extension::VK_KHR_descriptor_update_template
            | Extension::VK_KHR_maintenance1
            | Extension::VK_KHR_get_memory_requirements2
            | Extension::VK_KHR_get_physical_device_properties2
            | Extension::VK_KHR_maintenance2
            | Extension::VK_KHR_storage_buffer_storage_class
            | Extension::VK_KHR_relaxed_block_layout
            | Extension::VK_KHR_shader_draw_parameters => extensions![],
            Extension::VK_KHR_device_group => extensions![Extension::VK_KHR_device_group_creation],
            Extension::VK_KHR_sampler_ycbcr_conversion => extensions![
                Extension::VK_KHR_maintenance1,
                Extension::VK_KHR_bind_memory2,
                Extension::VK_KHR_get_memory_requirements2,
                Extension::VK_KHR_get_physical_device_properties2
            ],
            Extension::VK_KHR_maintenance3
            | Extension::VK_KHR_external_memory_capabilities
            | Extension::VK_KHR_external_fence_capabilities
            | Extension::VK_KHR_external_semaphore_capabilities
            | Extension::VK_KHR_multiview => {
                extensions![Extension::VK_KHR_get_physical_device_properties2]
            }
            Extension::VK_KHR_16bit_storage | Extension::VK_KHR_variable_pointers => extensions![
                Extension::VK_KHR_get_physical_device_properties2,
                Extension::VK_KHR_storage_buffer_storage_class
            ],
            Extension::VK_KHR_dedicated_allocation => {
                extensions![Extension::VK_KHR_get_memory_requirements2]
            }
            Extension::VK_KHR_external_fence => {
                extensions![Extension::VK_KHR_external_fence_capabilities]
            }
            Extension::VK_KHR_external_memory => {
                extensions![Extension::VK_KHR_external_memory_capabilities]
            }
            Extension::VK_KHR_external_semaphore => {
                extensions![Extension::VK_KHR_external_semaphore_capabilities]
            }
            Extension::VK_KHR_swapchain => extensions![Extension::VK_KHR_surface],
            #[cfg(target_os = "linux")]
            Extension::VK_KHR_xcb_surface => extensions![Extension::VK_KHR_surface],
        }
    }
    pub fn get_recursively_required_extensions(self) -> Extensions {
        let mut retval = self.get_required_extensions();
        let mut worklist: EnumMap<Extension, Extension> = enum_map! {_ => self};
        let worklist = worklist.as_mut_slice();
        let mut worklist_size = 1;
        while worklist_size > 0 {
            worklist_size -= 1;
            let extension = worklist[worklist_size];
            retval[extension] = true;
            for (extension, &v) in extension.get_required_extensions().iter() {
                if v && !retval[extension] {
                    worklist[worklist_size] = extension;
                    worklist_size += 1;
                }
            }
        }
        retval
    }
    pub fn get_name(self) -> &'static str {
        macro_rules! name {
            ($($(#[$attributes:meta])* $name:ident,)*) => {
                match self {
                    $($(#[$attributes])* Extension::$name => stringify!($name),)*
                }
            }
        }
        name!(
            VK_KHR_surface,
            VK_KHR_bind_memory2,
            VK_KHR_device_group,
            VK_KHR_device_group_creation,
            VK_KHR_descriptor_update_template,
            VK_KHR_maintenance1,
            VK_KHR_get_memory_requirements2,
            VK_KHR_get_physical_device_properties2,
            VK_KHR_sampler_ycbcr_conversion,
            VK_KHR_maintenance2,
            VK_KHR_maintenance3,
            VK_KHR_external_memory_capabilities,
            VK_KHR_external_fence_capabilities,
            VK_KHR_external_semaphore_capabilities,
            VK_KHR_16bit_storage,
            VK_KHR_storage_buffer_storage_class,
            VK_KHR_dedicated_allocation,
            VK_KHR_external_fence,
            VK_KHR_external_memory,
            VK_KHR_external_semaphore,
            VK_KHR_multiview,
            VK_KHR_relaxed_block_layout,
            VK_KHR_shader_draw_parameters,
            VK_KHR_variable_pointers,
            VK_KHR_swapchain,
            #[cfg(target_os = "linux")]
            VK_KHR_xcb_surface,
        )
    }
    pub fn get_spec_version(self) -> u32 {
        match self {
            Extension::VK_KHR_surface => api::VK_KHR_SURFACE_SPEC_VERSION,
            Extension::VK_KHR_bind_memory2 => api::VK_KHR_BIND_MEMORY_2_SPEC_VERSION,
            Extension::VK_KHR_device_group => api::VK_KHR_DEVICE_GROUP_SPEC_VERSION,
            Extension::VK_KHR_device_group_creation => {
                api::VK_KHR_DEVICE_GROUP_CREATION_SPEC_VERSION
            }
            Extension::VK_KHR_descriptor_update_template => {
                api::VK_KHR_DESCRIPTOR_UPDATE_TEMPLATE_SPEC_VERSION
            }
            Extension::VK_KHR_maintenance1 => api::VK_KHR_MAINTENANCE1_SPEC_VERSION,
            Extension::VK_KHR_get_memory_requirements2 => {
                api::VK_KHR_GET_MEMORY_REQUIREMENTS_2_SPEC_VERSION
            }
            Extension::VK_KHR_get_physical_device_properties2 => {
                api::VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_SPEC_VERSION
            }
            Extension::VK_KHR_sampler_ycbcr_conversion => {
                api::VK_KHR_SAMPLER_YCBCR_CONVERSION_SPEC_VERSION
            }
            Extension::VK_KHR_maintenance2 => api::VK_KHR_MAINTENANCE2_SPEC_VERSION,
            Extension::VK_KHR_maintenance3 => api::VK_KHR_MAINTENANCE3_SPEC_VERSION,
            Extension::VK_KHR_external_memory_capabilities => {
                api::VK_KHR_EXTERNAL_MEMORY_CAPABILITIES_SPEC_VERSION
            }
            Extension::VK_KHR_external_fence_capabilities => {
                api::VK_KHR_EXTERNAL_FENCE_CAPABILITIES_SPEC_VERSION
            }
            Extension::VK_KHR_external_semaphore_capabilities => {
                api::VK_KHR_EXTERNAL_SEMAPHORE_CAPABILITIES_SPEC_VERSION
            }
            Extension::VK_KHR_16bit_storage => api::VK_KHR_16BIT_STORAGE_SPEC_VERSION,
            Extension::VK_KHR_storage_buffer_storage_class => {
                api::VK_KHR_STORAGE_BUFFER_STORAGE_CLASS_SPEC_VERSION
            }
            Extension::VK_KHR_dedicated_allocation => api::VK_KHR_DEDICATED_ALLOCATION_SPEC_VERSION,
            Extension::VK_KHR_external_fence => api::VK_KHR_EXTERNAL_FENCE_SPEC_VERSION,
            Extension::VK_KHR_external_memory => api::VK_KHR_EXTERNAL_MEMORY_SPEC_VERSION,
            Extension::VK_KHR_external_semaphore => api::VK_KHR_EXTERNAL_SEMAPHORE_SPEC_VERSION,
            Extension::VK_KHR_multiview => api::VK_KHR_MULTIVIEW_SPEC_VERSION,
            Extension::VK_KHR_relaxed_block_layout => api::VK_KHR_RELAXED_BLOCK_LAYOUT_SPEC_VERSION,
            Extension::VK_KHR_shader_draw_parameters => {
                api::VK_KHR_SHADER_DRAW_PARAMETERS_SPEC_VERSION
            }
            Extension::VK_KHR_variable_pointers => api::VK_KHR_VARIABLE_POINTERS_SPEC_VERSION,
            Extension::VK_KHR_swapchain => api::VK_KHR_SWAPCHAIN_SPEC_VERSION,
            #[cfg(target_os = "linux")]
            Extension::VK_KHR_xcb_surface => api::VK_KHR_XCB_SURFACE_SPEC_VERSION,
        }
    }
    pub fn get_properties(self) -> api::VkExtensionProperties {
        let mut retval = api::VkExtensionProperties {
            extensionName: [0; api::VK_MAX_EXTENSION_NAME_SIZE as usize],
            specVersion: self.get_spec_version(),
        };
        util::copy_str_to_char_array(&mut retval.extensionName, self.get_name());
        retval
    }
    pub fn get_scope(self) -> ExtensionScope {
        match self {
            Extension::VK_KHR_surface
            | Extension::VK_KHR_device_group_creation
            | Extension::VK_KHR_get_physical_device_properties2
            | Extension::VK_KHR_external_memory_capabilities
            | Extension::VK_KHR_external_fence_capabilities
            | Extension::VK_KHR_external_semaphore_capabilities => ExtensionScope::Instance,
            Extension::VK_KHR_bind_memory2
            | Extension::VK_KHR_device_group
            | Extension::VK_KHR_descriptor_update_template
            | Extension::VK_KHR_maintenance1
            | Extension::VK_KHR_get_memory_requirements2
            | Extension::VK_KHR_sampler_ycbcr_conversion
            | Extension::VK_KHR_maintenance2
            | Extension::VK_KHR_maintenance3
            | Extension::VK_KHR_16bit_storage
            | Extension::VK_KHR_storage_buffer_storage_class
            | Extension::VK_KHR_dedicated_allocation
            | Extension::VK_KHR_external_fence
            | Extension::VK_KHR_external_memory
            | Extension::VK_KHR_external_semaphore
            | Extension::VK_KHR_multiview
            | Extension::VK_KHR_relaxed_block_layout
            | Extension::VK_KHR_shader_draw_parameters
            | Extension::VK_KHR_variable_pointers
            | Extension::VK_KHR_swapchain => ExtensionScope::Device,
            #[cfg(target_os = "linux")]
            Extension::VK_KHR_xcb_surface => ExtensionScope::Instance,
        }
    }
}

impl FromStr for Extension {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (i, _) in Extensions::default().iter() {
            if s == i.get_name() {
                return Ok(i);
            }
        }
        Err(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Extensions(EnumMap<Extension, bool>);

impl Extensions {
    pub fn create_empty() -> Self {
        Extensions(enum_map! {_ => false})
    }
    pub fn is_empty(&self) -> bool {
        self.iter().all(|(_, &v)| !v)
    }
    #[allow(dead_code)]
    pub fn is_full(&self) -> bool {
        self.iter().all(|(_, &v)| v)
    }
    pub fn get_allowed_extensions_from_instance_scope(&self) -> Self {
        let mut retval = Extensions::default();
        let instance_extensions = Self::instance_extensions();
        for (extension, value) in retval.iter_mut() {
            if extension.get_scope() == ExtensionScope::Instance {
                *value = self[extension];
                continue;
            }
            let required_extensions =
                instance_extensions & extension.get_recursively_required_extensions();
            *value = (!*self & required_extensions).is_empty();
        }
        retval
    }
    pub fn instance_extensions() -> Self {
        Extensions(
            (|extension: Extension| extension.get_scope() == ExtensionScope::Instance).into(),
        )
    }
    #[allow(dead_code)]
    pub fn device_extensions() -> Self {
        !Self::instance_extensions()
    }
}

impl FromIterator<Extension> for Extensions {
    fn from_iter<T: IntoIterator<Item = Extension>>(v: T) -> Extensions {
        let mut retval = Extensions::create_empty();
        for extension in v {
            retval[extension] = true;
        }
        retval
    }
}

impl Default for Extensions {
    fn default() -> Self {
        Self::create_empty()
    }
}

impl Deref for Extensions {
    type Target = EnumMap<Extension, bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Extensions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BitAnd for Extensions {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        let mut retval = Self::default();
        for (index, retval) in retval.iter_mut() {
            *retval = self[index] & rhs[index];
        }
        retval
    }
}

impl BitOr for Extensions {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        let mut retval = Self::default();
        for (index, retval) in retval.iter_mut() {
            *retval = self[index] | rhs[index];
        }
        retval
    }
}

impl BitXor for Extensions {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        let mut retval = Self::default();
        for (index, retval) in retval.iter_mut() {
            *retval = self[index] ^ rhs[index];
        }
        retval
    }
}

impl Not for Extensions {
    type Output = Self;
    fn not(mut self) -> Self {
        for v in self.values_mut() {
            *v = !*v;
        }
        self
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum GetProcAddressScope {
    Global,
    Instance,
    Device,
}

#[allow(clippy::cyclomatic_complexity)]
fn get_proc_address(
    name: *const c_char,
    scope: GetProcAddressScope,
    extensions: &Extensions,
) -> api::PFN_vkVoidFunction {
    let mut name = unsafe { CStr::from_ptr(name) }.to_str().ok()?;
    use crate::api::*;
    use std::mem::transmute;
    struct Scope {
        global: bool,
        instance: bool,
        device: bool,
    }
    let scope = Scope {
        global: scope != GetProcAddressScope::Device,
        instance: scope == GetProcAddressScope::Instance,
        device: scope != GetProcAddressScope::Global,
    };
    macro_rules! proc_alias_khr {
        ($base_name:ident, $required_extension:expr) => {
            if name == concat!(stringify!($base_name), "KHR") {
                if scope.instance && $required_extension {
                    name = stringify!($base_name);
                } else {
                    return None;
                }
            }
        };
    }
    proc_alias_khr!(
        vkBindBufferMemory2,
        extensions[Extension::VK_KHR_bind_memory2]
    );
    proc_alias_khr!(
        vkBindImageMemory2,
        extensions[Extension::VK_KHR_bind_memory2]
    );
    proc_alias_khr!(
        vkCmdDispatchBase,
        extensions[Extension::VK_KHR_device_group]
    );
    proc_alias_khr!(
        vkCmdSetDeviceMask,
        extensions[Extension::VK_KHR_device_group]
    );
    proc_alias_khr!(
        vkCreateDescriptorUpdateTemplate,
        extensions[Extension::VK_KHR_descriptor_update_template]
    );
    proc_alias_khr!(
        vkCreateSamplerYcbcrConversion,
        extensions[Extension::VK_KHR_sampler_ycbcr_conversion]
    );
    proc_alias_khr!(
        vkDestroyDescriptorUpdateTemplate,
        extensions[Extension::VK_KHR_descriptor_update_template]
    );
    proc_alias_khr!(
        vkDestroySamplerYcbcrConversion,
        extensions[Extension::VK_KHR_sampler_ycbcr_conversion]
    );
    proc_alias_khr!(
        vkEnumeratePhysicalDeviceGroups,
        extensions[Extension::VK_KHR_device_group_creation]
    );
    proc_alias_khr!(
        vkGetBufferMemoryRequirements2,
        extensions[Extension::VK_KHR_get_memory_requirements2]
    );
    proc_alias_khr!(
        vkGetDescriptorSetLayoutSupport,
        extensions[Extension::VK_KHR_maintenance3]
    );
    proc_alias_khr!(
        vkGetDeviceGroupPeerMemoryFeatures,
        extensions[Extension::VK_KHR_device_group]
    );
    proc_alias_khr!(
        vkGetImageMemoryRequirements2,
        extensions[Extension::VK_KHR_get_memory_requirements2]
    );
    proc_alias_khr!(
        vkGetImageSparseMemoryRequirements2,
        extensions[Extension::VK_KHR_get_memory_requirements2]
    );
    proc_alias_khr!(
        vkGetPhysicalDeviceExternalBufferProperties,
        extensions[Extension::VK_KHR_external_memory_capabilities]
    );
    proc_alias_khr!(
        vkGetPhysicalDeviceExternalFenceProperties,
        extensions[Extension::VK_KHR_external_fence_capabilities]
    );
    proc_alias_khr!(
        vkGetPhysicalDeviceExternalSemaphoreProperties,
        extensions[Extension::VK_KHR_external_semaphore_capabilities]
    );
    proc_alias_khr!(
        vkGetPhysicalDeviceFeatures2,
        extensions[Extension::VK_KHR_get_physical_device_properties2]
    );
    proc_alias_khr!(
        vkGetPhysicalDeviceFormatProperties2,
        extensions[Extension::VK_KHR_get_physical_device_properties2]
    );
    proc_alias_khr!(
        vkGetPhysicalDeviceImageFormatProperties2,
        extensions[Extension::VK_KHR_get_physical_device_properties2]
    );
    proc_alias_khr!(
        vkGetPhysicalDeviceMemoryProperties2,
        extensions[Extension::VK_KHR_get_physical_device_properties2]
    );
    proc_alias_khr!(
        vkGetPhysicalDeviceProperties2,
        extensions[Extension::VK_KHR_get_physical_device_properties2]
    );
    proc_alias_khr!(
        vkGetPhysicalDeviceQueueFamilyProperties2,
        extensions[Extension::VK_KHR_get_physical_device_properties2]
    );
    proc_alias_khr!(
        vkGetPhysicalDeviceSparseImageFormatProperties2,
        extensions[Extension::VK_KHR_get_physical_device_properties2]
    );
    proc_alias_khr!(
        vkTrimCommandPool,
        extensions[Extension::VK_KHR_maintenance1]
    );
    proc_alias_khr!(
        vkUpdateDescriptorSetWithTemplate,
        extensions[Extension::VK_KHR_descriptor_update_template]
    );
    macro_rules! proc_address {
        ($name:ident, $pfn_name:ident, $required_scope:ident, $required_extension:expr) => {
            if stringify!($name) == name {
                if scope.$required_scope && $required_extension {
                    let f: $pfn_name = Some($name);
                    return unsafe { transmute(f) };
                } else {
                    return None;
                }
            }
        };
    }
    #[cfg_attr(rustfmt, rustfmt_skip)]
    {
        proc_address!(vkCreateInstance, PFN_vkCreateInstance, global, true);
        proc_address!(vkEnumerateInstanceExtensionProperties, PFN_vkEnumerateInstanceExtensionProperties, global, true);
        proc_address!(vkEnumerateInstanceLayerProperties, PFN_vkEnumerateInstanceLayerProperties, global, true);
        proc_address!(vkEnumerateInstanceVersion, PFN_vkEnumerateInstanceVersion, global, true);

        proc_address!(vkAllocateCommandBuffers, PFN_vkAllocateCommandBuffers, device, true);
        proc_address!(vkAllocateDescriptorSets, PFN_vkAllocateDescriptorSets, device, true);
        proc_address!(vkAllocateMemory, PFN_vkAllocateMemory, device, true);
        proc_address!(vkBeginCommandBuffer, PFN_vkBeginCommandBuffer, device, true);
        proc_address!(vkBindBufferMemory, PFN_vkBindBufferMemory, device, true);
        proc_address!(vkBindBufferMemory2, PFN_vkBindBufferMemory2, device, true);
        proc_address!(vkBindImageMemory, PFN_vkBindImageMemory, device, true);
        proc_address!(vkBindImageMemory2, PFN_vkBindImageMemory2, device, true);
        proc_address!(vkCmdBeginQuery, PFN_vkCmdBeginQuery, device, true);
        proc_address!(vkCmdBeginRenderPass, PFN_vkCmdBeginRenderPass, device, true);
        proc_address!(vkCmdBindDescriptorSets, PFN_vkCmdBindDescriptorSets, device, true);
        proc_address!(vkCmdBindIndexBuffer, PFN_vkCmdBindIndexBuffer, device, true);
        proc_address!(vkCmdBindPipeline, PFN_vkCmdBindPipeline, device, true);
        proc_address!(vkCmdBindVertexBuffers, PFN_vkCmdBindVertexBuffers, device, true);
        proc_address!(vkCmdBlitImage, PFN_vkCmdBlitImage, device, true);
        proc_address!(vkCmdClearAttachments, PFN_vkCmdClearAttachments, device, true);
        proc_address!(vkCmdClearColorImage, PFN_vkCmdClearColorImage, device, true);
        proc_address!(vkCmdClearDepthStencilImage, PFN_vkCmdClearDepthStencilImage, device, true);
        proc_address!(vkCmdCopyBuffer, PFN_vkCmdCopyBuffer, device, true);
        proc_address!(vkCmdCopyBufferToImage, PFN_vkCmdCopyBufferToImage, device, true);
        proc_address!(vkCmdCopyImage, PFN_vkCmdCopyImage, device, true);
        proc_address!(vkCmdCopyImageToBuffer, PFN_vkCmdCopyImageToBuffer, device, true);
        proc_address!(vkCmdCopyQueryPoolResults, PFN_vkCmdCopyQueryPoolResults, device, true);
        proc_address!(vkCmdDispatch, PFN_vkCmdDispatch, device, true);
        proc_address!(vkCmdDispatchBase, PFN_vkCmdDispatchBase, device, true);
        proc_address!(vkCmdDispatchIndirect, PFN_vkCmdDispatchIndirect, device, true);
        proc_address!(vkCmdDraw, PFN_vkCmdDraw, device, true);
        proc_address!(vkCmdDrawIndexed, PFN_vkCmdDrawIndexed, device, true);
        proc_address!(vkCmdDrawIndexedIndirect, PFN_vkCmdDrawIndexedIndirect, device, true);
        proc_address!(vkCmdDrawIndirect, PFN_vkCmdDrawIndirect, device, true);
        proc_address!(vkCmdEndQuery, PFN_vkCmdEndQuery, device, true);
        proc_address!(vkCmdEndRenderPass, PFN_vkCmdEndRenderPass, device, true);
        proc_address!(vkCmdExecuteCommands, PFN_vkCmdExecuteCommands, device, true);
        proc_address!(vkCmdFillBuffer, PFN_vkCmdFillBuffer, device, true);
        proc_address!(vkCmdNextSubpass, PFN_vkCmdNextSubpass, device, true);
        proc_address!(vkCmdPipelineBarrier, PFN_vkCmdPipelineBarrier, device, true);
        proc_address!(vkCmdPushConstants, PFN_vkCmdPushConstants, device, true);
        proc_address!(vkCmdResetEvent, PFN_vkCmdResetEvent, device, true);
        proc_address!(vkCmdResetQueryPool, PFN_vkCmdResetQueryPool, device, true);
        proc_address!(vkCmdResolveImage, PFN_vkCmdResolveImage, device, true);
        proc_address!(vkCmdSetBlendConstants, PFN_vkCmdSetBlendConstants, device, true);
        proc_address!(vkCmdSetDepthBias, PFN_vkCmdSetDepthBias, device, true);
        proc_address!(vkCmdSetDepthBounds, PFN_vkCmdSetDepthBounds, device, true);
        proc_address!(vkCmdSetDeviceMask, PFN_vkCmdSetDeviceMask, device, true);
        proc_address!(vkCmdSetEvent, PFN_vkCmdSetEvent, device, true);
        proc_address!(vkCmdSetLineWidth, PFN_vkCmdSetLineWidth, device, true);
        proc_address!(vkCmdSetScissor, PFN_vkCmdSetScissor, device, true);
        proc_address!(vkCmdSetStencilCompareMask, PFN_vkCmdSetStencilCompareMask, device, true);
        proc_address!(vkCmdSetStencilReference, PFN_vkCmdSetStencilReference, device, true);
        proc_address!(vkCmdSetStencilWriteMask, PFN_vkCmdSetStencilWriteMask, device, true);
        proc_address!(vkCmdSetViewport, PFN_vkCmdSetViewport, device, true);
        proc_address!(vkCmdUpdateBuffer, PFN_vkCmdUpdateBuffer, device, true);
        proc_address!(vkCmdWaitEvents, PFN_vkCmdWaitEvents, device, true);
        proc_address!(vkCmdWriteTimestamp, PFN_vkCmdWriteTimestamp, device, true);
        proc_address!(vkCreateBuffer, PFN_vkCreateBuffer, device, true);
        proc_address!(vkCreateBufferView, PFN_vkCreateBufferView, device, true);
        proc_address!(vkCreateCommandPool, PFN_vkCreateCommandPool, device, true);
        proc_address!(vkCreateComputePipelines, PFN_vkCreateComputePipelines, device, true);
        proc_address!(vkCreateDescriptorPool, PFN_vkCreateDescriptorPool, device, true);
        proc_address!(vkCreateDescriptorSetLayout, PFN_vkCreateDescriptorSetLayout, device, true);
        proc_address!(vkCreateDescriptorUpdateTemplate, PFN_vkCreateDescriptorUpdateTemplate, device, true);
        proc_address!(vkCreateDevice, PFN_vkCreateDevice, instance, true);
        proc_address!(vkCreateEvent, PFN_vkCreateEvent, device, true);
        proc_address!(vkCreateFence, PFN_vkCreateFence, device, true);
        proc_address!(vkCreateFramebuffer, PFN_vkCreateFramebuffer, device, true);
        proc_address!(vkCreateGraphicsPipelines, PFN_vkCreateGraphicsPipelines, device, true);
        proc_address!(vkCreateImage, PFN_vkCreateImage, device, true);
        proc_address!(vkCreateImageView, PFN_vkCreateImageView, device, true);
        proc_address!(vkCreatePipelineCache, PFN_vkCreatePipelineCache, device, true);
        proc_address!(vkCreatePipelineLayout, PFN_vkCreatePipelineLayout, device, true);
        proc_address!(vkCreateQueryPool, PFN_vkCreateQueryPool, device, true);
        proc_address!(vkCreateRenderPass, PFN_vkCreateRenderPass, device, true);
        proc_address!(vkCreateSampler, PFN_vkCreateSampler, device, true);
        proc_address!(vkCreateSamplerYcbcrConversion, PFN_vkCreateSamplerYcbcrConversion, device, true);
        proc_address!(vkCreateSemaphore, PFN_vkCreateSemaphore, device, true);
        proc_address!(vkCreateShaderModule, PFN_vkCreateShaderModule, device, true);
        proc_address!(vkDestroyBuffer, PFN_vkDestroyBuffer, device, true);
        proc_address!(vkDestroyBufferView, PFN_vkDestroyBufferView, device, true);
        proc_address!(vkDestroyCommandPool, PFN_vkDestroyCommandPool, device, true);
        proc_address!(vkDestroyDescriptorPool, PFN_vkDestroyDescriptorPool, device, true);
        proc_address!(vkDestroyDescriptorSetLayout, PFN_vkDestroyDescriptorSetLayout, device, true);
        proc_address!(vkDestroyDescriptorUpdateTemplate, PFN_vkDestroyDescriptorUpdateTemplate, device, true);
        proc_address!(vkDestroyDevice, PFN_vkDestroyDevice, device, true);
        proc_address!(vkDestroyEvent, PFN_vkDestroyEvent, device, true);
        proc_address!(vkDestroyFence, PFN_vkDestroyFence, device, true);
        proc_address!(vkDestroyFramebuffer, PFN_vkDestroyFramebuffer, device, true);
        proc_address!(vkDestroyImage, PFN_vkDestroyImage, device, true);
        proc_address!(vkDestroyImageView, PFN_vkDestroyImageView, device, true);
        proc_address!(vkDestroyInstance, PFN_vkDestroyInstance, instance, true);
        proc_address!(vkDestroyPipeline, PFN_vkDestroyPipeline, device, true);
        proc_address!(vkDestroyPipelineCache, PFN_vkDestroyPipelineCache, device, true);
        proc_address!(vkDestroyPipelineLayout, PFN_vkDestroyPipelineLayout, device, true);
        proc_address!(vkDestroyQueryPool, PFN_vkDestroyQueryPool, device, true);
        proc_address!(vkDestroyRenderPass, PFN_vkDestroyRenderPass, device, true);
        proc_address!(vkDestroySampler, PFN_vkDestroySampler, device, true);
        proc_address!(vkDestroySamplerYcbcrConversion, PFN_vkDestroySamplerYcbcrConversion, device, true);
        proc_address!(vkDestroySemaphore, PFN_vkDestroySemaphore, device, true);
        proc_address!(vkDestroyShaderModule, PFN_vkDestroyShaderModule, device, true);
        proc_address!(vkDeviceWaitIdle, PFN_vkDeviceWaitIdle, device, true);
        proc_address!(vkEndCommandBuffer, PFN_vkEndCommandBuffer, device, true);
        proc_address!(vkEnumerateDeviceExtensionProperties, PFN_vkEnumerateDeviceExtensionProperties, instance, true);
        proc_address!(vkEnumerateDeviceLayerProperties, PFN_vkEnumerateDeviceLayerProperties, instance, true);
        proc_address!(vkEnumeratePhysicalDeviceGroups, PFN_vkEnumeratePhysicalDeviceGroups, instance, true);
        proc_address!(vkEnumeratePhysicalDevices, PFN_vkEnumeratePhysicalDevices, instance, true);
        proc_address!(vkFlushMappedMemoryRanges, PFN_vkFlushMappedMemoryRanges, device, true);
        proc_address!(vkFreeCommandBuffers, PFN_vkFreeCommandBuffers, device, true);
        proc_address!(vkFreeDescriptorSets, PFN_vkFreeDescriptorSets, device, true);
        proc_address!(vkFreeMemory, PFN_vkFreeMemory, device, true);
        proc_address!(vkGetBufferMemoryRequirements, PFN_vkGetBufferMemoryRequirements, device, true);
        proc_address!(vkGetBufferMemoryRequirements2, PFN_vkGetBufferMemoryRequirements2, device, true);
        proc_address!(vkGetDescriptorSetLayoutSupport, PFN_vkGetDescriptorSetLayoutSupport, device, true);
        proc_address!(vkGetDeviceGroupPeerMemoryFeatures, PFN_vkGetDeviceGroupPeerMemoryFeatures, device, true);
        proc_address!(vkGetDeviceMemoryCommitment, PFN_vkGetDeviceMemoryCommitment, device, true);
        proc_address!(vkGetDeviceProcAddr, PFN_vkGetDeviceProcAddr, device, true);
        proc_address!(vkGetDeviceQueue, PFN_vkGetDeviceQueue, device, true);
        proc_address!(vkGetDeviceQueue2, PFN_vkGetDeviceQueue2, device, true);
        proc_address!(vkGetEventStatus, PFN_vkGetEventStatus, device, true);
        proc_address!(vkGetFenceStatus, PFN_vkGetFenceStatus, device, true);
        proc_address!(vkGetImageMemoryRequirements, PFN_vkGetImageMemoryRequirements, device, true);
        proc_address!(vkGetImageMemoryRequirements2, PFN_vkGetImageMemoryRequirements2, device, true);
        proc_address!(vkGetImageSparseMemoryRequirements, PFN_vkGetImageSparseMemoryRequirements, device, true);
        proc_address!(vkGetImageSparseMemoryRequirements2, PFN_vkGetImageSparseMemoryRequirements2, device, true);
        proc_address!(vkGetImageSubresourceLayout, PFN_vkGetImageSubresourceLayout, device, true);
        proc_address!(vkGetInstanceProcAddr, PFN_vkGetInstanceProcAddr, device, true);
        proc_address!(vkGetPhysicalDeviceExternalBufferProperties, PFN_vkGetPhysicalDeviceExternalBufferProperties, instance, true);
        proc_address!(vkGetPhysicalDeviceExternalFenceProperties, PFN_vkGetPhysicalDeviceExternalFenceProperties, instance, true);
        proc_address!(vkGetPhysicalDeviceExternalSemaphoreProperties, PFN_vkGetPhysicalDeviceExternalSemaphoreProperties, instance, true);
        proc_address!(vkGetPhysicalDeviceFeatures, PFN_vkGetPhysicalDeviceFeatures, instance, true);
        proc_address!(vkGetPhysicalDeviceFeatures2, PFN_vkGetPhysicalDeviceFeatures2, instance, true);
        proc_address!(vkGetPhysicalDeviceFormatProperties, PFN_vkGetPhysicalDeviceFormatProperties, instance, true);
        proc_address!(vkGetPhysicalDeviceFormatProperties2, PFN_vkGetPhysicalDeviceFormatProperties2, instance, true);
        proc_address!(vkGetPhysicalDeviceImageFormatProperties, PFN_vkGetPhysicalDeviceImageFormatProperties, instance, true);
        proc_address!(vkGetPhysicalDeviceImageFormatProperties2, PFN_vkGetPhysicalDeviceImageFormatProperties2, instance, true);
        proc_address!(vkGetPhysicalDeviceMemoryProperties, PFN_vkGetPhysicalDeviceMemoryProperties, instance, true);
        proc_address!(vkGetPhysicalDeviceMemoryProperties2, PFN_vkGetPhysicalDeviceMemoryProperties2, instance, true);
        proc_address!(vkGetPhysicalDeviceProperties, PFN_vkGetPhysicalDeviceProperties, instance, true);
        proc_address!(vkGetPhysicalDeviceProperties2, PFN_vkGetPhysicalDeviceProperties2, instance, true);
        proc_address!(vkGetPhysicalDeviceQueueFamilyProperties, PFN_vkGetPhysicalDeviceQueueFamilyProperties, instance, true);
        proc_address!(vkGetPhysicalDeviceQueueFamilyProperties2, PFN_vkGetPhysicalDeviceQueueFamilyProperties2, instance, true);
        proc_address!(vkGetPhysicalDeviceSparseImageFormatProperties, PFN_vkGetPhysicalDeviceSparseImageFormatProperties, instance, true);
        proc_address!(vkGetPhysicalDeviceSparseImageFormatProperties2, PFN_vkGetPhysicalDeviceSparseImageFormatProperties2, instance, true);
        proc_address!(vkGetPipelineCacheData, PFN_vkGetPipelineCacheData, device, true);
        proc_address!(vkGetQueryPoolResults, PFN_vkGetQueryPoolResults, device, true);
        proc_address!(vkGetRenderAreaGranularity, PFN_vkGetRenderAreaGranularity, device, true);
        proc_address!(vkInvalidateMappedMemoryRanges, PFN_vkInvalidateMappedMemoryRanges, device, true);
        proc_address!(vkMapMemory, PFN_vkMapMemory, device, true);
        proc_address!(vkMergePipelineCaches, PFN_vkMergePipelineCaches, device, true);
        proc_address!(vkQueueBindSparse, PFN_vkQueueBindSparse, device, true);
        proc_address!(vkQueueSubmit, PFN_vkQueueSubmit, device, true);
        proc_address!(vkQueueWaitIdle, PFN_vkQueueWaitIdle, device, true);
        proc_address!(vkResetCommandBuffer, PFN_vkResetCommandBuffer, device, true);
        proc_address!(vkResetCommandPool, PFN_vkResetCommandPool, device, true);
        proc_address!(vkResetDescriptorPool, PFN_vkResetDescriptorPool, device, true);
        proc_address!(vkResetEvent, PFN_vkResetEvent, device, true);
        proc_address!(vkResetFences, PFN_vkResetFences, device, true);
        proc_address!(vkSetEvent, PFN_vkSetEvent, device, true);
        proc_address!(vkTrimCommandPool, PFN_vkTrimCommandPool, device, true);
        proc_address!(vkUnmapMemory, PFN_vkUnmapMemory, device, true);
        proc_address!(vkUpdateDescriptorSets, PFN_vkUpdateDescriptorSets, device, true);
        proc_address!(vkUpdateDescriptorSetWithTemplate, PFN_vkUpdateDescriptorSetWithTemplate, device, true);
        proc_address!(vkWaitForFences, PFN_vkWaitForFences, device, true);

        proc_address!(vkDestroySurfaceKHR, PFN_vkDestroySurfaceKHR, device, extensions[Extension::VK_KHR_surface]);
        proc_address!(vkGetPhysicalDeviceSurfaceSupportKHR, PFN_vkGetPhysicalDeviceSurfaceSupportKHR, device, extensions[Extension::VK_KHR_surface]);
        proc_address!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR, PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR, device, extensions[Extension::VK_KHR_surface]);
        proc_address!(vkGetPhysicalDeviceSurfaceFormatsKHR, PFN_vkGetPhysicalDeviceSurfaceFormatsKHR, device, extensions[Extension::VK_KHR_surface]);
        proc_address!(vkGetPhysicalDeviceSurfacePresentModesKHR, PFN_vkGetPhysicalDeviceSurfacePresentModesKHR, device, extensions[Extension::VK_KHR_surface]);

        proc_address!(vkCreateSwapchainKHR, PFN_vkCreateSwapchainKHR, device, extensions[Extension::VK_KHR_swapchain]);
        proc_address!(vkDestroySwapchainKHR, PFN_vkDestroySwapchainKHR, device, extensions[Extension::VK_KHR_swapchain]);
        proc_address!(vkGetSwapchainImagesKHR, PFN_vkGetSwapchainImagesKHR, device, extensions[Extension::VK_KHR_swapchain]);
        proc_address!(vkAcquireNextImageKHR, PFN_vkAcquireNextImageKHR, device, extensions[Extension::VK_KHR_swapchain]);
        proc_address!(vkQueuePresentKHR, PFN_vkQueuePresentKHR, device, extensions[Extension::VK_KHR_swapchain]);
        proc_address!(vkGetDeviceGroupPresentCapabilitiesKHR, PFN_vkGetDeviceGroupPresentCapabilitiesKHR, device, extensions[Extension::VK_KHR_swapchain]);
        proc_address!(vkGetDeviceGroupSurfacePresentModesKHR, PFN_vkGetDeviceGroupSurfacePresentModesKHR, device, extensions[Extension::VK_KHR_swapchain]);
        proc_address!(vkGetPhysicalDevicePresentRectanglesKHR, PFN_vkGetPhysicalDevicePresentRectanglesKHR, device, extensions[Extension::VK_KHR_swapchain]);
        proc_address!(vkAcquireNextImage2KHR, PFN_vkAcquireNextImage2KHR, device, extensions[Extension::VK_KHR_swapchain]);

        #[cfg(target_os = "linux")]
        proc_address!(vkCreateXcbSurfaceKHR, PFN_vkCreateXcbSurfaceKHR, device, extensions[Extension::VK_KHR_xcb_surface]);
        #[cfg(target_os = "linux")]
        proc_address!(vkGetPhysicalDeviceXcbPresentationSupportKHR, PFN_vkGetPhysicalDeviceXcbPresentationSupportKHR, device, extensions[Extension::VK_KHR_xcb_surface]);
        /*
        proc_address!(vkCmdBeginConditionalRenderingEXT, PFN_vkCmdBeginConditionalRenderingEXT, device, unknown);
        proc_address!(vkCmdBeginDebugUtilsLabelEXT, PFN_vkCmdBeginDebugUtilsLabelEXT, device, unknown);
        proc_address!(vkCmdBeginRenderPass2KHR, PFN_vkCmdBeginRenderPass2KHR, device, unknown);
        proc_address!(vkCmdBindShadingRateImageNV, PFN_vkCmdBindShadingRateImageNV, device, unknown);
        proc_address!(vkCmdDebugMarkerBeginEXT, PFN_vkCmdDebugMarkerBeginEXT, device, unknown);
        proc_address!(vkCmdDebugMarkerEndEXT, PFN_vkCmdDebugMarkerEndEXT, device, unknown);
        proc_address!(vkCmdDebugMarkerInsertEXT, PFN_vkCmdDebugMarkerInsertEXT, device, unknown);
        proc_address!(vkCmdDrawIndexedIndirectCountAMD, PFN_vkCmdDrawIndexedIndirectCountAMD, device, unknown);
        proc_address!(vkCmdDrawIndexedIndirectCountKHR, PFN_vkCmdDrawIndexedIndirectCountKHR, device, unknown);
        proc_address!(vkCmdDrawIndirectCountAMD, PFN_vkCmdDrawIndirectCountAMD, device, unknown);
        proc_address!(vkCmdDrawIndirectCountKHR, PFN_vkCmdDrawIndirectCountKHR, device, unknown);
        proc_address!(vkCmdDrawMeshTasksIndirectCountNV, PFN_vkCmdDrawMeshTasksIndirectCountNV, device, unknown);
        proc_address!(vkCmdDrawMeshTasksIndirectNV, PFN_vkCmdDrawMeshTasksIndirectNV, device, unknown);
        proc_address!(vkCmdDrawMeshTasksNV, PFN_vkCmdDrawMeshTasksNV, device, unknown);
        proc_address!(vkCmdEndConditionalRenderingEXT, PFN_vkCmdEndConditionalRenderingEXT, device, unknown);
        proc_address!(vkCmdEndDebugUtilsLabelEXT, PFN_vkCmdEndDebugUtilsLabelEXT, device, unknown);
        proc_address!(vkCmdEndRenderPass2KHR, PFN_vkCmdEndRenderPass2KHR, device, unknown);
        proc_address!(vkCmdInsertDebugUtilsLabelEXT, PFN_vkCmdInsertDebugUtilsLabelEXT, device, unknown);
        proc_address!(vkCmdNextSubpass2KHR, PFN_vkCmdNextSubpass2KHR, device, unknown);
        proc_address!(vkCmdPushDescriptorSetKHR, PFN_vkCmdPushDescriptorSetKHR, device, unknown);
        proc_address!(vkCmdPushDescriptorSetWithTemplateKHR, PFN_vkCmdPushDescriptorSetWithTemplateKHR, device, unknown);
        proc_address!(vkCmdSetCheckpointNV, PFN_vkCmdSetCheckpointNV, device, unknown);
        proc_address!(vkCmdSetCoarseSampleOrderNV, PFN_vkCmdSetCoarseSampleOrderNV, device, unknown);
        proc_address!(vkCmdSetDiscardRectangleEXT, PFN_vkCmdSetDiscardRectangleEXT, device, unknown);
        proc_address!(vkCmdSetExclusiveScissorNV, PFN_vkCmdSetExclusiveScissorNV, device, unknown);
        proc_address!(vkCmdSetSampleLocationsEXT, PFN_vkCmdSetSampleLocationsEXT, device, unknown);
        proc_address!(vkCmdSetViewportShadingRatePaletteNV, PFN_vkCmdSetViewportShadingRatePaletteNV, device, unknown);
        proc_address!(vkCmdSetViewportWScalingNV, PFN_vkCmdSetViewportWScalingNV, device, unknown);
        proc_address!(vkCmdWriteBufferMarkerAMD, PFN_vkCmdWriteBufferMarkerAMD, device, unknown);
        proc_address!(vkCreateDebugReportCallbackEXT, PFN_vkCreateDebugReportCallbackEXT, device, unknown);
        proc_address!(vkCreateDebugUtilsMessengerEXT, PFN_vkCreateDebugUtilsMessengerEXT, device, unknown);
        proc_address!(vkCreateDisplayModeKHR, PFN_vkCreateDisplayModeKHR, device, unknown);
        proc_address!(vkCreateDisplayPlaneSurfaceKHR, PFN_vkCreateDisplayPlaneSurfaceKHR, device, unknown);
        proc_address!(vkCreateRenderPass2KHR, PFN_vkCreateRenderPass2KHR, device, unknown);
        proc_address!(vkCreateSharedSwapchainsKHR, PFN_vkCreateSharedSwapchainsKHR, device, unknown);
        proc_address!(vkCreateValidationCacheEXT, PFN_vkCreateValidationCacheEXT, device, unknown);
        proc_address!(vkDebugMarkerSetObjectNameEXT, PFN_vkDebugMarkerSetObjectNameEXT, device, unknown);
        proc_address!(vkDebugMarkerSetObjectTagEXT, PFN_vkDebugMarkerSetObjectTagEXT, device, unknown);
        proc_address!(vkDebugReportCallbackEXT, PFN_vkDebugReportCallbackEXT, device, unknown);
        proc_address!(vkDebugReportMessageEXT, PFN_vkDebugReportMessageEXT, device, unknown);
        proc_address!(vkDebugUtilsMessengerCallbackEXT, PFN_vkDebugUtilsMessengerCallbackEXT, device, unknown);
        proc_address!(vkDestroyDebugReportCallbackEXT, PFN_vkDestroyDebugReportCallbackEXT, device, unknown);
        proc_address!(vkDestroyDebugUtilsMessengerEXT, PFN_vkDestroyDebugUtilsMessengerEXT, device, unknown);
        proc_address!(vkDestroyValidationCacheEXT, PFN_vkDestroyValidationCacheEXT, device, unknown);
        proc_address!(vkDisplayPowerControlEXT, PFN_vkDisplayPowerControlEXT, device, unknown);
        proc_address!(vkGetDisplayModeProperties2KHR, PFN_vkGetDisplayModeProperties2KHR, device, unknown);
        proc_address!(vkGetDisplayModePropertiesKHR, PFN_vkGetDisplayModePropertiesKHR, device, unknown);
        proc_address!(vkGetDisplayPlaneCapabilities2KHR, PFN_vkGetDisplayPlaneCapabilities2KHR, device, unknown);
        proc_address!(vkGetDisplayPlaneCapabilitiesKHR, PFN_vkGetDisplayPlaneCapabilitiesKHR, device, unknown);
        proc_address!(vkGetDisplayPlaneSupportedDisplaysKHR, PFN_vkGetDisplayPlaneSupportedDisplaysKHR, device, unknown);
        proc_address!(vkGetFenceFdKHR, PFN_vkGetFenceFdKHR, device, unknown);
        proc_address!(vkGetMemoryFdKHR, PFN_vkGetMemoryFdKHR, device, unknown);
        proc_address!(vkGetMemoryFdPropertiesKHR, PFN_vkGetMemoryFdPropertiesKHR, device, unknown);
        proc_address!(vkGetMemoryHostPointerPropertiesEXT, PFN_vkGetMemoryHostPointerPropertiesEXT, device, unknown);
        proc_address!(vkGetPastPresentationTimingGOOGLE, PFN_vkGetPastPresentationTimingGOOGLE, device, unknown);
        proc_address!(vkGetPhysicalDeviceDisplayPlaneProperties2KHR, PFN_vkGetPhysicalDeviceDisplayPlaneProperties2KHR, device, unknown);
        proc_address!(vkGetPhysicalDeviceDisplayPlanePropertiesKHR, PFN_vkGetPhysicalDeviceDisplayPlanePropertiesKHR, device, unknown);
        proc_address!(vkGetPhysicalDeviceDisplayProperties2KHR, PFN_vkGetPhysicalDeviceDisplayProperties2KHR, device, unknown);
        proc_address!(vkGetPhysicalDeviceDisplayPropertiesKHR, PFN_vkGetPhysicalDeviceDisplayPropertiesKHR, device, unknown);
        proc_address!(vkGetPhysicalDeviceExternalImageFormatPropertiesNV, PFN_vkGetPhysicalDeviceExternalImageFormatPropertiesNV, device, unknown);
        proc_address!(vkGetPhysicalDeviceMultisamplePropertiesEXT, PFN_vkGetPhysicalDeviceMultisamplePropertiesEXT, device, unknown);
        proc_address!(vkGetPhysicalDeviceSurfaceCapabilities2EXT, PFN_vkGetPhysicalDeviceSurfaceCapabilities2EXT, device, unknown);
        proc_address!(vkGetPhysicalDeviceSurfaceCapabilities2KHR, PFN_vkGetPhysicalDeviceSurfaceCapabilities2KHR, device, unknown);
        proc_address!(vkGetPhysicalDeviceSurfaceFormats2KHR, PFN_vkGetPhysicalDeviceSurfaceFormats2KHR, device, unknown);
        proc_address!(vkGetQueueCheckpointDataNV, PFN_vkGetQueueCheckpointDataNV, device, unknown);
        proc_address!(vkGetRefreshCycleDurationGOOGLE, PFN_vkGetRefreshCycleDurationGOOGLE, device, unknown);
        proc_address!(vkGetSemaphoreFdKHR, PFN_vkGetSemaphoreFdKHR, device, unknown);
        proc_address!(vkGetShaderInfoAMD, PFN_vkGetShaderInfoAMD, device, unknown);
        proc_address!(vkGetSwapchainCounterEXT, PFN_vkGetSwapchainCounterEXT, device, unknown);
        proc_address!(vkGetSwapchainStatusKHR, PFN_vkGetSwapchainStatusKHR, device, unknown);
        proc_address!(vkGetValidationCacheDataEXT, PFN_vkGetValidationCacheDataEXT, device, unknown);
        proc_address!(vkImportFenceFdKHR, PFN_vkImportFenceFdKHR, device, unknown);
        proc_address!(vkImportSemaphoreFdKHR, PFN_vkImportSemaphoreFdKHR, device, unknown);
        proc_address!(vkMergeValidationCachesEXT, PFN_vkMergeValidationCachesEXT, device, unknown);
        proc_address!(vkQueueBeginDebugUtilsLabelEXT, PFN_vkQueueBeginDebugUtilsLabelEXT, device, unknown);
        proc_address!(vkQueueEndDebugUtilsLabelEXT, PFN_vkQueueEndDebugUtilsLabelEXT, device, unknown);
        proc_address!(vkQueueInsertDebugUtilsLabelEXT, PFN_vkQueueInsertDebugUtilsLabelEXT, device, unknown);
        proc_address!(vkRegisterDeviceEventEXT, PFN_vkRegisterDeviceEventEXT, device, unknown);
        proc_address!(vkRegisterDisplayEventEXT, PFN_vkRegisterDisplayEventEXT, device, unknown);
        proc_address!(vkReleaseDisplayEXT, PFN_vkReleaseDisplayEXT, device, unknown);
        proc_address!(vkSetDebugUtilsObjectNameEXT, PFN_vkSetDebugUtilsObjectNameEXT, device, unknown);
        proc_address!(vkSetDebugUtilsObjectTagEXT, PFN_vkSetDebugUtilsObjectTagEXT, device, unknown);
        proc_address!(vkSetHdrMetadataEXT, PFN_vkSetHdrMetadataEXT, device, unknown);
        proc_address!(vkSubmitDebugUtilsMessageEXT, PFN_vkSubmitDebugUtilsMessageEXT, device, unknown);
        */
    }
    //eprintln!("unknown function: {:?}", name);
    None
}

#[derive(Debug, Copy, Clone)]
pub struct Features {
    features: api::VkPhysicalDeviceFeatures,
    physical_device_16bit_storage_features: api::VkPhysicalDevice16BitStorageFeatures,
    sampler_ycbcr_conversion_features: api::VkPhysicalDeviceSamplerYcbcrConversionFeatures,
    variable_pointer_features: api::VkPhysicalDeviceVariablePointerFeatures,
    shader_draw_parameter_features: api::VkPhysicalDeviceShaderDrawParameterFeatures,
    protected_memory_features: api::VkPhysicalDeviceProtectedMemoryFeatures,
    multiview_features: api::VkPhysicalDeviceMultiviewFeatures,
}

impl Features {
    fn new() -> Self {
        Self {
            features: api::VkPhysicalDeviceFeatures {
                robustBufferAccess: api::VK_TRUE,
                fullDrawIndexUint32: api::VK_TRUE,
                imageCubeArray: api::VK_TRUE,
                independentBlend: api::VK_FALSE,
                geometryShader: api::VK_FALSE,
                tessellationShader: api::VK_FALSE,
                sampleRateShading: api::VK_FALSE,
                dualSrcBlend: api::VK_FALSE,
                logicOp: api::VK_TRUE,
                multiDrawIndirect: api::VK_TRUE,
                drawIndirectFirstInstance: api::VK_TRUE,
                depthClamp: api::VK_FALSE,
                depthBiasClamp: api::VK_FALSE,
                fillModeNonSolid: api::VK_TRUE,
                depthBounds: api::VK_FALSE,
                wideLines: api::VK_FALSE,
                largePoints: api::VK_FALSE,
                alphaToOne: api::VK_TRUE,
                multiViewport: api::VK_TRUE,
                samplerAnisotropy: api::VK_FALSE,
                textureCompressionETC2: api::VK_FALSE, // FIXME: enable texture compression
                textureCompressionASTC_LDR: api::VK_FALSE, // FIXME: enable texture compression
                textureCompressionBC: api::VK_FALSE,   // FIXME: enable texture compression
                occlusionQueryPrecise: api::VK_FALSE,
                pipelineStatisticsQuery: api::VK_FALSE,
                vertexPipelineStoresAndAtomics: api::VK_TRUE,
                fragmentStoresAndAtomics: api::VK_TRUE,
                shaderTessellationAndGeometryPointSize: api::VK_FALSE,
                shaderImageGatherExtended: api::VK_FALSE,
                shaderStorageImageExtendedFormats: api::VK_FALSE,
                shaderStorageImageMultisample: api::VK_FALSE,
                shaderStorageImageReadWithoutFormat: api::VK_FALSE,
                shaderStorageImageWriteWithoutFormat: api::VK_FALSE,
                shaderUniformBufferArrayDynamicIndexing: api::VK_TRUE,
                shaderSampledImageArrayDynamicIndexing: api::VK_TRUE,
                shaderStorageBufferArrayDynamicIndexing: api::VK_TRUE,
                shaderStorageImageArrayDynamicIndexing: api::VK_TRUE,
                shaderClipDistance: api::VK_FALSE,
                shaderCullDistance: api::VK_FALSE,
                shaderFloat64: api::VK_TRUE,
                shaderInt64: api::VK_TRUE,
                shaderInt16: api::VK_TRUE,
                shaderResourceResidency: api::VK_FALSE,
                shaderResourceMinLod: api::VK_FALSE,
                sparseBinding: api::VK_FALSE,
                sparseResidencyBuffer: api::VK_FALSE,
                sparseResidencyImage2D: api::VK_FALSE,
                sparseResidencyImage3D: api::VK_FALSE,
                sparseResidency2Samples: api::VK_FALSE,
                sparseResidency4Samples: api::VK_FALSE,
                sparseResidency8Samples: api::VK_FALSE,
                sparseResidency16Samples: api::VK_FALSE,
                sparseResidencyAliased: api::VK_FALSE,
                variableMultisampleRate: api::VK_FALSE,
                inheritedQueries: api::VK_FALSE,
            },
            physical_device_16bit_storage_features: api::VkPhysicalDevice16BitStorageFeatures {
                sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES,
                pNext: null_mut(),
                storageBuffer16BitAccess: api::VK_TRUE,
                uniformAndStorageBuffer16BitAccess: api::VK_TRUE,
                storagePushConstant16: api::VK_TRUE,
                storageInputOutput16: api::VK_TRUE,
            },
            sampler_ycbcr_conversion_features:
                api::VkPhysicalDeviceSamplerYcbcrConversionFeatures {
                    sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES,
                    pNext: null_mut(),
                    samplerYcbcrConversion: api::VK_FALSE,
                },
            variable_pointer_features: api::VkPhysicalDeviceVariablePointerFeatures {
                sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES,
                pNext: null_mut(),
                variablePointersStorageBuffer: api::VK_TRUE,
                variablePointers: api::VK_TRUE,
            },
            shader_draw_parameter_features: api::VkPhysicalDeviceShaderDrawParameterFeatures {
                sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES,
                pNext: null_mut(),
                shaderDrawParameters: api::VK_TRUE,
            },
            protected_memory_features: api::VkPhysicalDeviceProtectedMemoryFeatures {
                sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES,
                pNext: null_mut(),
                protectedMemory: api::VK_FALSE,
            },
            multiview_features: api::VkPhysicalDeviceMultiviewFeatures {
                sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_FEATURES,
                pNext: null_mut(),
                multiview: api::VK_FALSE,
                multiviewGeometryShader: api::VK_FALSE,
                multiviewTessellationShader: api::VK_FALSE,
            },
        }
    }
    fn splat(value: bool) -> Self {
        let value32 = if value { api::VK_TRUE } else { api::VK_FALSE };
        Self {
            features: api::VkPhysicalDeviceFeatures {
                robustBufferAccess: value32,
                fullDrawIndexUint32: value32,
                imageCubeArray: value32,
                independentBlend: value32,
                geometryShader: value32,
                tessellationShader: value32,
                sampleRateShading: value32,
                dualSrcBlend: value32,
                logicOp: value32,
                multiDrawIndirect: value32,
                drawIndirectFirstInstance: value32,
                depthClamp: value32,
                depthBiasClamp: value32,
                fillModeNonSolid: value32,
                depthBounds: value32,
                wideLines: value32,
                largePoints: value32,
                alphaToOne: value32,
                multiViewport: value32,
                samplerAnisotropy: value32,
                textureCompressionETC2: value32,
                textureCompressionASTC_LDR: value32,
                textureCompressionBC: value32,
                occlusionQueryPrecise: value32,
                pipelineStatisticsQuery: value32,
                vertexPipelineStoresAndAtomics: value32,
                fragmentStoresAndAtomics: value32,
                shaderTessellationAndGeometryPointSize: value32,
                shaderImageGatherExtended: value32,
                shaderStorageImageExtendedFormats: value32,
                shaderStorageImageMultisample: value32,
                shaderStorageImageReadWithoutFormat: value32,
                shaderStorageImageWriteWithoutFormat: value32,
                shaderUniformBufferArrayDynamicIndexing: value32,
                shaderSampledImageArrayDynamicIndexing: value32,
                shaderStorageBufferArrayDynamicIndexing: value32,
                shaderStorageImageArrayDynamicIndexing: value32,
                shaderClipDistance: value32,
                shaderCullDistance: value32,
                shaderFloat64: value32,
                shaderInt64: value32,
                shaderInt16: value32,
                shaderResourceResidency: value32,
                shaderResourceMinLod: value32,
                sparseBinding: value32,
                sparseResidencyBuffer: value32,
                sparseResidencyImage2D: value32,
                sparseResidencyImage3D: value32,
                sparseResidency2Samples: value32,
                sparseResidency4Samples: value32,
                sparseResidency8Samples: value32,
                sparseResidency16Samples: value32,
                sparseResidencyAliased: value32,
                variableMultisampleRate: value32,
                inheritedQueries: value32,
            },
            physical_device_16bit_storage_features: api::VkPhysicalDevice16BitStorageFeatures {
                sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES,
                pNext: null_mut(),
                storageBuffer16BitAccess: value32,
                uniformAndStorageBuffer16BitAccess: value32,
                storagePushConstant16: value32,
                storageInputOutput16: value32,
            },
            sampler_ycbcr_conversion_features:
                api::VkPhysicalDeviceSamplerYcbcrConversionFeatures {
                    sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES,
                    pNext: null_mut(),
                    samplerYcbcrConversion: value32,
                },
            variable_pointer_features: api::VkPhysicalDeviceVariablePointerFeatures {
                sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES,
                pNext: null_mut(),
                variablePointersStorageBuffer: value32,
                variablePointers: value32,
            },
            shader_draw_parameter_features: api::VkPhysicalDeviceShaderDrawParameterFeatures {
                sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES,
                pNext: null_mut(),
                shaderDrawParameters: value32,
            },
            protected_memory_features: api::VkPhysicalDeviceProtectedMemoryFeatures {
                sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES,
                pNext: null_mut(),
                protectedMemory: value32,
            },
            multiview_features: api::VkPhysicalDeviceMultiviewFeatures {
                sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_FEATURES,
                pNext: null_mut(),
                multiview: value32,
                multiviewGeometryShader: value32,
                multiviewTessellationShader: value32,
            },
        }
    }
    fn visit2_mut<F: FnMut(&mut bool, &mut bool)>(&mut self, rhs: &mut Self, f: F) {
        struct VisitorStruct<F: FnMut(&mut bool, &mut bool)>(F);
        trait Visitor<T> {
            fn visit(&mut self, v1: &mut T, v2: &mut T);
        }
        impl<F: FnMut(&mut bool, &mut bool)> Visitor<bool> for VisitorStruct<F> {
            fn visit(&mut self, v1: &mut bool, v2: &mut bool) {
                (self.0)(v1, v2);
            }
        }
        impl<F: FnMut(&mut bool, &mut bool)> Visitor<api::VkBool32> for VisitorStruct<F> {
            fn visit(&mut self, value1: &mut api::VkBool32, value2: &mut api::VkBool32) {
                let mut temp1 = *value1 != api::VK_FALSE;
                let mut temp2 = *value2 != api::VK_FALSE;
                (self.0)(&mut temp1, &mut temp2);
                *value1 = if temp1 { api::VK_TRUE } else { api::VK_FALSE };
                *value2 = if temp2 { api::VK_TRUE } else { api::VK_FALSE };
            }
        }
        let mut visitor = VisitorStruct(f);
        macro_rules! visit {
            ($member1:ident.$member2:ident) => {
                visitor.visit(&mut self.$member1.$member2, &mut rhs.$member1.$member2)
            };
            ($member:ident) => {
                visitor.visit(&mut self.$member1, &mut rhs.$member1)
            };
        }
        visit!(features.robustBufferAccess);
        visit!(features.fullDrawIndexUint32);
        visit!(features.imageCubeArray);
        visit!(features.independentBlend);
        visit!(features.geometryShader);
        visit!(features.tessellationShader);
        visit!(features.sampleRateShading);
        visit!(features.dualSrcBlend);
        visit!(features.logicOp);
        visit!(features.multiDrawIndirect);
        visit!(features.drawIndirectFirstInstance);
        visit!(features.depthClamp);
        visit!(features.depthBiasClamp);
        visit!(features.fillModeNonSolid);
        visit!(features.depthBounds);
        visit!(features.wideLines);
        visit!(features.largePoints);
        visit!(features.alphaToOne);
        visit!(features.multiViewport);
        visit!(features.samplerAnisotropy);
        visit!(features.textureCompressionETC2);
        visit!(features.textureCompressionASTC_LDR);
        visit!(features.textureCompressionBC);
        visit!(features.occlusionQueryPrecise);
        visit!(features.pipelineStatisticsQuery);
        visit!(features.vertexPipelineStoresAndAtomics);
        visit!(features.fragmentStoresAndAtomics);
        visit!(features.shaderTessellationAndGeometryPointSize);
        visit!(features.shaderImageGatherExtended);
        visit!(features.shaderStorageImageExtendedFormats);
        visit!(features.shaderStorageImageMultisample);
        visit!(features.shaderStorageImageReadWithoutFormat);
        visit!(features.shaderStorageImageWriteWithoutFormat);
        visit!(features.shaderUniformBufferArrayDynamicIndexing);
        visit!(features.shaderSampledImageArrayDynamicIndexing);
        visit!(features.shaderStorageBufferArrayDynamicIndexing);
        visit!(features.shaderStorageImageArrayDynamicIndexing);
        visit!(features.shaderClipDistance);
        visit!(features.shaderCullDistance);
        visit!(features.shaderFloat64);
        visit!(features.shaderInt64);
        visit!(features.shaderInt16);
        visit!(features.shaderResourceResidency);
        visit!(features.shaderResourceMinLod);
        visit!(features.sparseBinding);
        visit!(features.sparseResidencyBuffer);
        visit!(features.sparseResidencyImage2D);
        visit!(features.sparseResidencyImage3D);
        visit!(features.sparseResidency2Samples);
        visit!(features.sparseResidency4Samples);
        visit!(features.sparseResidency8Samples);
        visit!(features.sparseResidency16Samples);
        visit!(features.sparseResidencyAliased);
        visit!(features.variableMultisampleRate);
        visit!(features.inheritedQueries);
        visit!(physical_device_16bit_storage_features.storageBuffer16BitAccess);
        visit!(physical_device_16bit_storage_features.uniformAndStorageBuffer16BitAccess);
        visit!(physical_device_16bit_storage_features.storagePushConstant16);
        visit!(physical_device_16bit_storage_features.storageInputOutput16);
        visit!(sampler_ycbcr_conversion_features.samplerYcbcrConversion);
        visit!(variable_pointer_features.variablePointersStorageBuffer);
        visit!(variable_pointer_features.variablePointers);
        visit!(shader_draw_parameter_features.shaderDrawParameters);
        visit!(protected_memory_features.protectedMemory);
        visit!(multiview_features.multiview);
        visit!(multiview_features.multiviewGeometryShader);
        visit!(multiview_features.multiviewTessellationShader);
    }
    fn visit2<F: FnMut(bool, bool)>(mut self, mut rhs: Self, mut f: F) {
        self.visit2_mut(&mut rhs, |v1, v2| f(*v1, *v2));
    }
    fn visit_mut<F: FnMut(&mut bool)>(&mut self, mut f: F) {
        let mut rhs = *self;
        self.visit2_mut(&mut rhs, |v, _| f(v));
    }
    #[allow(dead_code)]
    fn visit<F: FnMut(bool)>(mut self, mut f: F) {
        self.visit_mut(|v| f(*v));
    }
}

trait ImportExportFeatureSet<T> {
    fn import_feature_set(&mut self, features: &T);
    fn export_feature_set(&self, features: &mut T);
}

impl ImportExportFeatureSet<api::VkPhysicalDeviceFeatures> for Features {
    fn import_feature_set(&mut self, features: &api::VkPhysicalDeviceFeatures) {
        self.features = *features;
    }
    fn export_feature_set(&self, features: &mut api::VkPhysicalDeviceFeatures) {
        *features = self.features;
    }
}

impl ImportExportFeatureSet<api::VkPhysicalDeviceFeatures2> for Features {
    fn import_feature_set(&mut self, features: &api::VkPhysicalDeviceFeatures2) {
        self.features = features.features;
    }
    fn export_feature_set(&self, features: &mut api::VkPhysicalDeviceFeatures2) {
        features.features = self.features;
    }
}

macro_rules! impl_import_export_feature_set {
    ($type:ident, $member:ident) => {
        impl ImportExportFeatureSet<api::$type> for Features {
            fn import_feature_set(&mut self, features: &api::$type) {
                self.$member = api::$type {
                    sType: self.$member.sType,
                    pNext: self.$member.pNext,
                    ..*features
                };
            }
            fn export_feature_set(&self, features: &mut api::$type) {
                *features = api::$type {
                    sType: features.sType,
                    pNext: features.pNext,
                    ..self.$member
                };
            }
        }
    };
}

impl_import_export_feature_set!(
    VkPhysicalDevice16BitStorageFeatures,
    physical_device_16bit_storage_features
);

impl_import_export_feature_set!(
    VkPhysicalDeviceSamplerYcbcrConversionFeatures,
    sampler_ycbcr_conversion_features
);

impl_import_export_feature_set!(
    VkPhysicalDeviceVariablePointerFeatures,
    variable_pointer_features
);

impl_import_export_feature_set!(
    VkPhysicalDeviceShaderDrawParameterFeatures,
    shader_draw_parameter_features
);

impl_import_export_feature_set!(
    VkPhysicalDeviceProtectedMemoryFeatures,
    protected_memory_features
);

impl_import_export_feature_set!(VkPhysicalDeviceMultiviewFeatures, multiview_features);

impl Eq for Features {}

impl PartialEq for Features {
    fn eq(&self, rhs: &Self) -> bool {
        let mut equal = true;
        self.visit2(*rhs, |a, b| equal &= a == b);
        equal
    }
}

impl BitAndAssign for Features {
    fn bitand_assign(&mut self, mut rhs: Self) {
        self.visit2_mut(&mut rhs, |l, r| *l &= *r);
    }
}

impl BitOrAssign for Features {
    fn bitor_assign(&mut self, mut rhs: Self) {
        self.visit2_mut(&mut rhs, |l, r| *l |= *r);
    }
}

impl BitXorAssign for Features {
    fn bitxor_assign(&mut self, mut rhs: Self) {
        self.visit2_mut(&mut rhs, |l, r| *l ^= *r);
    }
}

impl BitAnd for Features {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self {
        self &= rhs;
        self
    }
}

impl BitOr for Features {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self {
        self |= rhs;
        self
    }
}

impl BitXor for Features {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self {
        self ^= rhs;
        self
    }
}

impl Not for Features {
    type Output = Self;
    fn not(mut self) -> Self {
        self.visit_mut(|v| *v = !*v);
        self
    }
}

pub struct Queue {}

pub struct Device {
    #[allow(dead_code)]
    physical_device: SharedHandle<api::VkPhysicalDevice>,
    extensions: Extensions,
    #[allow(dead_code)]
    features: Features,
    queues: Vec<Vec<OwnedHandle<api::VkQueue>>>,
}

impl Device {
    unsafe fn new(
        physical_device: SharedHandle<api::VkPhysicalDevice>,
        create_info: *const api::VkDeviceCreateInfo,
    ) -> Result<OwnedHandle<api::VkDevice>, api::VkResult> {
        parse_next_chain_const! {
            create_info,
            root = api::VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            device_group_device_create_info: api::VkDeviceGroupDeviceCreateInfo = api::VK_STRUCTURE_TYPE_DEVICE_GROUP_DEVICE_CREATE_INFO,
            physical_device_16bit_storage_features: api::VkPhysicalDevice16BitStorageFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES,
            physical_device_features_2: api::VkPhysicalDeviceFeatures2 = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2,
            physical_device_multiview_features: api::VkPhysicalDeviceMultiviewFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_FEATURES,
            physical_device_protected_memory_features: api::VkPhysicalDeviceProtectedMemoryFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES,
            physical_device_sampler_ycbcr_conversion_features: api::VkPhysicalDeviceSamplerYcbcrConversionFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES,
            physical_device_shader_draw_parameter_features: api::VkPhysicalDeviceShaderDrawParameterFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES,
            physical_device_variable_pointer_features: api::VkPhysicalDeviceVariablePointerFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES,
        }
        let create_info = &*create_info;
        if create_info.enabledLayerCount != 0 {
            return Err(api::VK_ERROR_LAYER_NOT_PRESENT);
        }
        let mut enabled_extensions = physical_device.enabled_extensions;
        for &extension_name in util::to_slice(
            create_info.ppEnabledExtensionNames,
            create_info.enabledExtensionCount as usize,
        ) {
            let extension: Extension = CStr::from_ptr(extension_name)
                .to_str()
                .map_err(|_| api::VK_ERROR_EXTENSION_NOT_PRESENT)?
                .parse()
                .map_err(|_| api::VK_ERROR_EXTENSION_NOT_PRESENT)?;
            assert_eq!(extension.get_scope(), ExtensionScope::Device);
            enabled_extensions[extension] = true;
        }
        for extension in enabled_extensions
            .iter()
            .filter_map(|(extension, &enabled)| if enabled { Some(extension) } else { None })
        {
            let missing_extensions = extension.get_required_extensions() & !enabled_extensions;
            for missing_extension in missing_extensions
                .iter()
                .filter_map(|(extension, &enabled)| if enabled { Some(extension) } else { None })
            {
                panic!(
                    "extension {} enabled but required extension {} is not enabled",
                    extension.get_name(),
                    missing_extension.get_name()
                );
            }
        }
        let mut selected_features = Features::splat(false);
        if !device_group_device_create_info.is_null() {
            let api::VkDeviceGroupDeviceCreateInfo {
                sType: _,
                pNext: _,
                physicalDeviceCount: physical_device_count,
                pPhysicalDevices: physical_devices,
            } = *device_group_device_create_info;
            assert_eq!(
                physical_device_count, 1,
                "multiple devices in a group are not implemented"
            );
            assert_eq!(
                *physical_devices,
                physical_device.get_handle(),
                "unknown physical_device"
            );
        }
        if !physical_device_16bit_storage_features.is_null() {
            selected_features.import_feature_set(&*physical_device_16bit_storage_features);
        }
        if !physical_device_features_2.is_null() {
            selected_features.import_feature_set(&*physical_device_features_2);
        } else if !create_info.pEnabledFeatures.is_null() {
            selected_features.import_feature_set(&*create_info.pEnabledFeatures);
        }
        if !physical_device_multiview_features.is_null() {
            selected_features.import_feature_set(&*physical_device_multiview_features);
        }
        if !physical_device_protected_memory_features.is_null() {
            selected_features.import_feature_set(&*physical_device_protected_memory_features);
        }
        if !physical_device_sampler_ycbcr_conversion_features.is_null() {
            selected_features
                .import_feature_set(&*physical_device_sampler_ycbcr_conversion_features);
        }
        if !physical_device_shader_draw_parameter_features.is_null() {
            selected_features.import_feature_set(&*physical_device_shader_draw_parameter_features);
        } else if enabled_extensions[Extension::VK_KHR_shader_draw_parameters] {
            selected_features
                .shader_draw_parameter_features
                .shaderDrawParameters = api::VK_TRUE;
        }
        if !physical_device_variable_pointer_features.is_null() {
            selected_features.import_feature_set(&*physical_device_variable_pointer_features);
        }
        if (selected_features & !physical_device.features) != Features::splat(false) {
            return Err(api::VK_ERROR_FEATURE_NOT_PRESENT);
        }
        assert_ne!(create_info.queueCreateInfoCount, 0);
        let queue_create_infos = util::to_slice(
            create_info.pQueueCreateInfos,
            create_info.queueCreateInfoCount as usize,
        );
        assert!(queue_create_infos.len() <= QUEUE_FAMILY_COUNT as usize);
        let mut total_queue_count = 0;
        let mut queue_counts: Vec<_> = Vec::new();
        for queue_create_info in queue_create_infos {
            parse_next_chain_const! {
                queue_create_info,
                root = api::VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            }
            let api::VkDeviceQueueCreateInfo {
                sType: _,
                pNext: _,
                flags,
                queueFamilyIndex: queue_family_index,
                queueCount: queue_count,
                pQueuePriorities: queue_priorities,
            } = *queue_create_info;
            assert_eq!(flags & api::VK_DEVICE_QUEUE_CREATE_PROTECTED_BIT, 0);
            assert!(queue_family_index < QUEUE_FAMILY_COUNT);
            assert!(queue_count <= QUEUE_COUNTS[queue_family_index as usize]);
            let queue_priorities = util::to_slice(queue_priorities, queue_count as usize);
            for &queue_priority in queue_priorities {
                assert!(queue_priority >= 0.0 && queue_priority <= 1.0);
            }
            assert_eq!(QUEUE_FAMILY_COUNT, 1, "multiple queues are not implemented");
            assert_eq!(
                QUEUE_COUNTS, [1; QUEUE_FAMILY_COUNT as usize],
                "multiple queues are not implemented"
            );
            queue_counts.push(queue_count as usize);
            total_queue_count += queue_count as usize;
        }
        assert!(total_queue_count <= TOTAL_QUEUE_COUNT);
        let mut queues = Vec::new();
        for queue_count in queue_counts {
            let mut queue_family_queues = Vec::new();
            for _queue_index in 0..queue_count {
                queue_family_queues.push(OwnedHandle::<api::VkQueue>::new(Queue {}));
            }
            queues.push(queue_family_queues);
        }
        Ok(OwnedHandle::<api::VkDevice>::new(Device {
            physical_device,
            extensions: enabled_extensions,
            features: selected_features,
            queues,
        }))
    }
}

pub struct PhysicalDevice {
    enabled_extensions: Extensions,
    allowed_extensions: Extensions,
    properties: api::VkPhysicalDeviceProperties,
    features: Features,
    system_memory_size: u64,
    point_clipping_properties: api::VkPhysicalDevicePointClippingProperties,
    multiview_properties: api::VkPhysicalDeviceMultiviewProperties,
    id_properties: api::VkPhysicalDeviceIDProperties,
    maintenance_3_properties: api::VkPhysicalDeviceMaintenance3Properties,
    protected_memory_properties: api::VkPhysicalDeviceProtectedMemoryProperties,
    subgroup_properties: api::VkPhysicalDeviceSubgroupProperties,
}

impl PhysicalDevice {
    pub fn get_pipeline_cache_uuid() -> uuid::Uuid {
        // FIXME: return real uuid
        uuid::Uuid::nil()
    }
    pub fn get_device_uuid() -> uuid::Uuid {
        // FIXME: return real uuid
        uuid::Uuid::nil()
    }
    pub fn get_driver_uuid() -> uuid::Uuid {
        // FIXME: return real uuid
        uuid::Uuid::nil()
    }
    pub fn get_limits() -> api::VkPhysicalDeviceLimits {
        #![allow(clippy::needless_update)]
        api::VkPhysicalDeviceLimits {
            maxImageDimension1D: !0,
            maxImageDimension2D: !0,
            maxImageDimension3D: !0,
            maxImageDimensionCube: !0,
            maxImageArrayLayers: !0,
            maxTexelBufferElements: !0,
            maxUniformBufferRange: !0,
            maxStorageBufferRange: !0,
            maxPushConstantsSize: !0,
            maxMemoryAllocationCount: !0,
            maxSamplerAllocationCount: !0,
            bufferImageGranularity: 1,
            sparseAddressSpaceSize: 0,
            maxBoundDescriptorSets: !0,
            maxPerStageDescriptorSamplers: !0,
            maxPerStageDescriptorUniformBuffers: !0,
            maxPerStageDescriptorStorageBuffers: !0,
            maxPerStageDescriptorSampledImages: !0,
            maxPerStageDescriptorStorageImages: !0,
            maxPerStageDescriptorInputAttachments: !0,
            maxPerStageResources: !0,
            maxDescriptorSetSamplers: !0,
            maxDescriptorSetUniformBuffers: !0,
            maxDescriptorSetUniformBuffersDynamic: !0,
            maxDescriptorSetStorageBuffers: !0,
            maxDescriptorSetStorageBuffersDynamic: !0,
            maxDescriptorSetSampledImages: !0,
            maxDescriptorSetStorageImages: !0,
            maxDescriptorSetInputAttachments: !0,
            maxVertexInputAttributes: !0,
            maxVertexInputBindings: !0,
            maxVertexInputAttributeOffset: !0,
            maxVertexInputBindingStride: !0,
            maxVertexOutputComponents: !0,
            maxTessellationGenerationLevel: 0,
            maxTessellationPatchSize: 0,
            maxTessellationControlPerVertexInputComponents: 0,
            maxTessellationControlPerVertexOutputComponents: 0,
            maxTessellationControlPerPatchOutputComponents: 0,
            maxTessellationControlTotalOutputComponents: 0,
            maxTessellationEvaluationInputComponents: 0,
            maxTessellationEvaluationOutputComponents: 0,
            maxGeometryShaderInvocations: 0,
            maxGeometryInputComponents: 0,
            maxGeometryOutputComponents: 0,
            maxGeometryOutputVertices: 0,
            maxGeometryTotalOutputComponents: 0,
            maxFragmentInputComponents: !0,
            maxFragmentOutputAttachments: !0,
            maxFragmentDualSrcAttachments: 0,
            maxFragmentCombinedOutputResources: !0,
            maxComputeSharedMemorySize: !0,
            maxComputeWorkGroupCount: [!0; 3],
            maxComputeWorkGroupInvocations: !0,
            maxComputeWorkGroupSize: [!0; 3],
            subPixelPrecisionBits: 4, // FIXME: update to correct value
            subTexelPrecisionBits: 4, // FIXME: update to correct value
            mipmapPrecisionBits: 4,   // FIXME: update to correct value
            maxDrawIndexedIndexValue: !0,
            maxDrawIndirectCount: !0,
            maxSamplerLodBias: 2.0, // FIXME: update to correct value
            maxSamplerAnisotropy: 1.0,
            maxViewports: 1,
            maxViewportDimensions: [4096; 2], // FIXME: update to correct value
            viewportBoundsRange: [-8192.0, 8191.0], // FIXME: update to correct value
            viewportSubPixelBits: 0,
            minMemoryMapAlignment: MIN_MEMORY_MAP_ALIGNMENT,
            minTexelBufferOffsetAlignment: 64, // FIXME: update to correct value
            minUniformBufferOffsetAlignment: 64, // FIXME: update to correct value
            minStorageBufferOffsetAlignment: 64, // FIXME: update to correct value
            minTexelOffset: -8,                // FIXME: update to correct value
            maxTexelOffset: 7,                 // FIXME: update to correct value
            minTexelGatherOffset: 0,
            maxTexelGatherOffset: 0,
            minInterpolationOffset: 0.0,
            maxInterpolationOffset: 0.0,
            subPixelInterpolationOffsetBits: 0,
            maxFramebufferWidth: 4096,  // FIXME: update to correct value
            maxFramebufferHeight: 4096, // FIXME: update to correct value
            maxFramebufferLayers: 256,  // FIXME: update to correct value
            framebufferColorSampleCounts: api::VK_SAMPLE_COUNT_1_BIT | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
            framebufferDepthSampleCounts: api::VK_SAMPLE_COUNT_1_BIT | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
            framebufferStencilSampleCounts: api::VK_SAMPLE_COUNT_1_BIT | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
            framebufferNoAttachmentsSampleCounts: api::VK_SAMPLE_COUNT_1_BIT
                | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
            maxColorAttachments: 4,
            sampledImageColorSampleCounts: api::VK_SAMPLE_COUNT_1_BIT | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
            sampledImageIntegerSampleCounts: api::VK_SAMPLE_COUNT_1_BIT
                | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
            sampledImageDepthSampleCounts: api::VK_SAMPLE_COUNT_1_BIT | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
            sampledImageStencilSampleCounts: api::VK_SAMPLE_COUNT_1_BIT
                | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
            storageImageSampleCounts: api::VK_SAMPLE_COUNT_1_BIT, // FIXME: update to correct value
            maxSampleMaskWords: 1,
            timestampComputeAndGraphics: api::VK_FALSE,
            timestampPeriod: 0.0,
            maxClipDistances: 0,
            maxCullDistances: 0,
            maxCombinedClipAndCullDistances: 0,
            discreteQueuePriorities: 2,
            pointSizeRange: [1.0; 2],
            lineWidthRange: [1.0; 2],
            pointSizeGranularity: 0.0,
            lineWidthGranularity: 0.0,
            strictLines: api::VK_FALSE,
            standardSampleLocations: api::VK_TRUE,
            optimalBufferCopyOffsetAlignment: 16,
            optimalBufferCopyRowPitchAlignment: 16,
            nonCoherentAtomSize: 1,     //TODO: check if this is correct
            ..unsafe { mem::zeroed() }  // for padding fields
        }
    }
    pub fn get_format_properties(format: api::VkFormat) -> api::VkFormatProperties {
        match format {
            api::VK_FORMAT_UNDEFINED => api::VkFormatProperties {
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R4G4_UNORM_PACK8 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R4G4B4A4_UNORM_PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B4G4R4A4_UNORM_PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R5G6B5_UNORM_PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B5G6R5_UNORM_PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R5G5B5A1_UNORM_PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B5G5R5A1_UNORM_PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A1R5G5B5_UNORM_PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8_SNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8_USCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8_SSCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8_SRGB => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8_SNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8_USCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8_SSCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8_SRGB => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8_SNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8_USCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8_SSCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8_SRGB => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8_SNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8_USCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8_SSCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8_SRGB => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8A8_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8A8_SNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8A8_USCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8A8_SSCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8A8_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8A8_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R8G8B8A8_SRGB => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8A8_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8A8_SNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8A8_USCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8A8_SSCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8A8_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8A8_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8A8_SRGB => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A8B8G8R8_UNORM_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A8B8G8R8_SNORM_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A8B8G8R8_USCALED_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A8B8G8R8_SSCALED_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A8B8G8R8_UINT_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A8B8G8R8_SINT_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A8B8G8R8_SRGB_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A2R10G10B10_UNORM_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A2R10G10B10_SNORM_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A2R10G10B10_USCALED_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A2R10G10B10_SSCALED_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A2R10G10B10_UINT_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A2R10G10B10_SINT_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A2B10G10R10_UNORM_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A2B10G10R10_SNORM_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A2B10G10R10_USCALED_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A2B10G10R10_SSCALED_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A2B10G10R10_UINT_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_A2B10G10R10_SINT_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16_SNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16_USCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16_SSCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16_SNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16_USCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16_SSCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16_SNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16_USCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16_SSCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16A16_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16A16_SNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16A16_USCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16A16_SSCALED => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16A16_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16A16_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R16G16B16A16_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R32_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R32_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R32_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R32G32_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R32G32_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R32G32_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R32G32B32_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R32G32B32_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R32G32B32_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R32G32B32A32_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R32G32B32A32_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R32G32B32A32_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R64_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R64_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R64_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R64G64_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R64G64_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R64G64_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R64G64B64_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R64G64B64_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R64G64B64_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R64G64B64A64_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R64G64B64A64_SINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R64G64B64A64_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B10G11R11_UFLOAT_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_E5B9G9R9_UFLOAT_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_D16_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_X8_D24_UNORM_PACK32 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_D32_SFLOAT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_S8_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_D16_UNORM_S8_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_D24_UNORM_S8_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_D32_SFLOAT_S8_UINT => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC1_RGB_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC1_RGB_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC1_RGBA_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC1_RGBA_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC2_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC2_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC3_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC3_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC4_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC4_SNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC5_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC5_SNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC6H_UFLOAT_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC6H_SFLOAT_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC7_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_BC7_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ETC2_R8G8B8_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ETC2_R8G8B8_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ETC2_R8G8B8A1_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ETC2_R8G8B8A1_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ETC2_R8G8B8A8_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ETC2_R8G8B8A8_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_EAC_R11_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_EAC_R11_SNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_EAC_R11G11_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_EAC_R11G11_SNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_4x4_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_4x4_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_5x4_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_5x4_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_5x5_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_5x5_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_6x5_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_6x5_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_6x6_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_6x6_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_8x5_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_8x5_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_8x6_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_8x6_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_8x8_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_8x8_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_10x5_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_10x5_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_10x6_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_10x6_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_10x8_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_10x8_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_10x10_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_10x10_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_12x10_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_12x10_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_12x12_UNORM_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_ASTC_12x12_SRGB_BLOCK => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G8B8G8R8_422_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B8G8R8G8_422_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G8_B8_R8_3PLANE_420_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G8_B8R8_2PLANE_420_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G8_B8_R8_3PLANE_422_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G8_B8R8_2PLANE_422_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G8_B8_R8_3PLANE_444_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R10X6_UNORM_PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R10X6G10X6_UNORM_2PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R10X6G10X6B10X6A10X6_UNORM_4PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G10X6B10X6G10X6R10X6_422_UNORM_4PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B10X6G10X6R10X6G10X6_422_UNORM_4PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R12X4_UNORM_PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R12X4G12X4_UNORM_2PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_R12X4G12X4B12X4A12X4_UNORM_4PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G12X4B12X4G12X4R12X4_422_UNORM_4PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B12X4G12X4R12X4G12X4_422_UNORM_4PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16 => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G16B16G16R16_422_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_B16G16R16G16_422_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G16_B16_R16_3PLANE_420_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G16_B16R16_2PLANE_420_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G16_B16_R16_3PLANE_422_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G16_B16R16_2PLANE_422_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            api::VK_FORMAT_G16_B16_R16_3PLANE_444_UNORM => api::VkFormatProperties {
                // FIXME: finish
                linearTilingFeatures: 0,
                optimalTilingFeatures: 0,
                bufferFeatures: 0,
            },
            _ => panic!("unknown format {}", format),
        }
    }
}

pub struct Instance {
    physical_device: OwnedHandle<api::VkPhysicalDevice>,
}

impl Instance {
    pub unsafe fn new(
        create_info: *const api::VkInstanceCreateInfo,
    ) -> Result<api::VkInstance, api::VkResult> {
        parse_next_chain_const! {
            create_info,
            root = api::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        }
        let create_info = &*create_info;
        if create_info.enabledLayerCount != 0 {
            return Err(api::VK_ERROR_LAYER_NOT_PRESENT);
        }
        let mut enabled_extensions = Extensions::create_empty();
        for &extension_name in util::to_slice(
            create_info.ppEnabledExtensionNames,
            create_info.enabledExtensionCount as usize,
        ) {
            let extension: Extension = CStr::from_ptr(extension_name)
                .to_str()
                .map_err(|_| api::VK_ERROR_EXTENSION_NOT_PRESENT)?
                .parse()
                .map_err(|_| api::VK_ERROR_EXTENSION_NOT_PRESENT)?;
            assert_eq!(extension.get_scope(), ExtensionScope::Instance);
            enabled_extensions[extension] = true;
        }
        for extension in enabled_extensions
            .iter()
            .filter_map(|(extension, &enabled)| if enabled { Some(extension) } else { None })
        {
            let missing_extensions = extension.get_required_extensions() & !enabled_extensions;
            for missing_extension in missing_extensions
                .iter()
                .filter_map(|(extension, &enabled)| if enabled { Some(extension) } else { None })
            {
                panic!(
                    "extension {} enabled but required extension {} is not enabled",
                    extension.get_name(),
                    missing_extension.get_name()
                );
            }
        }
        let system_memory_size;
        match sys_info::mem_info() {
            Err(error) => {
                eprintln!("mem_info error: {}", error);
                return Err(api::VK_ERROR_INITIALIZATION_FAILED);
            }
            Ok(info) => system_memory_size = info.total * 1024,
        }
        let mut device_name = [0; api::VK_MAX_PHYSICAL_DEVICE_NAME_SIZE as usize];
        util::copy_str_to_char_array(&mut device_name, KAZAN_DEVICE_NAME);
        #[allow(clippy::needless_update)]
        let retval = OwnedHandle::<api::VkInstance>::new(Instance {
            physical_device: OwnedHandle::new(PhysicalDevice {
                enabled_extensions,
                allowed_extensions: enabled_extensions.get_allowed_extensions_from_instance_scope(),
                properties: api::VkPhysicalDeviceProperties {
                    apiVersion: make_api_version(1, 1, api::VK_HEADER_VERSION),
                    driverVersion: make_api_version(
                        env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
                        env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
                        env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
                    ),
                    vendorID: api::VK_VENDOR_ID_KAZAN,
                    deviceID: 1,
                    deviceType: api::VK_PHYSICAL_DEVICE_TYPE_CPU,
                    deviceName: device_name,
                    pipelineCacheUUID: *PhysicalDevice::get_pipeline_cache_uuid().as_bytes(),
                    limits: PhysicalDevice::get_limits(),
                    sparseProperties: api::VkPhysicalDeviceSparseProperties {
                        residencyStandard2DBlockShape: api::VK_FALSE,
                        residencyStandard2DMultisampleBlockShape: api::VK_FALSE,
                        residencyStandard3DBlockShape: api::VK_FALSE,
                        residencyAlignedMipSize: api::VK_FALSE,
                        residencyNonResidentStrict: api::VK_FALSE,
                    },
                    ..mem::zeroed() // for padding fields
                },
                features: Features::new(),
                system_memory_size,
                point_clipping_properties: api::VkPhysicalDevicePointClippingProperties {
                    sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES,
                    pNext: null_mut(),
                    pointClippingBehavior: api::VK_POINT_CLIPPING_BEHAVIOR_ALL_CLIP_PLANES,
                },
                multiview_properties: api::VkPhysicalDeviceMultiviewProperties {
                    sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES,
                    pNext: null_mut(),
                    maxMultiviewViewCount: 6,
                    maxMultiviewInstanceIndex: !0,
                },
                id_properties: api::VkPhysicalDeviceIDProperties {
                    sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ID_PROPERTIES,
                    pNext: null_mut(),
                    deviceUUID: *PhysicalDevice::get_device_uuid().as_bytes(),
                    driverUUID: *PhysicalDevice::get_driver_uuid().as_bytes(),
                    deviceLUID: [0; api::VK_LUID_SIZE as usize],
                    deviceNodeMask: 1,
                    deviceLUIDValid: api::VK_FALSE,
                },
                maintenance_3_properties: api::VkPhysicalDeviceMaintenance3Properties {
                    sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES,
                    pNext: null_mut(),
                    maxPerSetDescriptors: !0,
                    maxMemoryAllocationSize: isize::max_value() as u64,
                    ..mem::zeroed() // for padding fields
                },
                protected_memory_properties: api::VkPhysicalDeviceProtectedMemoryProperties {
                    sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROTECTED_MEMORY_PROPERTIES,
                    pNext: null_mut(),
                    protectedNoFault: api::VK_FALSE,
                },
                subgroup_properties: api::VkPhysicalDeviceSubgroupProperties {
                    sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SUBGROUP_PROPERTIES,
                    pNext: null_mut(),
                    subgroupSize: 1, // FIXME fill in correct value
                    supportedStages: api::VK_SHADER_STAGE_COMPUTE_BIT,
                    supportedOperations: api::VK_SUBGROUP_FEATURE_BASIC_BIT,
                    quadOperationsInAllStages: api::VK_FALSE,
                },
            }),
        });
        Ok(retval.take())
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetInstanceProcAddr(
    instance: api::VkInstance,
    name: *const c_char,
) -> api::PFN_vkVoidFunction {
    match instance.get() {
        Some(_) => get_proc_address(
            name,
            GetProcAddressScope::Instance,
            &SharedHandle::from(instance)
                .unwrap()
                .physical_device
                .allowed_extensions,
        ),
        None => get_proc_address(
            name,
            GetProcAddressScope::Global,
            &Extensions::create_empty(),
        ),
    }
}

pub fn make_api_version(major: u32, minor: u32, patch: u32) -> u32 {
    assert!(major < (1 << 10));
    assert!(minor < (1 << 10));
    assert!(patch < (1 << 12));
    (major << 22) | (minor << 12) | patch
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkEnumerateInstanceVersion(api_version: *mut u32) -> api::VkResult {
    *api_version = make_api_version(1, 1, api::VK_HEADER_VERSION);
    api::VK_SUCCESS
}

pub unsafe fn enumerate_helper<T, Item, I: IntoIterator<Item = Item>, AF: FnMut(&mut T, Item)>(
    api_value_count: *mut u32,
    api_values: *mut T,
    values: I,
    mut assign_function: AF,
) -> api::VkResult {
    let mut retval = api::VK_SUCCESS;
    let mut api_values = if api_values.is_null() {
        None
    } else {
        Some(util::to_slice_mut(api_values, *api_value_count as usize))
    };
    let mut final_count = 0;
    for value in values {
        if let Some(api_values) = &mut api_values {
            if final_count >= api_values.len() {
                retval = api::VK_INCOMPLETE;
                break;
            } else {
                assign_function(&mut api_values[final_count], value);
                final_count += 1;
            }
        } else {
            final_count += 1;
        }
    }
    assert_eq!(final_count as u32 as usize, final_count);
    *api_value_count = final_count as u32;
    retval
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkEnumerateInstanceLayerProperties(
    property_count: *mut u32,
    properties: *mut api::VkLayerProperties,
) -> api::VkResult {
    enumerate_helper(property_count, properties, &[], |l, r| *l = *r)
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkEnumerateInstanceExtensionProperties(
    layer_name: *const c_char,
    property_count: *mut u32,
    properties: *mut api::VkExtensionProperties,
) -> api::VkResult {
    enumerate_extension_properties(
        layer_name,
        property_count,
        properties,
        ExtensionScope::Instance,
    )
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateInstance(
    create_info: *const api::VkInstanceCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    instance: *mut api::VkInstance,
) -> api::VkResult {
    *instance = Handle::null();
    match Instance::new(create_info) {
        Ok(v) => {
            *instance = v;
            api::VK_SUCCESS
        }
        Err(error) => error,
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyInstance(
    instance: api::VkInstance,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(instance);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkEnumeratePhysicalDevices(
    instance: api::VkInstance,
    physical_device_count: *mut u32,
    physical_devices: *mut api::VkPhysicalDevice,
) -> api::VkResult {
    let instance = SharedHandle::from(instance).unwrap();
    enumerate_helper(
        physical_device_count,
        physical_devices,
        iter::once(instance.physical_device.get_handle()),
        |l, r| *l = r,
    )
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceFeatures(
    physical_device: api::VkPhysicalDevice,
    features: *mut api::VkPhysicalDeviceFeatures,
) {
    let mut v = api::VkPhysicalDeviceFeatures2 {
        sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2,
        pNext: null_mut(),
        features: mem::zeroed(),
    };
    vkGetPhysicalDeviceFeatures2(physical_device, &mut v);
    *features = v.features;
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceFormatProperties(
    physical_device: api::VkPhysicalDevice,
    format: api::VkFormat,
    format_properties: *mut api::VkFormatProperties,
) {
    let mut format_properties2 = api::VkFormatProperties2 {
        sType: api::VK_STRUCTURE_TYPE_FORMAT_PROPERTIES_2,
        pNext: null_mut(),
        formatProperties: mem::zeroed(),
    };
    vkGetPhysicalDeviceFormatProperties2(physical_device, format, &mut format_properties2);
    *format_properties = format_properties2.formatProperties;
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceImageFormatProperties(
    _physicalDevice: api::VkPhysicalDevice,
    _format: api::VkFormat,
    _type_: api::VkImageType,
    _tiling: api::VkImageTiling,
    _usage: api::VkImageUsageFlags,
    _flags: api::VkImageCreateFlags,
    _pImageFormatProperties: *mut api::VkImageFormatProperties,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceProperties(
    physical_device: api::VkPhysicalDevice,
    properties: *mut api::VkPhysicalDeviceProperties,
) {
    let physical_device = SharedHandle::from(physical_device).unwrap();
    *properties = physical_device.properties;
}

unsafe fn get_physical_device_queue_family_properties(
    _physical_device: SharedHandle<api::VkPhysicalDevice>,
    queue_family_properties: &mut api::VkQueueFamilyProperties2,
    queue_count: u32,
) {
    parse_next_chain_mut! {
        queue_family_properties,
        root = api::VK_STRUCTURE_TYPE_QUEUE_FAMILY_PROPERTIES_2,
    }
    queue_family_properties.queueFamilyProperties = api::VkQueueFamilyProperties {
        queueFlags: api::VK_QUEUE_GRAPHICS_BIT
            | api::VK_QUEUE_COMPUTE_BIT
            | api::VK_QUEUE_TRANSFER_BIT,
        queueCount: queue_count,
        timestampValidBits: 0,
        minImageTransferGranularity: api::VkExtent3D {
            width: 1,
            height: 1,
            depth: 1,
        },
    };
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceQueueFamilyProperties(
    physical_device: api::VkPhysicalDevice,
    queue_family_property_count: *mut u32,
    queue_family_properties: *mut api::VkQueueFamilyProperties,
) {
    enumerate_helper(
        queue_family_property_count,
        queue_family_properties,
        QUEUE_COUNTS.iter(),
        |queue_family_properties, &count| {
            let mut queue_family_properties2 = api::VkQueueFamilyProperties2 {
                sType: api::VK_STRUCTURE_TYPE_QUEUE_FAMILY_PROPERTIES_2,
                pNext: null_mut(),
                queueFamilyProperties: mem::zeroed(),
            };
            get_physical_device_queue_family_properties(
                SharedHandle::from(physical_device).unwrap(),
                &mut queue_family_properties2,
                count,
            );
            *queue_family_properties = queue_family_properties2.queueFamilyProperties;
        },
    );
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceMemoryProperties(
    physical_device: api::VkPhysicalDevice,
    memory_properties: *mut api::VkPhysicalDeviceMemoryProperties,
) {
    let mut memory_properties2 = api::VkPhysicalDeviceMemoryProperties2 {
        sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MEMORY_PROPERTIES_2,
        pNext: null_mut(),
        memoryProperties: mem::zeroed(),
    };
    vkGetPhysicalDeviceMemoryProperties2(physical_device, &mut memory_properties2);
    *memory_properties = memory_properties2.memoryProperties;
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetDeviceProcAddr(
    device: api::VkDevice,
    name: *const c_char,
) -> api::PFN_vkVoidFunction {
    get_proc_address(
        name,
        GetProcAddressScope::Device,
        &SharedHandle::from(device).unwrap().extensions,
    )
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateDevice(
    physical_device: api::VkPhysicalDevice,
    create_info: *const api::VkDeviceCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    device: *mut api::VkDevice,
) -> api::VkResult {
    *device = Handle::null();
    match Device::new(SharedHandle::from(physical_device).unwrap(), create_info) {
        Ok(v) => {
            *device = v.take();
            api::VK_SUCCESS
        }
        Err(error) => error,
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyDevice(
    device: api::VkDevice,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(device);
}

unsafe fn enumerate_extension_properties(
    layer_name: *const c_char,
    property_count: *mut u32,
    properties: *mut api::VkExtensionProperties,
    extension_scope: ExtensionScope,
) -> api::VkResult {
    if !layer_name.is_null() {
        return api::VK_ERROR_LAYER_NOT_PRESENT;
    }
    enumerate_helper(
        property_count,
        properties,
        Extensions::default().iter().filter_map(
            |(extension, _): (Extension, _)| -> Option<api::VkExtensionProperties> {
                if extension.get_scope() == extension_scope {
                    Some(extension.get_properties())
                } else {
                    None
                }
            },
        ),
        |l, r| *l = r,
    )
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkEnumerateDeviceExtensionProperties(
    _physical_device: api::VkPhysicalDevice,
    layer_name: *const c_char,
    property_count: *mut u32,
    properties: *mut api::VkExtensionProperties,
) -> api::VkResult {
    enumerate_extension_properties(
        layer_name,
        property_count,
        properties,
        ExtensionScope::Device,
    )
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkEnumerateDeviceLayerProperties(
    _physicalDevice: api::VkPhysicalDevice,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkLayerProperties,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetDeviceQueue(
    device: api::VkDevice,
    queue_family_index: u32,
    queue_index: u32,
    queue: *mut api::VkQueue,
) {
    vkGetDeviceQueue2(
        device,
        &api::VkDeviceQueueInfo2 {
            sType: api::VK_STRUCTURE_TYPE_DEVICE_QUEUE_INFO_2,
            pNext: null(),
            flags: 0,
            queueFamilyIndex: queue_family_index,
            queueIndex: queue_index,
        },
        queue,
    );
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkQueueSubmit(
    _queue: api::VkQueue,
    _submitCount: u32,
    _pSubmits: *const api::VkSubmitInfo,
    _fence: api::VkFence,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkQueueWaitIdle(_queue: api::VkQueue) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDeviceWaitIdle(_device: api::VkDevice) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkAllocateMemory(
    _device: api::VkDevice,
    allocate_info: *const api::VkMemoryAllocateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    memory: *mut api::VkDeviceMemory,
) -> api::VkResult {
    parse_next_chain_const! {
        allocate_info,
        root = api::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
        export_memory_allocate_info: api::VkExportMemoryAllocateInfo = api::VK_STRUCTURE_TYPE_EXPORT_MEMORY_ALLOCATE_INFO,
        memory_allocate_flags_info: api::VkMemoryAllocateFlagsInfo = api::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_FLAGS_INFO,
        memory_dedicated_allocate_info: api::VkMemoryDedicatedAllocateInfo = api::VK_STRUCTURE_TYPE_MEMORY_DEDICATED_ALLOCATE_INFO,
    }
    let allocate_info = &*allocate_info;
    if !export_memory_allocate_info.is_null() {
        unimplemented!()
    }
    if !memory_allocate_flags_info.is_null() {
        unimplemented!()
    }
    if !memory_dedicated_allocate_info.is_null() {
        unimplemented!()
    }
    match DeviceMemoryType::from_index(allocate_info.memoryTypeIndex).unwrap() {
        DeviceMemoryType::Main => {
            if allocate_info.allocationSize > isize::max_value() as u64 {
                return api::VK_ERROR_OUT_OF_DEVICE_MEMORY;
            }
            match DeviceMemory::allocate_from_default_heap(DeviceMemoryLayout::calculate(
                allocate_info.allocationSize as usize,
                MIN_MEMORY_MAP_ALIGNMENT,
            )) {
                Ok(new_memory) => {
                    *memory = OwnedHandle::<api::VkDeviceMemory>::new(new_memory).take();
                    api::VK_SUCCESS
                }
                Err(_) => api::VK_ERROR_OUT_OF_DEVICE_MEMORY,
            }
        }
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkFreeMemory(
    _device: api::VkDevice,
    memory: api::VkDeviceMemory,
    _allocator: *const api::VkAllocationCallbacks,
) {
    if !memory.is_null() {
        OwnedHandle::from(memory);
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkMapMemory(
    _device: api::VkDevice,
    memory: api::VkDeviceMemory,
    offset: api::VkDeviceSize,
    _size: api::VkDeviceSize,
    _flags: api::VkMemoryMapFlags,
    data: *mut *mut c_void,
) -> api::VkResult {
    let memory = SharedHandle::from(memory).unwrap();
    // remember to keep vkUnmapMemory up to date
    *data = memory.get().as_ptr().offset(offset as isize) as *mut c_void;
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkUnmapMemory(_device: api::VkDevice, _memory: api::VkDeviceMemory) {}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkFlushMappedMemoryRanges(
    _device: api::VkDevice,
    _memoryRangeCount: u32,
    _pMemoryRanges: *const api::VkMappedMemoryRange,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkInvalidateMappedMemoryRanges(
    _device: api::VkDevice,
    _memoryRangeCount: u32,
    _pMemoryRanges: *const api::VkMappedMemoryRange,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetDeviceMemoryCommitment(
    _device: api::VkDevice,
    _memory: api::VkDeviceMemory,
    _pCommittedMemoryInBytes: *mut api::VkDeviceSize,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkBindBufferMemory(
    device: api::VkDevice,
    buffer: api::VkBuffer,
    memory: api::VkDeviceMemory,
    memory_offset: api::VkDeviceSize,
) -> api::VkResult {
    vkBindBufferMemory2(
        device,
        1,
        &api::VkBindBufferMemoryInfo {
            sType: api::VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_INFO,
            pNext: null(),
            buffer,
            memory,
            memoryOffset: memory_offset,
        },
    )
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkBindImageMemory(
    device: api::VkDevice,
    image: api::VkImage,
    memory: api::VkDeviceMemory,
    memory_offset: api::VkDeviceSize,
) -> api::VkResult {
    vkBindImageMemory2(
        device,
        1,
        &api::VkBindImageMemoryInfo {
            sType: api::VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_INFO,
            pNext: null(),
            image,
            memory,
            memoryOffset: memory_offset,
        },
    )
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetBufferMemoryRequirements(
    device: api::VkDevice,
    buffer: api::VkBuffer,
    memory_requirements: *mut api::VkMemoryRequirements,
) {
    let mut memory_requirements_2 = api::VkMemoryRequirements2 {
        sType: api::VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2,
        pNext: null_mut(),
        memoryRequirements: mem::zeroed(),
    };
    vkGetBufferMemoryRequirements2(
        device,
        &api::VkBufferMemoryRequirementsInfo2 {
            sType: api::VK_STRUCTURE_TYPE_BUFFER_MEMORY_REQUIREMENTS_INFO_2,
            pNext: null(),
            buffer,
        },
        &mut memory_requirements_2,
    );
    *memory_requirements = memory_requirements_2.memoryRequirements;
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetImageMemoryRequirements(
    device: api::VkDevice,
    image: api::VkImage,
    memory_requirements: *mut api::VkMemoryRequirements,
) {
    let mut memory_requirements_2 = api::VkMemoryRequirements2 {
        sType: api::VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2,
        pNext: null_mut(),
        memoryRequirements: mem::zeroed(),
    };
    vkGetImageMemoryRequirements2(
        device,
        &api::VkImageMemoryRequirementsInfo2 {
            sType: api::VK_STRUCTURE_TYPE_IMAGE_MEMORY_REQUIREMENTS_INFO_2,
            pNext: null(),
            image,
        },
        &mut memory_requirements_2,
    );
    *memory_requirements = memory_requirements_2.memoryRequirements;
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetImageSparseMemoryRequirements(
    _device: api::VkDevice,
    _image: api::VkImage,
    _pSparseMemoryRequirementCount: *mut u32,
    _pSparseMemoryRequirements: *mut api::VkSparseImageMemoryRequirements,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSparseImageFormatProperties(
    _physicalDevice: api::VkPhysicalDevice,
    _format: api::VkFormat,
    _type_: api::VkImageType,
    _samples: api::VkSampleCountFlagBits,
    _usage: api::VkImageUsageFlags,
    _tiling: api::VkImageTiling,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkSparseImageFormatProperties,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkQueueBindSparse(
    _queue: api::VkQueue,
    _bindInfoCount: u32,
    _pBindInfo: *const api::VkBindSparseInfo,
    _fence: api::VkFence,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateFence(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkFenceCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pFence: *mut api::VkFence,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyFence(
    _device: api::VkDevice,
    _fence: api::VkFence,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkResetFences(
    _device: api::VkDevice,
    _fenceCount: u32,
    _pFences: *const api::VkFence,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetFenceStatus(
    _device: api::VkDevice,
    _fence: api::VkFence,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkWaitForFences(
    _device: api::VkDevice,
    _fenceCount: u32,
    _pFences: *const api::VkFence,
    _waitAll: api::VkBool32,
    _timeout: u64,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateSemaphore(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkSemaphoreCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pSemaphore: *mut api::VkSemaphore,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroySemaphore(
    _device: api::VkDevice,
    _semaphore: api::VkSemaphore,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateEvent(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkEventCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pEvent: *mut api::VkEvent,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyEvent(
    _device: api::VkDevice,
    _event: api::VkEvent,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetEventStatus(
    _device: api::VkDevice,
    _event: api::VkEvent,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkSetEvent(
    _device: api::VkDevice,
    _event: api::VkEvent,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkResetEvent(
    _device: api::VkDevice,
    _event: api::VkEvent,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateQueryPool(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkQueryPoolCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pQueryPool: *mut api::VkQueryPool,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyQueryPool(
    _device: api::VkDevice,
    _queryPool: api::VkQueryPool,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetQueryPoolResults(
    _device: api::VkDevice,
    _queryPool: api::VkQueryPool,
    _firstQuery: u32,
    _queryCount: u32,
    _dataSize: usize,
    _pData: *mut c_void,
    _stride: api::VkDeviceSize,
    _flags: api::VkQueryResultFlags,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateBuffer(
    _device: api::VkDevice,
    create_info: *const api::VkBufferCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    buffer: *mut api::VkBuffer,
) -> api::VkResult {
    parse_next_chain_const! {
        create_info,
        root = api::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
        external_memory_buffer: api::VkExternalMemoryBufferCreateInfo = api::VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_BUFFER_CREATE_INFO,
    }
    let create_info = &*create_info;
    if !external_memory_buffer.is_null() {
        let external_memory_buffer = &*external_memory_buffer;
        assert_eq!(external_memory_buffer.handleTypes, 0);
    }
    if create_info.size > isize::max_value() as u64 {
        return api::VK_ERROR_OUT_OF_DEVICE_MEMORY;
    }
    *buffer = OwnedHandle::<api::VkBuffer>::new(Buffer {
        size: create_info.size as usize,
        memory: None,
    })
    .take();
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyBuffer(
    _device: api::VkDevice,
    buffer: api::VkBuffer,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(buffer);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateBufferView(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkBufferViewCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pView: *mut api::VkBufferView,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyBufferView(
    _device: api::VkDevice,
    _bufferView: api::VkBufferView,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateImage(
    _device: api::VkDevice,
    create_info: *const api::VkImageCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    image: *mut api::VkImage,
) -> api::VkResult {
    parse_next_chain_const! {
        create_info,
        root = api::VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
        external_memory_image_create_info: api::VkExternalMemoryImageCreateInfo = api::VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_IMAGE_CREATE_INFO,
        image_swapchain_create_info: api::VkImageSwapchainCreateInfoKHR = api::VK_STRUCTURE_TYPE_IMAGE_SWAPCHAIN_CREATE_INFO_KHR,
    }
    let create_info = &*create_info;
    if !external_memory_image_create_info.is_null() {
        unimplemented!();
    }
    if !image_swapchain_create_info.is_null() {
        unimplemented!();
    }
    *image = OwnedHandle::<api::VkImage>::new(Image {
        properties: ImageProperties {
            supported_tilings: match create_info.tiling {
                api::VK_IMAGE_TILING_OPTIMAL => SupportedTilings::Any,
                api::VK_IMAGE_TILING_LINEAR => SupportedTilings::LinearOnly,
                _ => unreachable!("invalid image tiling"),
            },
            format: create_info.format,
            extents: create_info.extent,
            array_layers: create_info.arrayLayers,
            mip_levels: create_info.mipLevels,
            multisample_count: match create_info.samples {
                api::VK_SAMPLE_COUNT_1_BIT => ImageMultisampleCount::Count1,
                api::VK_SAMPLE_COUNT_4_BIT => ImageMultisampleCount::Count4,
                _ => unreachable!("invalid sample count"),
            },
            swapchain_present_tiling: None,
        },
        memory: None,
    })
    .take();
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyImage(
    _device: api::VkDevice,
    image: api::VkImage,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(image);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetImageSubresourceLayout(
    _device: api::VkDevice,
    _image: api::VkImage,
    _pSubresource: *const api::VkImageSubresource,
    _pLayout: *mut api::VkSubresourceLayout,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateImageView(
    _device: api::VkDevice,
    create_info: *const api::VkImageViewCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    view: *mut api::VkImageView,
) -> api::VkResult {
    parse_next_chain_const! {
        create_info,
        root = api::VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
    }
    let create_info = &*create_info;
    let new_view = OwnedHandle::<api::VkImageView>::new(ImageView {
        image: SharedHandle::from(create_info.image).unwrap(),
        view_type: ImageViewType::from(create_info.viewType),
        format: create_info.format,
        component_mapping: ComponentMapping::from(create_info.components).unwrap(),
        subresource_range: create_info.subresourceRange,
    });
    *view = new_view.take();
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyImageView(
    _device: api::VkDevice,
    image_view: api::VkImageView,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(image_view);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateShaderModule(
    _device: api::VkDevice,
    create_info: *const api::VkShaderModuleCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    shader_module: *mut api::VkShaderModule,
) -> api::VkResult {
    parse_next_chain_const! {
        create_info,
        root = api::VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
    }
    let create_info = &*create_info;
    const U32_BYTE_COUNT: usize = 4;
    assert_eq!(U32_BYTE_COUNT, mem::size_of::<u32>());
    assert_eq!(create_info.codeSize % U32_BYTE_COUNT, 0);
    assert_ne!(create_info.codeSize, 0);
    let code = util::to_slice(create_info.pCode, create_info.codeSize / U32_BYTE_COUNT);
    *shader_module = OwnedHandle::<api::VkShaderModule>::new(ShaderModule {
        code: code.to_owned(),
    })
    .take();
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyShaderModule(
    _device: api::VkDevice,
    shader_module: api::VkShaderModule,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(shader_module);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreatePipelineCache(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkPipelineCacheCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pPipelineCache: *mut api::VkPipelineCache,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyPipelineCache(
    _device: api::VkDevice,
    _pipelineCache: api::VkPipelineCache,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPipelineCacheData(
    _device: api::VkDevice,
    _pipelineCache: api::VkPipelineCache,
    _pDataSize: *mut usize,
    _pData: *mut c_void,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkMergePipelineCaches(
    _device: api::VkDevice,
    _dstCache: api::VkPipelineCache,
    _srcCacheCount: u32,
    _pSrcCaches: *const api::VkPipelineCache,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateGraphicsPipelines(
    device: api::VkDevice,
    pipeline_cache: api::VkPipelineCache,
    create_info_count: u32,
    create_infos: *const api::VkGraphicsPipelineCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    pipelines: *mut api::VkPipeline,
) -> api::VkResult {
    pipeline::create_pipelines::<pipeline::GraphicsPipeline>(
        SharedHandle::from(device).unwrap(),
        SharedHandle::from(pipeline_cache),
        util::to_slice(create_infos, create_info_count as usize),
        util::to_slice_mut(pipelines, create_info_count as usize),
    )
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateComputePipelines(
    device: api::VkDevice,
    pipeline_cache: api::VkPipelineCache,
    create_info_count: u32,
    create_infos: *const api::VkComputePipelineCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    pipelines: *mut api::VkPipeline,
) -> api::VkResult {
    pipeline::create_pipelines::<pipeline::ComputePipeline>(
        SharedHandle::from(device).unwrap(),
        SharedHandle::from(pipeline_cache),
        util::to_slice(create_infos, create_info_count as usize),
        util::to_slice_mut(pipelines, create_info_count as usize),
    )
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyPipeline(
    _device: api::VkDevice,
    pipeline: api::VkPipeline,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(pipeline);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreatePipelineLayout(
    _device: api::VkDevice,
    create_info: *const api::VkPipelineLayoutCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    pipeline_layout: *mut api::VkPipelineLayout,
) -> api::VkResult {
    parse_next_chain_const! {
        create_info,
        root = api::VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
    }
    let create_info = &*create_info;
    let set_layouts = util::to_slice(create_info.pSetLayouts, create_info.setLayoutCount as usize);
    let push_constant_ranges = util::to_slice(
        create_info.pPushConstantRanges,
        create_info.pushConstantRangeCount as usize,
    );
    let push_constants_size = push_constant_ranges
        .iter()
        .map(|v| v.size as usize + v.offset as usize)
        .max()
        .unwrap_or(0);
    let descriptor_set_layouts: Vec<_> = set_layouts
        .iter()
        .map(|v| SharedHandle::from(*v).unwrap())
        .collect();
    let shader_compiler_pipeline_layout = shader_compiler::PipelineLayout {
        push_constants_size,
        descriptor_sets: descriptor_set_layouts
            .iter()
            .map(
                |descriptor_set_layout| shader_compiler::DescriptorSetLayout {
                    bindings: descriptor_set_layout
                        .bindings
                        .iter()
                        .map(|binding| {
                            Some(match *binding.as_ref()? {
                                DescriptorLayout::Sampler {
                                    count,
                                    immutable_samplers: _,
                                } => shader_compiler::DescriptorLayout::Sampler { count },
                                DescriptorLayout::CombinedImageSampler {
                                    count,
                                    immutable_samplers: _,
                                } => shader_compiler::DescriptorLayout::CombinedImageSampler {
                                    count,
                                },
                                DescriptorLayout::SampledImage { count } => {
                                    shader_compiler::DescriptorLayout::SampledImage { count }
                                }
                                DescriptorLayout::StorageImage { count } => {
                                    shader_compiler::DescriptorLayout::StorageImage { count }
                                }
                                DescriptorLayout::UniformTexelBuffer { count } => {
                                    shader_compiler::DescriptorLayout::UniformTexelBuffer { count }
                                }
                                DescriptorLayout::StorageTexelBuffer { count } => {
                                    shader_compiler::DescriptorLayout::StorageTexelBuffer { count }
                                }
                                DescriptorLayout::UniformBuffer { count } => {
                                    shader_compiler::DescriptorLayout::UniformBuffer { count }
                                }
                                DescriptorLayout::StorageBuffer { count } => {
                                    shader_compiler::DescriptorLayout::StorageBuffer { count }
                                }
                                DescriptorLayout::UniformBufferDynamic { count } => {
                                    shader_compiler::DescriptorLayout::UniformBufferDynamic {
                                        count,
                                    }
                                }
                                DescriptorLayout::StorageBufferDynamic { count } => {
                                    shader_compiler::DescriptorLayout::StorageBufferDynamic {
                                        count,
                                    }
                                }
                                DescriptorLayout::InputAttachment { count } => {
                                    shader_compiler::DescriptorLayout::InputAttachment { count }
                                }
                            })
                        })
                        .collect(),
                },
            )
            .collect(),
    };
    *pipeline_layout = OwnedHandle::<api::VkPipelineLayout>::new(PipelineLayout {
        push_constants_size,
        push_constant_ranges: push_constant_ranges.into(),
        descriptor_set_layouts,
        shader_compiler_pipeline_layout,
    })
    .take();
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyPipelineLayout(
    _device: api::VkDevice,
    pipeline_layout: api::VkPipelineLayout,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(pipeline_layout);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateSampler(
    _device: api::VkDevice,
    create_info: *const api::VkSamplerCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    sampler: *mut api::VkSampler,
) -> api::VkResult {
    parse_next_chain_const! {
        create_info,
        root = api::VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
    }
    let create_info = &*create_info;
    *sampler = OwnedHandle::<api::VkSampler>::new(Sampler {
        mag_filter: create_info.magFilter,
        min_filter: create_info.minFilter,
        mipmap_mode: create_info.mipmapMode,
        address_modes: [
            create_info.addressModeU,
            create_info.addressModeV,
            create_info.addressModeW,
        ],
        mip_lod_bias: create_info.mipLodBias,
        anisotropy: if create_info.anisotropyEnable != api::VK_FALSE {
            Some(sampler::AnisotropySettings {
                max: create_info.maxAnisotropy,
            })
        } else {
            None
        },
        compare_op: if create_info.compareEnable != api::VK_FALSE {
            Some(create_info.compareOp)
        } else {
            None
        },
        min_lod: create_info.minLod,
        max_lod: create_info.maxLod,
        border_color: create_info.borderColor,
        unnormalized_coordinates: create_info.unnormalizedCoordinates != api::VK_FALSE,
        sampler_ycbcr_conversion: None,
    })
    .take();
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroySampler(
    _device: api::VkDevice,
    sampler: api::VkSampler,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(sampler);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateDescriptorSetLayout(
    _device: api::VkDevice,
    create_info: *const api::VkDescriptorSetLayoutCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    set_layout: *mut api::VkDescriptorSetLayout,
) -> api::VkResult {
    parse_next_chain_const! {
        create_info,
        root = api::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
    }
    let create_info = &*create_info;
    let bindings = util::to_slice(create_info.pBindings, create_info.bindingCount as usize);
    let max_binding = bindings.iter().map(|v| v.binding).max().unwrap_or(0) as usize;
    let mut bindings_map: Vec<Option<DescriptorLayout>> = (0..=max_binding).map(|_| None).collect();
    for binding in bindings {
        let bindings_map_entry = &mut bindings_map[binding.binding as usize];
        assert!(
            bindings_map_entry.is_none(),
            "duplicate binding: {}",
            binding.binding
        );
        *bindings_map_entry = Some(DescriptorLayout::from(binding));
    }
    *set_layout = OwnedHandle::<api::VkDescriptorSetLayout>::new(DescriptorSetLayout {
        bindings: bindings_map,
    })
    .take();
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyDescriptorSetLayout(
    _device: api::VkDevice,
    descriptor_set_layout: api::VkDescriptorSetLayout,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(descriptor_set_layout);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateDescriptorPool(
    _device: api::VkDevice,
    create_info: *const api::VkDescriptorPoolCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    descriptor_pool: *mut api::VkDescriptorPool,
) -> api::VkResult {
    parse_next_chain_const! {
        create_info,
        root = api::VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
    }
    *descriptor_pool = OwnedHandle::<api::VkDescriptorPool>::new(DescriptorPool::new()).take();
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyDescriptorPool(
    _device: api::VkDevice,
    descriptor_pool: api::VkDescriptorPool,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(descriptor_pool);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkResetDescriptorPool(
    _device: api::VkDevice,
    descriptor_pool: api::VkDescriptorPool,
    _flags: api::VkDescriptorPoolResetFlags,
) -> api::VkResult {
    MutHandle::from(descriptor_pool).unwrap().reset();
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkAllocateDescriptorSets(
    _device: api::VkDevice,
    allocate_info: *const api::VkDescriptorSetAllocateInfo,
    descriptor_sets: *mut api::VkDescriptorSet,
) -> api::VkResult {
    parse_next_chain_const! {
        allocate_info,
        root = api::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
    }
    let allocate_info = &*allocate_info;
    let mut descriptor_pool = MutHandle::from(allocate_info.descriptorPool).unwrap();
    let descriptor_sets =
        util::to_slice_mut(descriptor_sets, allocate_info.descriptorSetCount as usize);
    let descriptor_set_layouts = util::to_slice(
        allocate_info.pSetLayouts,
        allocate_info.descriptorSetCount as usize,
    );
    descriptor_pool.allocate(
        descriptor_set_layouts
            .iter()
            .map(|descriptor_set_layout| DescriptorSet {
                bindings: SharedHandle::from(*descriptor_set_layout)
                    .unwrap()
                    .bindings
                    .iter()
                    .map(|layout| layout.as_ref().map(Descriptor::from))
                    .collect(),
            }),
        descriptor_sets,
    );
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkFreeDescriptorSets(
    _device: api::VkDevice,
    descriptor_pool: api::VkDescriptorPool,
    descriptor_set_count: u32,
    descriptor_sets: *const api::VkDescriptorSet,
) -> api::VkResult {
    let mut descriptor_pool = MutHandle::from(descriptor_pool).unwrap();
    let descriptor_sets = util::to_slice(descriptor_sets, descriptor_set_count as usize);
    descriptor_pool.free(descriptor_sets);
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkUpdateDescriptorSets(
    _device: api::VkDevice,
    descriptor_write_count: u32,
    descriptor_writes: *const api::VkWriteDescriptorSet,
    descriptor_copy_count: u32,
    descriptor_copies: *const api::VkCopyDescriptorSet,
) {
    let descriptor_writes = util::to_slice(descriptor_writes, descriptor_write_count as usize);
    let descriptor_copies = util::to_slice(descriptor_copies, descriptor_copy_count as usize);
    for descriptor_write in descriptor_writes {
        parse_next_chain_const! {
            descriptor_write,
            root = api::VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
        }
        let mut descriptor_set = MutHandle::from(descriptor_write.dstSet).unwrap();
        let mut binding_index = descriptor_write.dstBinding as usize;
        let mut elements = DescriptorWriteArg::from(descriptor_write);
        let mut start_element = Some(descriptor_write.dstArrayElement as usize);
        while elements.len() != 0 {
            let binding = descriptor_set.bindings[binding_index].as_mut().unwrap();
            binding_index += 1;
            assert_eq!(binding.descriptor_type(), descriptor_write.descriptorType);
            if binding.element_count() == 0 {
                assert_eq!(start_element, None);
                continue;
            }
            let start_element = start_element.take().unwrap_or(0);
            let used_elements = elements
                .len()
                .min(binding.element_count().checked_sub(start_element).unwrap());
            binding.write(start_element, elements.slice_to(..used_elements));
            elements = elements.slice_from(used_elements..);
        }
    }
    for descriptor_copy in descriptor_copies {
        parse_next_chain_const! {
            descriptor_copy,
            root = api::VK_STRUCTURE_TYPE_COPY_DESCRIPTOR_SET,
        }
        unimplemented!()
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateFramebuffer(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkFramebufferCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pFramebuffer: *mut api::VkFramebuffer,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyFramebuffer(
    _device: api::VkDevice,
    _framebuffer: api::VkFramebuffer,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateRenderPass(
    _device: api::VkDevice,
    create_info: *const api::VkRenderPassCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    render_pass: *mut api::VkRenderPass,
) -> api::VkResult {
    parse_next_chain_const! {
        create_info,
        root = api::VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
    }
    // FIXME: finish implementing
    *render_pass = OwnedHandle::<api::VkRenderPass>::new(RenderPass {}).take();
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyRenderPass(
    _device: api::VkDevice,
    render_pass: api::VkRenderPass,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(render_pass);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetRenderAreaGranularity(
    _device: api::VkDevice,
    _renderPass: api::VkRenderPass,
    _pGranularity: *mut api::VkExtent2D,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateCommandPool(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkCommandPoolCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pCommandPool: *mut api::VkCommandPool,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyCommandPool(
    _device: api::VkDevice,
    _commandPool: api::VkCommandPool,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkResetCommandPool(
    _device: api::VkDevice,
    _commandPool: api::VkCommandPool,
    _flags: api::VkCommandPoolResetFlags,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkAllocateCommandBuffers(
    _device: api::VkDevice,
    _pAllocateInfo: *const api::VkCommandBufferAllocateInfo,
    _pCommandBuffers: *mut api::VkCommandBuffer,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkFreeCommandBuffers(
    _device: api::VkDevice,
    _commandPool: api::VkCommandPool,
    _commandBufferCount: u32,
    _pCommandBuffers: *const api::VkCommandBuffer,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkBeginCommandBuffer(
    _commandBuffer: api::VkCommandBuffer,
    _pBeginInfo: *const api::VkCommandBufferBeginInfo,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkEndCommandBuffer(
    _commandBuffer: api::VkCommandBuffer,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkResetCommandBuffer(
    _commandBuffer: api::VkCommandBuffer,
    _flags: api::VkCommandBufferResetFlags,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdBindPipeline(
    _commandBuffer: api::VkCommandBuffer,
    _pipelineBindPoint: api::VkPipelineBindPoint,
    _pipeline: api::VkPipeline,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetViewport(
    _commandBuffer: api::VkCommandBuffer,
    _firstViewport: u32,
    _viewportCount: u32,
    _pViewports: *const api::VkViewport,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetScissor(
    _commandBuffer: api::VkCommandBuffer,
    _firstScissor: u32,
    _scissorCount: u32,
    _pScissors: *const api::VkRect2D,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetLineWidth(
    _commandBuffer: api::VkCommandBuffer,
    _lineWidth: f32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetDepthBias(
    _commandBuffer: api::VkCommandBuffer,
    _depthBiasConstantFactor: f32,
    _depthBiasClamp: f32,
    _depthBiasSlopeFactor: f32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetBlendConstants(
    _commandBuffer: api::VkCommandBuffer,
    _blendConstants: *const f32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetDepthBounds(
    _commandBuffer: api::VkCommandBuffer,
    _minDepthBounds: f32,
    _maxDepthBounds: f32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetStencilCompareMask(
    _commandBuffer: api::VkCommandBuffer,
    _faceMask: api::VkStencilFaceFlags,
    _compareMask: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetStencilWriteMask(
    _commandBuffer: api::VkCommandBuffer,
    _faceMask: api::VkStencilFaceFlags,
    _writeMask: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetStencilReference(
    _commandBuffer: api::VkCommandBuffer,
    _faceMask: api::VkStencilFaceFlags,
    _reference: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdBindDescriptorSets(
    _commandBuffer: api::VkCommandBuffer,
    _pipelineBindPoint: api::VkPipelineBindPoint,
    _layout: api::VkPipelineLayout,
    _firstSet: u32,
    _descriptorSetCount: u32,
    _pDescriptorSets: *const api::VkDescriptorSet,
    _dynamicOffsetCount: u32,
    _pDynamicOffsets: *const u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdBindIndexBuffer(
    _commandBuffer: api::VkCommandBuffer,
    _buffer: api::VkBuffer,
    _offset: api::VkDeviceSize,
    _indexType: api::VkIndexType,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdBindVertexBuffers(
    _commandBuffer: api::VkCommandBuffer,
    _firstBinding: u32,
    _bindingCount: u32,
    _pBuffers: *const api::VkBuffer,
    _pOffsets: *const api::VkDeviceSize,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdDraw(
    _commandBuffer: api::VkCommandBuffer,
    _vertexCount: u32,
    _instanceCount: u32,
    _firstVertex: u32,
    _firstInstance: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdDrawIndexed(
    _commandBuffer: api::VkCommandBuffer,
    _indexCount: u32,
    _instanceCount: u32,
    _firstIndex: u32,
    _vertexOffset: i32,
    _firstInstance: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdDrawIndirect(
    _commandBuffer: api::VkCommandBuffer,
    _buffer: api::VkBuffer,
    _offset: api::VkDeviceSize,
    _drawCount: u32,
    _stride: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdDrawIndexedIndirect(
    _commandBuffer: api::VkCommandBuffer,
    _buffer: api::VkBuffer,
    _offset: api::VkDeviceSize,
    _drawCount: u32,
    _stride: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdDispatch(
    _commandBuffer: api::VkCommandBuffer,
    _groupCountX: u32,
    _groupCountY: u32,
    _groupCountZ: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdDispatchIndirect(
    _commandBuffer: api::VkCommandBuffer,
    _buffer: api::VkBuffer,
    _offset: api::VkDeviceSize,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdCopyBuffer(
    _commandBuffer: api::VkCommandBuffer,
    _srcBuffer: api::VkBuffer,
    _dstBuffer: api::VkBuffer,
    _regionCount: u32,
    _pRegions: *const api::VkBufferCopy,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdCopyImage(
    _commandBuffer: api::VkCommandBuffer,
    _srcImage: api::VkImage,
    _srcImageLayout: api::VkImageLayout,
    _dstImage: api::VkImage,
    _dstImageLayout: api::VkImageLayout,
    _regionCount: u32,
    _pRegions: *const api::VkImageCopy,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdBlitImage(
    _commandBuffer: api::VkCommandBuffer,
    _srcImage: api::VkImage,
    _srcImageLayout: api::VkImageLayout,
    _dstImage: api::VkImage,
    _dstImageLayout: api::VkImageLayout,
    _regionCount: u32,
    _pRegions: *const api::VkImageBlit,
    _filter: api::VkFilter,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdCopyBufferToImage(
    _commandBuffer: api::VkCommandBuffer,
    _srcBuffer: api::VkBuffer,
    _dstImage: api::VkImage,
    _dstImageLayout: api::VkImageLayout,
    _regionCount: u32,
    _pRegions: *const api::VkBufferImageCopy,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdCopyImageToBuffer(
    _commandBuffer: api::VkCommandBuffer,
    _srcImage: api::VkImage,
    _srcImageLayout: api::VkImageLayout,
    _dstBuffer: api::VkBuffer,
    _regionCount: u32,
    _pRegions: *const api::VkBufferImageCopy,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdUpdateBuffer(
    _commandBuffer: api::VkCommandBuffer,
    _dstBuffer: api::VkBuffer,
    _dstOffset: api::VkDeviceSize,
    _dataSize: api::VkDeviceSize,
    _pData: *const c_void,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdFillBuffer(
    _commandBuffer: api::VkCommandBuffer,
    _dstBuffer: api::VkBuffer,
    _dstOffset: api::VkDeviceSize,
    _size: api::VkDeviceSize,
    _data: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdClearColorImage(
    _commandBuffer: api::VkCommandBuffer,
    _image: api::VkImage,
    _imageLayout: api::VkImageLayout,
    _pColor: *const api::VkClearColorValue,
    _rangeCount: u32,
    _pRanges: *const api::VkImageSubresourceRange,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdClearDepthStencilImage(
    _commandBuffer: api::VkCommandBuffer,
    _image: api::VkImage,
    _imageLayout: api::VkImageLayout,
    _pDepthStencil: *const api::VkClearDepthStencilValue,
    _rangeCount: u32,
    _pRanges: *const api::VkImageSubresourceRange,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdClearAttachments(
    _commandBuffer: api::VkCommandBuffer,
    _attachmentCount: u32,
    _pAttachments: *const api::VkClearAttachment,
    _rectCount: u32,
    _pRects: *const api::VkClearRect,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdResolveImage(
    _commandBuffer: api::VkCommandBuffer,
    _srcImage: api::VkImage,
    _srcImageLayout: api::VkImageLayout,
    _dstImage: api::VkImage,
    _dstImageLayout: api::VkImageLayout,
    _regionCount: u32,
    _pRegions: *const api::VkImageResolve,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetEvent(
    _commandBuffer: api::VkCommandBuffer,
    _event: api::VkEvent,
    _stageMask: api::VkPipelineStageFlags,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdResetEvent(
    _commandBuffer: api::VkCommandBuffer,
    _event: api::VkEvent,
    _stageMask: api::VkPipelineStageFlags,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdWaitEvents(
    _commandBuffer: api::VkCommandBuffer,
    _eventCount: u32,
    _pEvents: *const api::VkEvent,
    _srcStageMask: api::VkPipelineStageFlags,
    _dstStageMask: api::VkPipelineStageFlags,
    _memoryBarrierCount: u32,
    _pMemoryBarriers: *const api::VkMemoryBarrier,
    _bufferMemoryBarrierCount: u32,
    _pBufferMemoryBarriers: *const api::VkBufferMemoryBarrier,
    _imageMemoryBarrierCount: u32,
    _pImageMemoryBarriers: *const api::VkImageMemoryBarrier,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdPipelineBarrier(
    _commandBuffer: api::VkCommandBuffer,
    _srcStageMask: api::VkPipelineStageFlags,
    _dstStageMask: api::VkPipelineStageFlags,
    _dependencyFlags: api::VkDependencyFlags,
    _memoryBarrierCount: u32,
    _pMemoryBarriers: *const api::VkMemoryBarrier,
    _bufferMemoryBarrierCount: u32,
    _pBufferMemoryBarriers: *const api::VkBufferMemoryBarrier,
    _imageMemoryBarrierCount: u32,
    _pImageMemoryBarriers: *const api::VkImageMemoryBarrier,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdBeginQuery(
    _commandBuffer: api::VkCommandBuffer,
    _queryPool: api::VkQueryPool,
    _query: u32,
    _flags: api::VkQueryControlFlags,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdEndQuery(
    _commandBuffer: api::VkCommandBuffer,
    _queryPool: api::VkQueryPool,
    _query: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdResetQueryPool(
    _commandBuffer: api::VkCommandBuffer,
    _queryPool: api::VkQueryPool,
    _firstQuery: u32,
    _queryCount: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdWriteTimestamp(
    _commandBuffer: api::VkCommandBuffer,
    _pipelineStage: api::VkPipelineStageFlagBits,
    _queryPool: api::VkQueryPool,
    _query: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdCopyQueryPoolResults(
    _commandBuffer: api::VkCommandBuffer,
    _queryPool: api::VkQueryPool,
    _firstQuery: u32,
    _queryCount: u32,
    _dstBuffer: api::VkBuffer,
    _dstOffset: api::VkDeviceSize,
    _stride: api::VkDeviceSize,
    _flags: api::VkQueryResultFlags,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdPushConstants(
    _commandBuffer: api::VkCommandBuffer,
    _layout: api::VkPipelineLayout,
    _stageFlags: api::VkShaderStageFlags,
    _offset: u32,
    _size: u32,
    _pValues: *const c_void,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdBeginRenderPass(
    _commandBuffer: api::VkCommandBuffer,
    _pRenderPassBegin: *const api::VkRenderPassBeginInfo,
    _contents: api::VkSubpassContents,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdNextSubpass(
    _commandBuffer: api::VkCommandBuffer,
    _contents: api::VkSubpassContents,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdEndRenderPass(_commandBuffer: api::VkCommandBuffer) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdExecuteCommands(
    _commandBuffer: api::VkCommandBuffer,
    _commandBufferCount: u32,
    _pCommandBuffers: *const api::VkCommandBuffer,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkBindBufferMemory2(
    _device: api::VkDevice,
    bind_info_count: u32,
    bind_infos: *const api::VkBindBufferMemoryInfo,
) -> api::VkResult {
    assert_ne!(bind_info_count, 0);
    let bind_infos = util::to_slice(bind_infos, bind_info_count as usize);
    for bind_info in bind_infos {
        parse_next_chain_const! {
            bind_info,
            root = api::VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_INFO,
            device_group_info: api::VkBindBufferMemoryDeviceGroupInfo = api::VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO,
        }
        if !device_group_info.is_null() {
            let device_group_info = &*device_group_info;
            if device_group_info.deviceIndexCount == 0 {
            } else {
                assert_eq!(device_group_info.deviceIndexCount, 1);
                assert_eq!(*device_group_info.pDeviceIndices, 0);
            }
        }
        let bind_info = &*bind_info;
        let mut buffer = MutHandle::from(bind_info.buffer).unwrap();
        let device_memory = SharedHandle::from(bind_info.memory).unwrap();
        let device_memory_size = device_memory.size();
        assert!(bind_info.memoryOffset < device_memory_size as u64);
        let offset = bind_info.memoryOffset as usize;
        assert!(buffer.size.checked_add(offset).unwrap() <= device_memory_size);
        assert_eq!(offset % BUFFER_ALIGNMENT, 0);
        buffer.memory = Some(BufferMemory {
            device_memory,
            offset,
        });
    }
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkBindImageMemory2(
    _device: api::VkDevice,
    bind_info_count: u32,
    bind_infos: *const api::VkBindImageMemoryInfo,
) -> api::VkResult {
    assert_ne!(bind_info_count, 0);
    let bind_infos = util::to_slice(bind_infos, bind_info_count as usize);
    for bind_info in bind_infos {
        parse_next_chain_const! {
            bind_info,
            root = api::VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_INFO,
            device_group_info: api::VkBindImageMemoryDeviceGroupInfo = api::VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO,
            swapchain_info: api::VkBindImageMemorySwapchainInfoKHR = api::VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_SWAPCHAIN_INFO_KHR,
            plane_info: api::VkBindImagePlaneMemoryInfo = api::VK_STRUCTURE_TYPE_BIND_IMAGE_PLANE_MEMORY_INFO,
        }
        if !device_group_info.is_null() {
            let device_group_info = &*device_group_info;
            if device_group_info.deviceIndexCount == 0 {
            } else {
                assert_eq!(device_group_info.deviceIndexCount, 1);
                assert_eq!(*device_group_info.pDeviceIndices, 0);
            }
        }
        if !swapchain_info.is_null() {
            unimplemented!();
        }
        if !plane_info.is_null() {
            unimplemented!();
        }
        let bind_info = &*bind_info;
        let mut image = MutHandle::from(bind_info.image).unwrap();
        let device_memory = SharedHandle::from(bind_info.memory).unwrap();
        let device_memory_size = device_memory.size();
        let image_memory_layout = image.properties.computed_properties().memory_layout;
        assert!(bind_info.memoryOffset < device_memory_size as u64);
        let offset = bind_info.memoryOffset as usize;
        assert!(image_memory_layout.size.checked_add(offset).unwrap() <= device_memory_size);
        assert_eq!(offset % image_memory_layout.alignment, 0);
        image.memory = Some(ImageMemory {
            device_memory,
            offset,
        });
    }
    api::VK_SUCCESS
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetDeviceGroupPeerMemoryFeatures(
    _device: api::VkDevice,
    _heapIndex: u32,
    _localDeviceIndex: u32,
    _remoteDeviceIndex: u32,
    _pPeerMemoryFeatures: *mut api::VkPeerMemoryFeatureFlags,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetDeviceMask(
    _commandBuffer: api::VkCommandBuffer,
    _deviceMask: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdDispatchBase(
    _commandBuffer: api::VkCommandBuffer,
    _baseGroupX: u32,
    _baseGroupY: u32,
    _baseGroupZ: u32,
    _groupCountX: u32,
    _groupCountY: u32,
    _groupCountZ: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkEnumeratePhysicalDeviceGroups(
    instance: api::VkInstance,
    physical_device_group_count: *mut u32,
    physical_device_group_properties: *mut api::VkPhysicalDeviceGroupProperties,
) -> api::VkResult {
    enumerate_helper(
        physical_device_group_count,
        physical_device_group_properties,
        iter::once(()),
        |physical_device_group_properties, _| {
            parse_next_chain_mut! {
                physical_device_group_properties,
                root = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_GROUP_PROPERTIES,
            }
            let mut physical_devices = [Handle::null(); api::VK_MAX_DEVICE_GROUP_SIZE as usize];
            physical_devices[0] = SharedHandle::from(instance)
                .unwrap()
                .physical_device
                .get_handle();
            *physical_device_group_properties = api::VkPhysicalDeviceGroupProperties {
                sType: physical_device_group_properties.sType,
                pNext: physical_device_group_properties.pNext,
                physicalDeviceCount: 1,
                physicalDevices: physical_devices,
                subsetAllocation: api::VK_TRUE,
            };
        },
    )
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetImageMemoryRequirements2(
    _device: api::VkDevice,
    info: *const api::VkImageMemoryRequirementsInfo2,
    memory_requirements: *mut api::VkMemoryRequirements2,
) {
    #![allow(clippy::needless_update)]
    parse_next_chain_const! {
        info,
        root = api::VK_STRUCTURE_TYPE_IMAGE_MEMORY_REQUIREMENTS_INFO_2,
        image_plane_memory_requirements_info: api::VkImagePlaneMemoryRequirementsInfo = api::VK_STRUCTURE_TYPE_IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO,
    }
    parse_next_chain_mut! {
        memory_requirements,
        root = api::VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2,
        dedicated_requirements: api::VkMemoryDedicatedRequirements = api::VK_STRUCTURE_TYPE_MEMORY_DEDICATED_REQUIREMENTS,
    }
    if !image_plane_memory_requirements_info.is_null() {
        unimplemented!();
    }
    let info = &*info;
    let image = SharedHandle::from(info.image).unwrap();
    let memory_requirements = &mut *memory_requirements;
    let layout = image.properties.computed_properties().memory_layout;
    memory_requirements.memoryRequirements = api::VkMemoryRequirements {
        size: layout.size as u64,
        alignment: layout.alignment as u64,
        memoryTypeBits: DeviceMemoryType::Main.to_bits(),
        ..mem::zeroed() // for padding fields
    };
    if !dedicated_requirements.is_null() {
        let dedicated_requirements = &mut *dedicated_requirements;
        dedicated_requirements.prefersDedicatedAllocation = api::VK_FALSE;
        dedicated_requirements.requiresDedicatedAllocation = api::VK_FALSE;
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetBufferMemoryRequirements2(
    _device: api::VkDevice,
    info: *const api::VkBufferMemoryRequirementsInfo2,
    memory_requirements: *mut api::VkMemoryRequirements2,
) {
    #![allow(clippy::needless_update)]
    parse_next_chain_const! {
        info,
        root = api::VK_STRUCTURE_TYPE_BUFFER_MEMORY_REQUIREMENTS_INFO_2,
    }
    parse_next_chain_mut! {
        memory_requirements,
        root = api::VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2,
        dedicated_requirements: api::VkMemoryDedicatedRequirements = api::VK_STRUCTURE_TYPE_MEMORY_DEDICATED_REQUIREMENTS,
    }
    let memory_requirements = &mut *memory_requirements;
    let info = &*info;
    let buffer = SharedHandle::from(info.buffer).unwrap();
    let layout = DeviceMemoryLayout::calculate(buffer.size, BUFFER_ALIGNMENT);
    memory_requirements.memoryRequirements = api::VkMemoryRequirements {
        size: layout.size as u64,
        alignment: layout.alignment as u64,
        memoryTypeBits: DeviceMemoryType::Main.to_bits(),
        ..mem::zeroed() // for padding fields
    };
    if !dedicated_requirements.is_null() {
        let dedicated_requirements = &mut *dedicated_requirements;
        dedicated_requirements.prefersDedicatedAllocation = api::VK_FALSE;
        dedicated_requirements.requiresDedicatedAllocation = api::VK_FALSE;
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetImageSparseMemoryRequirements2(
    _device: api::VkDevice,
    _pInfo: *const api::VkImageSparseMemoryRequirementsInfo2,
    _pSparseMemoryRequirementCount: *mut u32,
    _pSparseMemoryRequirements: *mut api::VkSparseImageMemoryRequirements2,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceFeatures2(
    physical_device: api::VkPhysicalDevice,
    features: *mut api::VkPhysicalDeviceFeatures2,
) {
    parse_next_chain_mut! {
        features,
        root = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2,
        sampler_ycbcr_conversion_features: api::VkPhysicalDeviceSamplerYcbcrConversionFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES,
        physical_device_16bit_storage_features: api::VkPhysicalDevice16BitStorageFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES,
        variable_pointer_features: api::VkPhysicalDeviceVariablePointerFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES,
        physical_device_shader_draw_parameter_features: api::VkPhysicalDeviceShaderDrawParameterFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES,
        physical_device_protected_memory_features: api::VkPhysicalDeviceProtectedMemoryFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES,
        physical_device_multiview_features: api::VkPhysicalDeviceMultiviewFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_FEATURES,
    }
    let physical_device = SharedHandle::from(physical_device).unwrap();
    physical_device.features.export_feature_set(&mut *features);
    if !sampler_ycbcr_conversion_features.is_null() {
        physical_device
            .features
            .export_feature_set(&mut *sampler_ycbcr_conversion_features);
    }
    if !physical_device_16bit_storage_features.is_null() {
        physical_device
            .features
            .export_feature_set(&mut *physical_device_16bit_storage_features);
    }
    if !variable_pointer_features.is_null() {
        physical_device
            .features
            .export_feature_set(&mut *variable_pointer_features);
    }
    if !physical_device_shader_draw_parameter_features.is_null() {
        physical_device
            .features
            .export_feature_set(&mut *physical_device_shader_draw_parameter_features);
    }
    if !physical_device_protected_memory_features.is_null() {
        physical_device
            .features
            .export_feature_set(&mut *physical_device_protected_memory_features);
    }
    if !physical_device_multiview_features.is_null() {
        physical_device
            .features
            .export_feature_set(&mut *physical_device_multiview_features);
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceProperties2(
    physical_device: api::VkPhysicalDevice,
    properties: *mut api::VkPhysicalDeviceProperties2,
) {
    parse_next_chain_mut! {
        properties,
        root = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2,
        point_clipping_properties: api::VkPhysicalDevicePointClippingProperties = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES,
        multiview_properties: api::VkPhysicalDeviceMultiviewProperties = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES,
        id_properties: api::VkPhysicalDeviceIDProperties = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ID_PROPERTIES,
        maintenance_3_properties: api::VkPhysicalDeviceMaintenance3Properties = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES,
        protected_memory_properties: api::VkPhysicalDeviceProtectedMemoryProperties = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROTECTED_MEMORY_PROPERTIES,
        subgroup_properties: api::VkPhysicalDeviceSubgroupProperties = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SUBGROUP_PROPERTIES,
    }
    let properties = &mut *properties;
    let physical_device = SharedHandle::from(physical_device).unwrap();
    properties.properties = physical_device.properties;
    if !point_clipping_properties.is_null() {
        let point_clipping_properties = &mut *point_clipping_properties;
        *point_clipping_properties = api::VkPhysicalDevicePointClippingProperties {
            sType: point_clipping_properties.sType,
            pNext: point_clipping_properties.pNext,
            ..physical_device.point_clipping_properties
        };
    }
    if !multiview_properties.is_null() {
        let multiview_properties = &mut *multiview_properties;
        *multiview_properties = api::VkPhysicalDeviceMultiviewProperties {
            sType: multiview_properties.sType,
            pNext: multiview_properties.pNext,
            ..physical_device.multiview_properties
        };
    }
    if !id_properties.is_null() {
        let id_properties = &mut *id_properties;
        *id_properties = api::VkPhysicalDeviceIDProperties {
            sType: id_properties.sType,
            pNext: id_properties.pNext,
            ..physical_device.id_properties
        };
    }
    if !maintenance_3_properties.is_null() {
        let maintenance_3_properties = &mut *maintenance_3_properties;
        *maintenance_3_properties = api::VkPhysicalDeviceMaintenance3Properties {
            sType: maintenance_3_properties.sType,
            pNext: maintenance_3_properties.pNext,
            ..physical_device.maintenance_3_properties
        };
    }
    if !protected_memory_properties.is_null() {
        let protected_memory_properties = &mut *protected_memory_properties;
        *protected_memory_properties = api::VkPhysicalDeviceProtectedMemoryProperties {
            sType: protected_memory_properties.sType,
            pNext: protected_memory_properties.pNext,
            ..physical_device.protected_memory_properties
        };
    }
    if !subgroup_properties.is_null() {
        let subgroup_properties = &mut *subgroup_properties;
        *subgroup_properties = api::VkPhysicalDeviceSubgroupProperties {
            sType: subgroup_properties.sType,
            pNext: subgroup_properties.pNext,
            ..physical_device.subgroup_properties
        };
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceFormatProperties2(
    _physical_device: api::VkPhysicalDevice,
    format: api::VkFormat,
    format_properties: *mut api::VkFormatProperties2,
) {
    parse_next_chain_mut! {
        format_properties,
        root = api::VK_STRUCTURE_TYPE_FORMAT_PROPERTIES_2,
    }
    let format_properties = &mut *format_properties;
    format_properties.formatProperties = PhysicalDevice::get_format_properties(format);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceImageFormatProperties2(
    _physicalDevice: api::VkPhysicalDevice,
    _pImageFormatInfo: *const api::VkPhysicalDeviceImageFormatInfo2,
    _pImageFormatProperties: *mut api::VkImageFormatProperties2,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceQueueFamilyProperties2(
    physical_device: api::VkPhysicalDevice,
    queue_family_property_count: *mut u32,
    queue_family_properties: *mut api::VkQueueFamilyProperties2,
) {
    enumerate_helper(
        queue_family_property_count,
        queue_family_properties,
        QUEUE_COUNTS.iter(),
        |queue_family_properties, &count| {
            get_physical_device_queue_family_properties(
                SharedHandle::from(physical_device).unwrap(),
                queue_family_properties,
                count,
            );
        },
    );
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceMemoryProperties2(
    physical_device: api::VkPhysicalDevice,
    memory_properties: *mut api::VkPhysicalDeviceMemoryProperties2,
) {
    #![allow(clippy::needless_update)]
    let physical_device = SharedHandle::from(physical_device).unwrap();
    parse_next_chain_mut! {
        memory_properties,
        root = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MEMORY_PROPERTIES_2,
    }
    let memory_properties = &mut *memory_properties;
    let mut properties: api::VkPhysicalDeviceMemoryProperties = mem::zeroed();
    properties.memoryTypeCount = DeviceMemoryTypes::default().len() as u32;
    for (memory_type, _) in DeviceMemoryTypes::default().iter() {
        properties.memoryTypes[memory_type as usize] = api::VkMemoryType {
            propertyFlags: memory_type.flags(),
            heapIndex: memory_type.heap() as u32,
        };
    }
    properties.memoryHeapCount = DeviceMemoryHeaps::default().len() as u32;
    for (memory_heap, _) in DeviceMemoryHeaps::default().iter() {
        properties.memoryHeaps[memory_heap as usize] = api::VkMemoryHeap {
            size: match memory_heap {
                DeviceMemoryHeap::Main => physical_device.system_memory_size * 7 / 8,
            },
            flags: memory_heap.flags(),
            ..mem::zeroed() // for padding fields
        }
    }
    memory_properties.memoryProperties = properties;
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSparseImageFormatProperties2(
    _physicalDevice: api::VkPhysicalDevice,
    _pFormatInfo: *const api::VkPhysicalDeviceSparseImageFormatInfo2,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkSparseImageFormatProperties2,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkTrimCommandPool(
    _device: api::VkDevice,
    _commandPool: api::VkCommandPool,
    _flags: api::VkCommandPoolTrimFlags,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetDeviceQueue2(
    device: api::VkDevice,
    queue_info: *const api::VkDeviceQueueInfo2,
    queue: *mut api::VkQueue,
) {
    parse_next_chain_const! {
        queue_info,
        root = api::VK_STRUCTURE_TYPE_DEVICE_QUEUE_INFO_2,
    }
    let queue_info = &*queue_info;
    assert_eq!(queue_info.flags, 0);
    let device = SharedHandle::from(device).unwrap();
    *queue = device.queues[queue_info.queueFamilyIndex as usize][queue_info.queueIndex as usize]
        .get_handle();
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateSamplerYcbcrConversion(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkSamplerYcbcrConversionCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pYcbcrConversion: *mut api::VkSamplerYcbcrConversion,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroySamplerYcbcrConversion(
    _device: api::VkDevice,
    _ycbcrConversion: api::VkSamplerYcbcrConversion,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateDescriptorUpdateTemplate(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkDescriptorUpdateTemplateCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pDescriptorUpdateTemplate: *mut api::VkDescriptorUpdateTemplate,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyDescriptorUpdateTemplate(
    _device: api::VkDevice,
    _descriptorUpdateTemplate: api::VkDescriptorUpdateTemplate,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkUpdateDescriptorSetWithTemplate(
    _device: api::VkDevice,
    _descriptorSet: api::VkDescriptorSet,
    _descriptorUpdateTemplate: api::VkDescriptorUpdateTemplate,
    _pData: *const c_void,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceExternalBufferProperties(
    _physicalDevice: api::VkPhysicalDevice,
    _pExternalBufferInfo: *const api::VkPhysicalDeviceExternalBufferInfo,
    _pExternalBufferProperties: *mut api::VkExternalBufferProperties,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceExternalFenceProperties(
    _physicalDevice: api::VkPhysicalDevice,
    _pExternalFenceInfo: *const api::VkPhysicalDeviceExternalFenceInfo,
    _pExternalFenceProperties: *mut api::VkExternalFenceProperties,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceExternalSemaphoreProperties(
    _physicalDevice: api::VkPhysicalDevice,
    _pExternalSemaphoreInfo: *const api::VkPhysicalDeviceExternalSemaphoreInfo,
    _pExternalSemaphoreProperties: *mut api::VkExternalSemaphoreProperties,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetDescriptorSetLayoutSupport(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkDescriptorSetLayoutCreateInfo,
    _pSupport: *mut api::VkDescriptorSetLayoutSupport,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroySurfaceKHR(
    _instance: api::VkInstance,
    surface: api::VkSurfaceKHR,
    _allocator: *const api::VkAllocationCallbacks,
) {
    if let Some(surface) = SharedHandle::from(surface) {
        let surface_implementation = SurfacePlatform::from(surface.platform)
            .unwrap()
            .get_surface_implementation();
        surface_implementation.destroy_surface(surface.into_nonnull());
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceSupportKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _queueFamilyIndex: u32,
    _surface: api::VkSurfaceKHR,
    _pSupported: *mut api::VkBool32,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceCapabilitiesKHR(
    physical_device: api::VkPhysicalDevice,
    surface: api::VkSurfaceKHR,
    surface_capabilities: *mut api::VkSurfaceCapabilitiesKHR,
) -> api::VkResult {
    let mut surface_capabilities_2 = api::VkSurfaceCapabilities2KHR {
        sType: api::VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES_2_KHR,
        pNext: null_mut(),
        surfaceCapabilities: mem::zeroed(),
    };
    match vkGetPhysicalDeviceSurfaceCapabilities2KHR(
        physical_device,
        &api::VkPhysicalDeviceSurfaceInfo2KHR {
            sType: api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SURFACE_INFO_2_KHR,
            pNext: null(),
            surface: surface,
        },
        &mut surface_capabilities_2,
    ) {
        api::VK_SUCCESS => {
            *surface_capabilities = surface_capabilities_2.surfaceCapabilities;
            api::VK_SUCCESS
        }
        error => error,
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceFormatsKHR(
    _physical_device: api::VkPhysicalDevice,
    surface: api::VkSurfaceKHR,
    surface_format_count: *mut u32,
    surface_formats: *mut api::VkSurfaceFormatKHR,
) -> api::VkResult {
    let surface_implementation =
        SurfacePlatform::from(SharedHandle::from(surface).unwrap().platform)
            .unwrap()
            .get_surface_implementation();
    let returned_surface_formats = match surface_implementation.get_surface_formats(surface) {
        Ok(returned_surface_formats) => returned_surface_formats,
        Err(result) => return result,
    };
    enumerate_helper(
        surface_format_count,
        surface_formats,
        returned_surface_formats.iter(),
        |a, b| *a = *b,
    )
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfacePresentModesKHR(
    _physical_device: api::VkPhysicalDevice,
    surface: api::VkSurfaceKHR,
    present_mode_count: *mut u32,
    present_modes: *mut api::VkPresentModeKHR,
) -> api::VkResult {
    let surface_implementation =
        SurfacePlatform::from(SharedHandle::from(surface).unwrap().platform)
            .unwrap()
            .get_surface_implementation();
    let returned_present_modes = match surface_implementation.get_present_modes(surface) {
        Ok(returned_present_modes) => returned_present_modes,
        Err(result) => return result,
    };
    enumerate_helper(
        present_mode_count,
        present_modes,
        returned_present_modes.iter(),
        |a, b| *a = *b,
    )
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateSwapchainKHR(
    _device: api::VkDevice,
    create_info: *const api::VkSwapchainCreateInfoKHR,
    _allocator: *const api::VkAllocationCallbacks,
    swapchain: *mut api::VkSwapchainKHR,
) -> api::VkResult {
    parse_next_chain_const! {
        create_info,
        root = api::VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
        device_group_swapchain_create_info: api::VkDeviceGroupSwapchainCreateInfoKHR = api::VK_STRUCTURE_TYPE_DEVICE_GROUP_SWAPCHAIN_CREATE_INFO_KHR,
    }
    let create_info = &*create_info;
    let device_group_swapchain_create_info = if device_group_swapchain_create_info.is_null() {
        None
    } else {
        Some(&*device_group_swapchain_create_info)
    };
    *swapchain = Handle::null();
    let platform =
        SurfacePlatform::from(SharedHandle::from(create_info.surface).unwrap().platform).unwrap();
    match platform
        .get_surface_implementation()
        .build(create_info, device_group_swapchain_create_info)
    {
        Ok(new_swapchain) => {
            *swapchain = OwnedHandle::<api::VkSwapchainKHR>::new(new_swapchain).take();
            api::VK_SUCCESS
        }
        Err(error) => error,
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroySwapchainKHR(
    _device: api::VkDevice,
    swapchain: api::VkSwapchainKHR,
    _allocator: *const api::VkAllocationCallbacks,
) {
    OwnedHandle::from(swapchain);
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetSwapchainImagesKHR(
    _device: api::VkDevice,
    _swapchain: api::VkSwapchainKHR,
    _pSwapchainImageCount: *mut u32,
    _pSwapchainImages: *mut api::VkImage,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkAcquireNextImageKHR(
    _device: api::VkDevice,
    _swapchain: api::VkSwapchainKHR,
    _timeout: u64,
    _semaphore: api::VkSemaphore,
    _fence: api::VkFence,
    _pImageIndex: *mut u32,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkQueuePresentKHR(
    _queue: api::VkQueue,
    _pPresentInfo: *const api::VkPresentInfoKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetDeviceGroupPresentCapabilitiesKHR(
    _device: api::VkDevice,
    _pDeviceGroupPresentCapabilities: *mut api::VkDeviceGroupPresentCapabilitiesKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetDeviceGroupSurfacePresentModesKHR(
    _device: api::VkDevice,
    _surface: api::VkSurfaceKHR,
    _pModes: *mut api::VkDeviceGroupPresentModeFlagsKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDevicePresentRectanglesKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _surface: api::VkSurfaceKHR,
    _pRectCount: *mut u32,
    _pRects: *mut api::VkRect2D,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkAcquireNextImage2KHR(
    _device: api::VkDevice,
    _pAcquireInfo: *const api::VkAcquireNextImageInfoKHR,
    _pImageIndex: *mut u32,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetPhysicalDeviceDisplayPropertiesKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkDisplayPropertiesKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetPhysicalDeviceDisplayPlanePropertiesKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkDisplayPlanePropertiesKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetDisplayPlaneSupportedDisplaysKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _planeIndex: u32,
    _pDisplayCount: *mut u32,
    _pDisplays: *mut api::VkDisplayKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetDisplayModePropertiesKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _display: api::VkDisplayKHR,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkDisplayModePropertiesKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCreateDisplayModeKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _display: api::VkDisplayKHR,
    _pCreateInfo: *const api::VkDisplayModeCreateInfoKHR,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pMode: *mut api::VkDisplayModeKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetDisplayPlaneCapabilitiesKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _mode: api::VkDisplayModeKHR,
    _planeIndex: u32,
    _pCapabilities: *mut api::VkDisplayPlaneCapabilitiesKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCreateDisplayPlaneSurfaceKHR(
    _instance: api::VkInstance,
    _pCreateInfo: *const api::VkDisplaySurfaceCreateInfoKHR,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pSurface: *mut api::VkSurfaceKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCreateSharedSwapchainsKHR(
    _device: api::VkDevice,
    _swapchainCount: u32,
    _pCreateInfos: *const api::VkSwapchainCreateInfoKHR,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pSwapchains: *mut api::VkSwapchainKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetMemoryFdKHR(
    _device: api::VkDevice,
    _pGetFdInfo: *const api::VkMemoryGetFdInfoKHR,
    _pFd: *mut c_int,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetMemoryFdPropertiesKHR(
    _device: api::VkDevice,
    _handleType: api::VkExternalMemoryHandleTypeFlagBits,
    _fd: c_int,
    _pMemoryFdProperties: *mut api::VkMemoryFdPropertiesKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkImportSemaphoreFdKHR(
    _device: api::VkDevice,
    _pImportSemaphoreFdInfo: *const api::VkImportSemaphoreFdInfoKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetSemaphoreFdKHR(
    _device: api::VkDevice,
    _pGetFdInfo: *const api::VkSemaphoreGetFdInfoKHR,
    _pFd: *mut c_int,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdPushDescriptorSetKHR(
    _commandBuffer: api::VkCommandBuffer,
    _pipelineBindPoint: api::VkPipelineBindPoint,
    _layout: api::VkPipelineLayout,
    _set: u32,
    _descriptorWriteCount: u32,
    _pDescriptorWrites: *const api::VkWriteDescriptorSet,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdPushDescriptorSetWithTemplateKHR(
    _commandBuffer: api::VkCommandBuffer,
    _descriptorUpdateTemplate: api::VkDescriptorUpdateTemplate,
    _layout: api::VkPipelineLayout,
    _set: u32,
    _pData: *const c_void,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCreateRenderPass2KHR(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkRenderPassCreateInfo2KHR,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pRenderPass: *mut api::VkRenderPass,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdBeginRenderPass2KHR(
    _commandBuffer: api::VkCommandBuffer,
    _pRenderPassBegin: *const api::VkRenderPassBeginInfo,
    _pSubpassBeginInfo: *const api::VkSubpassBeginInfoKHR,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdNextSubpass2KHR(
    _commandBuffer: api::VkCommandBuffer,
    _pSubpassBeginInfo: *const api::VkSubpassBeginInfoKHR,
    _pSubpassEndInfo: *const api::VkSubpassEndInfoKHR,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdEndRenderPass2KHR(
    _commandBuffer: api::VkCommandBuffer,
    _pSubpassEndInfo: *const api::VkSubpassEndInfoKHR,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetSwapchainStatusKHR(
    _device: api::VkDevice,
    _swapchain: api::VkSwapchainKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkImportFenceFdKHR(
    _device: api::VkDevice,
    _pImportFenceFdInfo: *const api::VkImportFenceFdInfoKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetFenceFdKHR(
    _device: api::VkDevice,
    _pGetFdInfo: *const api::VkFenceGetFdInfoKHR,
    _pFd: *mut c_int,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceCapabilities2KHR(
    _physical_device: api::VkPhysicalDevice,
    surface_info: *const api::VkPhysicalDeviceSurfaceInfo2KHR,
    surface_capabilities: *mut api::VkSurfaceCapabilities2KHR,
) -> api::VkResult {
    parse_next_chain_const! {
        surface_info,
        root = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SURFACE_INFO_2_KHR,
    }
    let surface_info = &*surface_info;
    parse_next_chain_mut! {
        surface_capabilities,
        root = api::VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES_2_KHR,
    }
    let surface_capabilities = &mut *surface_capabilities;
    let surface_implementation =
        SurfacePlatform::from(SharedHandle::from(surface_info.surface).unwrap().platform)
            .unwrap()
            .get_surface_implementation();
    match surface_implementation.get_capabilities(surface_info.surface) {
        Ok(capabilities) => {
            surface_capabilities.surfaceCapabilities = capabilities;
            api::VK_SUCCESS
        }
        Err(result) => result,
    }
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceFormats2KHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pSurfaceInfo: *const api::VkPhysicalDeviceSurfaceInfo2KHR,
    _pSurfaceFormatCount: *mut u32,
    _pSurfaceFormats: *mut api::VkSurfaceFormat2KHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetPhysicalDeviceDisplayProperties2KHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkDisplayProperties2KHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetPhysicalDeviceDisplayPlaneProperties2KHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkDisplayPlaneProperties2KHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetDisplayModeProperties2KHR(
    _physicalDevice: api::VkPhysicalDevice,
    _display: api::VkDisplayKHR,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkDisplayModeProperties2KHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetDisplayPlaneCapabilities2KHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pDisplayPlaneInfo: *const api::VkDisplayPlaneInfo2KHR,
    _pCapabilities: *mut api::VkDisplayPlaneCapabilities2KHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdDrawIndirectCountKHR(
    _commandBuffer: api::VkCommandBuffer,
    _buffer: api::VkBuffer,
    _offset: api::VkDeviceSize,
    _countBuffer: api::VkBuffer,
    _countBufferOffset: api::VkDeviceSize,
    _maxDrawCount: u32,
    _stride: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdDrawIndexedIndirectCountKHR(
    _commandBuffer: api::VkCommandBuffer,
    _buffer: api::VkBuffer,
    _offset: api::VkDeviceSize,
    _countBuffer: api::VkBuffer,
    _countBufferOffset: api::VkDeviceSize,
    _maxDrawCount: u32,
    _stride: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCreateDebugReportCallbackEXT(
    _instance: api::VkInstance,
    _pCreateInfo: *const api::VkDebugReportCallbackCreateInfoEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pCallback: *mut api::VkDebugReportCallbackEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkDestroyDebugReportCallbackEXT(
    _instance: api::VkInstance,
    _callback: api::VkDebugReportCallbackEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkDebugReportMessageEXT(
    _instance: api::VkInstance,
    _flags: api::VkDebugReportFlagsEXT,
    _objectType: api::VkDebugReportObjectTypeEXT,
    _object: u64,
    _location: usize,
    _messageCode: i32,
    _pLayerPrefix: *const c_char,
    _pMessage: *const c_char,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkDebugMarkerSetObjectTagEXT(
    _device: api::VkDevice,
    _pTagInfo: *const api::VkDebugMarkerObjectTagInfoEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkDebugMarkerSetObjectNameEXT(
    _device: api::VkDevice,
    _pNameInfo: *const api::VkDebugMarkerObjectNameInfoEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdDebugMarkerBeginEXT(
    _commandBuffer: api::VkCommandBuffer,
    _pMarkerInfo: *const api::VkDebugMarkerMarkerInfoEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdDebugMarkerEndEXT(_commandBuffer: api::VkCommandBuffer) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdDebugMarkerInsertEXT(
    _commandBuffer: api::VkCommandBuffer,
    _pMarkerInfo: *const api::VkDebugMarkerMarkerInfoEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdDrawIndirectCountAMD(
    _commandBuffer: api::VkCommandBuffer,
    _buffer: api::VkBuffer,
    _offset: api::VkDeviceSize,
    _countBuffer: api::VkBuffer,
    _countBufferOffset: api::VkDeviceSize,
    _maxDrawCount: u32,
    _stride: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdDrawIndexedIndirectCountAMD(
    _commandBuffer: api::VkCommandBuffer,
    _buffer: api::VkBuffer,
    _offset: api::VkDeviceSize,
    _countBuffer: api::VkBuffer,
    _countBufferOffset: api::VkDeviceSize,
    _maxDrawCount: u32,
    _stride: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetShaderInfoAMD(
    _device: api::VkDevice,
    _pipeline: api::VkPipeline,
    _shaderStage: api::VkShaderStageFlagBits,
    _infoType: api::VkShaderInfoTypeAMD,
    _pInfoSize: *mut usize,
    _pInfo: *mut c_void,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetPhysicalDeviceExternalImageFormatPropertiesNV(
    _physicalDevice: api::VkPhysicalDevice,
    _format: api::VkFormat,
    _type_: api::VkImageType,
    _tiling: api::VkImageTiling,
    _usage: api::VkImageUsageFlags,
    _flags: api::VkImageCreateFlags,
    _externalHandleType: api::VkExternalMemoryHandleTypeFlagsNV,
    _pExternalImageFormatProperties: *mut api::VkExternalImageFormatPropertiesNV,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdBeginConditionalRenderingEXT(
    _commandBuffer: api::VkCommandBuffer,
    _pConditionalRenderingBegin: *const api::VkConditionalRenderingBeginInfoEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdEndConditionalRenderingEXT(
    _commandBuffer: api::VkCommandBuffer,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdSetViewportWScalingNV(
    _commandBuffer: api::VkCommandBuffer,
    _firstViewport: u32,
    _viewportCount: u32,
    _pViewportWScalings: *const api::VkViewportWScalingNV,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkReleaseDisplayEXT(
    _physicalDevice: api::VkPhysicalDevice,
    _display: api::VkDisplayKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceCapabilities2EXT(
    _physicalDevice: api::VkPhysicalDevice,
    _surface: api::VkSurfaceKHR,
    _pSurfaceCapabilities: *mut api::VkSurfaceCapabilities2EXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkDisplayPowerControlEXT(
    _device: api::VkDevice,
    _display: api::VkDisplayKHR,
    _pDisplayPowerInfo: *const api::VkDisplayPowerInfoEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkRegisterDeviceEventEXT(
    _device: api::VkDevice,
    _pDeviceEventInfo: *const api::VkDeviceEventInfoEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pFence: *mut api::VkFence,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkRegisterDisplayEventEXT(
    _device: api::VkDevice,
    _display: api::VkDisplayKHR,
    _pDisplayEventInfo: *const api::VkDisplayEventInfoEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pFence: *mut api::VkFence,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetSwapchainCounterEXT(
    _device: api::VkDevice,
    _swapchain: api::VkSwapchainKHR,
    _counter: api::VkSurfaceCounterFlagBitsEXT,
    _pCounterValue: *mut u64,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetRefreshCycleDurationGOOGLE(
    _device: api::VkDevice,
    _swapchain: api::VkSwapchainKHR,
    _pDisplayTimingProperties: *mut api::VkRefreshCycleDurationGOOGLE,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetPastPresentationTimingGOOGLE(
    _device: api::VkDevice,
    _swapchain: api::VkSwapchainKHR,
    _pPresentationTimingCount: *mut u32,
    _pPresentationTimings: *mut api::VkPastPresentationTimingGOOGLE,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdSetDiscardRectangleEXT(
    _commandBuffer: api::VkCommandBuffer,
    _firstDiscardRectangle: u32,
    _discardRectangleCount: u32,
    _pDiscardRectangles: *const api::VkRect2D,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkSetHdrMetadataEXT(
    _device: api::VkDevice,
    _swapchainCount: u32,
    _pSwapchains: *const api::VkSwapchainKHR,
    _pMetadata: *const api::VkHdrMetadataEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkSetDebugUtilsObjectNameEXT(
    _device: api::VkDevice,
    _pNameInfo: *const api::VkDebugUtilsObjectNameInfoEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkSetDebugUtilsObjectTagEXT(
    _device: api::VkDevice,
    _pTagInfo: *const api::VkDebugUtilsObjectTagInfoEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkQueueBeginDebugUtilsLabelEXT(
    _queue: api::VkQueue,
    _pLabelInfo: *const api::VkDebugUtilsLabelEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkQueueEndDebugUtilsLabelEXT(_queue: api::VkQueue) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkQueueInsertDebugUtilsLabelEXT(
    _queue: api::VkQueue,
    _pLabelInfo: *const api::VkDebugUtilsLabelEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdBeginDebugUtilsLabelEXT(
    _commandBuffer: api::VkCommandBuffer,
    _pLabelInfo: *const api::VkDebugUtilsLabelEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdEndDebugUtilsLabelEXT(_commandBuffer: api::VkCommandBuffer) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdInsertDebugUtilsLabelEXT(
    _commandBuffer: api::VkCommandBuffer,
    _pLabelInfo: *const api::VkDebugUtilsLabelEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCreateDebugUtilsMessengerEXT(
    _instance: api::VkInstance,
    _pCreateInfo: *const api::VkDebugUtilsMessengerCreateInfoEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pMessenger: *mut api::VkDebugUtilsMessengerEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkDestroyDebugUtilsMessengerEXT(
    _instance: api::VkInstance,
    _messenger: api::VkDebugUtilsMessengerEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkSubmitDebugUtilsMessageEXT(
    _instance: api::VkInstance,
    _messageSeverity: api::VkDebugUtilsMessageSeverityFlagBitsEXT,
    _messageTypes: api::VkDebugUtilsMessageTypeFlagsEXT,
    _pCallbackData: *const api::VkDebugUtilsMessengerCallbackDataEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdSetSampleLocationsEXT(
    _commandBuffer: api::VkCommandBuffer,
    _pSampleLocationsInfo: *const api::VkSampleLocationsInfoEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetPhysicalDeviceMultisamplePropertiesEXT(
    _physicalDevice: api::VkPhysicalDevice,
    _samples: api::VkSampleCountFlagBits,
    _pMultisampleProperties: *mut api::VkMultisamplePropertiesEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCreateValidationCacheEXT(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkValidationCacheCreateInfoEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pValidationCache: *mut api::VkValidationCacheEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkDestroyValidationCacheEXT(
    _device: api::VkDevice,
    _validationCache: api::VkValidationCacheEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkMergeValidationCachesEXT(
    _device: api::VkDevice,
    _dstCache: api::VkValidationCacheEXT,
    _srcCacheCount: u32,
    _pSrcCaches: *const api::VkValidationCacheEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetValidationCacheDataEXT(
    _device: api::VkDevice,
    _validationCache: api::VkValidationCacheEXT,
    _pDataSize: *mut usize,
    _pData: *mut c_void,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdBindShadingRateImageNV(
    _commandBuffer: api::VkCommandBuffer,
    _imageView: api::VkImageView,
    _imageLayout: api::VkImageLayout,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdSetViewportShadingRatePaletteNV(
    _commandBuffer: api::VkCommandBuffer,
    _firstViewport: u32,
    _viewportCount: u32,
    _pShadingRatePalettes: *const api::VkShadingRatePaletteNV,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdSetCoarseSampleOrderNV(
    _commandBuffer: api::VkCommandBuffer,
    _sampleOrderType: api::VkCoarseSampleOrderTypeNV,
    _customSampleOrderCount: u32,
    _pCustomSampleOrders: *const api::VkCoarseSampleOrderCustomNV,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetMemoryHostPointerPropertiesEXT(
    _device: api::VkDevice,
    _handleType: api::VkExternalMemoryHandleTypeFlagBits,
    _pHostPointer: *const c_void,
    _pMemoryHostPointerProperties: *mut api::VkMemoryHostPointerPropertiesEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdWriteBufferMarkerAMD(
    _commandBuffer: api::VkCommandBuffer,
    _pipelineStage: api::VkPipelineStageFlagBits,
    _dstBuffer: api::VkBuffer,
    _dstOffset: api::VkDeviceSize,
    _marker: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdDrawMeshTasksNV(
    _commandBuffer: api::VkCommandBuffer,
    _taskCount: u32,
    _firstTask: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdDrawMeshTasksIndirectNV(
    _commandBuffer: api::VkCommandBuffer,
    _buffer: api::VkBuffer,
    _offset: api::VkDeviceSize,
    _drawCount: u32,
    _stride: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdDrawMeshTasksIndirectCountNV(
    _commandBuffer: api::VkCommandBuffer,
    _buffer: api::VkBuffer,
    _offset: api::VkDeviceSize,
    _countBuffer: api::VkBuffer,
    _countBufferOffset: api::VkDeviceSize,
    _maxDrawCount: u32,
    _stride: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdSetExclusiveScissorNV(
    _commandBuffer: api::VkCommandBuffer,
    _firstExclusiveScissor: u32,
    _exclusiveScissorCount: u32,
    _pExclusiveScissors: *const api::VkRect2D,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkCmdSetCheckpointNV(
    _commandBuffer: api::VkCommandBuffer,
    _pCheckpointMarker: *const c_void,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[cfg(kazan_include_unused_vulkan_api)]
pub unsafe extern "system" fn vkGetQueueCheckpointDataNV(
    _queue: api::VkQueue,
    _pCheckpointDataCount: *mut u32,
    _pCheckpointData: *mut api::VkCheckpointDataNV,
) {
    unimplemented!()
}

#[cfg(target_os = "linux")]
#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateXcbSurfaceKHR(
    _instance: api::VkInstance,
    create_info: *const api::VkXcbSurfaceCreateInfoKHR,
    _allocator: *const api::VkAllocationCallbacks,
    surface: *mut api::VkSurfaceKHR,
) -> api::VkResult {
    parse_next_chain_const! {
        create_info,
        root = api::VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR,
    }
    let create_info = &*create_info;
    let new_surface = Box::new(api::VkIcdSurfaceXcb {
        base: api::VkIcdSurfaceBase {
            platform: api::VK_ICD_WSI_PLATFORM_XCB,
        },
        connection: create_info.connection,
        window: create_info.window,
    });
    *surface = api::VkSurfaceKHR::new(NonNull::new(
        Box::into_raw(new_surface) as *mut api::VkIcdSurfaceBase
    ));
    api::VK_SUCCESS
}

#[cfg(target_os = "linux")]
#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceXcbPresentationSupportKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _queueFamilyIndex: u32,
    _connection: *mut xcb::ffi::xcb_connection_t,
    _visual_id: xcb::ffi::xcb_visualid_t,
) -> api::VkBool32 {
    unimplemented!()
}
