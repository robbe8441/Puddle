use glam::Mat4;
use std::sync::Arc;

use super::setup::*;
use anyhow::Result;
use ash::{khr::swapchain, vk};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CameraUniformData {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
}

#[repr(C)]
pub struct VulkanDevice {
    entry: ash::Entry,
    instance: ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: ash::Device,
    queues: DeviceQueues,
}

pub struct FrameData {
    device: Arc<VulkanDevice>,
    graphics_command_buffers: Vec<vk::CommandBuffer>,
    compute_command_buffers: Vec<vk::CommandBuffer>,

    graphics_command_pool: vk::CommandPool,
    compute_command_pool: vk::CommandPool,

    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,

    frame_finished_fence: vk::Fence,
}

impl FrameData {
    pub unsafe fn new(device: Arc<VulkanDevice>) -> Result<Self> {
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

        Ok(Self {
            device,
            graphics_command_buffers: vec![],
            compute_command_buffers: vec![],
            graphics_command_pool: graphics_pool,
            compute_command_pool: compute_pool,
            image_available_semaphore,
            render_finished_semaphore,
            frame_finished_fence,
        })
    }

    pub unsafe fn render(
        &mut self,
        pipeline: vk::Pipeline,
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

        vk_device.cmd_draw(
            graphics_command_buffer,
            Vertex::VERTICES.len() as u32,
            1,
            0,
            0,
        );

        vk_device.cmd_end_render_pass(graphics_command_buffer);
        vk_device.end_command_buffer(graphics_command_buffer)?;

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

    pub unsafe fn destroy(self) {
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

        vk_device.destroy_command_pool(self.graphics_command_pool, None);
        vk_device.destroy_command_pool(self.compute_command_pool, None);

        vk_device.destroy_semaphore(self.image_available_semaphore, None);
        vk_device.destroy_semaphore(self.render_finished_semaphore, None);

        vk_device.destroy_fence(self.frame_finished_fence, None);
    }
}

pub struct Application {
    device: Arc<VulkanDevice>,

    pub surface: vk::SurfaceKHR,
    pub surface_loader: ash::khr::surface::Instance,
    swapchain: Swapchain,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,

    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,

    frames: Vec<FrameData>,
    frame_buffers: Vec<vk::Framebuffer>,

    frame: usize,
}

impl Application {
    pub unsafe fn new(window: &glfw::PWindow) -> Result<Self> {
        let (instance, entry) = create_instance(window)?;

        let (surface, surface_loader) = create_surface(&entry, &instance, window)?;

        let pdevice = get_physical_device(&instance, &surface_loader, surface)?;

        let (device, queues) = create_device(&instance, pdevice)?;

        let (win_width, win_height) = window.get_size();

        let swapchain = Swapchain::create_swapchain(
            pdevice,
            &device,
            &instance,
            &surface_loader,
            surface,
            [win_width as u32, win_height as u32],
        )?;

        let vertex_data = Vertex::VERTICES;

        let (vertex_buffer, vertex_buffer_memory) = create_buffer(
            &instance,
            &device,
            pdevice,
            std::mem::size_of_val(&vertex_data) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let (graphics_family, graphics_queue) = queues.get_graphics_queue();

        let startup_pool = create_command_pool(&device, graphics_family)?;

        // handle uploading startup stuff
        {
            let startup_buffer = create_command_buffers(&device, startup_pool, 1)?[0];

            let startup_fence = device.create_fence(&vk::FenceCreateInfo::default(), None)?;

            device.begin_command_buffer(
                startup_buffer,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;

            let vertex_bytes = bytemuck::cast_slice(&vertex_data);
            device.cmd_update_buffer(startup_buffer, vertex_buffer, 0, vertex_bytes);
            device.end_command_buffer(startup_buffer)?;

            let startup_buffers = [startup_buffer];
            let submits = [vk::SubmitInfo::default().command_buffers(&startup_buffers)];
            device.queue_submit(*graphics_queue.unwrap(), &submits, startup_fence)?;

            device.wait_for_fences(&[startup_fence], true, u64::MAX)?;

            device.destroy_fence(startup_fence, None);
            device.free_command_buffers(startup_pool, &startup_buffers);
        }

        device.destroy_command_pool(startup_pool, None);

        let layout_create_info = vk::PipelineLayoutCreateInfo::default();

        let pipeline_layout = device.create_pipeline_layout(&layout_create_info, None)?;

        let render_pass = create_render_pass(&device, &swapchain)?;

        let pipeline = create_pipeline(&device, &swapchain, pipeline_layout, render_pass)?;

        let frame_buffers = swapchain
            .image_views
            .iter()
            .map(|image_view| {
                let attachments = &[*image_view];
                let create_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(attachments)
                    .width(win_width as u32)
                    .height(win_height as u32)
                    .layers(1);

                device.create_framebuffer(&create_info, None).unwrap()
            })
            .collect();

        let vulkan_device = Arc::new(VulkanDevice {
            entry,
            instance,
            pdevice,
            device,
            queues,
        });

        let frames = swapchain
            .image_views
            .iter()
            .map(|_| FrameData::new(vulkan_device.clone()).unwrap())
            .collect();

        Ok(Self {
            device: vulkan_device,
            frames,
            frame_buffers,
            surface,
            surface_loader,
            swapchain,
            vertex_buffer,
            vertex_buffer_memory,
            pipeline,
            pipeline_layout,
            render_pass,
            frame: 0,
        })
    }

    pub unsafe fn on_resize(&mut self, new_size: [u32; 2]) -> Result<()> {
        self.device.device.device_wait_idle()?;
        self.frames.drain(..).for_each(|v| v.destroy());

        for buffer in &self.frame_buffers {
            self.device.device.destroy_framebuffer(*buffer, None);
        }

        self.swapchain.recreate(&self.device.device, new_size)?;

        self.frame_buffers = self.swapchain
            .image_views
            .iter()
            .map(|image_view| {
                let attachments = &[*image_view];
                let create_info = vk::FramebufferCreateInfo::default()
                    .render_pass(self.render_pass)
                    .attachments(attachments)
                    .width(new_size[0])
                    .height(new_size[1])
                    .layers(1);

                self.device.device.create_framebuffer(&create_info, None).unwrap()
            })
            .collect();

        self.frames = self
            .swapchain
            .image_views
            .iter()
            .map(|_| FrameData::new(self.device.clone()).unwrap())
            .collect();

        Ok(())
    }

    pub unsafe fn on_render(&mut self) -> Result<()> {
        self.frame = (self.frame + 1) % self.frames.len();

        self.frames[self.frame].render(
            self.pipeline,
            self.render_pass,
            self.vertex_buffer,
            &self.swapchain,
            &self.frame_buffers,
        )?;

        Ok(())
    }

    pub unsafe fn destroy(self) {
        let Application {
            device,
            surface,
            surface_loader,
            swapchain,
            vertex_buffer,
            vertex_buffer_memory,
            pipeline,
            pipeline_layout,
            render_pass,
            ..
        } = self;

        device.device.device_wait_idle().unwrap();

        for frame in self.frames {
            frame.destroy();
        }

        for buffer in self.frame_buffers {
            device.device.destroy_framebuffer(buffer, None);
        }

        device.device.destroy_buffer(vertex_buffer, None);
        device.device.free_memory(vertex_buffer_memory, None);

        swapchain.destroy(&device.device);
        surface_loader.destroy_surface(surface, None);

        device.device.destroy_pipeline(pipeline, None);
        device.device.destroy_render_pass(render_pass, None);
        device.device.destroy_pipeline_layout(pipeline_layout, None);

        device.device.destroy_device(None);
        device.instance.destroy_instance(None);
    }
}
