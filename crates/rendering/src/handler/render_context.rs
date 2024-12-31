use std::sync::Arc;

use ash::vk;
use bitflags::bitflags;
use log::error;

use crate::{types::Material, vulkan::VulkanDevice};

bitflags! {
    #[derive(Default)]
    pub struct DrawMode: u32 {
        const VERTEX_BUFFER = 0b1;
        const INDEX_BUFFER = 0b01;
        const INSTANCE_BUFFER = 0b001;
    }
}

#[derive(Default)]
pub struct DrawData {
    pub mode: DrawMode,
    /// must be set if mode contains ``VERTEX_BUFFER``
    pub vertex_buffer: vk::Buffer,
    /// must be set if mode contains ``INDEX_BUFFER``
    pub index_buffer: vk::Buffer,
    /// must be set if mode contains ``INDEX_BUFFER``
    pub index_type: vk::IndexType,
    /// must be set if mode contains ``INSTANCE_BUFFER``
    pub instance_buffer: vk::Buffer,
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

        if self.mode.contains(DrawMode::VERTEX_BUFFER) {
            vertex_input_desc.push(
                vk::VertexInputBindingDescription2EXT::default()
                    .stride(self.vertex_size)
                    .input_rate(vk::VertexInputRate::VERTEX),
            );
            vertex_buffers.push(self.vertex_buffer);
            vertex_attribute_desc.extend(self.vertex_attribute_descriptions.iter());
        }

        if self.mode.contains(DrawMode::INSTANCE_BUFFER) {
            vertex_input_desc.push(
                vk::VertexInputBindingDescription2EXT::default()
                    .stride(self.instance_size)
                    .input_rate(vk::VertexInputRate::INSTANCE),
            );
            vertex_buffers.push(self.instance_buffer);
            vertex_attribute_desc.extend(self.instance_attribute_descriptions.iter());
        }

        if self.mode.contains(DrawMode::INSTANCE_BUFFER)
            || self.mode.contains(DrawMode::VERTEX_BUFFER)
        {
            device.cmd_bind_vertex_buffers(cmd, 0, &vertex_buffers, &[]);
        }

        device
            .shader_device
            .cmd_set_vertex_input(cmd, &vertex_input_desc, &vertex_attribute_desc);

        if self.mode.contains(DrawMode::INDEX_BUFFER) {
            device.cmd_bind_index_buffer(cmd, self.index_buffer, 0, self.index_type);
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
