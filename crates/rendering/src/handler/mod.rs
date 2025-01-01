use crate::vulkan::{Swapchain, VulkanDevice};
use ash::{prelude::VkResult, vk};
use bindless::{BindlessHandler, BindlessResourceHandle};
use frame::FrameContext;
use render_context::RenderBatch;
use std::sync::Arc;

mod bindless;
mod frame;
pub mod render_context;

/// max frames that can be Prerecorded, makes the render smoother but more delayed
pub const FLYING_FRAMES: usize = 3;

pub struct RenderHandler {
    pub device: Arc<VulkanDevice>,
    swapchain: Swapchain,
    frames: [FrameContext; FLYING_FRAMES],
    batches: Vec<RenderBatch>,
    bindless_handler: BindlessHandler,
    frame_index: usize,
}

impl RenderHandler {
    /// # Errors
    /// # Panics
    pub fn new<T>(window: &T, window_size: [u32; 2]) -> VkResult<Self>
    where
        T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        let device = unsafe { Arc::new(VulkanDevice::new(window)?) };

        let swapchain = unsafe { Swapchain::new(device.clone(), window_size) }?;

        let frames = std::array::from_fn(|_| unsafe { FrameContext::new(&device).unwrap() });

        let bindless_handler = BindlessHandler::new(&device)?;

        Ok(Self {
            device,
            swapchain,
            frames,
            batches: vec![],
            bindless_handler,
            frame_index: 0,
        })
    }

    #[inline]
    pub fn add_render_batch(&mut self, batch: RenderBatch) {
        self.batches.push(batch);
    }

    #[inline]
    pub fn set_uniform_buffer(&mut self, buffer: vk::Buffer) -> BindlessResourceHandle {
        self.bindless_handler
            .set_uniform_buffer(&self.device, buffer)
    }

    #[inline]
    pub fn set_storage_buffer(&mut self, buffer: vk::Buffer) -> BindlessResourceHandle {
        self.bindless_handler
            .set_storage_buffer(&self.device, buffer)
    }

    #[inline]
    pub fn set_storage_image(
        &mut self,
        image: vk::ImageView,
        layout: vk::ImageLayout,
    ) -> BindlessResourceHandle {
        self.bindless_handler
            .set_storage_image(&self.device, image, layout)
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
    pub unsafe fn draw(&mut self) -> VkResult<()> {
        self.frames[self.frame_index].execute(&self.device, &self.swapchain, &self.batches)?;

        self.frame_index = (self.frame_index + 1) % FLYING_FRAMES;
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
            self.bindless_handler.destroy(&self.device);
        }
    }
}
