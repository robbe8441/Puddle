use ash::vk;
use std::{io::Cursor, sync::Arc, time::Instant};
use svo::OctreeNode;

use camera::Camera;
use math::{vec4, Mat4, Transform, Vec4};
use rendering::{
    handler::{
        render_batch::{DrawData, RenderBatch},
        RenderHandler,
    },
    types::{Material, MaterialCreateInfo, UDim2, VertexInput},
    vulkan::Buffer,
};

mod camera;
pub mod svo;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UniformData {
    view_proj: Mat4,
    cam_pos: Vec4,
    time: f32,
}

pub struct World {
    pub camera: Camera,
    pub start_time: Instant,
    pub uniform_buffer: Arc<Buffer>,
    pub material: Arc<Material>,
    pub voxel_octrees: Vec<OctreeNode>,
    pub voxel_buffers: Vec<Arc<Buffer>>,
}

impl World {
    /// # Panics
    /// if there is no space to allocate the uniform buffer
    pub fn new(renderer: &mut RenderHandler) -> Self {
        let image_res = renderer.get_swapchain_resolution();

        let camera = Camera {
            transform: Transform::IDENTITY,
            aspect: image_res.width as f32 / image_res.height as f32,
            fovy: 70.0,
            znear: 0.01,
            zfar: 100.0,
        };

        let uniform_buffer = Buffer::new(
            renderer.device.clone(),
            std::mem::size_of::<UniformData>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )
        .unwrap();

        let vertex_buffer = Buffer::new(
            renderer.device.clone(),
            (std::mem::size_of::<[f32; 4]>() * CUBE_VERTECIES.len()) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE, // TODO: make device Local
        )
        .unwrap();

        vertex_buffer.write(0, &CUBE_VERTECIES);

        let mut batch = RenderBatch::default();

        renderer.set_uniform_buffer(uniform_buffer.clone(), 0);

        let cube_draw = DrawData {
            vertex_count: CUBE_VERTECIES.len() as u32,
            vertex_buffer: Some(vertex_buffer),
            ..Default::default()
        };

        batch.add_draw_call(cube_draw);

        let vertex_input = VertexInput {
            attributes: vec![vk::VertexInputAttributeDescription::default()
                .format(vk::Format::R32G32B32A32_SFLOAT)],
            bindings: vec![vk::VertexInputBindingDescription::default()
                .input_rate(vk::VertexInputRate::VERTEX)
                .stride(std::mem::size_of::<[f32; 4]>() as u32)],
        };

        let mut code = Cursor::new(include_bytes!("../../shaders/shader.spv"));
        let byte_code = ash::util::read_spv(&mut code).unwrap();

        let module_info = vk::ShaderModuleCreateInfo::default().code(&byte_code);
        let module = unsafe { renderer.device.create_shader_module(&module_info, None) }.unwrap();

        let material_info = MaterialCreateInfo {
            cull_mode: rendering::types::CullingMode::Front,
            viewport: UDim2 {
                scale: [1.0, 1.0],
                offset: [0.0, 0.0],
            },
            vertex_input,
            shaders: vec![
                vk::PipelineShaderStageCreateInfo::default()
                    .name(c"main")
                    .stage(vk::ShaderStageFlags::VERTEX)
                    .module(module),
                vk::PipelineShaderStageCreateInfo::default()
                    .name(c"main")
                    .stage(vk::ShaderStageFlags::FRAGMENT)
                    .module(module),
            ],
        };

        let material = renderer.load_material(material_info);

        batch.set_material(material.clone());

        renderer.add_render_batch(batch);

        Self {
            camera,
            uniform_buffer,
            material,
            start_time: Instant::now(),
            voxel_buffers: vec![],
            voxel_octrees: vec![],
        }
    }

    pub fn update(&self) {
        let cam_pos = self.camera.transform.translation;

        self.uniform_buffer.write(
            0,
            &[UniformData {
                view_proj: self.camera.build_proj(),
                cam_pos: vec4(cam_pos.x, cam_pos.y, cam_pos.z, 1.0),
                time: self.start_time.elapsed().as_secs_f32(),
            }],
        );
    }
}

const CUBE_VERTECIES: [[f32; 4]; 36] = [
    // Vorderseite (CCW)
    [-0.5, -0.5, 0.5, 1.0], // unten links
    [-0.5, 0.5, 0.5, 1.0],  // oben links
    [0.5, 0.5, 0.5, 1.0],   // oben rechts
    [-0.5, -0.5, 0.5, 1.0], // unten links
    [0.5, 0.5, 0.5, 1.0],   // oben rechts
    [0.5, -0.5, 0.5, 1.0],  // unten rechts
    // Rückseite (CCW)
    [-0.5, -0.5, -0.5, 1.0], // unten links
    [0.5, -0.5, -0.5, 1.0],  // unten rechts
    [0.5, 0.5, -0.5, 1.0],   // oben rechts
    [-0.5, -0.5, -0.5, 1.0], // unten links
    [0.5, 0.5, -0.5, 1.0],   // oben rechts
    [-0.5, 0.5, -0.5, 1.0],  // oben links
    // Linke Seite (CCW)
    [-0.5, -0.5, -0.5, 1.0], // unten hinten
    [-0.5, 0.5, -0.5, 1.0],  // oben hinten
    [-0.5, 0.5, 0.5, 1.0],   // oben vorne
    [-0.5, -0.5, -0.5, 1.0], // unten hinten
    [-0.5, 0.5, 0.5, 1.0],   // oben vorne
    [-0.5, -0.5, 0.5, 1.0],  // unten vorne
    // Rechte Seite (CCW)
    [0.5, -0.5, -0.5, 1.0], // unten hinten
    [0.5, -0.5, 0.5, 1.0],  // unten vorne
    [0.5, 0.5, 0.5, 1.0],   // oben vorne
    [0.5, -0.5, -0.5, 1.0], // unten hinten
    [0.5, 0.5, 0.5, 1.0],   // oben vorne
    [0.5, 0.5, -0.5, 1.0],  // oben hinten
    // Oben (CCW)
    [-0.5, 0.5, -0.5, 1.0], // hinten links
    [0.5, 0.5, -0.5, 1.0],  // hinten rechts
    [0.5, 0.5, 0.5, 1.0],   // vorne rechts
    [-0.5, 0.5, -0.5, 1.0], // hinten links
    [0.5, 0.5, 0.5, 1.0],   // vorne rechts
    [-0.5, 0.5, 0.5, 1.0],  // vorne links
    // Unten (CCW)
    [-0.5, -0.5, -0.5, 1.0], // hinten links
    [-0.5, -0.5, 0.5, 1.0],  // vorne links
    [0.5, -0.5, 0.5, 1.0],   // vorne rechts
    [-0.5, -0.5, -0.5, 1.0], // hinten links
    [0.5, -0.5, 0.5, 1.0],   // vorne rechts
    [0.5, -0.5, -0.5, 1.0],  // hinten rechts
];
