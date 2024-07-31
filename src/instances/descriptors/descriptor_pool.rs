use std::sync::Arc;

use crate::instances::Device;
use anyhow::Result;
use ash::vk;

pub struct DescriptorPool {
    pub(super) intern: vk::DescriptorPool,
    pub(super) device: Arc<Device>,
}

impl DescriptorPool {
    pub fn new(
        device: Arc<Device>,
        sizes: &[vk::DescriptorPoolSize],
        max_sets: u32,
    ) -> Result<Arc<Self>> {
        let create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(sizes)
            .max_sets(max_sets);

        let pool = unsafe { device.as_raw().create_descriptor_pool(&create_info, None) }?;

        Ok(Arc::new(Self {
            intern: pool,
            device,
        }))
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device
                .as_raw()
                .destroy_descriptor_pool(self.intern, None)
        };
    }
}
