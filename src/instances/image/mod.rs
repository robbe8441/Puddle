mod view;
mod sampler;
pub use view::*;
pub use sampler::*;


use super::buffer::DeviceMemory;
use anyhow::Result;
use std::sync::Arc;

use ash::vk;

pub struct Image {
    intern: vk::Image,
    memory: Arc<DeviceMemory>,
    device: Arc<super::Device>,
    create_info: vk::ImageCreateInfo<'static>,
}

impl Image {
    pub fn new(device: Arc<super::Device>, create_info: vk::ImageCreateInfo<'static>) -> Result<Arc<Self>> {
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
            create_info,
            memory,
            device,
        }))
    }

    pub fn layout(&self) -> vk::ImageLayout {
        self.create_info.initial_layout
    }
    pub fn as_raw(&self) -> vk::Image {
        self.intern
    }
    pub fn device(&self) -> Arc<super::Device> {
        self.device.clone()
    }
    pub fn create_info(&self) -> vk::ImageCreateInfo {
        self.create_info
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe { self.device.as_raw().destroy_image(self.intern, None) };
    }
}
