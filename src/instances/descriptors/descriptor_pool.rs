use std::sync::Arc;

use crate::instances::{Device, Subbuffer};
use anyhow::{Context, Result};
use ash::vk;

pub struct DescriptorPool {
    intern: vk::DescriptorPool,
    device: Arc<Device>,
}

impl DescriptorPool {
    pub fn new(device: Arc<Device>) -> Result<Arc<Self>> {
        let pool_sizes = [vk::DescriptorPoolSize::default()
            .descriptor_count(1)
            .ty(vk::DescriptorType::STORAGE_BUFFER)];

        let create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(1);

        let pool = unsafe { device.as_raw().create_descriptor_pool(&create_info, None) }?;

        Ok(Arc::new(Self {
            intern: pool,
            device,
        }))
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device
                .as_raw()
                .destroy_descriptor_pool(self.intern, None)
        };
    }
}

pub struct DescriptorSet {
    intern: Vec<vk::DescriptorSet>,
    layout: Vec<vk::DescriptorSetLayout>,
}

impl DescriptorSet {
    pub fn new(pool: Arc<DescriptorPool>, buffer: Arc<Subbuffer<i32>>) -> Result<Arc<Self>> {
        let device_raw = pool.device.as_raw();

        let desc_layout_bindings = [vk::DescriptorSetLayoutBinding {
            descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::COMPUTE,
            ..Default::default()
        }];
        let descriptor_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&desc_layout_bindings);

        let desc_set_layouts =
            [unsafe { device_raw.create_descriptor_set_layout(&descriptor_info, None) }?];

        let desc_alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool.intern)
            .set_layouts(&desc_set_layouts);

        let descriptor_sets = unsafe { device_raw.allocate_descriptor_sets(&desc_alloc_info) }?;

        let buffer_descriptor = vk::DescriptorBufferInfo {
            buffer: buffer.raw_buffer(),
            offset: buffer.offset(),
            range: buffer.size(),
        };

        let write_desc_sets = [
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[0],
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_buffer_info: &buffer_descriptor,
                ..Default::default()
            },
        ];
        unsafe { device_raw.update_descriptor_sets(&write_desc_sets, &[]) };

        Ok(Arc::new(Self {
            intern: descriptor_sets,
            layout: desc_set_layouts.into(),
        }))
    }

    pub fn layout(&self) -> Vec<vk::DescriptorSetLayout> {
        self.layout.clone()
    }
    pub fn as_raw(&self) -> Vec<vk::DescriptorSet> {
        self.intern.clone()
    }
}
