use anyhow::{Context, Result};
use std::sync::Arc;

use ash::vk::{self, CommandPoolCreateFlags};

pub struct CommandPool {
    intern: vk::CommandPool,
    queue_family: u32,
    device: Arc<super::Device>,
}

impl CommandPool {
    pub fn new(device: Arc<super::Device>, queue_family: u32) -> Result<Arc<Self>> {
        let device_raw = device.as_raw();

        let create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family)
            .flags(CommandPoolCreateFlags::TRANSIENT);

        let command_pool = unsafe { device_raw.create_command_pool(&create_info, None) }?;

        Ok(Arc::new(Self {
            intern: command_pool,
            queue_family,
            device,
        }))
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe { self.device.as_raw().destroy_command_pool(self.intern, None) };
    }
}

pub struct CommandBuffer {
    intern: vk::CommandBuffer,
    pool: Arc<CommandPool>,
}

impl CommandBuffer {
    pub fn new(pool: Arc<CommandPool>) -> Result<Self> {
        let device_raw = pool.device.as_raw();

        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool.intern)
            .command_buffer_count(1);

        let command_buffer = unsafe { device_raw.allocate_command_buffers(&allocate_info) }?
            .into_iter()
            .next()
            .context("failed to allocate command buffer")?;

        Ok(Self {
            intern: command_buffer,
            pool,
        })
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe { self.pool.device.as_raw().free_command_buffers(self.pool.intern, &[self.intern]) };
    }
}
