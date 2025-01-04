use ash::vk;
use material::DefaultMaterial;
use std::{sync::Arc, time::Instant};
use svo::OctreeNode;

use camera::Camera;
use math::{vec4, Mat4, Transform, Vec4};
use rendering::{
    handler::{
        render_batch::{DrawData, RenderBatch},
        RenderHandler,
    },
    types::Material,
    vulkan::Buffer,
};

mod camera;
mod material;
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
    pub material: Arc<dyn Material>,
    pub voxel_octrees: Vec<OctreeNode>,
    pub voxel_buffers: Vec<Arc<Buffer>>,
}

impl World {
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
            vk::MemoryPropertyFlags::HOST_VISIBLE,
        )
        .unwrap();

        let vertex_buffer = Buffer::new(
            renderer.device.clone(),
            std::mem::size_of_val(&CUBE_VERTECIES) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE, // TODO: make device Local
        )
        .unwrap();

        vertex_buffer.write(0, &CUBE_VERTECIES);

        let code = include_bytes!("../../shaders/shader.spv");

        let vertex_shader = renderer.load_shader(
            code,
            c"main",
            vk::ShaderStageFlags::VERTEX,
            vk::ShaderStageFlags::FRAGMENT,
        );

        let fragment_shader = renderer.load_shader(
            code,
            c"main",
            vk::ShaderStageFlags::FRAGMENT,
            vk::ShaderStageFlags::empty(),
        );

        let material = Arc::new(DefaultMaterial {
            shaders: [vertex_shader, fragment_shader],
        });

        let mut batch = RenderBatch::default();
        batch.set_material(material.clone());

        renderer.set_uniform_buffer(uniform_buffer.clone());

        let cube_draw = DrawData {
            vertex_count: CUBE_VERTECIES.len() as u32,
            vertex_size: std::mem::size_of::<[f32; 4]>() as u32,
            vertex_buffer: Some(vertex_buffer),
            vertex_attribute_descriptions: vec![vk::VertexInputAttributeDescription2EXT::default()
                .format(vk::Format::R32G32B32A32_SFLOAT)],
            ..Default::default()
        };

        batch.add_draw_call(cube_draw);
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
