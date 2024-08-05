use anyhow::Result;
use ash::vk::{self, ShaderStageFlags};
use std::sync::Arc;

use super::{DescriptorPool, WriteDescriptorSet};

pub struct DescriptorSet {
    intern: Vec<vk::DescriptorSet>,
    layout: Vec<vk::DescriptorSetLayout>,
    pool: Arc<DescriptorPool>,
}

impl DescriptorSet {
    pub fn new(
        pool: Arc<DescriptorPool>,
        writes: &[super::WriteDescriptorSet],
    ) -> Result<Arc<Self>> {
        let device_raw = pool.device.as_raw();

        let desc_layout_bindings: Vec<_> = writes
            .iter()
            .map(|desc| vk::DescriptorSetLayoutBinding {
                binding: desc.dst_binding,
                descriptor_type: desc.descriptor_type,
                descriptor_count: desc.descriptor_count,
                stage_flags: ShaderStageFlags::ALL,
                ..Default::default()
            })
            .collect();

        let descriptor_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&desc_layout_bindings);

        let desc_set_layouts =
            [unsafe { device_raw.create_descriptor_set_layout(&descriptor_info, None) }?];

        let desc_alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool.intern)
            .set_layouts(&desc_set_layouts);

        let descriptor_sets = unsafe { device_raw.allocate_descriptor_sets(&desc_alloc_info) }?;

        let writes: Vec<_> = writes
            .iter()
            .map(|write| {
                let mut vkwrite = vk::WriteDescriptorSet::default()
                    .dst_binding(write.dst_binding)
                    .descriptor_count(write.descriptor_count)
                    .descriptor_type(write.descriptor_type)
                    .dst_set(descriptor_sets[write.dst_set as usize]);

                if let Some(info) = write.image_info {
                    vkwrite = vkwrite.image_info(info);
                } else if let Some(info) = write.buffer_info {
                    vkwrite = vkwrite.buffer_info(info);
                }

                vkwrite
            })
            .collect();

        unsafe { device_raw.update_descriptor_sets(&writes, &[]) };

        Ok(Arc::new(Self {
            intern: descriptor_sets,
            layout: desc_set_layouts.into(),
            pool,
        }))
    }

    pub fn update(&self, writes: &[WriteDescriptorSet]) {
        let writes: Vec<_> = writes
            .iter()
            .map(|write| {
                let mut vkwrite = vk::WriteDescriptorSet::default()
                    .dst_binding(write.dst_binding)
                    .descriptor_count(write.descriptor_count)
                    .descriptor_type(write.descriptor_type)
                    .dst_set(self.intern[write.dst_set as usize]);

                if let Some(info) = write.image_info {
                    vkwrite = vkwrite.image_info(info);
                } else if let Some(info) = write.buffer_info {
                    vkwrite = vkwrite.buffer_info(info);
                }

                vkwrite
            })
            .collect();

        unsafe {
            self.pool
                .device
                .as_raw()
                .update_descriptor_sets(&writes, &[])
        };
    }

    pub fn layout(&self) -> Vec<vk::DescriptorSetLayout> {
        self.layout.clone()
    }
    pub fn as_raw(&self) -> Vec<vk::DescriptorSet> {
        self.intern.clone()
    }
}
