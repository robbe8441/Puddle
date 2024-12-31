mod frame;
pub mod render_context;
use std::cell::{Cell, UnsafeCell};

use ash::prelude::VkResult;
use frame::FrameContext;
use render_context::RenderBatch;

use crate::vulkan::{Swapchain, VulkanDevice};

/// max frames that can be Prerecorded, makes the render smoother but more delayed
pub const FLYING_FRAMES: usize = 3;

pub struct RenderHandler {
    pub device: VulkanDevice,
    swapchain: Swapchain,
    frames: [FrameContext; FLYING_FRAMES],
    batches: UnsafeCell<Vec<RenderBatch>>,
    frame_index: Cell<usize>,
}

impl RenderHandler {
    /// # Errors
    /// # Panics
    pub fn new<T>(window: &T, window_size: [u32; 2]) -> VkResult<Self>
    where
        T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        let device = unsafe { VulkanDevice::new(window)? };

        let swapchain = unsafe { Swapchain::new(&device, window_size) }?;

        let frames = std::array::from_fn(|_| unsafe { FrameContext::new(&device).unwrap() });

        Ok(Self {
            device,
            swapchain,
            frames,
            batches: UnsafeCell::new(vec![]),
            frame_index: Cell::new(0),
        })
    }

    pub fn add_render_batch(&self, batch: RenderBatch) {
        let batches = unsafe {&mut *self.batches.get()};
        batches.push(batch);
    }

    /// # Errors
    /// # Safety
    pub unsafe fn resize(&self, new_size: [u32; 2]) -> VkResult<()> {
        self.device.device_wait_idle()?;
        self.swapchain.recreate(&self.device, new_size)?;
        Ok(())
    }

    /// # Safety
    /// # Errors
    pub unsafe fn draw(&self) -> VkResult<()> {
        let frame = self.frame_index.get();
        self.frames[frame].execute(&self.device, &self.swapchain, &*self.batches.get())?;

        self.frame_index.set((frame + 1) % FLYING_FRAMES);
        Ok(())
    }
}

impl Drop for RenderHandler {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            for frame in &self.frames {
                frame.destroy(&self.device);
            }
            self.swapchain.destroy(&self.device);
            self.device.destroy();
        }
    }
}
