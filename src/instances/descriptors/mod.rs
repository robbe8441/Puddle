use std::sync::Arc;

use ash::vk;

mod descriptor_pool;
mod descriptor_set;

pub use descriptor_pool::*;
pub use descriptor_set::*;

pub use vk::DescriptorPoolSize;

use super::BufferAllocation;

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
pub enum DescriptorType {
    UniformBuffer,
    StorageBuffer,
    StorageImage,
    SampledImage,
    StorageTexelBuffer,
    UniformTexelBuffer,
    Sampler,
}

impl Into<vk::DescriptorType> for DescriptorType {
    fn into(self) -> vk::DescriptorType {
        match self {
            Self::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
            Self::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
            Self::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
            Self::SampledImage => vk::DescriptorType::SAMPLED_IMAGE,
            Self::StorageTexelBuffer => vk::DescriptorType::STORAGE_TEXEL_BUFFER,
            Self::UniformTexelBuffer => vk::DescriptorType::UNIFORM_TEXEL_BUFFER,
            Self::Sampler => vk::DescriptorType::SAMPLER,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BindingDescriptor {
    pub binding: u32,
    pub count: u32,
    pub shader_stage: vk::ShaderStageFlags,
    pub ty: DescriptorType,
}

impl<'a> Into<vk::DescriptorSetLayoutBinding<'a>> for &BindingDescriptor {
    fn into(self) -> vk::DescriptorSetLayoutBinding<'a> {
        vk::DescriptorSetLayoutBinding {
            binding: self.binding,
            descriptor_count: self.count,
            stage_flags: self.shader_stage,
            descriptor_type: self.ty.into(),
            ..Default::default()
        }
    }
}

pub enum WriteDescriptorSet {
    Buffers(u32, Vec<Arc<dyn BufferAllocation>>),
    // Image(u32),
}
