use std::sync::Arc;

use ash::{prelude::VkResult, vk};

use crate::vulkan::VulkanDevice;

use super::MemoryBlock;

pub struct Buffer {
    memory: Arc<MemoryBlock>,
    handle: vk::Buffer,
}

impl Buffer {
    /// # Errors
    pub fn new(
        device: Arc<VulkanDevice>,
        size: u64,
        usage: vk::BufferUsageFlags,
        property_flags: vk::MemoryPropertyFlags,
    ) -> VkResult<Self> {
        let create_info = vk::BufferCreateInfo::default().size(size).usage(usage);

        let buffer = unsafe { device.create_buffer(&create_info, None) }?;
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        let memory = MemoryBlock::new(device.clone(), requirements, property_flags)?;

        unsafe { device.bind_buffer_memory(buffer, memory.memory, 0) }?;

        Ok(Self {
            memory: Arc::new(memory),
            handle: buffer,
        })
    }

    #[must_use]
    pub fn handle(&self) -> vk::Buffer {
        self.handle
    }
    #[must_use]
    pub fn mem_ref(&self) -> &MemoryBlock {
        &self.memory
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.memory.device.destroy_buffer(self.handle, None);
        }
    }
}
