// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use api;
use handle::{OwnedHandle, SharedHandle};
use std::fmt;
use std::ops::Deref;

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
pub struct ComputePipeline {}

impl GenericPipeline for ComputePipeline {}

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
        unimplemented!()
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
