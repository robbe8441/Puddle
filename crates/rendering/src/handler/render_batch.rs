use crate::{
    types::Material,
    vulkan::{Buffer, VulkanDevice},
};
use ash::vk;
use log::error;
use std::sync::Arc;

#[derive(Default)]
pub struct DrawData {
    /// must be set if mode contains ``VERTEX_BUFFER``
    pub vertex_buffer: Option<Arc<Buffer>>,
    /// must be set if mode contains ``INDEX_BUFFER``
    pub index_buffer: Option<Arc<Buffer>>,
    /// must be set if mode contains ``INDEX_BUFFER``
    pub index_type: vk::IndexType,
    /// must be set if mode contains ``INSTANCE_BUFFER``
    pub instance_buffer: Option<Arc<Buffer>>,
    /// defaults to 1 (cant be 0)
    pub instance_count: u32,
    /// must be set if mode contains ``INDEX_BUFFER``
    pub index_count: u32,
    /// must be always set (otherwise nothing is drawn)
    pub vertex_count: u32,
    /// size of one vertex in bytes (only if mode contains ``VERTEX_BUFFER``)
    pub vertex_size: u32,
    /// size of one instance in bytes (only if mode contains ``INSTANCE_BUFFER``)
    pub instance_size: u32,
    /// must be set if mode contains ``VERTEX_BUFFER``
    pub vertex_attribute_descriptions: Vec<vk::VertexInputAttributeDescription2EXT<'static>>,
    /// must be set if mode contains ``INSTANCE_BUFFER``
    pub instance_attribute_descriptions: Vec<vk::VertexInputAttributeDescription2EXT<'static>>,
}

impl DrawData {
    unsafe fn execute(&self, device: &VulkanDevice, cmd: vk::CommandBuffer) {
        let mut vertex_input_desc = vec![];
        let mut vertex_attribute_desc = vec![];
        let mut vertex_buffers = vec![];

        if let Some(vertex_b) = &self.vertex_buffer {
            debug_assert!(self.vertex_size > 0);
            vertex_input_desc.push(
                vk::VertexInputBindingDescription2EXT::default()
                    .stride(self.vertex_size)
                    .divisor(1)
                    .input_rate(vk::VertexInputRate::VERTEX),
            );
            vertex_buffers.push(vertex_b.handle());
            vertex_attribute_desc.extend(self.vertex_attribute_descriptions.iter());
        }

        if let Some(instance_b) = &self.instance_buffer {
            debug_assert!(self.instance_size > 0);
            vertex_input_desc.push(
                vk::VertexInputBindingDescription2EXT::default()
                    .stride(self.instance_size)
                    .divisor(1)
                    .input_rate(vk::VertexInputRate::INSTANCE),
            );
            vertex_buffers.push(instance_b.handle()); // instance buffer is also in vertex buffers
            vertex_attribute_desc.extend(self.instance_attribute_descriptions.iter());
        }

        if !vertex_buffers.is_empty() {
            let offsets = vec![0; vertex_buffers.len()];
            device.cmd_bind_vertex_buffers(cmd, 0, &vertex_buffers, &offsets);
        }

        device
            .shader_device
            .cmd_set_vertex_input(cmd, &vertex_input_desc, &vertex_attribute_desc);

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
    draws: Vec<DrawData>,
    material: Option<Arc<dyn Material>>,
}

impl RenderBatch {
    pub fn set_material(&mut self, material: Arc<dyn Material>) {
        self.material = Some(material);
    }

    pub fn add_draw_call(&mut self, draw_data: DrawData) {
        self.draws.push(draw_data);
    }

    pub(crate) unsafe fn execute(
        &self,
        device: &VulkanDevice,
        cmd: vk::CommandBuffer,
        color_attachments: &[vk::RenderingAttachmentInfo],
        render_area: vk::Rect2D,
    ) {
        let Some(material) = &self.material else {
            error!("no material has been set before rendering");
            return;
        };

        let rendering_info = vk::RenderingInfo::default()
            .render_area(render_area)
            .layer_count(1)
            .view_mask(0)
            .color_attachments(color_attachments);

        device.cmd_begin_rendering(cmd, &rendering_info);

        device.cmd_set_scissor_with_count(cmd, &[render_area]);
        let view_size = render_area.extent;
        device.cmd_set_viewport_with_count(
            cmd,
            &[vk::Viewport::default()
                .width(view_size.width as f32)
                .height(view_size.height as f32)],
        );

        material.setup_material(&device.shader_device, cmd);

        for command in &self.draws {
            command.execute(device, cmd);
        }

        device.cmd_end_rendering(cmd);
    }
}
