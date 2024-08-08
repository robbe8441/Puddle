use anyhow::Result;
use ash::vk;
use graphics::RenderPass;
use std::{sync::Arc, time::Instant};

use vk_render::instances::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct CameraUniform {
    proj: [[f32; 4]; 4],
    pos: [f32; 4],
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let window = Arc::new(WindowBuilder::new().build(&event_loop)?);

    let extensions = Surface::enumerate_required_extensions(&event_loop)?;

    let instance = unsafe { Instance::from_extensions(extensions) }?;

    let (device, queue) = Device::new_default(instance.clone())?;

    let surface = Surface::new(instance.clone(), window.clone())?;

    let mut swapchain =
        Swapchain::new(device.clone(), surface.clone(), window.inner_size().into())?;

    let command_pool = CommandPool::new(device.clone(), queue.family_index())?;

    let renderpass = RenderPass::new_deafult(device.clone(), swapchain.format().format)?;

    let mut framebuffers: Vec<Arc<Framebuffer>> = swapchain
        .image_views
        .iter()
        .map(|&present_image_view| {
            let framebuffer_attachments = [present_image_view, swapchain.depth_image_view];
            let res = swapchain.resolution();
            let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                .render_pass(renderpass.as_raw())
                .attachments(&framebuffer_attachments)
                .width(res.width)
                .height(res.height)
                .layers(1);

            vk_render::instances::Framebuffer::new(device.clone(), &frame_buffer_create_info)
                .unwrap()
        })
        .collect();

    let egui_ctx = egui::Context::default();
    let mut egui_winit = egui_winit::State::new(4096, &window);

    let painter = vk_render::egui::Painter::new(device.clone(), queue.clone(), renderpass)?;

    event_loop.run(|event, target| match event {
        Event::WindowEvent {
            window_id: _,
            event,
        } => match event {
            WindowEvent::RedrawRequested => {
                let fence = Fence::new(device.clone()).unwrap();

                let (image_index, _suboptimal) = swapchain.aquire_next_image(fence.clone());

                let command_buffer = CommandBuffer::new(command_pool.clone()).unwrap();

                let frame_start = Instant::now();
                egui_ctx.begin_frame(*egui_winit.egui_input());

                let mut name = "ferris";
                let mut age = 10;

                egui::Window::new("Hello World").show(&egui_ctx, |ui| {
                    ui.heading("My egui Application");
                    ui.horizontal(|ui| {
                        ui.label("Your name: ");
                        ui.text_edit_singleline(&mut name);
                    });
                    ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
                    if ui.button("Increment").clicked() {
                        age += 1;
                    }
                    ui.label(format!("Hello '{name}', age {age}"));
                });

                let egui_output = egui_ctx.end_frame();

                let size = window.inner_size();
                let sf: f32 = window.scale_factor() as f32;

                painter
                    .draw(
                        &mut command_buffer,
                        [(size.width as f32) / sf, (size.height as f32) / sf],
                        &egui_ctx,
                        egui_output.shapes,
                    )
                    .unwrap();

                command_buffer.end();

                fence
                    .submit_buffers(&[command_buffer], queue.clone())
                    .unwrap();

                fence.wait_for_finished(u64::MAX).unwrap();

                swapchain.present(queue.clone(), image_index);
            }

            WindowEvent::Resized(size) => {
                swapchain = Swapchain::new(device.clone(), surface.clone(), size.into()).unwrap();
                // pipeline = PipelineGraphics::test(device.clone(), swapchain.format().format);

                framebuffers = swapchain
                    .image_views
                    .iter()
                    .map(|&present_image_view| {
                        let framebuffer_attachments =
                            [present_image_view, swapchain.depth_image_view];
                        let res = swapchain.resolution();
                        let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                            .render_pass(renderpass.as_raw())
                            .attachments(&framebuffer_attachments)
                            .width(res.width)
                            .height(res.height)
                            .layers(1);

                        vk_render::instances::Framebuffer::new(
                            device.clone(),
                            &frame_buffer_create_info,
                        )
                        .unwrap()
                    })
                    .collect();
            }
            WindowEvent::CloseRequested => target.exit(),
            _ => {}
        },
        Event::AboutToWait => window.request_redraw(),
        _ => {}
    })?;

    Ok(())
}
