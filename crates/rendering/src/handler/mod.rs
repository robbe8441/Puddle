use crate::vulkan::{Buffer, Swapchain, VulkanDevice};
use ash::{prelude::VkResult, vk};
use bindless::{BindlessHandler, BindlessResourceHandle};
use frame::FrameContext;
use render_batch::RenderBatch;
use std::{ffi::CStr, io::Cursor, sync::Arc};

mod bindless;
mod frame;
pub mod render_batch;

/// max frames that can be Prerecorded, makes the render smoother but more delayed
pub const FLYING_FRAMES: usize = 3;

pub struct RenderHandler {
    pub device: Arc<VulkanDevice>,
    swapchain: Swapchain,
    frames: [FrameContext; FLYING_FRAMES],
    batches: Vec<RenderBatch>,
    bindless_handler: BindlessHandler,
    frame_index: usize,
    loaded_shaders: Vec<vk::ShaderEXT>,
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
            loaded_shaders: vec![],
        })
    }

    /// # Panics
    pub fn load_shader(
        &mut self,
        code: &[u8],
        entry: &CStr,
        stage: vk::ShaderStageFlags,
        next_stage: vk::ShaderStageFlags,
    ) -> vk::ShaderEXT {
        let mut code = Cursor::new(code);
        let byte_code = ash::util::read_spv(&mut code).unwrap();

        let create_info = [vk::ShaderCreateInfoEXT {
            p_code: byte_code.as_ptr().cast(),
            code_size: byte_code.len() * size_of::<u32>(),
            p_set_layouts: &self.bindless_handler.descriptor_layout,
            set_layout_count: 1,
            code_type: vk::ShaderCodeTypeEXT::SPIRV,
            stage,
            next_stage,
            p_name: entry.as_ptr(),
            ..Default::default()
        }];

        let shader =
            unsafe { self.device.shader_device.create_shaders(&create_info, None) }.unwrap()[0];
        self.loaded_shaders.push(shader);

        shader
    }

    #[inline]
    pub fn add_render_batch(&mut self, batch: RenderBatch) {
        self.batches.push(batch);
    }

    #[inline]
    pub fn set_uniform_buffer(&mut self, buffer: Arc<Buffer>) -> BindlessResourceHandle {
        self.bindless_handler
            .set_uniform_buffer(&self.device, buffer)
    }

    #[inline]
    pub fn set_storage_buffer(&mut self, buffer: Arc<Buffer>) -> BindlessResourceHandle {
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
        self.swapchain.recreate(self.device.clone(), new_size)?;
        Ok(())
    }

    /// # Safety
    /// # Errors
    pub unsafe fn draw(&mut self) -> VkResult<()> {
        self.frames[self.frame_index].execute(
            &self.device,
            &self.swapchain,
            &self.batches,
            &self.bindless_handler,
        )?;

        self.frame_index = (self.frame_index + 1) % FLYING_FRAMES;
        Ok(())
    }

    pub fn get_swapchain_resolution(&self) -> vk::Extent2D {
        unsafe { (*self.swapchain.create_info.get()).image_extent }
    }
}

impl Drop for RenderHandler {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            for shader in &self.loaded_shaders {
                self.device.shader_device.destroy_shader(*shader, None);
            }
            for frame in &self.frames {
                frame.destroy(&self.device);
            }
            self.bindless_handler.destroy(&self.device);
        }
    }
}
