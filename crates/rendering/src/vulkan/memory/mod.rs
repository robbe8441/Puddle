use std::sync::Arc;
use ash::{prelude::VkResult, vk};
use super::VulkanDevice;
pub use buffer::Buffer;

mod buffer;

pub struct MemoryBlock {
    device: Arc<VulkanDevice>,
    memory: vk::DeviceMemory,
}

impl MemoryBlock {
    /// # Errors
    /// if there is no space left to allocate
    /// # Panics
    /// if the requested memory type doesn't exist
    pub fn new(
        device: Arc<VulkanDevice>,
        memory_requirements: vk::MemoryRequirements,
        memory_props: vk::MemoryPropertyFlags,
    ) -> VkResult<Self> {
        let mem_props = unsafe {
            device
                .instance
                .get_physical_device_memory_properties(device.pdevice)
        };

        let memory_index = find_memorytype_index(memory_requirements, mem_props, memory_props)
            .expect("failed to find memory type index");

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_index);

        let memory = unsafe { device.allocate_memory(&alloc_info, None) }?;

        Ok(Self { device, memory })
    }

    #[must_use]
    pub fn handle(&self) -> vk::DeviceMemory {
        self.memory
    }
}

impl Drop for MemoryBlock {
    fn drop(&mut self) {
        unsafe { self.device.free_memory(self.memory, None) };
    }
}


#[must_use]
pub fn find_memorytype_index(
    memory_req: vk::MemoryRequirements,
    memory_prop: vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
    .map(|(index, _memory_type)| index as u32)
}
