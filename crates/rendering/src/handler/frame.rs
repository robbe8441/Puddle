use super::{bindless::BindlessHandler, render_batch::RenderBatch};
use crate::vulkan::{Swapchain, VulkanDevice};
use ash::{
    prelude::VkResult,
    vk::{self, Handle},
};

pub struct FrameContext {
    /// tells if this ``FrameContext`` is currently executing
    is_executing_fence: vk::Fence,
    /// tells when the image is ready to be drawn on to
    image_available_semaphore: vk::Semaphore,
    /// tells when the render has finished and is ready to be presented
    render_finished_semaphore: vk::Semaphore,

    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
}

impl FrameContext {
    pub unsafe fn new(device: &VulkanDevice) -> VkResult<Self> {
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let semaphore_info = vk::SemaphoreCreateInfo::default();

        let is_executing_fence = device.create_fence(&fence_info, None)?;
        let image_available_semaphore = device.create_semaphore(&semaphore_info, None)?;
        let render_finished_semaphore = device.create_semaphore(&semaphore_info, None)?;

        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = device.create_command_pool(&pool_info, None)?;

        let command_buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffer = device.allocate_command_buffers(&command_buffer_info)?[0];

        // device.begin_command_buffer(command_buffer, &vk::CommandBufferBeginInfo::default())?;
        // device.end_command_buffer(command_buffer)?;

        Ok(Self {
            is_executing_fence,
            image_available_semaphore,
            render_finished_semaphore,
            command_pool,
            command_buffer,
        })
    }

    pub unsafe fn destroy(&self, device: &VulkanDevice) {
        let _ = device.wait_for_fences(&[self.is_executing_fence], true, u64::MAX);
        device.destroy_fence(self.is_executing_fence, None);
        device.destroy_semaphore(self.image_available_semaphore, None);
        device.destroy_semaphore(self.render_finished_semaphore, None);
        device.destroy_command_pool(self.command_pool, None);
    }

    unsafe fn request_image_index(&self, swapchain: &Swapchain) -> VkResult<(u32, bool)> {
        let handle = *swapchain.handle.get();
        swapchain.loader.acquire_next_image(
            handle,
            u64::MAX,
            self.image_available_semaphore,
            vk::Fence::null(),
        )
    }

    unsafe fn submit(
        &self,
        device: &VulkanDevice,
        swapchain: &Swapchain,
        image_index: u32,
    ) -> VkResult<()> {
        let wait_semaphores = [self.image_available_semaphore];
        let signal_semaphores = [self.render_finished_semaphore];
        let command_buffers = [self.command_buffer];

        let submits = [vk::SubmitInfo::default()
            .command_buffers(&command_buffers)
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .signal_semaphores(&signal_semaphores)];

        device.queue_submit(device.queues.graphics.1, &submits, self.is_executing_fence)?;

        let swapchains = [*swapchain.handle.get()];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        swapchain
            .loader
            .queue_present(device.queues.graphics.1, &present_info)?;

        Ok(())
    }

    pub unsafe fn execute(
        &self,
        device: &VulkanDevice,
        swapchain: &Swapchain,
        batches: &[RenderBatch],
        bindless_handler: &BindlessHandler,
    ) -> VkResult<()> {
        // wait for the commandbuffer to finish executing before resetting it
        device.wait_for_fences(&[self.is_executing_fence], true, u64::MAX)?;

        let (image_index, _suboptimal) = self.request_image_index(swapchain)?;

        // if there is still being rendered to the image, then we need to wait
        let wait_fence = &mut (*swapchain.images.get())[image_index as usize].available;
        if !wait_fence.is_null() {
            device.wait_for_fences(&[*wait_fence], true, u64::MAX)?;
        }
        *wait_fence = self.is_executing_fence;

        device.reset_fences(&[self.is_executing_fence])?;

        device.reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())?;

        self.record_command_buffer(device, swapchain, image_index, batches, bindless_handler)?;

        self.submit(device, swapchain, image_index)?;
        Ok(())
    }

    unsafe fn record_command_buffer(
        &self,
        device: &VulkanDevice,
        swapchain: &Swapchain,
        image_index: u32,
        batches: &[RenderBatch],
        bindless_handler: &BindlessHandler,
    ) -> VkResult<()> {
        let command_buffer = self.command_buffer;
        let swapchain_views = &*swapchain.images.get();
        let swapchain_images = &*swapchain
            .loader
            .get_swapchain_images(*swapchain.handle.get())?;

        device.begin_command_buffer(self.command_buffer, &vk::CommandBufferBeginInfo::default())?;

        // bind bindless descriptor set
        device.cmd_bind_descriptor_sets(
            self.command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            bindless_handler.pipeline_layout,
            0,
            &[bindless_handler.descriptor_set],
            &[],
        );

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.05, 0.01, 0.07, 1.0],
            },
        };
        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0],
            },
        };

        // convert the swapchain image in to the right format to be written on
        let barrier = vk::ImageMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .image(swapchain_images[image_index as usize])
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        device.cmd_pipeline_barrier(
            self.command_buffer,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );

        let clear_attachments = [
            vk::RenderingAttachmentInfo::default()
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(color_clear_value)
                .image_view(swapchain_views[image_index as usize].main_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
            vk::RenderingAttachmentInfo::default()
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(clear_value)
                .image_view(swapchain_views[image_index as usize].normal_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
            vk::RenderingAttachmentInfo::default()
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(clear_value)
                .image_view(swapchain_views[image_index as usize].depth_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
        ];

        let render_attachments: Vec<_> = clear_attachments
            .iter()
            .map(|v| v.load_op(vk::AttachmentLoadOp::LOAD))
            .collect();

        let image_size = (*swapchain.create_info.get()).image_extent;
        let render_area = vk::Rect2D::default().extent(image_size);

        // the first draw clears the attachments
        if let Some(batch) = batches.first() {
            batch.execute(device, command_buffer, &clear_attachments, render_area);
        }

        for batch in batches.iter().skip(1) {
            batch.execute(device, command_buffer, &render_attachments, render_area);
        }

        // convert the swapchain image back to a presentable format
        let present_barrier = vk::ImageMemoryBarrier {
            src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_access_mask: vk::AccessFlags::empty(),
            old_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..barrier
        };

        device.cmd_pipeline_barrier(
            self.command_buffer,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[present_barrier],
        );
        device.end_command_buffer(self.command_buffer)?;
        Ok(())
    }
}
