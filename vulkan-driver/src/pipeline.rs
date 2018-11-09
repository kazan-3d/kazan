// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use api;
use handle::{OwnedHandle, SharedHandle};
use shader_compiler;
use shader_compiler_backend;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fmt;
use std::iter;
use std::ops::Deref;
use util;

#[derive(Debug)]
pub struct PipelineLayout {
    pub push_constants_size: usize,
    pub push_constant_ranges: Vec<api::VkPushConstantRange>,
    pub descriptor_set_layouts: Vec<SharedHandle<api::VkDescriptorSetLayout>>,
}

pub trait GenericPipeline: fmt::Debug + Sync + 'static {}

pub trait GenericPipelineSized: GenericPipeline + Sized {
    type PipelineCreateInfo;
    unsafe fn create(
        device: SharedHandle<api::VkDevice>,
        pipeline_cache: Option<SharedHandle<api::VkPipelineCache>>,
        create_info: &Self::PipelineCreateInfo,
    ) -> Self;
    fn to_pipeline(self) -> Pipeline;
}

#[derive(Debug)]
pub struct ComputePipeline {
    pipeline: shader_compiler::ComputePipeline,
}

impl GenericPipeline for ComputePipeline {}

unsafe fn get_specializations<'a>(
    specializations: *const api::VkSpecializationInfo,
) -> Vec<shader_compiler::Specialization<'a>> {
    if specializations.is_null() {
        return Vec::new();
    }
    let specializations = &*specializations;
    let data = util::to_slice(
        specializations.pData as *const u8,
        specializations.dataSize as usize,
    );
    util::to_slice(
        specializations.pMapEntries,
        specializations.mapEntryCount as usize,
    )
    .iter()
    .map(|map_entry| shader_compiler::Specialization {
        id: map_entry.constantID,
        bytes: &data[map_entry.offset as usize..][..map_entry.size as usize],
    })
    .collect()
}

macro_rules! get_shader_stages {
    {
        $stages:expr,
        $($required_name:ident = $required_stage:ident,)*
        $(#[optional] $optional_name:ident = $optional_stage:ident,)*
    } => {
        let mut shader_stages = HashMap::new();
        for stage in $stages {
            assert!(shader_stages.insert(stage.stage, stage).is_none(), "duplicate stage: {:#X}", stage.stage);
        }
        $(
            let stage = shader_stages
                .remove(&api::$required_stage)
                .expect(concat!("missing stage: ", stringify!($required_stage)));
            let source = SharedHandle::from(stage.module).unwrap();
            let specializations = get_specializations(stage.pSpecializationInfo);
            let $required_name = shader_compiler::ShaderStageCreateInfo {
                code: &source.code,
                entry_point_name: CStr::from_ptr(stage.pName).to_str().unwrap(),
                specializations: &specializations,
            };
        )*
        $(
            let stage = shader_stages
                .remove(&api::$optional_stage);
            let source = stage.as_ref().map(|stage| SharedHandle::from(stage.module).unwrap());
            let specializations = stage.as_ref().map(|stage| get_specializations(stage.pSpecializationInfo)).unwrap_or(Vec::new());
            let $optional_name = match (&stage, &source) {
                (Some(stage), Some(source)) => {
                    Some(shader_compiler::ShaderStageCreateInfo {
                        code: &source.code,
                        entry_point_name: CStr::from_ptr(stage.pName).to_str().unwrap(),
                        specializations: &specializations,
                    })
                },
                _ => None,
            };
        )*
    };
}

fn get_generic_pipeline_options(
    flags: api::VkPipelineCreateFlags,
) -> shader_compiler::GenericPipelineOptions {
    shader_compiler::GenericPipelineOptions {
        optimization_mode: if (flags & api::VK_PIPELINE_CREATE_DISABLE_OPTIMIZATION_BIT) != 0 {
            shader_compiler_backend::OptimizationMode::NoOptimizations
        } else {
            shader_compiler_backend::OptimizationMode::Normal
        },
    }
}

impl GenericPipelineSized for ComputePipeline {
    type PipelineCreateInfo = api::VkComputePipelineCreateInfo;
    unsafe fn create(
        _device: SharedHandle<api::VkDevice>,
        _pipeline_cache: Option<SharedHandle<api::VkPipelineCache>>,
        create_info: &api::VkComputePipelineCreateInfo,
    ) -> Self {
        parse_next_chain_const!{
            create_info,
            root = api::VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO,
        }
        if (create_info.flags & api::VK_PIPELINE_CREATE_VIEW_INDEX_FROM_DEVICE_INDEX_BIT) != 0 {
            unimplemented!();
        }
        get_shader_stages!{
            iter::once(&create_info.stage),
            compute_stage = VK_SHADER_STAGE_COMPUTE_BIT,
        }
        Self {
            pipeline: shader_compiler::ComputePipeline::new(
                &shader_compiler::ComputePipelineOptions {
                    generic_options: get_generic_pipeline_options(create_info.flags),
                },
                compute_stage,
            ),
        }
    }
    fn to_pipeline(self) -> Pipeline {
        Pipeline::Compute(self)
    }
}

#[derive(Debug)]
pub struct GraphicsPipeline {}

impl GenericPipeline for GraphicsPipeline {}

impl GenericPipelineSized for GraphicsPipeline {
    type PipelineCreateInfo = api::VkGraphicsPipelineCreateInfo;
    unsafe fn create(
        _device: SharedHandle<api::VkDevice>,
        _pipeline_cache: Option<SharedHandle<api::VkPipelineCache>>,
        create_info: &api::VkGraphicsPipelineCreateInfo,
    ) -> Self {
        parse_next_chain_const!{
            create_info,
            root = api::VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
        }
        get_shader_stages!{
            util::to_slice(create_info.pStages, create_info.stageCount as usize),
            vertex_stage = VK_SHADER_STAGE_VERTEX_BIT,
            #[optional]
            fragment_stage = VK_SHADER_STAGE_FRAGMENT_BIT,
        }
        unimplemented!()
    }
    fn to_pipeline(self) -> Pipeline {
        Pipeline::Graphics(self)
    }
}

#[derive(Debug)]
pub enum Pipeline {
    Compute(ComputePipeline),
    Graphics(GraphicsPipeline),
}

impl Deref for Pipeline {
    type Target = dyn GenericPipeline;
    fn deref(&self) -> &dyn GenericPipeline {
        match self {
            Pipeline::Compute(v) => v,
            Pipeline::Graphics(v) => v,
        }
    }
}

pub unsafe fn create_pipelines<T: GenericPipelineSized>(
    device: SharedHandle<api::VkDevice>,
    pipeline_cache: Option<SharedHandle<api::VkPipelineCache>>,
    create_infos: &[T::PipelineCreateInfo],
    pipelines: &mut [api::VkPipeline],
) -> api::VkResult {
    assert_eq!(create_infos.len(), pipelines.len());
    for (pipeline, create_info) in pipelines.iter_mut().zip(create_infos.iter()) {
        *pipeline = OwnedHandle::<api::VkPipeline>::new(
            T::create(device, pipeline_cache, create_info).to_pipeline(),
        )
        .take();
    }
    api::VK_SUCCESS
}
