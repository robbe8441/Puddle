use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use glam::{Mat4, Vec4};

use crate::{
    application::{CameraUniformData, VulkanDevice},
    setup::{create_buffer, Swapchain, Vertex},
};

pub struct FrameData {
    pub device: Arc<VulkanDevice>,
    pub graphics_command_buffers: Vec<vk::CommandBuffer>,
    pub compute_command_buffers: Vec<vk::CommandBuffer>,

    pub graphics_command_pool: vk::CommandPool,
    pub compute_command_pool: vk::CommandPool,

    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,

    pub frame_finished_fence: vk::Fence,

    pub descriptor_set: vk::DescriptorSet,

    pub camera_data: CameraUniformData,
    pub camera_uniform_buffer: vk::Buffer,
    pub camera_uniform_memory: vk::DeviceMemory,
    pub camera_align: ash::util::Align<CameraUniformData>,
}

impl FrameData {
    pub unsafe fn new(
        device: Arc<VulkanDevice>,
        descriptor_layout: vk::DescriptorSetLayout,
        descriptor_pool: vk::DescriptorPool,
    ) -> Result<Self> {
        let graphics_create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(device.queues.get_graphics_queue().0);

        let graphics_pool = device
            .device
            .create_command_pool(&graphics_create_info, None)?;

        let compute_create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(device.queues.get_compute_queue().0);

        let compute_pool = device
            .device
            .create_command_pool(&compute_create_info, None)?;

        let image_available_semaphore = device
            .device
            .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?;

        let render_finished_semaphore = device
            .device
            .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?;

        let frame_finished_fence = device.device.create_fence(
            &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
            None,
        )?;

        let descriptor_set = device.device.allocate_descriptor_sets(
            &vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&[descriptor_layout]),
        )?[0];

        let (camera_uniform_buffer, camera_uniform_memory) = create_buffer(
            &device.instance,
            &device.device,
            device.pdevice,
            std::mem::size_of::<CameraUniformData>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let ptr = device.device.map_memory(
            camera_uniform_memory,
            0,
            std::mem::size_of::<CameraUniformData>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;

        let requirements = device
            .device
            .get_buffer_memory_requirements(camera_uniform_buffer);

        let camera_align = ash::util::Align::new(ptr, requirements.alignment, requirements.size);

        let camera_buffer_info = [vk::DescriptorBufferInfo::default()
            .buffer(camera_uniform_buffer)
            .offset(0)
            .range(std::mem::size_of::<CameraUniformData>() as u64)];

        let write_descriptors = [vk::WriteDescriptorSet::default()
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .dst_binding(0)
            .dst_set(descriptor_set)
            .dst_array_element(0)
            .buffer_info(&camera_buffer_info)];

        device
            .device
            .update_descriptor_sets(&write_descriptors, &[]);

        Ok(Self {
            device,
            graphics_command_buffers: vec![],
            compute_command_buffers: vec![],
            graphics_command_pool: graphics_pool,
            compute_command_pool: compute_pool,
            image_available_semaphore,
            render_finished_semaphore,
            frame_finished_fence,
            descriptor_set,
            camera_uniform_buffer,
            camera_uniform_memory,
            camera_align,
            camera_data: CameraUniformData::default(),
        })
    }

    pub unsafe fn render(
        &mut self,
        pipeline: vk::Pipeline,
        pipeline_layout: vk::PipelineLayout,
        renderpass: vk::RenderPass,
        vertex_buffer: vk::Buffer,
        swapchain: &Swapchain,
        frame_buffers: &[vk::Framebuffer],
    ) -> Result<()> {
        let vk_device = &self.device.device;

        vk_device.wait_for_fences(&[self.frame_finished_fence], true, u64::MAX)?;
        vk_device.reset_fences(&[self.frame_finished_fence])?;

        let (image_index, _suboptimal) = swapchain.loader.acquire_next_image(
            swapchain.handle,
            u64::MAX,
            self.image_available_semaphore,
            vk::Fence::null(),
        )?;

        if !self.graphics_command_buffers.is_empty() {
            vk_device
                .free_command_buffers(self.graphics_command_pool, &self.graphics_command_buffers);
            self.graphics_command_buffers.clear();
        }
        if !self.compute_command_buffers.is_empty() {
            vk_device
                .free_command_buffers(self.compute_command_pool, &self.compute_command_buffers);
            self.compute_command_buffers.clear();
        }

        let graphics_command_buffer = vk_device.allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::default()
                .command_pool(self.graphics_command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1),
        )?[0];

        let image_extent = swapchain.create_info.image_extent;

        vk_device.begin_command_buffer(
            graphics_command_buffer,
            &vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
        )?;

        let render_area = vk::Rect2D::default()
            .offset(vk::Offset2D::default())
            .extent(image_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let clear_values = &[color_clear_value];

        let viewports = [vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(image_extent.width as f32)
            .height(image_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)];

        let scissors = [vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(image_extent)];

        vk_device.cmd_set_viewport(graphics_command_buffer, 0, &viewports);
        vk_device.cmd_set_scissor(graphics_command_buffer, 0, &scissors);

        let info = vk::RenderPassBeginInfo::default()
            .render_pass(renderpass)
            .framebuffer(frame_buffers[image_index as usize])
            .render_area(render_area)
            .clear_values(clear_values);

        vk_device.cmd_begin_render_pass(
            graphics_command_buffer,
            &info,
            vk::SubpassContents::INLINE,
        );

        vk_device.cmd_bind_pipeline(
            graphics_command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline,
        );

        vk_device.cmd_bind_vertex_buffers(graphics_command_buffer, 0, &[vertex_buffer], &[0]);

        vk_device.cmd_bind_descriptor_sets(
            graphics_command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout,
            0,
            &[self.descriptor_set],
            &[],
        );

        vk_device.cmd_draw(
            graphics_command_buffer,
            Vertex::VERTICES.len() as u32,
            1,
            0,
            0,
        );

        vk_device.cmd_end_render_pass(graphics_command_buffer);
        vk_device.end_command_buffer(graphics_command_buffer)?;

        self.camera_align.copy_from_slice(&[self.camera_data]);

        self.graphics_command_buffers.push(graphics_command_buffer);

        let graphics_queue = self.device.queues.get_graphics_queue().1.unwrap();

        let wait_semaphores = &[self.image_available_semaphore];
        let signal_semaphores = &[self.render_finished_semaphore];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(&self.graphics_command_buffers)
            .signal_semaphores(signal_semaphores);

        vk_device.queue_submit(*graphics_queue, &[submit_info], self.frame_finished_fence)?;

        let swapchains = &[swapchain.handle];
        let image_indices = &[image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        swapchain
            .loader
            .queue_present(*graphics_queue, &present_info)?;

        Ok(())
    }

    pub unsafe fn destroy(self, descriptor_pool: vk::DescriptorPool) {
        let vk_device = &self.device.device;

        vk_device
            .wait_for_fences(&[self.frame_finished_fence], true, u64::MAX)
            .unwrap();

        if !self.graphics_command_buffers.is_empty() {
            vk_device
                .free_command_buffers(self.graphics_command_pool, &self.graphics_command_buffers);
        }
        if !self.compute_command_buffers.is_empty() {
            vk_device
                .free_command_buffers(self.compute_command_pool, &self.compute_command_buffers);
        }

        let _ = vk_device.free_descriptor_sets(descriptor_pool, &[self.descriptor_set]);

        vk_device.destroy_buffer(self.camera_uniform_buffer, None);
        vk_device.free_memory(self.camera_uniform_memory, None);

        vk_device.destroy_command_pool(self.graphics_command_pool, None);
        vk_device.destroy_command_pool(self.compute_command_pool, None);

        vk_device.destroy_semaphore(self.image_available_semaphore, None);
        vk_device.destroy_semaphore(self.render_finished_semaphore, None);

        vk_device.destroy_fence(self.frame_finished_fence, None);
    }
}
