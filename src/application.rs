use std::{sync::Arc, time::Instant};

use crate::{frame::FrameData, transform::Transform};

use super::setup::*;
use anyhow::Result;
use ash::vk;
use glam::{vec4, Mat4, Vec3, Vec4};

pub struct Camera {
    transform: Transform,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn build_proj(&self) -> CameraUniformData {
        let view = Mat4::look_at_rh(
            self.transform.translation,
            self.transform.forward(),
            self.transform.down(),
        );

        let mut proj =
            Mat4::perspective_rh_gl(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);

        proj.x_axis.x *= -1.0;

        let dir = self.transform.forward();
        let uniform_data = CameraUniformData {
            view_proj: proj * view,
            look_dir: vec4(dir.x, dir.y, dir.z, 1.0),
        };

        uniform_data
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct CameraUniformData {
    view_proj: Mat4,
    look_dir: Vec4,
}

#[repr(C)]
pub struct VulkanDevice {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub pdevice: vk::PhysicalDevice,
    pub device: ash::Device,
    pub queues: DeviceQueues,
}

pub struct Application {
    pub device: Arc<VulkanDevice>,

    pub surface: vk::SurfaceKHR,
    pub surface_loader: ash::khr::surface::Instance,
    pub swapchain: Swapchain,

    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,

    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,

    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_layout: vk::DescriptorSetLayout,

    pub frames: Vec<FrameData>,
    pub frame_buffers: Vec<vk::Framebuffer>,

    pub frame: usize,
    pub start: Instant,

    pub camera: Camera,
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

        let bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .stage_flags(
                vk::ShaderStageFlags::COMPUTE
                    | vk::ShaderStageFlags::VERTEX
                    | vk::ShaderStageFlags::FRAGMENT,
            )
            .descriptor_count(1)];

        let descriptor_layout = device.create_descriptor_set_layout(
            &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
            None,
        )?;

        let set_layouts = [descriptor_layout];
        let layout_create_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);

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

        let sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(10)];

        let descriptor_pool = device.create_descriptor_pool(
            &vk::DescriptorPoolCreateInfo::default()
                .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
                .max_sets(10)
                .pool_sizes(&sizes),
            None,
        )?;

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
            .map(|_| {
                FrameData::new(vulkan_device.clone(), descriptor_layout, descriptor_pool).unwrap()
            })
            .collect();

        let camera = Camera {
            transform: Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
            aspect: win_width as f32 / win_height as f32,
            fovy: 70.0,
            zfar: 100.0,
            znear: 0.01,
        };

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
            descriptor_layout,
            descriptor_pool,
            frame: 0,
            start: Instant::now(),
            camera,
        })
    }

    pub unsafe fn on_resize(&mut self, new_size: [u32; 2]) -> Result<()> {
        self.device.device.device_wait_idle()?;
        self.frames
            .drain(..)
            .for_each(|v| v.destroy(self.descriptor_pool));

        for buffer in &self.frame_buffers {
            self.device.device.destroy_framebuffer(*buffer, None);
        }

        self.swapchain.recreate(&self.device.device, new_size)?;

        self.camera.aspect = new_size[0] as f32 / new_size[1] as f32;

        self.frame_buffers = self
            .swapchain
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

                self.device
                    .device
                    .create_framebuffer(&create_info, None)
                    .unwrap()
            })
            .collect();

        self.frames = self
            .swapchain
            .image_views
            .iter()
            .map(|_| {
                FrameData::new(
                    self.device.clone(),
                    self.descriptor_layout,
                    self.descriptor_pool,
                )
                .unwrap()
            })
            .collect();

        Ok(())
    }

    pub unsafe fn on_render(&mut self) -> Result<()> {
        self.frame = (self.frame + 1) % self.frames.len();

        let t = self.start.elapsed().as_secs_f32();

        self.camera.transform =
            Transform::from_xyz(t.cos() * 3.0, 1.0, t.sin()).looking_at(Vec3::ZERO, Vec3::Y);

        self.frames[self.frame].camera_data = self.camera.build_proj();

        self.frames[self.frame].render(
            self.pipeline,
            self.pipeline_layout,
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
            descriptor_pool,
            descriptor_layout,
            ..
        } = self;

        device.device.device_wait_idle().unwrap();

        for frame in self.frames {
            frame.destroy(self.descriptor_pool);
        }

        device
            .device
            .destroy_descriptor_set_layout(descriptor_layout, None);
        device.device.destroy_descriptor_pool(descriptor_pool, None);

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
