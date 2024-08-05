use anyhow::Result;
use std::sync::Arc;

use super::BindingDescriptor;
use crate::instances::Device;
use ash::vk;

pub struct DescriptorPool {
    pub(super) intern: vk::DescriptorPool,
    pub(super) device: Arc<Device>,
    pub(super) bindings: Arc<[BindingDescriptor]>,
}

impl DescriptorPool {
    pub fn new(
        device: Arc<Device>,
        bindings: &[BindingDescriptor],
        max_sets: u32,
    ) -> Result<Arc<Self>> {
        let sizes: Vec<_> = bindings
            .iter()
            .map(|b| {
                vk::DescriptorPoolSize::default()
                    .descriptor_count(b.count)
                    .ty(b.ty.into())
            })
            .collect();

        let create_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(max_sets)
            .pool_sizes(&sizes);

        let pool = unsafe { device.as_raw().create_descriptor_pool(&create_info, None) }?;

        Ok(Self {
            intern: pool,
            bindings: bindings.into(),
            device,
        }
        .into())
    }
}
