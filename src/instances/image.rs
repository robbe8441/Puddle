use super::buffer::DeviceMemory;
use anyhow::Result;
use std::sync::Arc;

use ash::vk;

pub struct Image {
    intern: vk::Image,
    memory: Arc<DeviceMemory>,
    device: Arc<super::Device>,
    layout: vk::ImageLayout,
}

impl Image {
    pub fn new(device: Arc<super::Device>, create_info: vk::ImageCreateInfo) -> Result<Arc<Self>> {
        let device_raw = device.as_raw();

        let image = unsafe { device_raw.create_image(&create_info, None) }?;

        let memory_req = unsafe { device_raw.get_image_memory_requirements(image) };

        let memory = DeviceMemory::new(
            device.clone(),
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            memory_req,
        )?;

        unsafe { device_raw.bind_image_memory(image, memory.as_raw(), 0) }?;

        Ok(Arc::new(Self {
            intern: image,
            layout: create_info.initial_layout,
            memory,
            device,
        }))
    }

    pub fn as_raw(&self) -> vk::Image {
        self.intern
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe { self.device.as_raw().destroy_image(self.intern, None) };
    }
}
