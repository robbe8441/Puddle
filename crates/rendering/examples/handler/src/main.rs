use math::{Mat4, Transform, Vec3};
use std::{sync::Arc, time::Instant};

use ash::vk;
use rendering::{
    handler::{
        render_batch::{DrawData, RenderBatch},
        RenderHandler,
    },
    types::Material,
    vulkan::Buffer,
};

pub struct DefaultMaterial {
    shaders: [vk::ShaderEXT; 2],
}

pub struct Camera {
    transform: Transform,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    #[must_use]
    pub fn build_proj(&self) -> Mat4 {
        let view = Mat4::look_at_rh(
            self.transform.translation,
            self.transform.forward(),
            self.transform.down(),
        );

        let mut proj =
            Mat4::perspective_rh_gl(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);

        proj.x_axis.x *= -1.0;
        proj * view
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UniformData {
    camera: Mat4,
    time: f32,
}

impl Material for DefaultMaterial {
    fn shaders(&self) -> (&[vk::ShaderEXT], &[vk::ShaderStageFlags]) {
        (
            &self.shaders,
            &[vk::ShaderStageFlags::VERTEX, vk::ShaderStageFlags::FRAGMENT],
        )
    }

    fn set_polygon_mode(&self) -> vk::PolygonMode {
        vk::PolygonMode::FILL
    }

    fn set_color_write_mask(&self) -> (u32, &[vk::ColorComponentFlags]) {
        (
            0,
            &[vk::ColorComponentFlags::RGBA, vk::ColorComponentFlags::R],
        )
    }
}

#[allow(clippy::too_many_lines)]
fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut glfw_ctx = glfw::init(glfw::fail_on_errors).unwrap();

    let (mut window, glfw_events) = glfw_ctx
        .create_window(800, 600, "Puddle triangle", glfw::WindowMode::Windowed)
        .unwrap();

    let window_size = {
        let v = window.get_size();
        [v.0 as u32, v.1 as u32]
    };

    let mut handler = RenderHandler::new(&window, window_size).unwrap();

    let vertex_shader = handler.load_shader(
        include_bytes!("../shaders/shader.spv"),
        c"main",
        vk::ShaderStageFlags::VERTEX,
        vk::ShaderStageFlags::FRAGMENT,
    );

    let fragment_shader = handler.load_shader(
        include_bytes!("../shaders/shader.spv"),
        c"main",
        vk::ShaderStageFlags::FRAGMENT,
        vk::ShaderStageFlags::empty(),
    );

    const QUAD: [[f32; 4]; 6] = [
        [-0.5, 0.5, 0.0, 1.0],
        [-0.5, -0.5, 0.0, 1.0],
        [0.5, -0.5, 0.0, 1.0],
        [-0.5, 0.5, 0.0, 1.0],
        [0.5, 0.5, 0.0, 1.0],
        [0.5, -0.5, 0.0, 1.0],
    ];

    let vertex_buffer = Buffer::new(
        handler.device.clone(),
        std::mem::size_of_val(&QUAD) as u64,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE,
    )
    .unwrap();

    let uniform_buffer = Buffer::new(
        handler.device.clone(),
        std::mem::size_of::<UniformData>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE,
    )
    .unwrap();

    let _uniform_handle = handler.set_uniform_buffer(uniform_buffer.clone());

    let material = Arc::new(DefaultMaterial {
        shaders: [vertex_shader, fragment_shader],
    });

    let mut render_batch = RenderBatch::default();
    render_batch.set_material(material.clone());

    let draw_data = DrawData {
        vertex_buffer: Some(vertex_buffer.clone()),
        vertex_count: 6,
        vertex_size: std::mem::size_of::<[f32; 4]>() as u32,
        vertex_attribute_descriptions: vec![vk::VertexInputAttributeDescription2EXT::default()
            .format(vk::Format::R32G32B32A32_SFLOAT)],
        ..Default::default()
    };

    render_batch.add_draw_call(draw_data);
    handler.add_render_batch(render_batch);

    window.set_all_polling(true);
    let start = Instant::now();

    let mut camera = Camera {
        transform: Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        fovy: 70.0,
        aspect: window_size[0] as f32 / window_size[1] as f32,
        znear: 0.001,
        zfar: 100.0,
    };

    vertex_buffer.write(0, &QUAD);

    while !window.should_close() {
        glfw_ctx.poll_events();
        unsafe { handler.draw() }.unwrap();
        uniform_buffer.write(
            0,
            &[UniformData {
                time: start.elapsed().as_secs_f32().sin().abs(),
                camera: camera.build_proj(),
            }],
        );
        let t = start.elapsed().as_secs_f32();

        camera.transform =
            Transform::from_xyz(t.cos() * 4.0, 1.0, t.sin() * 4.0).looking_at(Vec3::ZERO, Vec3::Y);

        for (_, event) in glfw::flush_messages(&glfw_events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, ..) | glfw::WindowEvent::Close => {
                    window.set_should_close(true);
                }

                glfw::WindowEvent::Size(x, y) => {
                    unsafe {
                        handler.resize([x as u32, y as u32]).unwrap();
                    };
                }
                _ => {}
            }
        }
    }

    unsafe {
        handler
            .device
            .shader_device
            .destroy_shader(material.shaders[0], None);
        handler
            .device
            .shader_device
            .destroy_shader(material.shaders[1], None);
    }
}
