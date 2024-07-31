use crate::instances::Device;
use std::sync::Arc;

use anyhow::Result;
use ash::vk;

pub struct Framebuffer {
    intern: vk::Framebuffer,
    size: [u32; 2],
    device: Arc<Device>,
}

impl Framebuffer {
    pub fn new(device: Arc<Device>, create_info: &vk::FramebufferCreateInfo) -> Result<Arc<Self>> {
        let framebuffer = unsafe { device.as_raw().create_framebuffer(create_info, None) }?;
        let size = [create_info.width, create_info.height];

        Ok(Arc::new(Self {
            device,
            size,
            intern: framebuffer,
        }))
    }

    pub fn size(&self) -> vk::Extent2D {
        vk::Extent2D {
            width: self.size[0],
            height: self.size[1],
        }
    }

    pub fn as_raw(&self) -> vk::Framebuffer {
        self.intern
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe { self.device.as_raw().destroy_framebuffer(self.intern, None) }
    }
}
