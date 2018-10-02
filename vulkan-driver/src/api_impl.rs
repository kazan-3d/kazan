// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
#![allow(dead_code)]
use api;
use constants::*;
use enum_map::EnumMap;
use handle::{Handle, OwnedHandle, SharedHandle};
use std::ffi::CStr;
use std::iter;
use std::iter::FromIterator;
use std::mem;
use std::ops::*;
use std::os::raw::c_char;
use std::ptr::null;
use std::ptr::null_mut;
use std::slice;
use std::str::FromStr;
use sys_info;
use uuid;
use xcb;

/// structure types the driver should know about
fn is_supported_structure_type(v: api::VkStructureType) -> bool {
    match v {
        api::VK_STRUCTURE_TYPE_APPLICATION_INFO
        | api::VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO
        | api::VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_INFO
        | api::VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO
        | api::VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_INFO
        | api::VK_STRUCTURE_TYPE_BIND_IMAGE_PLANE_MEMORY_INFO
        | api::VK_STRUCTURE_TYPE_BIND_SPARSE_INFO
        | api::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER
        | api::VK_STRUCTURE_TYPE_BUFFER_MEMORY_REQUIREMENTS_INFO_2
        | api::VK_STRUCTURE_TYPE_BUFFER_VIEW_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO
        | api::VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO
        | api::VK_STRUCTURE_TYPE_COMMAND_BUFFER_INHERITANCE_INFO
        | api::VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_COPY_DESCRIPTOR_SET
        | api::VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO
        | api::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_SUPPORT
        | api::VK_STRUCTURE_TYPE_DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_DEVICE_GROUP_BIND_SPARSE_INFO
        | api::VK_STRUCTURE_TYPE_DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO
        | api::VK_STRUCTURE_TYPE_DEVICE_GROUP_DEVICE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_DEVICE_GROUP_RENDER_PASS_BEGIN_INFO
        | api::VK_STRUCTURE_TYPE_DEVICE_GROUP_SUBMIT_INFO
        | api::VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_DEVICE_QUEUE_INFO_2
        | api::VK_STRUCTURE_TYPE_EVENT_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_EXPORT_FENCE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_EXPORT_MEMORY_ALLOCATE_INFO
        | api::VK_STRUCTURE_TYPE_EXPORT_SEMAPHORE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_EXTERNAL_BUFFER_PROPERTIES
        | api::VK_STRUCTURE_TYPE_EXTERNAL_FENCE_PROPERTIES
        | api::VK_STRUCTURE_TYPE_EXTERNAL_IMAGE_FORMAT_PROPERTIES
        | api::VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_BUFFER_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_IMAGE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_EXTERNAL_SEMAPHORE_PROPERTIES
        | api::VK_STRUCTURE_TYPE_FENCE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_FORMAT_PROPERTIES_2
        | api::VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_IMAGE_FORMAT_PROPERTIES_2
        | api::VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER
        | api::VK_STRUCTURE_TYPE_IMAGE_MEMORY_REQUIREMENTS_INFO_2
        | api::VK_STRUCTURE_TYPE_IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO
        | api::VK_STRUCTURE_TYPE_IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2
        | api::VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_IMAGE_VIEW_USAGE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_MAPPED_MEMORY_RANGE
        | api::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_FLAGS_INFO
        | api::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO
        | api::VK_STRUCTURE_TYPE_MEMORY_BARRIER
        | api::VK_STRUCTURE_TYPE_MEMORY_DEDICATED_ALLOCATE_INFO
        | api::VK_STRUCTURE_TYPE_MEMORY_DEDICATED_REQUIREMENTS
        | api::VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_GROUP_PROPERTIES
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ID_PROPERTIES
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MEMORY_PROPERTIES_2
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_FEATURES
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROTECTED_MEMORY_PROPERTIES
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SUBGROUP_PROPERTIES
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES
        | api::VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PIPELINE_TESSELLATION_STATE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_PROTECTED_SUBMIT_INFO
        | api::VK_STRUCTURE_TYPE_QUERY_POOL_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_QUEUE_FAMILY_PROPERTIES_2
        | api::VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO
        | api::VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_RENDER_PASS_MULTIVIEW_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES
        | api::VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_INFO
        | api::VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO
        | api::VK_STRUCTURE_TYPE_SPARSE_IMAGE_FORMAT_PROPERTIES_2
        | api::VK_STRUCTURE_TYPE_SPARSE_IMAGE_MEMORY_REQUIREMENTS_2
        | api::VK_STRUCTURE_TYPE_SUBMIT_INFO
        | api::VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET => true,
        _ => false,
    }
}

fn verify_structure_type_is_supported(v: api::VkStructureType) {
    assert!(
        is_supported_structure_type(v),
        "missing structure type in is_supported_structure_type: {:?}",
        v
    );
}

unsafe fn parse_next_chain_const(
    root: *const api::VkBaseInStructure,
    expected_root_struct_type: api::VkStructureType,
    expected_child_structs: &[(api::VkStructureType, *mut *const api::VkBaseInStructure)],
) {
    verify_structure_type_is_supported(expected_root_struct_type);
    let ref root = *root;
    assert_eq!(root.sType, expected_root_struct_type);
    for &(child_struct_type, child_struct) in expected_child_structs.iter() {
        verify_structure_type_is_supported(child_struct_type);
        *child_struct = null();
    }
    let mut child = root.pNext as *const api::VkBaseInStructure;
    while !child.is_null() {
        let ref child_ref = *child;
        let search_for_type = child_ref.sType;
        let mut found = false;
        for &(child_struct_type, child_struct) in expected_child_structs.iter() {
            if child_struct_type == search_for_type {
                assert!(
                    (*child_struct).is_null(),
                    "duplicate struct type in pNext chain: {:?}",
                    search_for_type
                );
                *child_struct = child;
                found = true;
                break;
            }
        }
        assert!(
            found || !is_supported_structure_type(search_for_type),
            "unexpected struct type in pNext chain: {:?}",
            search_for_type
        );
        child = child_ref.pNext as *const _;
    }
}

unsafe fn parse_next_chain_mut(
    root: *mut api::VkBaseOutStructure,
    expected_root_struct_type: api::VkStructureType,
    expected_child_structs: &[(api::VkStructureType, *mut *mut api::VkBaseOutStructure)],
) {
    parse_next_chain_const(
        root as *const api::VkBaseInStructure,
        expected_root_struct_type,
        mem::transmute(expected_child_structs),
    )
}

macro_rules! parse_next_chain_const {
    {
        $root:expr,
        root = $root_type:expr,
        $($name:ident: $var_type:ty = $struct_type:expr,)*
    } => {
        $(let mut $name: *const $var_type = null();)*
        parse_next_chain_const(
            $root as *const api::VkBaseInStructure,
            $root_type,
            &[$(($struct_type, &mut $name as *mut *const $var_type as *mut *const api::VkBaseInStructure)),*]
        );
    };
}

macro_rules! parse_next_chain_mut {
    {
        $root:expr,
        root = $root_type:expr,
        $($name:ident: $var_type:ty = $struct_type:expr,)*
    } => {
        $(let mut $name: *mut $var_type = null_mut();)*
        parse_next_chain_mut(
            $root as *mut api::VkBaseOutStructure,
            $root_type,
            &[$(($struct_type, &mut $name as *mut *mut $var_type as *mut *mut api::VkBaseOutStructure)),*]
        );
    };
}

fn copy_str_to_char_array(dest: &mut [c_char], src: &str) {
    assert!(dest.len() >= src.len() + 1);
    let src = src.as_bytes();
    for i in 0..src.len() {
        dest[i] = src[i] as c_char;
    }
    for i in src.len()..dest.len() {
        dest[i] = 0;
    }
}

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
        }
    }
    pub fn get_recursively_required_extensions(self) -> Extensions {
        let mut retval = self.get_required_extensions();
        let mut worklist: EnumMap<Extension, Extension> = enum_map!{_ => self};
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
            ($($name:ident,)*) => {
                match self {
                    $(Extension::$name => stringify!($name),)*
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
        }
    }
    pub fn get_properties(self) -> api::VkExtensionProperties {
        let mut retval = api::VkExtensionProperties {
            extensionName: [0; api::VK_MAX_EXTENSION_NAME_SIZE as usize],
            specVersion: self.get_spec_version(),
        };
        copy_str_to_char_array(&mut retval.extensionName, self.get_name());
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
            | Extension::VK_KHR_variable_pointers => ExtensionScope::Device,
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
        Extensions(enum_map!{_ => false})
    }
    pub fn is_empty(&self) -> bool {
        self.iter().all(|(_, &v)| !v)
    }
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

fn get_proc_address(
    name: *const c_char,
    has_instance_or_device: bool,
    extensions: &Extensions,
) -> api::PFN_vkVoidFunction {
    let mut name = unsafe { CStr::from_ptr(name) }.to_str().ok()?;
    use api::*;
    use std::mem::transmute;
    struct Scope {
        global: bool,
        instance: bool,
    }
    let scope = Scope {
        global: true,
        instance: has_instance_or_device,
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

        proc_address!(vkAllocateCommandBuffers, PFN_vkAllocateCommandBuffers, instance, true);
        proc_address!(vkAllocateDescriptorSets, PFN_vkAllocateDescriptorSets, instance, true);
        proc_address!(vkAllocateMemory, PFN_vkAllocateMemory, instance, true);
        proc_address!(vkBeginCommandBuffer, PFN_vkBeginCommandBuffer, instance, true);
        proc_address!(vkBindBufferMemory, PFN_vkBindBufferMemory, instance, true);
        proc_address!(vkBindBufferMemory2, PFN_vkBindBufferMemory2, instance, true);
        proc_address!(vkBindImageMemory, PFN_vkBindImageMemory, instance, true);
        proc_address!(vkBindImageMemory2, PFN_vkBindImageMemory2, instance, true);
        proc_address!(vkCmdBeginQuery, PFN_vkCmdBeginQuery, instance, true);
        proc_address!(vkCmdBeginRenderPass, PFN_vkCmdBeginRenderPass, instance, true);
        proc_address!(vkCmdBindDescriptorSets, PFN_vkCmdBindDescriptorSets, instance, true);
        proc_address!(vkCmdBindIndexBuffer, PFN_vkCmdBindIndexBuffer, instance, true);
        proc_address!(vkCmdBindPipeline, PFN_vkCmdBindPipeline, instance, true);
        proc_address!(vkCmdBindVertexBuffers, PFN_vkCmdBindVertexBuffers, instance, true);
        proc_address!(vkCmdBlitImage, PFN_vkCmdBlitImage, instance, true);
        proc_address!(vkCmdClearAttachments, PFN_vkCmdClearAttachments, instance, true);
        proc_address!(vkCmdClearColorImage, PFN_vkCmdClearColorImage, instance, true);
        proc_address!(vkCmdClearDepthStencilImage, PFN_vkCmdClearDepthStencilImage, instance, true);
        proc_address!(vkCmdCopyBuffer, PFN_vkCmdCopyBuffer, instance, true);
        proc_address!(vkCmdCopyBufferToImage, PFN_vkCmdCopyBufferToImage, instance, true);
        proc_address!(vkCmdCopyImage, PFN_vkCmdCopyImage, instance, true);
        proc_address!(vkCmdCopyImageToBuffer, PFN_vkCmdCopyImageToBuffer, instance, true);
        proc_address!(vkCmdCopyQueryPoolResults, PFN_vkCmdCopyQueryPoolResults, instance, true);
        proc_address!(vkCmdDispatch, PFN_vkCmdDispatch, instance, true);
        proc_address!(vkCmdDispatchBase, PFN_vkCmdDispatchBase, instance, true);
        proc_address!(vkCmdDispatchIndirect, PFN_vkCmdDispatchIndirect, instance, true);
        proc_address!(vkCmdDraw, PFN_vkCmdDraw, instance, true);
        proc_address!(vkCmdDrawIndexed, PFN_vkCmdDrawIndexed, instance, true);
        proc_address!(vkCmdDrawIndexedIndirect, PFN_vkCmdDrawIndexedIndirect, instance, true);
        proc_address!(vkCmdDrawIndirect, PFN_vkCmdDrawIndirect, instance, true);
        proc_address!(vkCmdEndQuery, PFN_vkCmdEndQuery, instance, true);
        proc_address!(vkCmdEndRenderPass, PFN_vkCmdEndRenderPass, instance, true);
        proc_address!(vkCmdExecuteCommands, PFN_vkCmdExecuteCommands, instance, true);
        proc_address!(vkCmdFillBuffer, PFN_vkCmdFillBuffer, instance, true);
        proc_address!(vkCmdNextSubpass, PFN_vkCmdNextSubpass, instance, true);
        proc_address!(vkCmdPipelineBarrier, PFN_vkCmdPipelineBarrier, instance, true);
        proc_address!(vkCmdPushConstants, PFN_vkCmdPushConstants, instance, true);
        proc_address!(vkCmdResetEvent, PFN_vkCmdResetEvent, instance, true);
        proc_address!(vkCmdResetQueryPool, PFN_vkCmdResetQueryPool, instance, true);
        proc_address!(vkCmdResolveImage, PFN_vkCmdResolveImage, instance, true);
        proc_address!(vkCmdSetBlendConstants, PFN_vkCmdSetBlendConstants, instance, true);
        proc_address!(vkCmdSetDepthBias, PFN_vkCmdSetDepthBias, instance, true);
        proc_address!(vkCmdSetDepthBounds, PFN_vkCmdSetDepthBounds, instance, true);
        proc_address!(vkCmdSetDeviceMask, PFN_vkCmdSetDeviceMask, instance, true);
        proc_address!(vkCmdSetEvent, PFN_vkCmdSetEvent, instance, true);
        proc_address!(vkCmdSetLineWidth, PFN_vkCmdSetLineWidth, instance, true);
        proc_address!(vkCmdSetScissor, PFN_vkCmdSetScissor, instance, true);
        proc_address!(vkCmdSetStencilCompareMask, PFN_vkCmdSetStencilCompareMask, instance, true);
        proc_address!(vkCmdSetStencilReference, PFN_vkCmdSetStencilReference, instance, true);
        proc_address!(vkCmdSetStencilWriteMask, PFN_vkCmdSetStencilWriteMask, instance, true);
        proc_address!(vkCmdSetViewport, PFN_vkCmdSetViewport, instance, true);
        proc_address!(vkCmdUpdateBuffer, PFN_vkCmdUpdateBuffer, instance, true);
        proc_address!(vkCmdWaitEvents, PFN_vkCmdWaitEvents, instance, true);
        proc_address!(vkCmdWriteTimestamp, PFN_vkCmdWriteTimestamp, instance, true);
        proc_address!(vkCreateBuffer, PFN_vkCreateBuffer, instance, true);
        proc_address!(vkCreateBufferView, PFN_vkCreateBufferView, instance, true);
        proc_address!(vkCreateCommandPool, PFN_vkCreateCommandPool, instance, true);
        proc_address!(vkCreateComputePipelines, PFN_vkCreateComputePipelines, instance, true);
        proc_address!(vkCreateDescriptorPool, PFN_vkCreateDescriptorPool, instance, true);
        proc_address!(vkCreateDescriptorSetLayout, PFN_vkCreateDescriptorSetLayout, instance, true);
        proc_address!(vkCreateDescriptorUpdateTemplate, PFN_vkCreateDescriptorUpdateTemplate, instance, true);
        proc_address!(vkCreateDevice, PFN_vkCreateDevice, instance, true);
        proc_address!(vkCreateEvent, PFN_vkCreateEvent, instance, true);
        proc_address!(vkCreateFence, PFN_vkCreateFence, instance, true);
        proc_address!(vkCreateFramebuffer, PFN_vkCreateFramebuffer, instance, true);
        proc_address!(vkCreateGraphicsPipelines, PFN_vkCreateGraphicsPipelines, instance, true);
        proc_address!(vkCreateImage, PFN_vkCreateImage, instance, true);
        proc_address!(vkCreateImageView, PFN_vkCreateImageView, instance, true);
        proc_address!(vkCreatePipelineCache, PFN_vkCreatePipelineCache, instance, true);
        proc_address!(vkCreatePipelineLayout, PFN_vkCreatePipelineLayout, instance, true);
        proc_address!(vkCreateQueryPool, PFN_vkCreateQueryPool, instance, true);
        proc_address!(vkCreateRenderPass, PFN_vkCreateRenderPass, instance, true);
        proc_address!(vkCreateSampler, PFN_vkCreateSampler, instance, true);
        proc_address!(vkCreateSamplerYcbcrConversion, PFN_vkCreateSamplerYcbcrConversion, instance, true);
        proc_address!(vkCreateSemaphore, PFN_vkCreateSemaphore, instance, true);
        proc_address!(vkCreateShaderModule, PFN_vkCreateShaderModule, instance, true);
        proc_address!(vkDestroyBuffer, PFN_vkDestroyBuffer, instance, true);
        proc_address!(vkDestroyBufferView, PFN_vkDestroyBufferView, instance, true);
        proc_address!(vkDestroyCommandPool, PFN_vkDestroyCommandPool, instance, true);
        proc_address!(vkDestroyDescriptorPool, PFN_vkDestroyDescriptorPool, instance, true);
        proc_address!(vkDestroyDescriptorSetLayout, PFN_vkDestroyDescriptorSetLayout, instance, true);
        proc_address!(vkDestroyDescriptorUpdateTemplate, PFN_vkDestroyDescriptorUpdateTemplate, instance, true);
        proc_address!(vkDestroyDevice, PFN_vkDestroyDevice, instance, true);
        proc_address!(vkDestroyEvent, PFN_vkDestroyEvent, instance, true);
        proc_address!(vkDestroyFence, PFN_vkDestroyFence, instance, true);
        proc_address!(vkDestroyFramebuffer, PFN_vkDestroyFramebuffer, instance, true);
        proc_address!(vkDestroyImage, PFN_vkDestroyImage, instance, true);
        proc_address!(vkDestroyImageView, PFN_vkDestroyImageView, instance, true);
        proc_address!(vkDestroyInstance, PFN_vkDestroyInstance, instance, true);
        proc_address!(vkDestroyPipeline, PFN_vkDestroyPipeline, instance, true);
        proc_address!(vkDestroyPipelineCache, PFN_vkDestroyPipelineCache, instance, true);
        proc_address!(vkDestroyPipelineLayout, PFN_vkDestroyPipelineLayout, instance, true);
        proc_address!(vkDestroyQueryPool, PFN_vkDestroyQueryPool, instance, true);
        proc_address!(vkDestroyRenderPass, PFN_vkDestroyRenderPass, instance, true);
        proc_address!(vkDestroySampler, PFN_vkDestroySampler, instance, true);
        proc_address!(vkDestroySamplerYcbcrConversion, PFN_vkDestroySamplerYcbcrConversion, instance, true);
        proc_address!(vkDestroySemaphore, PFN_vkDestroySemaphore, instance, true);
        proc_address!(vkDestroyShaderModule, PFN_vkDestroyShaderModule, instance, true);
        proc_address!(vkDeviceWaitIdle, PFN_vkDeviceWaitIdle, instance, true);
        proc_address!(vkEndCommandBuffer, PFN_vkEndCommandBuffer, instance, true);
        proc_address!(vkEnumerateDeviceExtensionProperties, PFN_vkEnumerateDeviceExtensionProperties, instance, true);
        proc_address!(vkEnumerateDeviceLayerProperties, PFN_vkEnumerateDeviceLayerProperties, instance, true);
        proc_address!(vkEnumeratePhysicalDeviceGroups, PFN_vkEnumeratePhysicalDeviceGroups, instance, true);
        proc_address!(vkEnumeratePhysicalDevices, PFN_vkEnumeratePhysicalDevices, instance, true);
        proc_address!(vkFlushMappedMemoryRanges, PFN_vkFlushMappedMemoryRanges, instance, true);
        proc_address!(vkFreeCommandBuffers, PFN_vkFreeCommandBuffers, instance, true);
        proc_address!(vkFreeDescriptorSets, PFN_vkFreeDescriptorSets, instance, true);
        proc_address!(vkFreeMemory, PFN_vkFreeMemory, instance, true);
        proc_address!(vkGetBufferMemoryRequirements, PFN_vkGetBufferMemoryRequirements, instance, true);
        proc_address!(vkGetBufferMemoryRequirements2, PFN_vkGetBufferMemoryRequirements2, instance, true);
        proc_address!(vkGetDescriptorSetLayoutSupport, PFN_vkGetDescriptorSetLayoutSupport, instance, true);
        proc_address!(vkGetDeviceGroupPeerMemoryFeatures, PFN_vkGetDeviceGroupPeerMemoryFeatures, instance, true);
        proc_address!(vkGetDeviceMemoryCommitment, PFN_vkGetDeviceMemoryCommitment, instance, true);
        proc_address!(vkGetDeviceProcAddr, PFN_vkGetDeviceProcAddr, instance, true);
        proc_address!(vkGetDeviceQueue, PFN_vkGetDeviceQueue, instance, true);
        proc_address!(vkGetDeviceQueue2, PFN_vkGetDeviceQueue2, instance, true);
        proc_address!(vkGetEventStatus, PFN_vkGetEventStatus, instance, true);
        proc_address!(vkGetFenceStatus, PFN_vkGetFenceStatus, instance, true);
        proc_address!(vkGetImageMemoryRequirements, PFN_vkGetImageMemoryRequirements, instance, true);
        proc_address!(vkGetImageMemoryRequirements2, PFN_vkGetImageMemoryRequirements2, instance, true);
        proc_address!(vkGetImageSparseMemoryRequirements, PFN_vkGetImageSparseMemoryRequirements, instance, true);
        proc_address!(vkGetImageSparseMemoryRequirements2, PFN_vkGetImageSparseMemoryRequirements2, instance, true);
        proc_address!(vkGetImageSubresourceLayout, PFN_vkGetImageSubresourceLayout, instance, true);
        proc_address!(vkGetInstanceProcAddr, PFN_vkGetInstanceProcAddr, instance, true);
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
        proc_address!(vkGetPipelineCacheData, PFN_vkGetPipelineCacheData, instance, true);
        proc_address!(vkGetQueryPoolResults, PFN_vkGetQueryPoolResults, instance, true);
        proc_address!(vkGetRenderAreaGranularity, PFN_vkGetRenderAreaGranularity, instance, true);
        proc_address!(vkInvalidateMappedMemoryRanges, PFN_vkInvalidateMappedMemoryRanges, instance, true);
        proc_address!(vkMapMemory, PFN_vkMapMemory, instance, true);
        proc_address!(vkMergePipelineCaches, PFN_vkMergePipelineCaches, instance, true);
        proc_address!(vkQueueBindSparse, PFN_vkQueueBindSparse, instance, true);
        proc_address!(vkQueueSubmit, PFN_vkQueueSubmit, instance, true);
        proc_address!(vkQueueWaitIdle, PFN_vkQueueWaitIdle, instance, true);
        proc_address!(vkResetCommandBuffer, PFN_vkResetCommandBuffer, instance, true);
        proc_address!(vkResetCommandPool, PFN_vkResetCommandPool, instance, true);
        proc_address!(vkResetDescriptorPool, PFN_vkResetDescriptorPool, instance, true);
        proc_address!(vkResetEvent, PFN_vkResetEvent, instance, true);
        proc_address!(vkResetFences, PFN_vkResetFences, instance, true);
        proc_address!(vkSetEvent, PFN_vkSetEvent, instance, true);
        proc_address!(vkTrimCommandPool, PFN_vkTrimCommandPool, instance, true);
        proc_address!(vkUnmapMemory, PFN_vkUnmapMemory, instance, true);
        proc_address!(vkUpdateDescriptorSets, PFN_vkUpdateDescriptorSets, instance, true);
        proc_address!(vkUpdateDescriptorSetWithTemplate, PFN_vkUpdateDescriptorSetWithTemplate, instance, true);
        proc_address!(vkWaitForFences, PFN_vkWaitForFences, instance, true);

        proc_address!(vkDestroySurfaceKHR, PFN_vkDestroySurfaceKHR, instance, extensions[Extension::VK_KHR_surface]);
        proc_address!(vkGetPhysicalDeviceSurfaceSupportKHR, PFN_vkGetPhysicalDeviceSurfaceSupportKHR, instance, extensions[Extension::VK_KHR_surface]);
        proc_address!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR, PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR, instance, extensions[Extension::VK_KHR_surface]);
        proc_address!(vkGetPhysicalDeviceSurfaceFormatsKHR, PFN_vkGetPhysicalDeviceSurfaceFormatsKHR, instance, extensions[Extension::VK_KHR_surface]);
        proc_address!(vkGetPhysicalDeviceSurfacePresentModesKHR, PFN_vkGetPhysicalDeviceSurfacePresentModesKHR, instance, extensions[Extension::VK_KHR_surface]);

        /*
        proc_address!(vkGetDeviceGroupPresentCapabilitiesKHR, PFN_vkGetDeviceGroupPresentCapabilitiesKHR, instance, extensions[Extension::VK_KHR_swapchain]);
        proc_address!(vkGetDeviceGroupSurfacePresentModesKHR, PFN_vkGetDeviceGroupSurfacePresentModesKHR, instance, extensions[Extension::VK_KHR_swapchain]);
        proc_address!(vkGetPhysicalDevicePresentRectanglesKHR, PFN_vkGetPhysicalDevicePresentRectanglesKHR, instance, extensions[Extension::VK_KHR_swapchain]);
        proc_address!(vkAcquireNextImage2KHR, PFN_vkAcquireNextImage2KHR, instance, extensions[Extension::VK_KHR_swapchain]);

        proc_address!(vkAcquireNextImageKHR, PFN_vkAcquireNextImageKHR, instance, unknown);
        proc_address!(vkCmdBeginConditionalRenderingEXT, PFN_vkCmdBeginConditionalRenderingEXT, instance, unknown);
        proc_address!(vkCmdBeginDebugUtilsLabelEXT, PFN_vkCmdBeginDebugUtilsLabelEXT, instance, unknown);
        proc_address!(vkCmdBeginRenderPass2KHR, PFN_vkCmdBeginRenderPass2KHR, instance, unknown);
        proc_address!(vkCmdBindShadingRateImageNV, PFN_vkCmdBindShadingRateImageNV, instance, unknown);
        proc_address!(vkCmdDebugMarkerBeginEXT, PFN_vkCmdDebugMarkerBeginEXT, instance, unknown);
        proc_address!(vkCmdDebugMarkerEndEXT, PFN_vkCmdDebugMarkerEndEXT, instance, unknown);
        proc_address!(vkCmdDebugMarkerInsertEXT, PFN_vkCmdDebugMarkerInsertEXT, instance, unknown);
        proc_address!(vkCmdDrawIndexedIndirectCountAMD, PFN_vkCmdDrawIndexedIndirectCountAMD, instance, unknown);
        proc_address!(vkCmdDrawIndexedIndirectCountKHR, PFN_vkCmdDrawIndexedIndirectCountKHR, instance, unknown);
        proc_address!(vkCmdDrawIndirectCountAMD, PFN_vkCmdDrawIndirectCountAMD, instance, unknown);
        proc_address!(vkCmdDrawIndirectCountKHR, PFN_vkCmdDrawIndirectCountKHR, instance, unknown);
        proc_address!(vkCmdDrawMeshTasksIndirectCountNV, PFN_vkCmdDrawMeshTasksIndirectCountNV, instance, unknown);
        proc_address!(vkCmdDrawMeshTasksIndirectNV, PFN_vkCmdDrawMeshTasksIndirectNV, instance, unknown);
        proc_address!(vkCmdDrawMeshTasksNV, PFN_vkCmdDrawMeshTasksNV, instance, unknown);
        proc_address!(vkCmdEndConditionalRenderingEXT, PFN_vkCmdEndConditionalRenderingEXT, instance, unknown);
        proc_address!(vkCmdEndDebugUtilsLabelEXT, PFN_vkCmdEndDebugUtilsLabelEXT, instance, unknown);
        proc_address!(vkCmdEndRenderPass2KHR, PFN_vkCmdEndRenderPass2KHR, instance, unknown);
        proc_address!(vkCmdInsertDebugUtilsLabelEXT, PFN_vkCmdInsertDebugUtilsLabelEXT, instance, unknown);
        proc_address!(vkCmdNextSubpass2KHR, PFN_vkCmdNextSubpass2KHR, instance, unknown);
        proc_address!(vkCmdPushDescriptorSetKHR, PFN_vkCmdPushDescriptorSetKHR, instance, unknown);
        proc_address!(vkCmdPushDescriptorSetWithTemplateKHR, PFN_vkCmdPushDescriptorSetWithTemplateKHR, instance, unknown);
        proc_address!(vkCmdSetCheckpointNV, PFN_vkCmdSetCheckpointNV, instance, unknown);
        proc_address!(vkCmdSetCoarseSampleOrderNV, PFN_vkCmdSetCoarseSampleOrderNV, instance, unknown);
        proc_address!(vkCmdSetDiscardRectangleEXT, PFN_vkCmdSetDiscardRectangleEXT, instance, unknown);
        proc_address!(vkCmdSetExclusiveScissorNV, PFN_vkCmdSetExclusiveScissorNV, instance, unknown);
        proc_address!(vkCmdSetSampleLocationsEXT, PFN_vkCmdSetSampleLocationsEXT, instance, unknown);
        proc_address!(vkCmdSetViewportShadingRatePaletteNV, PFN_vkCmdSetViewportShadingRatePaletteNV, instance, unknown);
        proc_address!(vkCmdSetViewportWScalingNV, PFN_vkCmdSetViewportWScalingNV, instance, unknown);
        proc_address!(vkCmdWriteBufferMarkerAMD, PFN_vkCmdWriteBufferMarkerAMD, instance, unknown);
        proc_address!(vkCreateDebugReportCallbackEXT, PFN_vkCreateDebugReportCallbackEXT, instance, unknown);
        proc_address!(vkCreateDebugUtilsMessengerEXT, PFN_vkCreateDebugUtilsMessengerEXT, instance, unknown);
        proc_address!(vkCreateDisplayModeKHR, PFN_vkCreateDisplayModeKHR, instance, unknown);
        proc_address!(vkCreateDisplayPlaneSurfaceKHR, PFN_vkCreateDisplayPlaneSurfaceKHR, instance, unknown);
        proc_address!(vkCreateRenderPass2KHR, PFN_vkCreateRenderPass2KHR, instance, unknown);
        proc_address!(vkCreateSharedSwapchainsKHR, PFN_vkCreateSharedSwapchainsKHR, instance, unknown);
        proc_address!(vkCreateSwapchainKHR, PFN_vkCreateSwapchainKHR, instance, unknown);
        proc_address!(vkCreateValidationCacheEXT, PFN_vkCreateValidationCacheEXT, instance, unknown);
        proc_address!(vkCreateXcbSurfaceKHR, PFN_vkCreateXcbSurfaceKHR, instance, unknown);
        proc_address!(vkDebugMarkerSetObjectNameEXT, PFN_vkDebugMarkerSetObjectNameEXT, instance, unknown);
        proc_address!(vkDebugMarkerSetObjectTagEXT, PFN_vkDebugMarkerSetObjectTagEXT, instance, unknown);
        proc_address!(vkDebugReportCallbackEXT, PFN_vkDebugReportCallbackEXT, instance, unknown);
        proc_address!(vkDebugReportMessageEXT, PFN_vkDebugReportMessageEXT, instance, unknown);
        proc_address!(vkDebugUtilsMessengerCallbackEXT, PFN_vkDebugUtilsMessengerCallbackEXT, instance, unknown);
        proc_address!(vkDestroyDebugReportCallbackEXT, PFN_vkDestroyDebugReportCallbackEXT, instance, unknown);
        proc_address!(vkDestroyDebugUtilsMessengerEXT, PFN_vkDestroyDebugUtilsMessengerEXT, instance, unknown);
        proc_address!(vkDestroySwapchainKHR, PFN_vkDestroySwapchainKHR, instance, unknown);
        proc_address!(vkDestroyValidationCacheEXT, PFN_vkDestroyValidationCacheEXT, instance, unknown);
        proc_address!(vkDisplayPowerControlEXT, PFN_vkDisplayPowerControlEXT, instance, unknown);
        proc_address!(vkGetDisplayModeProperties2KHR, PFN_vkGetDisplayModeProperties2KHR, instance, unknown);
        proc_address!(vkGetDisplayModePropertiesKHR, PFN_vkGetDisplayModePropertiesKHR, instance, unknown);
        proc_address!(vkGetDisplayPlaneCapabilities2KHR, PFN_vkGetDisplayPlaneCapabilities2KHR, instance, unknown);
        proc_address!(vkGetDisplayPlaneCapabilitiesKHR, PFN_vkGetDisplayPlaneCapabilitiesKHR, instance, unknown);
        proc_address!(vkGetDisplayPlaneSupportedDisplaysKHR, PFN_vkGetDisplayPlaneSupportedDisplaysKHR, instance, unknown);
        proc_address!(vkGetFenceFdKHR, PFN_vkGetFenceFdKHR, instance, unknown);
        proc_address!(vkGetMemoryFdKHR, PFN_vkGetMemoryFdKHR, instance, unknown);
        proc_address!(vkGetMemoryFdPropertiesKHR, PFN_vkGetMemoryFdPropertiesKHR, instance, unknown);
        proc_address!(vkGetMemoryHostPointerPropertiesEXT, PFN_vkGetMemoryHostPointerPropertiesEXT, instance, unknown);
        proc_address!(vkGetPastPresentationTimingGOOGLE, PFN_vkGetPastPresentationTimingGOOGLE, instance, unknown);
        proc_address!(vkGetPhysicalDeviceDisplayPlaneProperties2KHR, PFN_vkGetPhysicalDeviceDisplayPlaneProperties2KHR, instance, unknown);
        proc_address!(vkGetPhysicalDeviceDisplayPlanePropertiesKHR, PFN_vkGetPhysicalDeviceDisplayPlanePropertiesKHR, instance, unknown);
        proc_address!(vkGetPhysicalDeviceDisplayProperties2KHR, PFN_vkGetPhysicalDeviceDisplayProperties2KHR, instance, unknown);
        proc_address!(vkGetPhysicalDeviceDisplayPropertiesKHR, PFN_vkGetPhysicalDeviceDisplayPropertiesKHR, instance, unknown);
        proc_address!(vkGetPhysicalDeviceExternalImageFormatPropertiesNV, PFN_vkGetPhysicalDeviceExternalImageFormatPropertiesNV, instance, unknown);
        proc_address!(vkGetPhysicalDeviceMultisamplePropertiesEXT, PFN_vkGetPhysicalDeviceMultisamplePropertiesEXT, instance, unknown);
        proc_address!(vkGetPhysicalDeviceSurfaceCapabilities2EXT, PFN_vkGetPhysicalDeviceSurfaceCapabilities2EXT, instance, unknown);
        proc_address!(vkGetPhysicalDeviceSurfaceCapabilities2KHR, PFN_vkGetPhysicalDeviceSurfaceCapabilities2KHR, instance, unknown);
        proc_address!(vkGetPhysicalDeviceSurfaceFormats2KHR, PFN_vkGetPhysicalDeviceSurfaceFormats2KHR, instance, unknown);
        proc_address!(vkGetPhysicalDeviceXcbPresentationSupportKHR, PFN_vkGetPhysicalDeviceXcbPresentationSupportKHR, instance, unknown);
        proc_address!(vkGetQueueCheckpointDataNV, PFN_vkGetQueueCheckpointDataNV, instance, unknown);
        proc_address!(vkGetRefreshCycleDurationGOOGLE, PFN_vkGetRefreshCycleDurationGOOGLE, instance, unknown);
        proc_address!(vkGetSemaphoreFdKHR, PFN_vkGetSemaphoreFdKHR, instance, unknown);
        proc_address!(vkGetShaderInfoAMD, PFN_vkGetShaderInfoAMD, instance, unknown);
        proc_address!(vkGetSwapchainCounterEXT, PFN_vkGetSwapchainCounterEXT, instance, unknown);
        proc_address!(vkGetSwapchainImagesKHR, PFN_vkGetSwapchainImagesKHR, instance, unknown);
        proc_address!(vkGetSwapchainStatusKHR, PFN_vkGetSwapchainStatusKHR, instance, unknown);
        proc_address!(vkGetValidationCacheDataEXT, PFN_vkGetValidationCacheDataEXT, instance, unknown);
        proc_address!(vkImportFenceFdKHR, PFN_vkImportFenceFdKHR, instance, unknown);
        proc_address!(vkImportSemaphoreFdKHR, PFN_vkImportSemaphoreFdKHR, instance, unknown);
        proc_address!(vkMergeValidationCachesEXT, PFN_vkMergeValidationCachesEXT, instance, unknown);
        proc_address!(vkQueueBeginDebugUtilsLabelEXT, PFN_vkQueueBeginDebugUtilsLabelEXT, instance, unknown);
        proc_address!(vkQueueEndDebugUtilsLabelEXT, PFN_vkQueueEndDebugUtilsLabelEXT, instance, unknown);
        proc_address!(vkQueueInsertDebugUtilsLabelEXT, PFN_vkQueueInsertDebugUtilsLabelEXT, instance, unknown);
        proc_address!(vkQueuePresentKHR, PFN_vkQueuePresentKHR, instance, unknown);
        proc_address!(vkRegisterDeviceEventEXT, PFN_vkRegisterDeviceEventEXT, instance, unknown);
        proc_address!(vkRegisterDisplayEventEXT, PFN_vkRegisterDisplayEventEXT, instance, unknown);
        proc_address!(vkReleaseDisplayEXT, PFN_vkReleaseDisplayEXT, instance, unknown);
        proc_address!(vkSetDebugUtilsObjectNameEXT, PFN_vkSetDebugUtilsObjectNameEXT, instance, unknown);
        proc_address!(vkSetDebugUtilsObjectTagEXT, PFN_vkSetDebugUtilsObjectTagEXT, instance, unknown);
        proc_address!(vkSetHdrMetadataEXT, PFN_vkSetHdrMetadataEXT, instance, unknown);
        proc_address!(vkSubmitDebugUtilsMessageEXT, PFN_vkSubmitDebugUtilsMessageEXT, instance, unknown);
        */
    }
    eprintln!("unknown function: {:?}", name);
    None
}

#[derive(Copy, Clone)]
pub struct Features {
    features: api::VkPhysicalDeviceFeatures,
    physical_device_16bit_storage_features: api::VkPhysicalDevice16BitStorageFeatures,
    sampler_ycbcr_conversion_features: api::VkPhysicalDeviceSamplerYcbcrConversionFeatures,
    variable_pointer_features: api::VkPhysicalDeviceVariablePointerFeatures,
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
    }
    fn visit2<F: FnMut(bool, bool)>(mut self, mut rhs: Self, mut f: F) {
        self.visit2_mut(&mut rhs, |v1, v2| f(*v1, *v2));
    }
    fn visit_mut<F: FnMut(&mut bool)>(&mut self, mut f: F) {
        let mut rhs = *self;
        self.visit2_mut(&mut rhs, |v, _| f(v));
    }
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

pub struct Device {
    physical_device: SharedHandle<api::VkPhysicalDevice>,
    extensions: Extensions,
    features: Features,
}

impl Device {
    unsafe fn new(
        physical_device: SharedHandle<api::VkPhysicalDevice>,
        create_info: *const api::VkDeviceCreateInfo,
    ) -> Result<OwnedHandle<api::VkDevice>, api::VkResult> {
        parse_next_chain_const!{
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
        let ref create_info = *create_info;
        let mut selected_features = Features::splat(false);
        if !device_group_device_create_info.is_null() {
            // FIXME: implement
            unimplemented!()
        }
        if !physical_device_16bit_storage_features.is_null() {
            selected_features.import_feature_set(&*physical_device_16bit_storage_features);
        }
        if !physical_device_features_2.is_null() {
            selected_features.import_feature_set(&*physical_device_features_2);
        } else {
            selected_features.import_feature_set(&*create_info.pEnabledFeatures);
        }
        if !physical_device_multiview_features.is_null() {
            // FIXME: implement
            unimplemented!()
        }
        if !physical_device_protected_memory_features.is_null() {
            // FIXME: implement
            unimplemented!()
        }
        if !physical_device_sampler_ycbcr_conversion_features.is_null() {
            selected_features
                .import_feature_set(&*physical_device_sampler_ycbcr_conversion_features);
        }
        if !physical_device_shader_draw_parameter_features.is_null() {
            // FIXME: implement
            unimplemented!()
        }
        if !physical_device_variable_pointer_features.is_null() {
            selected_features.import_feature_set(&*physical_device_variable_pointer_features);
        }
        unimplemented!()
    }
}

pub struct PhysicalDevice {
    enabled_extensions: Extensions,
    allowed_extensions: Extensions,
    properties: api::VkPhysicalDeviceProperties,
    features: Features,
    system_memory_size: u64,
}

impl PhysicalDevice {
    pub fn get_pipeline_cache_uuid() -> uuid::Uuid {
        // FIXME: return real uuid
        uuid::Uuid::nil()
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
        parse_next_chain_const!{
            create_info,
            root = api::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        }
        let ref create_info = *create_info;
        if create_info.enabledLayerCount != 0 {
            return Err(api::VK_ERROR_LAYER_NOT_PRESENT);
        }
        let mut enabled_extensions = Extensions::create_empty();
        for &extension_name in slice::from_raw_parts(
            create_info.ppEnabledExtensionNames,
            create_info.enabledExtensionCount as usize,
        ) {
            let extension: Extension = CStr::from_ptr(extension_name)
                .to_str()
                .map_err(|_| api::VK_ERROR_EXTENSION_NOT_PRESENT)?
                .parse()
                .map_err(|_| api::VK_ERROR_EXTENSION_NOT_PRESENT)?;
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
        copy_str_to_char_array(&mut device_name, KAZAN_DEVICE_NAME);
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
                    limits: api::VkPhysicalDeviceLimits {
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
                        minTexelBufferOffsetAlignment: 256, // FIXME: update to correct value
                        minUniformBufferOffsetAlignment: 256, // FIXME: update to correct value
                        minStorageBufferOffsetAlignment: 256, // FIXME: update to correct value
                        minTexelOffset: -8,                 // FIXME: update to correct value
                        maxTexelOffset: 7,                  // FIXME: update to correct value
                        minTexelGatherOffset: 0,
                        maxTexelGatherOffset: 0,
                        minInterpolationOffset: 0.0,
                        maxInterpolationOffset: 0.0,
                        subPixelInterpolationOffsetBits: 0,
                        maxFramebufferWidth: 4096, // FIXME: update to correct value
                        maxFramebufferHeight: 4096, // FIXME: update to correct value
                        maxFramebufferLayers: 256, // FIXME: update to correct value
                        framebufferColorSampleCounts: api::VK_SAMPLE_COUNT_1_BIT
                            | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
                        framebufferDepthSampleCounts: api::VK_SAMPLE_COUNT_1_BIT
                            | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
                        framebufferStencilSampleCounts: api::VK_SAMPLE_COUNT_1_BIT
                            | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
                        framebufferNoAttachmentsSampleCounts: api::VK_SAMPLE_COUNT_1_BIT
                            | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
                        maxColorAttachments: 4,
                        sampledImageColorSampleCounts: api::VK_SAMPLE_COUNT_1_BIT
                            | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
                        sampledImageIntegerSampleCounts: api::VK_SAMPLE_COUNT_1_BIT
                            | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
                        sampledImageDepthSampleCounts: api::VK_SAMPLE_COUNT_1_BIT
                            | api::VK_SAMPLE_COUNT_4_BIT, // FIXME: update to correct value
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
                        nonCoherentAtomSize: 1, //TODO: check if this is correct
                    },
                    sparseProperties: api::VkPhysicalDeviceSparseProperties {
                        residencyStandard2DBlockShape: api::VK_FALSE,
                        residencyStandard2DMultisampleBlockShape: api::VK_FALSE,
                        residencyStandard3DBlockShape: api::VK_FALSE,
                        residencyAlignedMipSize: api::VK_FALSE,
                        residencyNonResidentStrict: api::VK_FALSE,
                    },
                },
                features: Features::new(),
                system_memory_size,
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
            true,
            &SharedHandle::from(instance)
                .physical_device
                .allowed_extensions,
        ),
        None => get_proc_address(name, false, &Extensions::create_empty()),
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
        Some(slice::from_raw_parts_mut(
            api_values,
            *api_value_count as usize,
        ))
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
    layer_name: *const ::std::os::raw::c_char,
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
    let instance = SharedHandle::from(instance);
    enumerate_helper(
        physical_device_count,
        physical_devices,
        iter::once(*instance.physical_device.get_handle()),
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
    let physical_device = SharedHandle::from(physical_device);
    *properties = physical_device.properties;
}

unsafe fn get_physical_device_queue_family_properties(
    _physical_device: SharedHandle<api::VkPhysicalDevice>,
    queue_family_properties: &mut api::VkQueueFamilyProperties2,
    queue_count: u32,
) {
    parse_next_chain_mut!{
        queue_family_properties as *mut api::VkQueueFamilyProperties2,
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
                SharedHandle::from(physical_device),
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
    _device: api::VkDevice,
    _pName: *const ::std::os::raw::c_char,
) -> api::PFN_vkVoidFunction {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateDevice(
    physical_device: api::VkPhysicalDevice,
    create_info: *const api::VkDeviceCreateInfo,
    _allocator: *const api::VkAllocationCallbacks,
    device: *mut api::VkDevice,
) -> api::VkResult {
    *device = Handle::null();
    match Device::new(SharedHandle::from(physical_device), create_info) {
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
    layer_name: *const ::std::os::raw::c_char,
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
    layer_name: *const ::std::os::raw::c_char,
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
    _device: api::VkDevice,
    _queueFamilyIndex: u32,
    _queueIndex: u32,
    _pQueue: *mut api::VkQueue,
) {
    unimplemented!()
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
    _pAllocateInfo: *const api::VkMemoryAllocateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pMemory: *mut api::VkDeviceMemory,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkFreeMemory(
    _device: api::VkDevice,
    _memory: api::VkDeviceMemory,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkMapMemory(
    _device: api::VkDevice,
    _memory: api::VkDeviceMemory,
    _offset: api::VkDeviceSize,
    _size: api::VkDeviceSize,
    _flags: api::VkMemoryMapFlags,
    _ppData: *mut *mut ::std::os::raw::c_void,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkUnmapMemory(_device: api::VkDevice, _memory: api::VkDeviceMemory) {
    unimplemented!()
}

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
    _device: api::VkDevice,
    _buffer: api::VkBuffer,
    _memory: api::VkDeviceMemory,
    _memoryOffset: api::VkDeviceSize,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkBindImageMemory(
    _device: api::VkDevice,
    _image: api::VkImage,
    _memory: api::VkDeviceMemory,
    _memoryOffset: api::VkDeviceSize,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetBufferMemoryRequirements(
    _device: api::VkDevice,
    _buffer: api::VkBuffer,
    _pMemoryRequirements: *mut api::VkMemoryRequirements,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetImageMemoryRequirements(
    _device: api::VkDevice,
    _image: api::VkImage,
    _pMemoryRequirements: *mut api::VkMemoryRequirements,
) {
    unimplemented!()
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
    _pData: *mut ::std::os::raw::c_void,
    _stride: api::VkDeviceSize,
    _flags: api::VkQueryResultFlags,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateBuffer(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkBufferCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pBuffer: *mut api::VkBuffer,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyBuffer(
    _device: api::VkDevice,
    _buffer: api::VkBuffer,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
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
    _pCreateInfo: *const api::VkImageCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pImage: *mut api::VkImage,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyImage(
    _device: api::VkDevice,
    _image: api::VkImage,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
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
    _pCreateInfo: *const api::VkImageViewCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pView: *mut api::VkImageView,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyImageView(
    _device: api::VkDevice,
    _imageView: api::VkImageView,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateShaderModule(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkShaderModuleCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pShaderModule: *mut api::VkShaderModule,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyShaderModule(
    _device: api::VkDevice,
    _shaderModule: api::VkShaderModule,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
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
    _pData: *mut ::std::os::raw::c_void,
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
    _device: api::VkDevice,
    _pipelineCache: api::VkPipelineCache,
    _createInfoCount: u32,
    _pCreateInfos: *const api::VkGraphicsPipelineCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pPipelines: *mut api::VkPipeline,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateComputePipelines(
    _device: api::VkDevice,
    _pipelineCache: api::VkPipelineCache,
    _createInfoCount: u32,
    _pCreateInfos: *const api::VkComputePipelineCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pPipelines: *mut api::VkPipeline,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyPipeline(
    _device: api::VkDevice,
    _pipeline: api::VkPipeline,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreatePipelineLayout(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkPipelineLayoutCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pPipelineLayout: *mut api::VkPipelineLayout,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyPipelineLayout(
    _device: api::VkDevice,
    _pipelineLayout: api::VkPipelineLayout,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateSampler(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkSamplerCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pSampler: *mut api::VkSampler,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroySampler(
    _device: api::VkDevice,
    _sampler: api::VkSampler,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateDescriptorSetLayout(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkDescriptorSetLayoutCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pSetLayout: *mut api::VkDescriptorSetLayout,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyDescriptorSetLayout(
    _device: api::VkDevice,
    _descriptorSetLayout: api::VkDescriptorSetLayout,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateDescriptorPool(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkDescriptorPoolCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pDescriptorPool: *mut api::VkDescriptorPool,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyDescriptorPool(
    _device: api::VkDevice,
    _descriptorPool: api::VkDescriptorPool,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkResetDescriptorPool(
    _device: api::VkDevice,
    _descriptorPool: api::VkDescriptorPool,
    _flags: api::VkDescriptorPoolResetFlags,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkAllocateDescriptorSets(
    _device: api::VkDevice,
    _pAllocateInfo: *const api::VkDescriptorSetAllocateInfo,
    _pDescriptorSets: *mut api::VkDescriptorSet,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkFreeDescriptorSets(
    _device: api::VkDevice,
    _descriptorPool: api::VkDescriptorPool,
    _descriptorSetCount: u32,
    _pDescriptorSets: *const api::VkDescriptorSet,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkUpdateDescriptorSets(
    _device: api::VkDevice,
    _descriptorWriteCount: u32,
    _pDescriptorWrites: *const api::VkWriteDescriptorSet,
    _descriptorCopyCount: u32,
    _pDescriptorCopies: *const api::VkCopyDescriptorSet,
) {
    unimplemented!()
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
    _pCreateInfo: *const api::VkRenderPassCreateInfo,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pRenderPass: *mut api::VkRenderPass,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyRenderPass(
    _device: api::VkDevice,
    _renderPass: api::VkRenderPass,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
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
    _pData: *const ::std::os::raw::c_void,
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
    _pValues: *const ::std::os::raw::c_void,
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
    _bindInfoCount: u32,
    _pBindInfos: *const api::VkBindBufferMemoryInfo,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkBindImageMemory2(
    _device: api::VkDevice,
    _bindInfoCount: u32,
    _pBindInfos: *const api::VkBindImageMemoryInfo,
) -> api::VkResult {
    unimplemented!()
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
    _instance: api::VkInstance,
    _pPhysicalDeviceGroupCount: *mut u32,
    _pPhysicalDeviceGroupProperties: *mut api::VkPhysicalDeviceGroupProperties,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetImageMemoryRequirements2(
    _device: api::VkDevice,
    _pInfo: *const api::VkImageMemoryRequirementsInfo2,
    _pMemoryRequirements: *mut api::VkMemoryRequirements2,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetBufferMemoryRequirements2(
    _device: api::VkDevice,
    _pInfo: *const api::VkBufferMemoryRequirementsInfo2,
    _pMemoryRequirements: *mut api::VkMemoryRequirements2,
) {
    unimplemented!()
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
    parse_next_chain_mut!{
        features,
        root = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2,
        sampler_ycbcr_conversion_features: api::VkPhysicalDeviceSamplerYcbcrConversionFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES,
        physical_device_16bit_storage_features: api::VkPhysicalDevice16BitStorageFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES,
        variable_pointer_features: api::VkPhysicalDeviceVariablePointerFeatures = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES,
    }
    SharedHandle::from(physical_device)
        .features
        .export_feature_set(&mut *features);
    if !sampler_ycbcr_conversion_features.is_null() {
        SharedHandle::from(physical_device)
            .features
            .export_feature_set(&mut *sampler_ycbcr_conversion_features);
    }
    if !physical_device_16bit_storage_features.is_null() {
        SharedHandle::from(physical_device)
            .features
            .export_feature_set(&mut *physical_device_16bit_storage_features);
    }
    if !variable_pointer_features.is_null() {
        SharedHandle::from(physical_device)
            .features
            .export_feature_set(&mut *variable_pointer_features);
        //FIXME: finish
        let ref mut variable_pointer_features = *variable_pointer_features;
        variable_pointer_features.variablePointersStorageBuffer = api::VK_TRUE;
        variable_pointer_features.variablePointers = api::VK_TRUE;
    }
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceProperties2(
    physical_device: api::VkPhysicalDevice,
    properties: *mut api::VkPhysicalDeviceProperties2,
) {
    parse_next_chain_mut!{
        properties,
        root = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2,
    }
    let ref mut properties = *properties;
    let physical_device = SharedHandle::from(physical_device);
    properties.properties = physical_device.properties;
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceFormatProperties2(
    _physical_device: api::VkPhysicalDevice,
    format: api::VkFormat,
    format_properties: *mut api::VkFormatProperties2,
) {
    parse_next_chain_mut!{
        format_properties,
        root = api::VK_STRUCTURE_TYPE_FORMAT_PROPERTIES_2,
    }
    let ref mut format_properties = *format_properties;
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
                SharedHandle::from(physical_device),
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
    let physical_device = SharedHandle::from(physical_device);
    parse_next_chain_mut!{
        memory_properties,
        root = api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MEMORY_PROPERTIES_2,
    }
    let ref mut memory_properties = *memory_properties;
    let mut properties: api::VkPhysicalDeviceMemoryProperties = mem::zeroed();
    properties.memoryTypeCount = 1;
    properties.memoryTypes[0] = api::VkMemoryType {
        propertyFlags: api::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT
            | api::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
            | api::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT
            | api::VK_MEMORY_PROPERTY_HOST_CACHED_BIT,
        heapIndex: 0,
    };
    properties.memoryHeapCount = 1;
    properties.memoryHeaps[0] = api::VkMemoryHeap {
        size: physical_device.system_memory_size * 7 / 8,
        flags: api::VK_MEMORY_HEAP_DEVICE_LOCAL_BIT,
    };
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
    _device: api::VkDevice,
    _pQueueInfo: *const api::VkDeviceQueueInfo2,
    _pQueue: *mut api::VkQueue,
) {
    unimplemented!()
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
    _pData: *const ::std::os::raw::c_void,
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
    _surface: api::VkSurfaceKHR,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
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
    _physicalDevice: api::VkPhysicalDevice,
    _surface: api::VkSurfaceKHR,
    _pSurfaceCapabilities: *mut api::VkSurfaceCapabilitiesKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceFormatsKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _surface: api::VkSurfaceKHR,
    _pSurfaceFormatCount: *mut u32,
    _pSurfaceFormats: *mut api::VkSurfaceFormatKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfacePresentModesKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _surface: api::VkSurfaceKHR,
    _pPresentModeCount: *mut u32,
    _pPresentModes: *mut api::VkPresentModeKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateSwapchainKHR(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkSwapchainCreateInfoKHR,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pSwapchain: *mut api::VkSwapchainKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroySwapchainKHR(
    _device: api::VkDevice,
    _swapchain: api::VkSwapchainKHR,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
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
pub unsafe extern "system" fn vkGetPhysicalDeviceDisplayPropertiesKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkDisplayPropertiesKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceDisplayPlanePropertiesKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkDisplayPlanePropertiesKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetDisplayPlaneSupportedDisplaysKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _planeIndex: u32,
    _pDisplayCount: *mut u32,
    _pDisplays: *mut api::VkDisplayKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetDisplayModePropertiesKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _display: api::VkDisplayKHR,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkDisplayModePropertiesKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
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
pub unsafe extern "system" fn vkGetDisplayPlaneCapabilitiesKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _mode: api::VkDisplayModeKHR,
    _planeIndex: u32,
    _pCapabilities: *mut api::VkDisplayPlaneCapabilitiesKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateDisplayPlaneSurfaceKHR(
    _instance: api::VkInstance,
    _pCreateInfo: *const api::VkDisplaySurfaceCreateInfoKHR,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pSurface: *mut api::VkSurfaceKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
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
pub unsafe extern "system" fn vkGetMemoryFdKHR(
    _device: api::VkDevice,
    _pGetFdInfo: *const api::VkMemoryGetFdInfoKHR,
    _pFd: *mut ::std::os::raw::c_int,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetMemoryFdPropertiesKHR(
    _device: api::VkDevice,
    _handleType: api::VkExternalMemoryHandleTypeFlagBits,
    _fd: ::std::os::raw::c_int,
    _pMemoryFdProperties: *mut api::VkMemoryFdPropertiesKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkImportSemaphoreFdKHR(
    _device: api::VkDevice,
    _pImportSemaphoreFdInfo: *const api::VkImportSemaphoreFdInfoKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetSemaphoreFdKHR(
    _device: api::VkDevice,
    _pGetFdInfo: *const api::VkSemaphoreGetFdInfoKHR,
    _pFd: *mut ::std::os::raw::c_int,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
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
pub unsafe extern "system" fn vkCmdPushDescriptorSetWithTemplateKHR(
    _commandBuffer: api::VkCommandBuffer,
    _descriptorUpdateTemplate: api::VkDescriptorUpdateTemplate,
    _layout: api::VkPipelineLayout,
    _set: u32,
    _pData: *const ::std::os::raw::c_void,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateRenderPass2KHR(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkRenderPassCreateInfo2KHR,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pRenderPass: *mut api::VkRenderPass,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdBeginRenderPass2KHR(
    _commandBuffer: api::VkCommandBuffer,
    _pRenderPassBegin: *const api::VkRenderPassBeginInfo,
    _pSubpassBeginInfo: *const api::VkSubpassBeginInfoKHR,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdNextSubpass2KHR(
    _commandBuffer: api::VkCommandBuffer,
    _pSubpassBeginInfo: *const api::VkSubpassBeginInfoKHR,
    _pSubpassEndInfo: *const api::VkSubpassEndInfoKHR,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdEndRenderPass2KHR(
    _commandBuffer: api::VkCommandBuffer,
    _pSubpassEndInfo: *const api::VkSubpassEndInfoKHR,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetSwapchainStatusKHR(
    _device: api::VkDevice,
    _swapchain: api::VkSwapchainKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkImportFenceFdKHR(
    _device: api::VkDevice,
    _pImportFenceFdInfo: *const api::VkImportFenceFdInfoKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetFenceFdKHR(
    _device: api::VkDevice,
    _pGetFdInfo: *const api::VkFenceGetFdInfoKHR,
    _pFd: *mut ::std::os::raw::c_int,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceCapabilities2KHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pSurfaceInfo: *const api::VkPhysicalDeviceSurfaceInfo2KHR,
    _pSurfaceCapabilities: *mut api::VkSurfaceCapabilities2KHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceFormats2KHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pSurfaceInfo: *const api::VkPhysicalDeviceSurfaceInfo2KHR,
    _pSurfaceFormatCount: *mut u32,
    _pSurfaceFormats: *mut api::VkSurfaceFormat2KHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceDisplayProperties2KHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkDisplayProperties2KHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceDisplayPlaneProperties2KHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkDisplayPlaneProperties2KHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetDisplayModeProperties2KHR(
    _physicalDevice: api::VkPhysicalDevice,
    _display: api::VkDisplayKHR,
    _pPropertyCount: *mut u32,
    _pProperties: *mut api::VkDisplayModeProperties2KHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetDisplayPlaneCapabilities2KHR(
    _physicalDevice: api::VkPhysicalDevice,
    _pDisplayPlaneInfo: *const api::VkDisplayPlaneInfo2KHR,
    _pCapabilities: *mut api::VkDisplayPlaneCapabilities2KHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
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
pub unsafe extern "system" fn vkCreateDebugReportCallbackEXT(
    _instance: api::VkInstance,
    _pCreateInfo: *const api::VkDebugReportCallbackCreateInfoEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pCallback: *mut api::VkDebugReportCallbackEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyDebugReportCallbackEXT(
    _instance: api::VkInstance,
    _callback: api::VkDebugReportCallbackEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDebugReportMessageEXT(
    _instance: api::VkInstance,
    _flags: api::VkDebugReportFlagsEXT,
    _objectType: api::VkDebugReportObjectTypeEXT,
    _object: u64,
    _location: usize,
    _messageCode: i32,
    _pLayerPrefix: *const ::std::os::raw::c_char,
    _pMessage: *const ::std::os::raw::c_char,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDebugMarkerSetObjectTagEXT(
    _device: api::VkDevice,
    _pTagInfo: *const api::VkDebugMarkerObjectTagInfoEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDebugMarkerSetObjectNameEXT(
    _device: api::VkDevice,
    _pNameInfo: *const api::VkDebugMarkerObjectNameInfoEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdDebugMarkerBeginEXT(
    _commandBuffer: api::VkCommandBuffer,
    _pMarkerInfo: *const api::VkDebugMarkerMarkerInfoEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdDebugMarkerEndEXT(_commandBuffer: api::VkCommandBuffer) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdDebugMarkerInsertEXT(
    _commandBuffer: api::VkCommandBuffer,
    _pMarkerInfo: *const api::VkDebugMarkerMarkerInfoEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
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
pub unsafe extern "system" fn vkGetShaderInfoAMD(
    _device: api::VkDevice,
    _pipeline: api::VkPipeline,
    _shaderStage: api::VkShaderStageFlagBits,
    _infoType: api::VkShaderInfoTypeAMD,
    _pInfoSize: *mut usize,
    _pInfo: *mut ::std::os::raw::c_void,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
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
pub unsafe extern "system" fn vkCmdBeginConditionalRenderingEXT(
    _commandBuffer: api::VkCommandBuffer,
    _pConditionalRenderingBegin: *const api::VkConditionalRenderingBeginInfoEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdEndConditionalRenderingEXT(
    _commandBuffer: api::VkCommandBuffer,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetViewportWScalingNV(
    _commandBuffer: api::VkCommandBuffer,
    _firstViewport: u32,
    _viewportCount: u32,
    _pViewportWScalings: *const api::VkViewportWScalingNV,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkReleaseDisplayEXT(
    _physicalDevice: api::VkPhysicalDevice,
    _display: api::VkDisplayKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceCapabilities2EXT(
    _physicalDevice: api::VkPhysicalDevice,
    _surface: api::VkSurfaceKHR,
    _pSurfaceCapabilities: *mut api::VkSurfaceCapabilities2EXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDisplayPowerControlEXT(
    _device: api::VkDevice,
    _display: api::VkDisplayKHR,
    _pDisplayPowerInfo: *const api::VkDisplayPowerInfoEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkRegisterDeviceEventEXT(
    _device: api::VkDevice,
    _pDeviceEventInfo: *const api::VkDeviceEventInfoEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pFence: *mut api::VkFence,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
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
pub unsafe extern "system" fn vkGetSwapchainCounterEXT(
    _device: api::VkDevice,
    _swapchain: api::VkSwapchainKHR,
    _counter: api::VkSurfaceCounterFlagBitsEXT,
    _pCounterValue: *mut u64,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetRefreshCycleDurationGOOGLE(
    _device: api::VkDevice,
    _swapchain: api::VkSwapchainKHR,
    _pDisplayTimingProperties: *mut api::VkRefreshCycleDurationGOOGLE,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPastPresentationTimingGOOGLE(
    _device: api::VkDevice,
    _swapchain: api::VkSwapchainKHR,
    _pPresentationTimingCount: *mut u32,
    _pPresentationTimings: *mut api::VkPastPresentationTimingGOOGLE,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetDiscardRectangleEXT(
    _commandBuffer: api::VkCommandBuffer,
    _firstDiscardRectangle: u32,
    _discardRectangleCount: u32,
    _pDiscardRectangles: *const api::VkRect2D,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkSetHdrMetadataEXT(
    _device: api::VkDevice,
    _swapchainCount: u32,
    _pSwapchains: *const api::VkSwapchainKHR,
    _pMetadata: *const api::VkHdrMetadataEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkSetDebugUtilsObjectNameEXT(
    _device: api::VkDevice,
    _pNameInfo: *const api::VkDebugUtilsObjectNameInfoEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkSetDebugUtilsObjectTagEXT(
    _device: api::VkDevice,
    _pTagInfo: *const api::VkDebugUtilsObjectTagInfoEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkQueueBeginDebugUtilsLabelEXT(
    _queue: api::VkQueue,
    _pLabelInfo: *const api::VkDebugUtilsLabelEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkQueueEndDebugUtilsLabelEXT(_queue: api::VkQueue) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkQueueInsertDebugUtilsLabelEXT(
    _queue: api::VkQueue,
    _pLabelInfo: *const api::VkDebugUtilsLabelEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdBeginDebugUtilsLabelEXT(
    _commandBuffer: api::VkCommandBuffer,
    _pLabelInfo: *const api::VkDebugUtilsLabelEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdEndDebugUtilsLabelEXT(_commandBuffer: api::VkCommandBuffer) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdInsertDebugUtilsLabelEXT(
    _commandBuffer: api::VkCommandBuffer,
    _pLabelInfo: *const api::VkDebugUtilsLabelEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateDebugUtilsMessengerEXT(
    _instance: api::VkInstance,
    _pCreateInfo: *const api::VkDebugUtilsMessengerCreateInfoEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pMessenger: *mut api::VkDebugUtilsMessengerEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyDebugUtilsMessengerEXT(
    _instance: api::VkInstance,
    _messenger: api::VkDebugUtilsMessengerEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkSubmitDebugUtilsMessageEXT(
    _instance: api::VkInstance,
    _messageSeverity: api::VkDebugUtilsMessageSeverityFlagBitsEXT,
    _messageTypes: api::VkDebugUtilsMessageTypeFlagsEXT,
    _pCallbackData: *const api::VkDebugUtilsMessengerCallbackDataEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetSampleLocationsEXT(
    _commandBuffer: api::VkCommandBuffer,
    _pSampleLocationsInfo: *const api::VkSampleLocationsInfoEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceMultisamplePropertiesEXT(
    _physicalDevice: api::VkPhysicalDevice,
    _samples: api::VkSampleCountFlagBits,
    _pMultisampleProperties: *mut api::VkMultisamplePropertiesEXT,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateValidationCacheEXT(
    _device: api::VkDevice,
    _pCreateInfo: *const api::VkValidationCacheCreateInfoEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pValidationCache: *mut api::VkValidationCacheEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkDestroyValidationCacheEXT(
    _device: api::VkDevice,
    _validationCache: api::VkValidationCacheEXT,
    _pAllocator: *const api::VkAllocationCallbacks,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkMergeValidationCachesEXT(
    _device: api::VkDevice,
    _dstCache: api::VkValidationCacheEXT,
    _srcCacheCount: u32,
    _pSrcCaches: *const api::VkValidationCacheEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetValidationCacheDataEXT(
    _device: api::VkDevice,
    _validationCache: api::VkValidationCacheEXT,
    _pDataSize: *mut usize,
    _pData: *mut ::std::os::raw::c_void,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdBindShadingRateImageNV(
    _commandBuffer: api::VkCommandBuffer,
    _imageView: api::VkImageView,
    _imageLayout: api::VkImageLayout,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetViewportShadingRatePaletteNV(
    _commandBuffer: api::VkCommandBuffer,
    _firstViewport: u32,
    _viewportCount: u32,
    _pShadingRatePalettes: *const api::VkShadingRatePaletteNV,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetCoarseSampleOrderNV(
    _commandBuffer: api::VkCommandBuffer,
    _sampleOrderType: api::VkCoarseSampleOrderTypeNV,
    _customSampleOrderCount: u32,
    _pCustomSampleOrders: *const api::VkCoarseSampleOrderCustomNV,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetMemoryHostPointerPropertiesEXT(
    _device: api::VkDevice,
    _handleType: api::VkExternalMemoryHandleTypeFlagBits,
    _pHostPointer: *const ::std::os::raw::c_void,
    _pMemoryHostPointerProperties: *mut api::VkMemoryHostPointerPropertiesEXT,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
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
pub unsafe extern "system" fn vkCmdDrawMeshTasksNV(
    _commandBuffer: api::VkCommandBuffer,
    _taskCount: u32,
    _firstTask: u32,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
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
pub unsafe extern "system" fn vkCmdSetExclusiveScissorNV(
    _commandBuffer: api::VkCommandBuffer,
    _firstExclusiveScissor: u32,
    _exclusiveScissorCount: u32,
    _pExclusiveScissors: *const api::VkRect2D,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCmdSetCheckpointNV(
    _commandBuffer: api::VkCommandBuffer,
    _pCheckpointMarker: *const ::std::os::raw::c_void,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetQueueCheckpointDataNV(
    _queue: api::VkQueue,
    _pCheckpointDataCount: *mut u32,
    _pCheckpointData: *mut api::VkCheckpointDataNV,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkCreateXcbSurfaceKHR(
    _instance: api::VkInstance,
    _pCreateInfo: *const api::VkXcbSurfaceCreateInfoKHR,
    _pAllocator: *const api::VkAllocationCallbacks,
    _pSurface: *mut api::VkSurfaceKHR,
) -> api::VkResult {
    unimplemented!()
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn vkGetPhysicalDeviceXcbPresentationSupportKHR(
    _physicalDevice: api::VkPhysicalDevice,
    _queueFamilyIndex: u32,
    _connection: *mut xcb::ffi::xcb_connection_t,
    _visual_id: xcb::ffi::xcb_visualid_t,
) -> api::VkBool32 {
    unimplemented!()
}
