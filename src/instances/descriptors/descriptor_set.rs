use crate::instances::Device;
use anyhow::Result;
use ash::vk;
use std::sync::Arc;

use super::WriteDescriptorSet;

pub struct DescriptorSet {
    intern: vk::DescriptorSet,
    layout: vk::DescriptorSetLayout,
    pool: Arc<super::DescriptorPool>,
}

impl DescriptorSet {
    pub fn new(device: Arc<Device>, bindings: &[super::BindingDescriptor]) -> Result<Arc<Self>> {
        let pool = super::DescriptorPool::new(device, bindings, 1)?;
        Self::from_pool(pool)
    }

    pub fn from_pool(pool: Arc<super::DescriptorPool>) -> Result<Arc<Self>> {
        let desc_layout_bindings: Vec<vk::DescriptorSetLayoutBinding> =
            pool.bindings.iter().map(Into::into).collect();

        let descriptor_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&desc_layout_bindings);

        let desc_set_layout = unsafe {
            pool.device
                .as_raw()
                .create_descriptor_set_layout(&descriptor_info, None)
        }?;

        let layouts = [desc_set_layout];
        let desc_alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool.intern)
            .set_layouts(&layouts);

        let descriptor_set = unsafe {
            pool.device
                .as_raw()
                .allocate_descriptor_sets(&desc_alloc_info)
        }?[0];

        Ok(Self {
            intern: descriptor_set,
            layout: desc_set_layout,
            pool,
        }
        .into())
    }

    pub fn write(&self, writes: &[super::WriteDescriptorSet]) {
        let buffer_infos: Vec<Option<Vec<_>>> = writes
            .iter()
            .map(|v| match v {
                WriteDescriptorSet::Buffers(_, buffers) => Some(
                    buffers
                        .iter()
                        .map(|b| vk::DescriptorBufferInfo {
                            offset: b.offset(),
                            range: b.size(),
                            buffer: b.buffer_raw(),
                        })
                        .collect(),
                ),
                _ => None,
            })
            .collect();

        let image_infos: Vec<Option<Vec<_>>> = writes
            .iter()
            .map(|v| match v {
                WriteDescriptorSet::ImageViews(_, views) => Some(
                    views
                        .iter()
                        .map(|b| vk::DescriptorImageInfo {
                            sampler: vk::Sampler::null(),
                            image_view: b.as_raw(),
                            image_layout:  vk::ImageLayout::GENERAL, //b.image().layout(),
                        })
                        .collect(),
                ),
                _ => None,
            })
            .collect();

        let write_descriptors: Vec<_> = writes
            .into_iter()
            .enumerate()
            .map(|(i, desc)| match desc {
                WriteDescriptorSet::Buffers(binding, buffers) => vk::WriteDescriptorSet::default()
                    .descriptor_type(self.pool.bindings[*binding as usize].ty.into())
                    .descriptor_count(buffer_infos.len() as u32)
                    .dst_set(self.intern)
                    .dst_binding(*binding)
                    .buffer_info(&buffer_infos[i].as_ref().unwrap()),
                WriteDescriptorSet::ImageViews(binding, views) => vk::WriteDescriptorSet::default()
                    .descriptor_type(self.pool.bindings[*binding as usize].ty.into())
                    .descriptor_count(buffer_infos.len() as u32)
                    .dst_set(self.intern)
                    .dst_binding(*binding)
                    .image_info(&image_infos[i].as_ref().unwrap()),
            })
            .collect();

        unsafe {
            self.pool
                .device
                .as_raw()
                .update_descriptor_sets(&write_descriptors, &[])
        };
    }

    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.layout
    }

    pub fn as_raw(&self) -> vk::DescriptorSet {
        self.intern
    }
}
