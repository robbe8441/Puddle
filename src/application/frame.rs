use crate::vulkan::{Buffer, PushConstant, Swapchain, VulkanContext};
use ash::vk::{self, Handle};

use super::camera::CameraUniformData;

pub struct FrameData {
    graphics_command_pool: vk::CommandPool,
    graphics_command_buffers: Vec<vk::CommandBuffer>,

    compute_command_pool: vk::CommandPool,
    compute_command_buffers: Vec<vk::CommandBuffer>,

    vk_ctx: *const VulkanContext,
    camera_buffer: Buffer,
    camera_handle: u32,

    // wait for the image to be available before rendering
    pub image_available_semaphore: vk::Semaphore,

    // wait for rendering before presenting
    pub render_complete_semaphore: vk::Semaphore,

    // tells if the current frame_data is currently rendering
    // if so, then the next frame needs to wait
    in_use_fence: vk::Fence,
}

impl FrameData {
    pub fn new(vk_ctx: &VulkanContext) -> Result<Self, vk::Result> {
        let graphics_family = vk_ctx.queues.get_graphics_queue().0;
        let compute_family = vk_ctx.queues.get_compute_queue().0;

        let graphics_pool_info =
            vk::CommandPoolCreateInfo::default().queue_family_index(graphics_family);

        let compute_pool_info =
            vk::CommandPoolCreateInfo::default().queue_family_index(compute_family);

        let graphics_command_pool =
            unsafe { vk_ctx.device.create_command_pool(&graphics_pool_info, None) }?;

        let compute_command_pool =
            unsafe { vk_ctx.device.create_command_pool(&compute_pool_info, None) }?;

        let camera_buffer = vk_ctx.create_buffer(
            std::mem::size_of::<CameraUniformData>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let camera_handle = vk_ctx
            .bindless_handler
            .store_buffer(&vk_ctx.device, &camera_buffer)
            .0;

        let in_use_fence = unsafe {
            vk_ctx.device.create_fence(
                &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                None,
            )
        }?;

        let image_available_semaphore = unsafe {
            vk_ctx
                .device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        }?;

        let render_complete_semaphore = unsafe {
            vk_ctx
                .device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        }?;

        Ok(Self {
            graphics_command_pool,
            graphics_command_buffers: vec![],
            compute_command_buffers: vec![],
            compute_command_pool,
            vk_ctx: std::ptr::from_ref(vk_ctx),
            camera_buffer,
            camera_handle,
            image_available_semaphore,
            render_complete_semaphore,
            in_use_fence,
        })
    }

    #[allow(clippy::too_many_lines)]
    pub unsafe fn render(
        &mut self,
        swapchain: &mut Swapchain,
        pipeline: vk::Pipeline,
        vk_ctx: &VulkanContext,
    ) -> Result<(), vk::Result> {
        vk_ctx
            .device
            .wait_for_fences(&[self.in_use_fence], true, u64::MAX)?;

        let (image_index, _) = unsafe {
            swapchain.loader.acquire_next_image(
                swapchain.handle,
                u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
            )
        }?;

        // check if the image is still in use
        // if so, wait for it to finish
        let image_fence = &mut swapchain.image_use_fences[image_index as usize];
        if !image_fence.is_null() {
            vk_ctx
                .device
                .wait_for_fences(&[*image_fence], true, u64::MAX)?;
        }

        *image_fence = self.in_use_fence;
        vk_ctx.device.reset_fences(&[self.in_use_fence])?;

        // free all the command_buffers that are now finished executing
        if !self.graphics_command_buffers.is_empty() {
            vk_ctx
                .device
                .free_command_buffers(self.graphics_command_pool, &self.graphics_command_buffers);
            self.graphics_command_buffers.clear();
        }

        if !self.compute_command_buffers.is_empty() {
            vk_ctx
                .device
                .free_command_buffers(self.compute_command_pool, &self.compute_command_buffers);
            self.compute_command_buffers.clear();
        }

        let command_buffer = vk_ctx.device.allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::default()
                .command_pool(self.graphics_command_pool)
                .command_buffer_count(1),
        )?[0];

        self.graphics_command_buffers.push(command_buffer);

        vk_ctx.device.begin_command_buffer(
            command_buffer,
            &vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
        )?;

        vk_ctx.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            vk_ctx.bindless_handler.pipeline_layout,
            0,
            &[vk_ctx.bindless_handler.descriptor_set],
            &[],
        );

        let constants = [PushConstant {
            camera: self.camera_handle,
            swapchain_image: swapchain.image_handles[image_index as usize],
            insance_array: 0,
            instance_count: 0,
        }];

        vk_ctx.device.cmd_push_constants(
            command_buffer,
            vk_ctx.bindless_handler.pipeline_layout,
            vk::ShaderStageFlags::COMPUTE,
            0,
            bytemuck::cast_slice(&constants),
        );

        vk_ctx
            .device
            .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline);

        let barrier_to_general = vk::ImageMemoryBarrier {
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::SHADER_WRITE,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::GENERAL,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: swapchain.images[image_index as usize],
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };

        vk_ctx.device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier_to_general],
        );

        // let image_size = swapchain.create_info.image_extent;

        vk_ctx
            .device
            .cmd_dispatch(command_buffer, 1920 / 32, 1080 / 32, 1);

        let barrier_to_present = vk::ImageMemoryBarrier {
            src_access_mask: vk::AccessFlags::SHADER_WRITE,
            dst_access_mask: vk::AccessFlags::MEMORY_READ,
            old_layout: vk::ImageLayout::GENERAL,
            new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..barrier_to_general
        };

        vk_ctx.device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier_to_present],
        );

        vk_ctx.device.end_command_buffer(command_buffer)?;

        let wait_semaphores = [self.image_available_semaphore];
        let signal_semaphores = [self.render_complete_semaphore];
        let command_buffers = [command_buffer];

        let submit_info = [vk::SubmitInfo::default()
            .command_buffers(&command_buffers)
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::BOTTOM_OF_PIPE])
            .signal_semaphores(&signal_semaphores)];

        let graphics_queue = vk_ctx.queues.get_graphics_queue().1.unwrap();

        vk_ctx
            .device
            .queue_submit(*graphics_queue, &submit_info, self.in_use_fence)?;

        let image_indices = [image_index];
        let swapchains = [swapchain.handle];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .image_indices(&image_indices)
            .swapchains(&swapchains);

        swapchain
            .loader
            .queue_present(*graphics_queue, &present_info)?;

        Ok(())
    }

    pub unsafe fn destroy(&self) {
        unsafe {
            let ctx = &*self.vk_ctx;

            // wait for the frame to finish rendering
            let _ = ctx
                .device
                .wait_for_fences(&[self.in_use_fence], true, u64::MAX);

            ctx.device.destroy_fence(self.in_use_fence, None);
            ctx.device
                .destroy_command_pool(self.graphics_command_pool, None);
            ctx.device
                .destroy_command_pool(self.compute_command_pool, None);
            ctx.device
                .destroy_semaphore(self.image_available_semaphore, None);
            ctx.device
                .destroy_semaphore(self.render_complete_semaphore, None);

            self.camera_buffer.destroy(&ctx.device);
        }
    }
}
