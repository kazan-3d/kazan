// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::api;
use std::os::raw::c_char;
use std::ptr::null;
use std::slice;

/// like `std::util::to_slice` except that the pointer can be null when the length is zero
pub unsafe fn to_slice<'a, T>(p: *const T, len: usize) -> &'a [T] {
    if len == 0 {
        &[]
    } else {
        assert!(!p.is_null());
        slice::from_raw_parts(p, len)
    }
}

/// like `std::util::to_slice_mut` except that the pointer can be null when the length is zero
pub unsafe fn to_slice_mut<'a, T>(p: *mut T, len: usize) -> &'a mut [T] {
    if len == 0 {
        &mut []
    } else {
        assert!(!p.is_null());
        slice::from_raw_parts_mut(p, len)
    }
}

/// structure types the driver should know about
pub fn is_supported_structure_type(v: api::VkStructureType) -> bool {
    #[cfg(target_os = "linux")]
    {
        #[allow(clippy::single_match)]
        match v {
            api::VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR
            | api::VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR => return true,
            _ => {}
        }
    }
    match v {
        api::VK_STRUCTURE_TYPE_ACQUIRE_NEXT_IMAGE_INFO_KHR
        | api::VK_STRUCTURE_TYPE_APPLICATION_INFO
        | api::VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO
        | api::VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_INFO
        | api::VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO
        | api::VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_INFO
        | api::VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_SWAPCHAIN_INFO_KHR
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
        | api::VK_STRUCTURE_TYPE_DEVICE_GROUP_PRESENT_CAPABILITIES_KHR
        | api::VK_STRUCTURE_TYPE_DEVICE_GROUP_PRESENT_INFO_KHR
        | api::VK_STRUCTURE_TYPE_DEVICE_GROUP_RENDER_PASS_BEGIN_INFO
        | api::VK_STRUCTURE_TYPE_DEVICE_GROUP_SUBMIT_INFO
        | api::VK_STRUCTURE_TYPE_DEVICE_GROUP_SWAPCHAIN_CREATE_INFO_KHR
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
        | api::VK_STRUCTURE_TYPE_IMAGE_SWAPCHAIN_CREATE_INFO_KHR
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
        | api::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SURFACE_INFO_2_KHR
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
        | api::VK_STRUCTURE_TYPE_PRESENT_INFO_KHR
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
        | api::VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES_2_KHR
        | api::VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR
        | api::VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET => true,
        _ => false,
    }
}

pub fn verify_structure_type_is_supported(v: api::VkStructureType) {
    assert!(
        is_supported_structure_type(v),
        "missing structure type in is_supported_structure_type: {:?}",
        v
    );
}

pub unsafe fn parse_next_chain_const(
    root: *const api::VkBaseInStructure,
    expected_root_struct_type: api::VkStructureType,
    expected_child_structs: &[(api::VkStructureType, *mut *const api::VkBaseInStructure)],
) {
    verify_structure_type_is_supported(expected_root_struct_type);
    let root = &*root;
    assert_eq!(root.sType, expected_root_struct_type);
    for &(child_struct_type, child_struct) in expected_child_structs.iter() {
        verify_structure_type_is_supported(child_struct_type);
        *child_struct = null();
    }
    let mut child = root.pNext as *const api::VkBaseInStructure;
    while !child.is_null() {
        let child_ref = &*child;
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

pub unsafe fn parse_next_chain_mut(
    root: *mut api::VkBaseOutStructure,
    expected_root_struct_type: api::VkStructureType,
    expected_child_structs: &[(api::VkStructureType, *mut *mut api::VkBaseOutStructure)],
) {
    parse_next_chain_const(
        root as *const api::VkBaseInStructure,
        expected_root_struct_type,
        &*(expected_child_structs as *const [(u32, *mut *mut api::VkBaseOutStructure)]
            as *const [(u32, *mut *const api::VkBaseInStructure)]),
    )
}

macro_rules! parse_next_chain_const {
    {
        $root:expr,
        root = $root_type:expr,
        $($name:ident: $var_type:ty = $struct_type:expr,)*
    } => {
        $(let mut $name: *const $var_type = ::std::ptr::null();)*
        $crate::util::parse_next_chain_const(
            $root as *const _ as *const $crate::api::VkBaseInStructure,
            $root_type,
            &[$(($struct_type, &mut $name as *mut *const $var_type as *mut *const $crate::api::VkBaseInStructure)),*]
        );
    };
}

macro_rules! parse_next_chain_mut {
    {
        $root:expr,
        root = $root_type:expr,
        $($name:ident: $var_type:ty = $struct_type:expr,)*
    } => {
        $(let mut $name: *mut $var_type = ::std::ptr::null_mut();)*
        $crate::util::parse_next_chain_mut(
            $root as *mut _ as *mut $crate::api::VkBaseOutStructure,
            $root_type,
            &[$(($struct_type, &mut $name as *mut *mut $var_type as *mut *mut $crate::api::VkBaseOutStructure)),*]
        );
    };
}

pub fn copy_str_to_char_array(dest: &mut [c_char], src: &str) {
    #![allow(clippy::int_plus_one)]
    assert!(dest.len() >= src.len() + 1);
    let src = src.as_bytes();
    for i in 0..src.len() {
        dest[i] = src[i] as c_char;
    }
    for v in dest.iter_mut().skip(src.len()) {
        *v = 0;
    }
}
