use std::sync::Arc;

use anyhow::{Context, Result};
use ash::vk;

use crate::instances::Device;

#[allow(unused)]
pub struct RawBuffer {
    intern: vk::Buffer,
    memory_req: vk::MemoryRequirements,
    memory_index: u32,
    property_flags: vk::MemoryPropertyFlags,
    device: Arc<Device>,
}

impl RawBuffer {
    pub fn new(device: Arc<Device>, create_info: vk::BufferCreateInfo, property_flags: vk::MemoryPropertyFlags) -> Result<Arc<Self>> {
        let device_raw = device.as_raw();

        let buffer = unsafe { device_raw.create_buffer(&create_info, None) }?;

        let memory_req = unsafe { device_raw.get_buffer_memory_requirements(buffer) };

        let memory_index = find_memorytype_index(
            &memory_req,
            &device.memory_priorities(),
            property_flags,
        )
        .context("Unable to find suitable memorytype for the vertex buffer.")?;

        Ok(Arc::new(Self {
            intern: buffer,
            memory_index,
            memory_req,
            property_flags,
            device,
        }))
    }

    pub fn memory_req(&self) -> vk::MemoryRequirements {
        self.memory_req.clone()
    }
    pub fn properties(&self) -> vk::MemoryPropertyFlags {
        self.property_flags
    }
    pub fn memory_index(&self) -> u32 {
        self.memory_index
    }
    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }
    pub fn as_raw(&self) -> vk::Buffer {
        self.intern.clone()
    }
}

impl Drop for RawBuffer {
    fn drop(&mut self) {
        unsafe { self.device.as_raw().destroy_buffer(self.intern, None) };
    }
}

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
