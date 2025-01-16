use crate::{
    types::Material,
    vulkan::{Buffer, VulkanDevice},
};
use ash::vk;
use std::sync::Arc;

use super::material::MaterialHandler;

/// ``DrawData`` contains all the data needed for a single Draw call
#[derive(Default)]
pub struct DrawData {
    /// if this is Some then ``vertex_attribute_descriptions`` must be set
    pub vertex_buffer: Option<Arc<Buffer>>,
    /// if this is Some then ``instance_attribute_descriptions`` must be set
    pub instance_buffer: Option<Arc<Buffer>>,
    pub index_buffer: Option<Arc<Buffer>>,
    pub index_type: vk::IndexType,
    pub instance_count: u32,
    pub index_count: u32,
    pub vertex_count: u32,
}

impl DrawData {
    unsafe fn execute(&self, device: &VulkanDevice, cmd: vk::CommandBuffer) {
        let mut vertex_buffers = vec![];

        if let Some(vertex_b) = &self.vertex_buffer {
            vertex_buffers.push(vertex_b.handle());
        }

        if let Some(instance_b) = &self.instance_buffer {
            vertex_buffers.push(instance_b.handle()); // instance buffer is also in vertex buffers
        }

        // if there is no Vertex/Instance input then we don't need to bind it
        if !vertex_buffers.is_empty() {
            let offsets = vec![0; vertex_buffers.len()];
            device.cmd_bind_vertex_buffers(cmd, 0, &vertex_buffers, &offsets);
        }

        if let Some(index_b) = &self.index_buffer {
            device.cmd_bind_index_buffer(cmd, index_b.handle(), 0, self.index_type);
            device.cmd_draw_indexed(cmd, self.index_count, self.instance_count.max(1), 0, 0, 0);
        } else {
            device.cmd_draw(cmd, self.vertex_count, self.instance_count.max(1), 0, 0);
        }
    }
}

#[derive(Default)]
pub struct RenderBatch {
    material: Option<Arc<Material>>,
    draws: Vec<DrawData>,
}

impl RenderBatch {
    pub fn set_material(&mut self, material: Arc<Material>) {
        self.material = Some(material);
    }

    pub fn add_draw_call(&mut self, draw_data: DrawData) {
        self.draws.push(draw_data);
    }

    pub(crate) unsafe fn execute(
        &self,
        device: &VulkanDevice,
        cmd: vk::CommandBuffer,
    ) {
        let Some(material) = &self.material else {
            panic!("no material set when rendering")
        };
        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, material.pipeline);

        for command in &self.draws {
            command.execute(device, cmd);
        }
    }
}
