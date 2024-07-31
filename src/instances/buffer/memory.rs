use crate::instances::Device;
use std::sync::Arc;

use anyhow::{Context, Result};
use ash::vk;

pub struct DeviceMemory {
    intern: vk::DeviceMemory,
    memory_index: u32,
    property_flags: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
    device: Arc<Device>,
}

impl DeviceMemory {
    pub fn new(
        device: Arc<Device>,
        property_flags: vk::MemoryPropertyFlags,
        memory_req: vk::MemoryRequirements,
    ) -> Result<Arc<Self>> {
        let memory_index =
            find_memorytype_index(&memory_req, &device.memory_priorities(), property_flags)
                .context("Unable to find suitable memorytype for the vertex buffer.")?;

        let allocate_info = vk::MemoryAllocateInfo {
            allocation_size: memory_req.size,
            memory_type_index: memory_index,
            ..Default::default()
        };

        let mem = unsafe { device.as_raw().allocate_memory(&allocate_info, None) }?;

        Ok(Arc::new(Self {
            device,
            intern: mem,
            property_flags,
            memory_index,
            requirements: memory_req,
        }))
    }

    pub fn requirements(&self) -> vk::MemoryRequirements {
        self.requirements
    }

    pub fn as_raw(&self) -> vk::DeviceMemory {
        self.intern
    }
}

impl Drop for DeviceMemory {
    fn drop(&mut self) {
        unsafe { self.device.as_raw().free_memory(self.intern, None) };
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
