use crate::instances::CommandBuffer;
use crate::instances::{queue::Queue, Device};
use anyhow::Result;
use std::sync::Arc;

use ash::vk;

pub struct Fence {
    intern: vk::Fence,
    device: Arc<Device>,
}

impl Fence {
    pub fn new(device: Arc<Device>) -> Result<Arc<Self>> {
        let create_info = vk::FenceCreateInfo::default();

        let fence = unsafe { device.as_raw().create_fence(&create_info, None) }?;

        Ok(Arc::new(Self {
            intern: fence,
            device,
        }))
    }

    pub fn submit_buffers(&self, buffers: &[CommandBuffer], queue: Arc<Queue>) -> Result<()> {
        let buffers: Vec<vk::CommandBuffer> = buffers.into_iter().map(|v| v.as_raw()).collect();
        let submits = [vk::SubmitInfo::default().command_buffers(&buffers)];

        unsafe {
            self.device
                .as_raw()
                .queue_submit(queue.as_raw(), &submits, self.intern)
        }?;

        Ok(())
    }

    pub fn wait_for_finished(&self, timeout: u64) -> Result<()> {
        let res = unsafe {
            self.device
                .as_raw()
                .wait_for_fences(&[self.intern], true, timeout)
        };

        if let Err(e) = res {
            match e {
                ash::vk::Result::TIMEOUT => return Ok(()),
                _ => return Err(e.into()),
            }
        }

        Ok(())
    }

    pub fn as_raw(&self) -> vk::Fence {
        self.intern
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe { self.device.as_raw().destroy_fence(self.intern, None) };
    }
}
