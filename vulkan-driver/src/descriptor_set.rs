// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2019 Jacob Lifshay

use crate::api;
use crate::buffer::BufferSlice;
use crate::handle::OwnedHandle;
use crate::handle::SharedHandle;
use crate::image;
use crate::util;
use std::ops;

#[derive(Debug)]
pub enum DescriptorLayout {
    Sampler {
        count: usize,
        immutable_samplers: Option<Vec<SharedHandle<api::VkSampler>>>,
    },
    CombinedImageSampler {
        count: usize,
        immutable_samplers: Option<Vec<SharedHandle<api::VkSampler>>>,
    },
    SampledImage {
        count: usize,
    },
    StorageImage {
        count: usize,
    },
    UniformTexelBuffer {
        count: usize,
    },
    StorageTexelBuffer {
        count: usize,
    },
    UniformBuffer {
        count: usize,
    },
    StorageBuffer {
        count: usize,
    },
    UniformBufferDynamic {
        count: usize,
    },
    StorageBufferDynamic {
        count: usize,
    },
    InputAttachment {
        count: usize,
    },
}

impl DescriptorLayout {
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        match *self {
            DescriptorLayout::Sampler { count, .. } => count,
            DescriptorLayout::CombinedImageSampler { count, .. } => count,
            DescriptorLayout::SampledImage { count } => count,
            DescriptorLayout::StorageImage { count } => count,
            DescriptorLayout::UniformTexelBuffer { count } => count,
            DescriptorLayout::StorageTexelBuffer { count } => count,
            DescriptorLayout::UniformBuffer { count } => count,
            DescriptorLayout::StorageBuffer { count } => count,
            DescriptorLayout::UniformBufferDynamic { count } => count,
            DescriptorLayout::StorageBufferDynamic { count } => count,
            DescriptorLayout::InputAttachment { count } => count,
        }
    }
    #[allow(dead_code)]
    pub fn immutable_samplers(&self) -> Option<&[SharedHandle<api::VkSampler>]> {
        match self {
            DescriptorLayout::Sampler {
                immutable_samplers: Some(immutable_samplers),
                ..
            } => Some(immutable_samplers),
            DescriptorLayout::CombinedImageSampler {
                immutable_samplers: Some(immutable_samplers),
                ..
            } => Some(immutable_samplers),
            _ => None,
        }
    }
    pub fn descriptor_type(&self) -> api::VkDescriptorType {
        match self {
            DescriptorLayout::Sampler { .. } => api::VK_DESCRIPTOR_TYPE_SAMPLER,
            DescriptorLayout::CombinedImageSampler { .. } => {
                api::VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER
            }
            DescriptorLayout::SampledImage { .. } => api::VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
            DescriptorLayout::StorageImage { .. } => api::VK_DESCRIPTOR_TYPE_STORAGE_IMAGE,
            DescriptorLayout::UniformTexelBuffer { .. } => {
                api::VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER
            }
            DescriptorLayout::StorageTexelBuffer { .. } => {
                api::VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER
            }
            DescriptorLayout::UniformBuffer { .. } => api::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
            DescriptorLayout::StorageBuffer { .. } => api::VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
            DescriptorLayout::UniformBufferDynamic { .. } => {
                api::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC
            }
            DescriptorLayout::StorageBufferDynamic { .. } => {
                api::VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC
            }
            DescriptorLayout::InputAttachment { .. } => api::VK_DESCRIPTOR_TYPE_INPUT_ATTACHMENT,
        }
    }
    pub unsafe fn from(v: &api::VkDescriptorSetLayoutBinding) -> Self {
        let get_immutable_samplers = || {
            if v.pImmutableSamplers.is_null() {
                None
            } else {
                let immutable_samplers =
                    util::to_slice(v.pImmutableSamplers, v.descriptorCount as usize);
                Some(
                    immutable_samplers
                        .iter()
                        .map(|sampler| SharedHandle::from(*sampler).unwrap())
                        .collect(),
                )
            }
        };
        match v.descriptorType {
            api::VK_DESCRIPTOR_TYPE_SAMPLER => DescriptorLayout::Sampler {
                count: v.descriptorCount as usize,
                immutable_samplers: get_immutable_samplers(),
            },
            api::VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER => {
                DescriptorLayout::CombinedImageSampler {
                    count: v.descriptorCount as usize,
                    immutable_samplers: get_immutable_samplers(),
                }
            }
            api::VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE => DescriptorLayout::SampledImage {
                count: v.descriptorCount as usize,
            },
            api::VK_DESCRIPTOR_TYPE_STORAGE_IMAGE => DescriptorLayout::StorageImage {
                count: v.descriptorCount as usize,
            },
            api::VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER => DescriptorLayout::UniformTexelBuffer {
                count: v.descriptorCount as usize,
            },
            api::VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER => DescriptorLayout::StorageTexelBuffer {
                count: v.descriptorCount as usize,
            },
            api::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER => DescriptorLayout::UniformBuffer {
                count: v.descriptorCount as usize,
            },
            api::VK_DESCRIPTOR_TYPE_STORAGE_BUFFER => DescriptorLayout::StorageBuffer {
                count: v.descriptorCount as usize,
            },
            api::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC => {
                DescriptorLayout::UniformBufferDynamic {
                    count: v.descriptorCount as usize,
                }
            }
            api::VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC => {
                DescriptorLayout::StorageBufferDynamic {
                    count: v.descriptorCount as usize,
                }
            }
            api::VK_DESCRIPTOR_TYPE_INPUT_ATTACHMENT => DescriptorLayout::InputAttachment {
                count: v.descriptorCount as usize,
            },
            _ => unreachable!("invalid VkDescriptorType: {}", v.descriptorType),
        }
    }
}

#[derive(Debug)]
pub struct DescriptorSetLayout {
    pub bindings: Vec<Option<DescriptorLayout>>,
}

#[derive(Debug)]
pub struct DescriptorPool {
    descriptor_sets: Vec<OwnedHandle<api::VkDescriptorSet>>,
}

impl DescriptorPool {
    pub fn new() -> Self {
        Self {
            descriptor_sets: Vec::new(),
        }
    }
    pub fn reset(&mut self) {
        self.descriptor_sets.clear()
    }
    pub unsafe fn allocate<I: IntoIterator<Item = DescriptorSet>>(
        &mut self,
        descriptor_sets: I,
        output_descriptor_sets: &mut [api::VkDescriptorSet],
    ) {
        let start_index = self.descriptor_sets.len();
        self.descriptor_sets
            .extend(descriptor_sets.into_iter().map(OwnedHandle::new));
        assert_eq!(
            self.descriptor_sets[start_index..].len(),
            output_descriptor_sets.len()
        );
        for (output_descriptor_set, descriptor_set) in output_descriptor_sets
            .iter_mut()
            .zip(self.descriptor_sets[start_index..].iter())
        {
            *output_descriptor_set = descriptor_set.get_handle();
        }
    }
    pub unsafe fn free(&mut self, descriptor_sets: &[api::VkDescriptorSet]) {
        self.descriptor_sets
            .retain(|descriptor_set| !descriptor_sets.contains(&descriptor_set.get_handle()))
    }
}

#[derive(Debug)]
pub struct DescriptorImage(SharedHandle<api::VkImageView>, image::Tiling);

impl DescriptorImage {
    pub unsafe fn from(v: &api::VkDescriptorImageInfo) -> Self {
        let image_view = SharedHandle::from(v.imageView).unwrap();
        let tiling = image_view.image.properties.get_tiling(v.imageLayout);
        DescriptorImage(image_view, tiling)
    }
}

#[derive(Debug)]
pub struct DescriptorCombinedImageSampler {
    image: Option<DescriptorImage>,
    sampler: Option<SharedHandle<api::VkSampler>>,
}

#[derive(Debug)]
pub enum Descriptor {
    Sampler {
        elements: Vec<Option<SharedHandle<api::VkSampler>>>,
        immutable_samplers: bool,
    },
    CombinedImageSampler {
        elements: Vec<DescriptorCombinedImageSampler>,
        immutable_samplers: bool,
    },
    SampledImage {
        elements: Vec<Option<DescriptorImage>>,
    },
    StorageImage {
        elements: Vec<Option<DescriptorImage>>,
    },
    UniformTexelBuffer {
        elements: Vec<Option<SharedHandle<api::VkBufferView>>>,
    },
    StorageTexelBuffer {
        elements: Vec<Option<SharedHandle<api::VkBufferView>>>,
    },
    UniformBuffer {
        elements: Vec<Option<BufferSlice>>,
    },
    StorageBuffer {
        elements: Vec<Option<BufferSlice>>,
    },
    UniformBufferDynamic {
        elements: Vec<Option<BufferSlice>>,
    },
    StorageBufferDynamic {
        elements: Vec<Option<BufferSlice>>,
    },
    InputAttachment {
        elements: Vec<Option<DescriptorImage>>,
    },
}

#[derive(Copy, Clone)]
pub enum DescriptorWriteArg<'a> {
    Image(&'a [api::VkDescriptorImageInfo]),
    Buffer(&'a [api::VkDescriptorBufferInfo]),
    TexelBuffer(&'a [api::VkBufferView]),
}

impl<'a> DescriptorWriteArg<'a> {
    pub unsafe fn from(v: &'a api::VkWriteDescriptorSet) -> Self {
        match v.descriptorType {
            api::VK_DESCRIPTOR_TYPE_SAMPLER
            | api::VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER
            | api::VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE
            | api::VK_DESCRIPTOR_TYPE_STORAGE_IMAGE
            | api::VK_DESCRIPTOR_TYPE_INPUT_ATTACHMENT => {
                assert!(!v.pImageInfo.is_null());
                DescriptorWriteArg::Image(util::to_slice(v.pImageInfo, v.descriptorCount as usize))
            }
            api::VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER
            | api::VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER => {
                assert!(!v.pTexelBufferView.is_null());
                DescriptorWriteArg::TexelBuffer(util::to_slice(
                    v.pTexelBufferView,
                    v.descriptorCount as usize,
                ))
            }
            api::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER
            | api::VK_DESCRIPTOR_TYPE_STORAGE_BUFFER
            | api::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC
            | api::VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC => {
                assert!(!v.pBufferInfo.is_null());
                DescriptorWriteArg::Buffer(util::to_slice(
                    v.pBufferInfo,
                    v.descriptorCount as usize,
                ))
            }
            _ => unreachable!("invalid VkDescriptorType: {}", v.descriptorType),
        }
    }
    pub fn len(self) -> usize {
        match self {
            DescriptorWriteArg::Image(v) => v.len(),
            DescriptorWriteArg::Buffer(v) => v.len(),
            DescriptorWriteArg::TexelBuffer(v) => v.len(),
        }
    }
    pub fn slice(self, range: ops::Range<usize>) -> Self {
        match self {
            DescriptorWriteArg::Image(v) => DescriptorWriteArg::Image(&v[range]),
            DescriptorWriteArg::Buffer(v) => DescriptorWriteArg::Buffer(&v[range]),
            DescriptorWriteArg::TexelBuffer(v) => DescriptorWriteArg::TexelBuffer(&v[range]),
        }
    }
    pub fn slice_from(self, range: ops::RangeFrom<usize>) -> Self {
        match self {
            DescriptorWriteArg::Image(v) => DescriptorWriteArg::Image(&v[range]),
            DescriptorWriteArg::Buffer(v) => DescriptorWriteArg::Buffer(&v[range]),
            DescriptorWriteArg::TexelBuffer(v) => DescriptorWriteArg::TexelBuffer(&v[range]),
        }
    }
    pub fn slice_to(self, range: ops::RangeTo<usize>) -> Self {
        match self {
            DescriptorWriteArg::Image(v) => DescriptorWriteArg::Image(&v[range]),
            DescriptorWriteArg::Buffer(v) => DescriptorWriteArg::Buffer(&v[range]),
            DescriptorWriteArg::TexelBuffer(v) => DescriptorWriteArg::TexelBuffer(&v[range]),
        }
    }
    pub fn image(self) -> Option<&'a [api::VkDescriptorImageInfo]> {
        match self {
            DescriptorWriteArg::Image(v) => Some(v),
            _ => None,
        }
    }
    pub fn buffer(self) -> Option<&'a [api::VkDescriptorBufferInfo]> {
        match self {
            DescriptorWriteArg::Buffer(v) => Some(v),
            _ => None,
        }
    }
    pub fn texel_buffer(self) -> Option<&'a [api::VkBufferView]> {
        match self {
            DescriptorWriteArg::TexelBuffer(v) => Some(v),
            _ => None,
        }
    }
}

fn descriptor_write_helper<Element, T, F: FnMut(&mut Element, &T)>(
    start_element: usize,
    elements: &mut [Element],
    args: &[T],
    mut f: F,
) {
    for (element, arg) in elements[start_element..][..args.len()]
        .iter_mut()
        .zip(args.iter())
    {
        f(element, arg);
    }
}

impl Descriptor {
    pub fn element_count(&self) -> usize {
        match self {
            Descriptor::Sampler { elements, .. } => elements.len(),
            Descriptor::CombinedImageSampler { elements, .. } => elements.len(),
            Descriptor::SampledImage { elements, .. } => elements.len(),
            Descriptor::StorageImage { elements, .. } => elements.len(),
            Descriptor::UniformTexelBuffer { elements, .. } => elements.len(),
            Descriptor::StorageTexelBuffer { elements, .. } => elements.len(),
            Descriptor::UniformBuffer { elements, .. } => elements.len(),
            Descriptor::StorageBuffer { elements, .. } => elements.len(),
            Descriptor::UniformBufferDynamic { elements, .. } => elements.len(),
            Descriptor::StorageBufferDynamic { elements, .. } => elements.len(),
            Descriptor::InputAttachment { elements, .. } => elements.len(),
        }
    }
    pub fn from(layout: &DescriptorLayout) -> Self {
        match layout {
            DescriptorLayout::Sampler {
                count,
                immutable_samplers,
            } => {
                let mut elements: Vec<_> = (0..*count).map(|_| None).collect();
                let immutable_samplers = if let Some(immutable_samplers) = immutable_samplers {
                    assert_eq!(immutable_samplers.len(), *count);
                    for (element, sampler) in elements.iter_mut().zip(immutable_samplers.iter()) {
                        *element = Some(*sampler);
                    }
                    true
                } else {
                    false
                };
                Descriptor::Sampler {
                    elements,
                    immutable_samplers,
                }
            }
            DescriptorLayout::CombinedImageSampler {
                count,
                immutable_samplers,
            } => {
                let mut elements: Vec<_> = (0..*count)
                    .map(|_| DescriptorCombinedImageSampler {
                        image: None,
                        sampler: None,
                    })
                    .collect();
                let immutable_samplers = if let Some(immutable_samplers) = immutable_samplers {
                    assert_eq!(immutable_samplers.len(), *count);
                    for (element, sampler) in elements.iter_mut().zip(immutable_samplers.iter()) {
                        element.sampler = Some(*sampler);
                    }
                    true
                } else {
                    false
                };
                Descriptor::CombinedImageSampler {
                    elements,
                    immutable_samplers,
                }
            }
            DescriptorLayout::SampledImage { count } => Descriptor::SampledImage {
                elements: (0..*count).map(|_| None).collect(),
            },
            DescriptorLayout::StorageImage { count } => Descriptor::StorageImage {
                elements: (0..*count).map(|_| None).collect(),
            },
            DescriptorLayout::UniformTexelBuffer { count } => Descriptor::UniformTexelBuffer {
                elements: (0..*count).map(|_| None).collect(),
            },
            DescriptorLayout::StorageTexelBuffer { count } => Descriptor::StorageTexelBuffer {
                elements: (0..*count).map(|_| None).collect(),
            },
            DescriptorLayout::UniformBuffer { count } => Descriptor::UniformBuffer {
                elements: (0..*count).map(|_| None).collect(),
            },
            DescriptorLayout::StorageBuffer { count } => Descriptor::StorageBuffer {
                elements: (0..*count).map(|_| None).collect(),
            },
            DescriptorLayout::UniformBufferDynamic { count } => Descriptor::UniformBufferDynamic {
                elements: (0..*count).map(|_| None).collect(),
            },
            DescriptorLayout::StorageBufferDynamic { count } => Descriptor::StorageBufferDynamic {
                elements: (0..*count).map(|_| None).collect(),
            },
            DescriptorLayout::InputAttachment { count } => Descriptor::InputAttachment {
                elements: (0..*count).map(|_| None).collect(),
            },
        }
    }
    pub unsafe fn write(&mut self, start_element: usize, arg: DescriptorWriteArg) {
        assert_eq!(arg.len() + start_element, self.element_count());
        match self {
            Descriptor::Sampler {
                elements,
                immutable_samplers,
            } => descriptor_write_helper(
                start_element,
                elements,
                arg.image().unwrap(),
                |_element, _arg| assert!(!*immutable_samplers),
            ),
            Descriptor::CombinedImageSampler {
                elements,
                immutable_samplers,
            } => descriptor_write_helper(
                start_element,
                elements,
                arg.image().unwrap(),
                |element, arg| {
                    if !*immutable_samplers {
                        element.sampler = Some(SharedHandle::from(arg.sampler).unwrap());
                    }
                    element.image = Some(DescriptorImage::from(arg));
                },
            ),
            Descriptor::SampledImage { elements } => descriptor_write_helper(
                start_element,
                elements,
                arg.image().unwrap(),
                |element, arg| {
                    *element = Some(DescriptorImage::from(arg));
                },
            ),
            Descriptor::StorageImage { elements } => descriptor_write_helper(
                start_element,
                elements,
                arg.image().unwrap(),
                |element, arg| {
                    *element = Some(DescriptorImage::from(arg));
                },
            ),
            Descriptor::UniformTexelBuffer { elements } => unimplemented!(),
            Descriptor::StorageTexelBuffer { elements } => unimplemented!(),
            Descriptor::UniformBuffer { elements } => descriptor_write_helper(
                start_element,
                elements,
                arg.buffer().unwrap(),
                |element, arg| *element = Some(BufferSlice::from(arg)),
            ),
            Descriptor::StorageBuffer { elements } => descriptor_write_helper(
                start_element,
                elements,
                arg.buffer().unwrap(),
                |element, arg| *element = Some(BufferSlice::from(arg)),
            ),
            Descriptor::UniformBufferDynamic { elements } => unimplemented!(),
            Descriptor::StorageBufferDynamic { elements } => unimplemented!(),
            Descriptor::InputAttachment { elements } => descriptor_write_helper(
                start_element,
                elements,
                arg.image().unwrap(),
                |element, arg| {
                    *element = Some(DescriptorImage::from(arg));
                },
            ),
        }
    }
    pub fn descriptor_type(&self) -> api::VkDescriptorType {
        match self {
            Descriptor::Sampler { .. } => api::VK_DESCRIPTOR_TYPE_SAMPLER,
            Descriptor::CombinedImageSampler { .. } => {
                api::VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER
            }
            Descriptor::SampledImage { .. } => api::VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
            Descriptor::StorageImage { .. } => api::VK_DESCRIPTOR_TYPE_STORAGE_IMAGE,
            Descriptor::UniformTexelBuffer { .. } => api::VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER,
            Descriptor::StorageTexelBuffer { .. } => api::VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER,
            Descriptor::UniformBuffer { .. } => api::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
            Descriptor::StorageBuffer { .. } => api::VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
            Descriptor::UniformBufferDynamic { .. } => {
                api::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC
            }
            Descriptor::StorageBufferDynamic { .. } => {
                api::VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC
            }
            Descriptor::InputAttachment { .. } => api::VK_DESCRIPTOR_TYPE_INPUT_ATTACHMENT,
        }
    }
}

#[derive(Debug)]
pub struct DescriptorSet {
    pub bindings: Vec<Option<Descriptor>>,
}
