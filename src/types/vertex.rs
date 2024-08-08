use ash::vk;
use bytemuck::offset_of;
use glam::*;

pub trait VertexInput {
    fn desc() -> Vec<vk::VertexInputAttributeDescription>;
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Vertex {
    pub pos: [f32; 4],
}

impl Vertex {
    pub fn from_pos3(pos: Vec3) -> Self {
        Self {
            pos: [pos.x, pos.y, pos.z, 1.0],
        }
    }
    pub fn from_pos2(pos: Vec2) -> Self {
        Self {
            pos: [pos.x, pos.y, 0.0, 1.0],
        }
    }
}

impl VertexInput for Vertex {
    fn desc() -> Vec<vk::VertexInputAttributeDescription> {
        vec![vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(Vertex, pos) as u32,
        }]
    }
}
