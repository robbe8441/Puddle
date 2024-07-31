use anyhow::Result;
use ash::vk;
use descriptors::{DescriptorPool, DescriptorSet, WriteDescriptorSet};
use glam::{Mat4, Vec4Swizzles};
use graphics::PipelineGraphics;
use std::{sync::Arc, time::Instant};

use vk_render::{
    instances::*,
    types::{Transform, Vertex},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

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

    let vertices = [
        Vertex {
            pos: [-1.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            pos: [1.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            pos: [0.0, -1.0, 0.0, 1.0],
        },
    ];

    let instances = [
        glam::Vec4::new(-0.5, 0.0, 0.0, 0.0).to_array(),
        glam::Vec4::new(0.5, 0.0, 0.0, 0.0).to_array(),
    ];

    dbg!(&instances);

    let instance_buffer: Arc<Subbuffer<[f32; 4]>> = Subbuffer::from_data(
        device.clone(),
        vk::BufferCreateInfo {
            size: std::mem::size_of_val(&vertices) as u64,
            usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
            ..Default::default()
        },
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        &instances,
    )
    .unwrap();

    let sizes = [vk::DescriptorPoolSize {
        ty: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 1,
    }];

    let descriptor_pool = DescriptorPool::new(device.clone(), &sizes, 1)?;

    let buffer_info = [instance_buffer.desc()];

    let writes = [WriteDescriptorSet {
        buffer_info: Some(&buffer_info),
        image_info: None,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
        dst_binding: 0,
        dst_set: 0,
    }];

    let model_matrix = DescriptorSet::new(descriptor_pool.clone(), &writes)?;

    let pipeline = PipelineGraphics::test(
        device.clone(),
        swapchain.format().format,
        model_matrix.clone(),
    );

    let mut framebuffers: Vec<Arc<Framebuffer>> = swapchain
        .image_views
        .iter()
        .map(|&present_image_view| {
            let framebuffer_attachments = [present_image_view, swapchain.depth_image_view];
            let res = swapchain.resolution();
            let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                .render_pass(pipeline.renderpass)
                .attachments(&framebuffer_attachments)
                .width(res.width)
                .height(res.height)
                .layers(1);

            vk_render::instances::Framebuffer::new(device.clone(), &frame_buffer_create_info)
                .unwrap()
        })
        .collect();

    let vertex_buffer = Subbuffer::from_data(
        device.clone(),
        vk::BufferCreateInfo {
            size: std::mem::size_of_val(&vertices) as u64,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            ..Default::default()
        },
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        &vertices,
    )
    .unwrap();

    let mut delta = Instant::now();

    event_loop.run(|event, target| match event {
        Event::WindowEvent {
            window_id: _,
            event,
        } => match event {
            WindowEvent::RedrawRequested => {
                let fence = Fence::new(device.clone()).unwrap();

                let (image_index, _suboptimal) = swapchain.aquire_next_image(fence.clone());

                let command_buffer = CommandBuffer::new(command_pool.clone()).unwrap();

                command_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

                command_buffer.bind_descriptor_set(model_matrix.clone(), 0, &pipeline, &[0]);

                // println!("fps {}", 1.0 / delta.elapsed().as_secs_f64());
                delta = Instant::now();

                unsafe {
                    pipeline.draw(
                        &command_buffer,
                        framebuffers[image_index as usize].clone(),
                        &[vertex_buffer.clone()],
                    )
                };

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
                            .render_pass(pipeline.renderpass)
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
