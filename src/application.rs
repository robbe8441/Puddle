use super::setup::*;
use anyhow::Result;
use ash::vk;

#[repr(C)]
pub struct Application {
    instance: ash::Instance,
    entry: ash::Entry,

    pub pdevice: vk::PhysicalDevice,
    device: ash::Device,
    queues: DeviceQueues,

    pub surface: vk::SurfaceKHR,
    pub surface_loader: ash::khr::surface::Instance,
    swapchain: Swapchain,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,

    command_buffers: Vec<vk::CommandBuffer>,
    frame_buffers: Vec<vk::Framebuffer>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    in_flight_images: Vec<vk::Fence>,

    graphics_command_pool: vk::CommandPool,

    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,

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

        let (graphics_family, graphics_queue) = queues.get_graphics_queue().unwrap();

        let graphics_command_pool = create_command_pool(&device, graphics_family)?;

        // handle uploading startup stuff
        {
            let startup_buffer = create_command_buffers(&device, graphics_command_pool, 1)?[0];

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
            device.queue_submit(*graphics_queue, &submits, startup_fence)?;

            device.wait_for_fences(&[startup_fence], true, u64::MAX)?;

            device.destroy_fence(startup_fence, None);
            device.free_command_buffers(graphics_command_pool, &startup_buffers);
        }

        let layout_create_info = vk::PipelineLayoutCreateInfo::default();

        let pipeline_layout = device.create_pipeline_layout(&layout_create_info, None)?;

        let render_pass = create_render_pass(&device, &swapchain)?;

        let pipeline = create_pipeline(&device, &swapchain, pipeline_layout, render_pass)?;

        let frame_buffers = create_framebuffers(
            &device,
            render_pass,
            [win_width as u32, win_height as u32],
            &swapchain.image_views,
        );

        let command_buffers =
            create_command_buffers(&device, graphics_command_pool, frame_buffers.len() as u32)?;

        for (i, command_buffer) in command_buffers.iter().enumerate() {
            record_buffer(
                &device,
                *command_buffer,
                [win_width as u32, win_height as u32],
                render_pass,
                frame_buffers[i],
                pipeline,
                vertex_buffer,
            )?;
        }

        drop(graphics_queue);

        let image_available_semaphores: Vec<_> = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                device
                    .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                    .unwrap()
            })
            .collect();

        let render_finished_semaphores: Vec<_> = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                device
                    .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                    .unwrap()
            })
            .collect();

        let in_flight_fences: Vec<_> = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                device
                    .create_fence(
                        &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                        None,
                    )
                    .unwrap()
            })
            .collect();

        let in_flight_images = vec![vk::Fence::null(); frame_buffers.len()];

        Ok(Self {
            instance,
            entry,
            pdevice,
            device,
            queues,
            surface,
            surface_loader,
            swapchain,
            vertex_buffer,
            vertex_buffer_memory,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            in_flight_images,
            graphics_command_pool,
            command_buffers,
            frame_buffers,
            pipeline,
            pipeline_layout,
            render_pass,
            frame: 0,
        })
    }

    pub unsafe fn on_resize(&mut self, new_size: [u32; 2]) -> Result<()> {
        self.device.device_wait_idle()?;

        for buffer in &self.frame_buffers {
            unsafe { self.device.destroy_framebuffer(*buffer, None) };
        }

        self.device
            .free_command_buffers(self.graphics_command_pool, &self.command_buffers);

        self.swapchain.recreate(&self.device, new_size)?;

        self.frame_buffers = create_framebuffers(
            &self.device,
            self.render_pass,
            new_size,
            &self.swapchain.image_views,
        );

        self.command_buffers = create_command_buffers(
            &self.device,
            self.graphics_command_pool,
            self.frame_buffers.len() as u32,
        )?;

        for (i, command_buffer) in self.command_buffers.iter().enumerate() {
            record_buffer(
                &self.device,
                *command_buffer,
                new_size,
                self.render_pass,
                self.frame_buffers[i],
                self.pipeline,
                self.vertex_buffer,
            )?;
        }

        Ok(())
    }

    pub unsafe fn on_render(&mut self) -> Result<()> {
        let (_graphics_family, graphics_queue) = self.queues.get_graphics_queue().unwrap();

        render(
            &self.device,
            *graphics_queue,
            &self.swapchain,
            &self.command_buffers,
            &self.image_available_semaphores,
            &self.render_finished_semaphores,
            &self.in_flight_fences,
            &mut self.in_flight_images,
            self.frame,
        )?;

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    pub unsafe fn destroy(self) {
        let Application {
            instance,
            device,
            surface,
            surface_loader,
            swapchain,
            vertex_buffer,
            vertex_buffer_memory,
            command_buffers,
            frame_buffers,
            pipeline,
            pipeline_layout,
            render_pass,
            in_flight_fences,
            image_available_semaphores,
            render_finished_semaphores,
            graphics_command_pool,
            ..
        } = self;

        device.device_wait_idle().unwrap();

        device.destroy_buffer(vertex_buffer, None);
        device.free_memory(vertex_buffer_memory, None);

        for buffer in frame_buffers {
            device.destroy_framebuffer(buffer, None);
        }
        swapchain.destroy(&device);
        surface_loader.destroy_surface(surface, None);

        for semaphore in image_available_semaphores {
            device.destroy_semaphore(semaphore, None);
        }

        for semaphore in render_finished_semaphores {
            device.destroy_semaphore(semaphore, None);
        }

        for fence in in_flight_fences {
            device.destroy_fence(fence, None);
        }

        device.free_command_buffers(graphics_command_pool, &command_buffers);
        device.destroy_command_pool(graphics_command_pool, None);

        device.destroy_pipeline(pipeline, None);
        device.destroy_render_pass(render_pass, None);
        device.destroy_pipeline_layout(pipeline_layout, None);

        device.destroy_device(None);
        instance.destroy_instance(None);
    }
}
