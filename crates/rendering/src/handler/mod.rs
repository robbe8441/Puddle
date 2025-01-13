use crate::vulkan::{Buffer, Swapchain, VulkanDevice};
use ash::{prelude::VkResult, vk};
use bindless::{get_free_slot, BindlessHandler, BindlessResourceHandle, ResourceSlot};
use frame::FrameContext;
use material::MaterialHandler;
use render_batch::RenderBatch;
use std::sync::Arc;

mod bindless;
mod frame;
pub mod material;
pub mod render_batch;

/// max frames that can be Prerecorded, makes the render smoother but more delayed
pub const FLYING_FRAMES: usize = 2;

pub struct RenderHandler {
    pub device: Arc<VulkanDevice>,
    swapchain: Swapchain,
    materials: MaterialHandler,
    frames: [FrameContext; FLYING_FRAMES],
    batches: Vec<RenderBatch>,
    bindless_handler: BindlessHandler,
    frame_index: usize,
    // a queue of resources that are supposed to be destroyed but need to wait for a fence
    destroy_queue: Vec<(vk::Fence, DestroyResource)>,
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

        let mut materials = MaterialHandler::new(device.clone(), &swapchain)?;

        let frames = std::array::from_fn(|_| unsafe { FrameContext::new(&device).unwrap() });

        let bindless_handler = BindlessHandler::new(&device)?;

        materials.create_pipeline(
            bindless_handler.pipeline_layout,
            swapchain.get_image_extent(),
        );

        Ok(Self {
            device,
            swapchain,
            materials,
            frames,
            batches: vec![],
            bindless_handler,
            frame_index: 0,
            destroy_queue: vec![],
        })
    }

    #[inline]
    pub fn add_render_batch(&mut self, batch: RenderBatch) {
        self.batches.push(batch);
    }

    /// sets the given index in the array to be this buffer
    pub fn set_uniform_buffer(
        &mut self,
        buffer: Arc<Buffer>,
        index: usize,
    ) -> BindlessResourceHandle {
        let handle = BindlessResourceHandle {
            index,
            ty: bindless::BindlessResourceType::UniformBuffer,
        };

        self.bindless_handler
            .upload_buffer(buffer, handle, self.frame_index);

        self.bindless_handler.uniform_buffers[index] = ResourceSlot::Submited;

        handle
    }

    /// sets the first free index to be this buffer
    pub fn push_uniform_buffer(&mut self, buffer: Arc<Buffer>) -> Option<BindlessResourceHandle> {
        let index = get_free_slot(&self.bindless_handler.uniform_buffers)?;
        Some(self.set_uniform_buffer(buffer, index))
    }

    /// sets the given index in the array to be this buffer
    pub fn set_storage_buffer(
        &mut self,
        buffer: Arc<Buffer>,
        index: usize,
    ) -> BindlessResourceHandle {
        let handle = BindlessResourceHandle {
            index,
            ty: bindless::BindlessResourceType::StorageBuffer,
        };

        self.bindless_handler
            .upload_buffer(buffer, handle, self.frame_index);

        self.bindless_handler.uniform_buffers[index] = ResourceSlot::Submited;

        handle
    }

    /// sets the first free index to be this buffer
    pub fn push_storage_buffer(&mut self, buffer: Arc<Buffer>) -> Option<BindlessResourceHandle> {
        let index = get_free_slot(&self.bindless_handler.storage_buffers)?;
        Some(self.set_storage_buffer(buffer, index))
    }

    // TODO
    // pub fn set_storage_image() {}

    /// # Errors
    /// if there was an issue creating a new swapchain
    /// for example if there is no memory left
    pub fn on_window_resize(&mut self, new_size: [u32; 2]) -> VkResult<()> {
        unsafe {
            self.device.device_wait_idle()?;
            self.swapchain.recreate(self.device.clone(), new_size)
        }
    }

    /// # Safety
    /// # Errors
    pub fn on_render(&mut self) -> VkResult<()> {
        self.frame_index = (self.frame_index + 1) % FLYING_FRAMES;

        unsafe {
            self.frames[self.frame_index].execute(
                &self.device,
                &self.materials,
                &mut self.swapchain,
                &self.batches,
                &self.bindless_handler,
                self.frame_index,
            )?;
        }

        self.bindless_handler
            .update_descriptor_set(&self.device, self.frame_index);

        self.clean_resources();

        Ok(())
    }

    pub fn get_swapchain_resolution(&self) -> vk::Extent2D {
        self.swapchain.create_info.image_extent
    }

    /// resizes a buffer buffer that bound
    /// the buffer must not be currently used somewhere except by the renderer it self
    /// the handle stays valid and doesn't need to be updated
    /// # Panics
    /// if the handle doesn't point to a valid resource
    /// # Errors
    /// if there is no space to allocate
    pub fn resize_buffer(
        &mut self,
        handle: &BindlessResourceHandle,
        new_size: u64,
    ) -> VkResult<Arc<Buffer>> {
        // pull the buffer out of the bindless array
        let buffer = match handle.ty {
            bindless::BindlessResourceType::StorageBuffer => {
                self.bindless_handler.storage_buffers[handle.index].take()
            }
            bindless::BindlessResourceType::UniformBuffer => {
                self.bindless_handler.uniform_buffers[handle.index].take()
            }
            bindless::BindlessResourceType::StorageImage => unimplemented!(),
        }
        .expect("the given handle is invalid and doesnt point to a resource");

        // we need ownership and ensure that nothing else is currently using the buffer
        // as we want to destroy the old one
        let buffer_owned =
            Arc::into_inner(buffer).expect("the buffer is still being used somewhere else");

        let new_buffer = buffer_owned.resize(self.device.clone(), new_size)?;

        match handle.ty {
            bindless::BindlessResourceType::StorageBuffer => {
                self.set_storage_buffer(new_buffer.clone(), handle.index)
            }
            bindless::BindlessResourceType::UniformBuffer => {
                self.set_uniform_buffer(new_buffer.clone(), handle.index)
            }
            bindless::BindlessResourceType::StorageImage => unimplemented!(),
        };

        // we need to wait until the last frame using the old buffer is finished executing
        let wait_for_fence = &self.frames[self.frame_index].is_executing_fence;

        self.destroy_queue
            .push((*wait_for_fence, DestroyResource::Buffer(buffer_owned)));

        Ok(new_buffer)
    }

    pub fn clean_resources(&mut self) {
        unsafe {
            let mut i = 0;
            while let Some((fence, _)) = self.destroy_queue.get(i) {
                if self.device.wait_for_fences(&[*fence], true, 0).is_ok() {
                    self.destroy_queue.remove(i);
                }

                i += 1;
            }
        }
    }
}

pub enum DestroyResource {
    Buffer(Buffer),
    Image(vk::Image),
    ImageView(vk::ImageView),
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
