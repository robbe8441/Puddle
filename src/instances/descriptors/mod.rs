use std::sync::Arc;

use ash::vk;

use super::BufferAllocation;

mod descriptor_pool;
mod descriptor_set;

pub use descriptor_pool::*;
pub use descriptor_set::*;

pub use vk::DescriptorPoolSize;

#[allow(unused)]
pub enum DescriptorType {
    UniformBuffer,
    StorageBuffer,
    SampledImage,
    StorageTexelBuffer,
    UniformTexelBuffer,
}

pub struct WriteDescriptorSet<'a> {
    pub buffer_info: Option<&'a [vk::DescriptorBufferInfo]>,
    pub image_info: Option<&'a [vk::DescriptorImageInfo]>,
    pub dst_binding: u32,
    pub descriptor_count: u32,
    pub dst_set: u32,
    pub descriptor_type: vk::DescriptorType,
}

impl Into<vk::DescriptorType> for DescriptorType {
    fn into(self) -> vk::DescriptorType {
        match self {
            Self::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
            Self::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
            Self::SampledImage => vk::DescriptorType::SAMPLED_IMAGE,
            Self::StorageTexelBuffer => vk::DescriptorType::STORAGE_TEXEL_BUFFER,
            Self::UniformTexelBuffer => vk::DescriptorType::UNIFORM_TEXEL_BUFFER,
        }
    }
}
