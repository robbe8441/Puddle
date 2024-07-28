use std::sync::Arc;

use anyhow::Result;
use ash::vk;

use crate::instances::Device;

#[allow(unused)]
pub struct RawBuffer {
    intern: vk::Buffer,
    memory: Arc<super::DeviceMemory>,
    usage: vk::BufferUsageFlags,
    size: vk::DeviceSize,
    device: Arc<Device>,
}

impl RawBuffer {
    pub fn new(
        device: Arc<Device>,
        create_info: vk::BufferCreateInfo,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Arc<Self>> {
        let device_raw = device.as_raw();

        let buffer = unsafe { device_raw.create_buffer(&create_info, None) }?;

        let memory_req = unsafe { device_raw.get_buffer_memory_requirements(buffer) };

        let mem = super::DeviceMemory::new(device.clone(), property_flags, memory_req)?;

        unsafe { device_raw.bind_buffer_memory(buffer, mem.as_raw(), 0) }.unwrap();

        Ok(Arc::new(Self {
            intern: buffer,
            memory: mem,
            usage: create_info.usage,
            size: create_info.size,
            device,
        }))
    }

    pub fn size(&self) -> vk::DeviceSize {
        self.size
    }
    pub fn memory(&self) -> Arc<super::DeviceMemory> {
        self.memory.clone()
    }
    pub fn usage(&self) -> vk::BufferUsageFlags {
        self.usage
    }
    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }
    pub fn as_raw(&self) -> vk::Buffer {
        self.intern
    }
}

impl Drop for RawBuffer {
    fn drop(&mut self) {
        unsafe { self.device.as_raw().destroy_buffer(self.intern, None) };
    }
}

