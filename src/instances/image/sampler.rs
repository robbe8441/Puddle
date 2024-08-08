use std::sync::Arc;

use crate::instances::Device;
use anyhow::Result;
use ash::vk;

pub struct Sampler {
    intern: vk::Sampler,
    device: Arc<Device>,
}

impl Sampler {
    pub fn new(device: Arc<Device>, info: &vk::SamplerCreateInfo) -> Result<Arc<Self>> {
        let sampler = unsafe { device.as_raw().create_sampler(info, None) }?;
        Ok(Self {
            intern: sampler,
            device,
        }
        .into())
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe { self.device.as_raw().destroy_sampler(self.intern, None) };
    }
}
