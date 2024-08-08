use anyhow::Result;
use ash::vk;
use descriptors::{BindingDescriptor, DescriptorPool, DescriptorSet, WriteDescriptorSet};
use glam::{Mat4, Vec3};
use graphics::{PipelineCreateInfo, PipelineGraphics, RenderPass, ViewportMode};
use std::{sync::Arc, time::Instant};

use vk_render::{
    instances::{ShaderModule, *},
    types::{Transform, Vertex},
};
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

    let vertices = read_teapod_file();

    let instances = [Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::ONE.normalize(), Vec3::Y)];

    let instance_mat: Vec<_> = instances
        .iter()
        .map(|v| v.compute_matrix().to_cols_array_2d())
        .collect();
    let inversed_instance_mat: Vec<_> = instances
        .iter()
        .map(|v| v.compute_matrix().inverse().to_cols_array_2d())
        .collect();

    let inverted_instance_buffer: Arc<Subbuffer<[[f32; 4]; 4]>> = Subbuffer::from_data(
        device.clone(),
        vk::BufferCreateInfo {
            usage: vk::BufferUsageFlags::STORAGE_BUFFER,
            ..Default::default()
        },
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        &inversed_instance_mat,
    )
    .unwrap();
    let instance_buffer: Arc<Subbuffer<[[f32; 4]; 4]>> = Subbuffer::from_data(
        device.clone(),
        vk::BufferCreateInfo {
            usage: vk::BufferUsageFlags::STORAGE_BUFFER,
            ..Default::default()
        },
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        &instance_mat,
    )
    .unwrap();

    let cam_data = [CameraUniform {
        pos: [0.0; 4],
        proj: [[0.0; 4]; 4],
    }];

    let camera_unifrom_buffer: Arc<Subbuffer<CameraUniform>> = Subbuffer::from_data(
        device.clone(),
        vk::BufferCreateInfo {
            usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
            ..Default::default()
        },
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        &cam_data,
    )
    .unwrap();

    let image_create_info = vk::ImageCreateInfo {
        image_type: vk::ImageType::TYPE_3D,
        format: vk::Format::R8_UINT,
        extent: vk::Extent3D {
            width: 320,
            height: 320,
            depth: 320,
        },
        mip_levels: 1,
        array_layers: 1,
        samples: vk::SampleCountFlags::TYPE_1,
        tiling: vk::ImageTiling::OPTIMAL,
        usage: vk::ImageUsageFlags::STORAGE,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        initial_layout: vk::ImageLayout::GENERAL,
        ..Default::default()
    };

    let image = Image::new(device.clone(), &image_create_info)?;

    let image_view_info = ImageViewCreateInfo {
        view_type: vk::ImageViewType::TYPE_3D,
        format: image_create_info.format,
        components: vk::ComponentMapping {
            r: vk::ComponentSwizzle::R,
            g: vk::ComponentSwizzle::G,
            b: vk::ComponentSwizzle::B,
            a: vk::ComponentSwizzle::A,
        },
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            level_count: 1,
            layer_count: 1,
            ..Default::default()
        },
    };

    let compute_shader = ShaderModule::from_source(
        device.clone(),
        include_str!("./shaders/generate_sphere.glsl"),
        ShaderKind::Compute,
        "main",
    )?;

    let image_view = ImageView::new(image, &image_view_info)?;

    let set_layout = [
        BindingDescriptor {
            ty: descriptors::DescriptorType::StorageBuffer,
            count: 1,
            binding: 0,
            shader_stage: vk::ShaderStageFlags::ALL,
        },
        BindingDescriptor {
            ty: descriptors::DescriptorType::StorageBuffer,
            count: 1,
            binding: 1,
            shader_stage: vk::ShaderStageFlags::ALL,
        },
        BindingDescriptor {
            ty: descriptors::DescriptorType::UniformBuffer,
            count: 1,
            binding: 2,
            shader_stage: vk::ShaderStageFlags::ALL,
        },
        BindingDescriptor {
            ty: descriptors::DescriptorType::StorageImage,
            count: 1,
            binding: 3,
            shader_stage: vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::FRAGMENT,
        },
    ];

    let writes = [
        WriteDescriptorSet::Buffers(0, vec![instance_buffer.clone()]),
        WriteDescriptorSet::Buffers(1, vec![inverted_instance_buffer.clone()]),
        WriteDescriptorSet::Buffers(2, vec![camera_unifrom_buffer.clone()]),
        WriteDescriptorSet::ImageViews(3, vec![image_view.clone()]),
    ];

    let model_matrix = DescriptorSet::new(device.clone(), &set_layout)?;
    model_matrix.write(&writes);

    let compute_pipeline = vk_render::instances::compute::PipelineCompute::new(
        device.clone(),
        compute_shader,
        model_matrix.clone(),
    )?;

    let fence = Fence::new(device.clone())?;

    let render_pass = RenderPass::new_deafult(device.clone(), swapchain.format().format)?;

    let vertex_shader = ShaderModule::from_source(
        device.clone(),
        include_str!("./shaders/vertex.glsl"),
        ShaderKind::Vertex,
        "main",
    )?;

    let fragment_shader = ShaderModule::from_source(
        device.clone(),
        include_str!("./shaders/fragment.glsl"),
        ShaderKind::Fragment,
        "main",
    )?;

    let pipeline_info = PipelineCreateInfo {
        device: device.clone(),
        vertex_shader,
        fragment_shader,
        descriptor_layouts: vec![model_matrix.layout()],
        cull_mode: graphics::CullMode::Back,
        render_pass,
    };

    let pipeline = PipelineGraphics::new(pipeline_info)?;

    let mut framebuffers: Vec<Arc<Framebuffer>> = swapchain
        .image_views
        .iter()
        .map(|&present_image_view| {
            let framebuffer_attachments = [present_image_view, swapchain.depth_image_view];
            let res = swapchain.resolution();
            let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                .render_pass(pipeline.render_pass().as_raw())
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
    let start_time = Instant::now();

    event_loop.run(|event, target| match event {
        Event::WindowEvent {
            window_id: _,
            event,
        } => match event {
            WindowEvent::RedrawRequested => {
                let fence = Fence::new(device.clone()).unwrap();

                let (image_index, _suboptimal) = swapchain.aquire_next_image(fence.clone());

                let command_buffer = CommandBuffer::new(command_pool.clone()).unwrap();

                let t = start_time.elapsed().as_secs_f32() / 2.0;

                let cam_transform = Transform::from_xyz(t.cos() * 3.0, 1.0, t.sin() * 3.0)
                    .looking_at(Vec3::ZERO, -Vec3::Y);

                let view = Mat4::look_to_rh(
                    cam_transform.translation,
                    cam_transform.forward(),
                    cam_transform.up(),
                );

                let window_size = window.inner_size();
                let proj = Mat4::perspective_rh_gl(
                    (90.0_f32).to_radians(),
                    window_size.width as f32 / window_size.height as f32,
                    0.1,
                    100.0,
                );

                let pos = cam_transform.translation;

                let data = [CameraUniform {
                    proj: (proj * view).to_cols_array_2d(),
                    pos: [pos.x, pos.y, pos.z, 1.0],
                }];

                camera_unifrom_buffer.write(&data).unwrap();

                command_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

                command_buffer.bind_pipeline(compute_pipeline.clone());

                command_buffer.bind_descriptor_set(
                    model_matrix.clone(),
                    0,
                    compute_pipeline.clone(),
                    &[0],
                );

                command_buffer.dispatch(10, 10, 320);

                // println!("fps {}", 1.0 / delta.elapsed().as_secs_f64());
                delta = Instant::now();

                let clear_values = [
                    vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [0.1, 0.1, 0.1, 1.0],
                        },
                    },
                    vk::ClearValue {
                        depth_stencil: vk::ClearDepthStencilValue {
                            depth: 1.0,
                            stencil: 0,
                        },
                    },
                ];

                let viewport = ViewportMode::Relative(0.25, 1.0, 1.0, 0.5);
                let framebuffer = framebuffers[image_index as usize].clone();

                let scissors = [framebuffer.size().into()];
                let viewports = [viewport.get_size(framebuffer.size().into())];

                let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                    .render_pass(pipeline.render_pass().as_raw())
                    .framebuffer(framebuffer.as_raw())
                    .render_area(framebuffer.size().into())
                    .clear_values(&clear_values);

                command_buffer
                    .begin_render_pass(&render_pass_begin_info, vk::SubpassContents::INLINE);

                command_buffer.bind_descriptor_set(model_matrix.clone(), 0, pipeline.clone(), &[0]);

                command_buffer.bind_pipeline(pipeline.clone());
                command_buffer.set_viewport(0, &viewports);
                command_buffer.set_scissor(0, &scissors);

                command_buffer.bind_vertex_buffers(0, &[vertex_buffer.clone()], &[0]);
                command_buffer.draw(vertices.len() as u32, instances.len() as u32, 0, 0);
                command_buffer.end_render_pass();

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
                            .render_pass(pipeline.render_pass().as_raw())
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

pub fn read_teapod_file() -> Vec<Vertex> {
    let mut vertecies = vec![];
    let mut faces = vec![];

    for line in include_str!("./assets/cube.obj")
        .lines()
        .filter(|line| line.starts_with("v ") || line.starts_with("f "))
    {
        match &line[..2] {
            "v " => {
                let pos: [f32; 3] = line[2..]
                    .split_whitespace()
                    .filter_map(|v| v.parse::<f32>().ok())
                    .collect::<Vec<f32>>()
                    .try_into()
                    .unwrap();

                vertecies.push(Vertex {
                    pos: [pos[0], pos[1], pos[2], 1.0],
                })
            }
            "f " => {
                let indexes: [usize; 3] = line[2..]
                    .split_whitespace()
                    .map(|v| v.split("/").next().unwrap())
                    .filter_map(|v| v.parse::<usize>().ok())
                    .map(|v| v - 1)
                    .collect::<Vec<usize>>()
                    .try_into()
                    .unwrap();

                faces.push(vertecies[indexes[0]]);
                faces.push(vertecies[indexes[1]]);
                faces.push(vertecies[indexes[2]]);

                // faces.push(vertecies[indexes[0]]);
                // faces.push(vertecies[indexes[2]]);
                // faces.push(vertecies[indexes[3]]);
            }
            _ => {}
        }
    }

    faces
}
