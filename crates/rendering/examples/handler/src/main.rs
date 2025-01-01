use std::{io::Cursor, sync::Arc};

use ash::vk;
use rendering::{
    handler::{
        render_context::{DrawData, RenderBatch},
        RenderHandler,
    },
    types::Material,
    vulkan::Buffer,
};

pub struct DefaultMaterial {
    shaders: [vk::ShaderEXT; 2],
}

impl Material for DefaultMaterial {
    fn set_color_write_mask(&self) -> (u32, &[vk::ColorComponentFlags]) {
        (0, &[vk::ColorComponentFlags::RGBA])
    }
    fn shaders(&self) -> (&[vk::ShaderEXT], &[vk::ShaderStageFlags]) {
        (
            &self.shaders,
            &[vk::ShaderStageFlags::VERTEX, vk::ShaderStageFlags::FRAGMENT],
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

    let mut code = Cursor::new(include_bytes!("../shaders/shader.spv"));
    let byte_code = ash::util::read_spv(&mut code).unwrap();

    let shader_info = vk::ShaderCreateInfoEXT::default()
        .code_type(vk::ShaderCodeTypeEXT::SPIRV)
        .code(bytemuck::cast_slice(&byte_code));

    let shader_crate_infos = [
        vk::ShaderCreateInfoEXT {
            stage: vk::ShaderStageFlags::VERTEX,
            next_stage: vk::ShaderStageFlags::FRAGMENT,
            p_name: c"main".as_ptr(),
            ..shader_info
        },
        vk::ShaderCreateInfoEXT {
            stage: vk::ShaderStageFlags::FRAGMENT,
            p_name: c"main".as_ptr(),
            ..shader_info
        },
    ];

    let shaders = unsafe {
        handler
            .device
            .shader_device
            .create_shaders(&shader_crate_infos, None)
    }
    .unwrap()
    .try_into()
    .unwrap();

    let vertex_data = [
        [-0.5, 0.5, 1.0, 1.0],
        [-0.5, -0.5, 1.0, 1.0],
        [0.5, -0.5, 1.0, 1.0],
        [-0.5, 0.5, 1.0, 1.0],
        [0.5, -0.5, 1.0, 1.0],
        [0.5, 0.5, 1.0, 1.0],
    ];

    let vertex_buffer = Buffer::new(
        handler.device.clone(),
        std::mem::size_of_val(&vertex_data) as u64,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE,
    )
    .unwrap();

    unsafe {
        let ptr = handler
            .device
            .map_memory(
                vertex_buffer.mem_ref().handle(),
                0,
                vk::WHOLE_SIZE,
                vk::MemoryMapFlags::empty(),
            )
            .unwrap();

        let mut align: ash::util::Align<[f32; 4]> = ash::util::Align::new(
            ptr,
            std::mem::align_of::<f32>() as u64,
            std::mem::size_of_val(&vertex_data) as u64,
        );

        align.copy_from_slice(&vertex_data);
    }

    let material = DefaultMaterial { shaders };

    let mut render_batch = RenderBatch::default();
    render_batch.set_material(Arc::new(material));

    let draw_data = DrawData {
        vertex_buffer: Some(vertex_buffer),
        vertex_count: 6,
        vertex_size: std::mem::size_of::<[f32; 4]>() as u32,
        vertex_attribute_descriptions: vec![vk::VertexInputAttributeDescription2EXT::default()
            .format(vk::Format::R32G32B32A32_SFLOAT)],
        ..Default::default()
    };

    render_batch.add_draw_call(draw_data);
    handler.add_render_batch(render_batch);

    window.set_all_polling(true);

    while !window.should_close() {
        glfw_ctx.poll_events();
        unsafe { handler.draw() }.unwrap();

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
            .destroy_shader(shaders[0], None);
        handler
            .device
            .shader_device
            .destroy_shader(shaders[1], None);
    }
}
