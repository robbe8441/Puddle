use std::{cell::UnsafeCell, io::Cursor};

use ash::vk;
use ash::prelude::VkResult;
use rendering::vulkan::{Swapchain, VulkanDevice};

pub struct Application {
    vk_device: VulkanDevice,
    swapchain: Swapchain,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    window: glfw::PWindow,
    glfw_ctx: glfw::Glfw,
    glfw_events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    render_finished_semaphore: vk::Semaphore,
    image_available_semaphore: vk::Semaphore,
    execution_finished_fence: vk::Fence,
    command_bufer: UnsafeCell<Option<vk::CommandBuffer>>,
    shaders: [vk::ShaderEXT; 3],
}

impl Application {
    unsafe fn new() -> VkResult<Self> {
        let mut glfw_ctx = glfw::init(glfw::fail_on_errors).unwrap();

        let (mut window, glfw_events) = glfw_ctx
            .create_window(800, 600, "Puddle triangle", glfw::WindowMode::Windowed)
            .unwrap();

        window.set_all_polling(true);

        let vk_device = VulkanDevice::new(&window)?;

        #[allow(clippy::cast_sign_loss)]
        let window_size = {
            let v = window.get_size();
            [v.0 as u32, v.1 as u32]
        };

        let swapchain = Swapchain::new(&vk_device, window_size)?;

        let (queue_family, queue) = vk_device.queues.graphics;

        let command_pool = {
            let create_info = vk::CommandPoolCreateInfo::default().queue_family_index(queue_family);
            vk_device.create_command_pool(&create_info, None)
        }?;

        let image_available_semaphore =
            vk_device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?;

        let render_finished_semaphore =
            vk_device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?;

        let execution_finished_fence = vk_device.create_fence(
            &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
            None,
        )?;

        let mut code = Cursor::new(include_bytes!("../shaders/shader_opt.spv"));
        let byte_code = ash::util::read_spv(&mut code).unwrap();

        let mut code2 = Cursor::new(include_bytes!("../shaders/quad.spv"));
        let byte_code2 = ash::util::read_spv(&mut code2).unwrap();

        let shader_info = vk::ShaderCreateInfoEXT::default()
            .code_type(vk::ShaderCodeTypeEXT::SPIRV)
            .code(bytemuck::cast_slice(&byte_code));

        let shader_info2 = vk::ShaderCreateInfoEXT::default()
            .code_type(vk::ShaderCodeTypeEXT::SPIRV)
            .code(bytemuck::cast_slice(&byte_code2));

        let shader_crate_infos = [
            vk::ShaderCreateInfoEXT {
                stage: vk::ShaderStageFlags::VERTEX,
                next_stage: vk::ShaderStageFlags::FRAGMENT,
                p_name: c"main".as_ptr(),
                ..shader_info
            },
            vk::ShaderCreateInfoEXT {
                stage: vk::ShaderStageFlags::VERTEX,
                p_name: c"main".as_ptr(),
                next_stage: vk::ShaderStageFlags::FRAGMENT,
                ..shader_info2
            },
            vk::ShaderCreateInfoEXT {
                stage: vk::ShaderStageFlags::FRAGMENT,
                p_name: c"main".as_ptr(),
                ..shader_info
            },
        ];
        let shaders = vk_device
            .shader_device
            .create_shaders(&shader_crate_infos, None)
            .unwrap()
            .try_into()
            .unwrap();

        Ok(Self {
            vk_device,
            swapchain,
            command_pool,
            shaders,
            queue,
            window,
            glfw_ctx,
            glfw_events,
            render_finished_semaphore,
            image_available_semaphore,
            execution_finished_fence,
            command_bufer: UnsafeCell::new(None),
        })
    }

    #[allow(clippy::too_many_lines)]
    unsafe fn draw(&self) {
        let vk_device = &self.vk_device;

        let _ = vk_device.wait_for_fences(&[self.execution_finished_fence], true, u64::MAX);
        vk_device
            .reset_fences(&[self.execution_finished_fence])
            .unwrap();

        if let Some(buffer) = *self.command_bufer.get() {
            vk_device.free_command_buffers(self.command_pool, &[buffer]);
        }

        let (image_index, _suboptimal) = self
            .swapchain
            .loader
            .acquire_next_image(
                *self.swapchain.handle.get(),
                u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
            )
            .unwrap();

        let command_buffer = unsafe {
            let create_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(self.command_pool)
                .command_buffer_count(1);
            vk_device.allocate_command_buffers(&create_info).unwrap()[0]
        };

        vk_device
            .begin_command_buffer(
                command_buffer,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )
            .unwrap();


        let swapchain_views = &*self.swapchain.image_views.get();
        let swapchain_images = self
            .swapchain
            .loader
            .get_swapchain_images(*self.swapchain.handle.get())
            .unwrap();


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

        let present_barrier = vk::ImageMemoryBarrier {
            src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_access_mask: vk::AccessFlags::empty(),
            old_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..barrier
        };

        vk_device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );

        let image_size = (*self.swapchain.create_info.get()).image_extent;

        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.05, 0.01, 0.07, 1.0],
            },
        };

        let color_attachments = [vk::RenderingAttachmentInfo::default()
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(clear_value)
            .image_view(swapchain_views[image_index as usize])
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

        let render_begin = vk::RenderingInfo::default()
            .render_area(vk::Rect2D::default().extent(image_size))
            .layer_count(1)
            .view_mask(0)
            .color_attachments(&color_attachments);

        vk_device.cmd_begin_rendering(command_buffer, &render_begin);

        vk_device.cmd_set_viewport_with_count(
            command_buffer,
            &[vk::Viewport::default()
                .width(image_size.width as f32)
                .height(image_size.height as f32)],
        );

        vk_device.cmd_set_scissor_with_count(
            command_buffer,
            &[vk::Rect2D::default().extent(image_size)],
        );

        let s_device = &vk_device.shader_device;

        s_device.cmd_set_vertex_input(command_buffer, &[], &[]);
        s_device.cmd_set_rasterizer_discard_enable(command_buffer, false);
        s_device.cmd_set_polygon_mode(command_buffer, vk::PolygonMode::FILL);
        s_device.cmd_set_rasterization_samples(command_buffer, vk::SampleCountFlags::TYPE_1);
        s_device.cmd_set_sample_mask(command_buffer, vk::SampleCountFlags::TYPE_1, &[1]);
        s_device.cmd_set_alpha_to_coverage_enable(command_buffer, false);
        s_device.cmd_set_cull_mode(command_buffer, vk::CullModeFlags::NONE);
        s_device.cmd_set_depth_test_enable(command_buffer, false);
        s_device.cmd_set_depth_write_enable(command_buffer, false);
        s_device.cmd_set_depth_bias_enable(command_buffer, false);
        s_device.cmd_set_stencil_test_enable(command_buffer, false);
        s_device.cmd_set_primitive_topology(command_buffer, vk::PrimitiveTopology::TRIANGLE_LIST);
        s_device.cmd_set_primitive_restart_enable(command_buffer, false);
        s_device.cmd_set_color_blend_enable(command_buffer, 0, &[0]);
        s_device.cmd_set_color_blend_equation(
            command_buffer,
            0,
            &[vk::ColorBlendEquationEXT::default()],
        );
        s_device.cmd_set_color_write_mask(command_buffer, 0, &[vk::ColorComponentFlags::RGBA]);

        let stages = [vk::ShaderStageFlags::VERTEX, vk::ShaderStageFlags::FRAGMENT];
        let vertex1 = self.shaders[0];
        let vertex2 = self.shaders[1];
        let fragment = self.shaders[2];

        vk_device
            .shader_device
            .cmd_bind_shaders(command_buffer, &stages, &[vertex2, fragment]);

        vk_device.cmd_draw(command_buffer, 6, 1, 0, 0);

        vk_device
            .shader_device
            .cmd_bind_shaders(command_buffer, &stages, &[vertex1, fragment]);

        vk_device.cmd_draw(command_buffer, 3, 1, 0, 0);

        vk_device.cmd_end_rendering(command_buffer);

        vk_device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[present_barrier],
        );

        vk_device.end_command_buffer(command_buffer).unwrap();

        let command_bufers = [command_buffer];
        let wait_semaphores = [self.image_available_semaphore];
        let signal_semaphores = [self.render_finished_semaphore];

        let submit_info = [vk::SubmitInfo::default()
            .command_buffers(&command_bufers)
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .signal_semaphores(&signal_semaphores)];

        vk_device
            .queue_submit(self.queue, &submit_info, self.execution_finished_fence)
            .unwrap();

        let swapchains = [*self.swapchain.handle.get()];
        let image_indecies = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indecies);

        self.swapchain
            .loader
            .queue_present(self.queue, &present_info)
            .unwrap();

        *self.command_bufer.get() = Some(command_buffer);
    }

    fn destroy(&self) {
        let vk_device = &self.vk_device;

        unsafe {
            let _ = vk_device.device_wait_idle();
            let _ = vk_device.wait_for_fences(&[self.execution_finished_fence], true, u64::MAX);

            if let Some(buffer) = *self.command_bufer.get() {
                vk_device.free_command_buffers(self.command_pool, &[buffer]);
            }

            vk_device.destroy_command_pool(self.command_pool, None);
            vk_device.destroy_semaphore(self.image_available_semaphore, None);
            vk_device.destroy_semaphore(self.render_finished_semaphore, None);
            vk_device.destroy_fence(self.execution_finished_fence, None);

            for shader in self.shaders {
                vk_device.shader_device.destroy_shader(shader, None);
            }

            self.swapchain.destroy(vk_device);
            vk_device.destroy();
        }
    }
}

impl Default for Application {
    fn default() -> Self {
        unsafe { Self::new() }.unwrap()
    }
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut app = Application::default();

    while !app.window.should_close() {
        app.glfw_ctx.poll_events();

        for (_, event) in glfw::flush_messages(&app.glfw_events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, ..) | glfw::WindowEvent::Close => {
                    app.window.set_should_close(true);
                }

                glfw::WindowEvent::Size(x, y) => {
                    unsafe {
                        let _ = app.vk_device.device_wait_idle();
                        app.swapchain
                            .recreate(&app.vk_device, [x as u32, y as u32])
                            .unwrap();
                    };
                }
                _ => {}
            }
        }

        unsafe {
            app.draw();
        }
    }

    app.destroy();
}
