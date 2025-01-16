#![allow(unused)]

use ash::{khr::swapchain, vk};

use crate::vulkan::VulkanDevice;

use super::MemoryAccessFlags;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum CullingMode {
    #[default]
    None,
    Front,
    Back,
}

impl From<CullingMode> for vk::CullModeFlags {
    fn from(value: CullingMode) -> Self {
        match value {
            CullingMode::None => Self::NONE,
            CullingMode::Front => Self::FRONT,
            CullingMode::Back => Self::BACK,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct UDim2 {
    /// the size relative to the swapchain in percent (0.0 - 1.0)
    pub scale: [f32; 2],
    /// the size in pixels
    pub offset: [f32; 2],
}

#[derive(Debug, Clone, Default)]
pub struct VertexInput {
    pub bindings: Vec<vk::VertexInputBindingDescription>,
    pub attributes: Vec<vk::VertexInputAttributeDescription>,
}

pub struct ColorAttachmentInfo {
    access: MemoryAccessFlags,
}

#[derive(Debug, Default, Clone)]
pub struct MaterialCreateInfo {
    pub cull_mode: CullingMode,
    pub viewport: UDim2,
    pub vertex_input: VertexInput,
    pub shaders: Vec<vk::PipelineShaderStageCreateInfo<'static>>,
}

pub struct Material {
    pub pipeline: vk::Pipeline,
    pub info: MaterialCreateInfo,
}

impl MaterialCreateInfo {
    pub(crate) fn build(
        &self,
        device: &VulkanDevice,
        rpass: vk::RenderPass,
        layout: vk::PipelineLayout,
        swapchain_size: [u32; 2],
    ) -> Material {
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&self.vertex_input.bindings)
            .vertex_attribute_descriptions(&self.vertex_input.attributes);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(self.cull_mode.into())
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let screen_size = [
            self.viewport.scale[0] * swapchain_size[0] as f32 + self.viewport.offset[0],
            self.viewport.scale[1] * swapchain_size[1] as f32 + self.viewport.offset[1],
        ];

        let viewport = vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(screen_size[0])
            .height(screen_size[1])
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(
                vk::Extent2D::default()
                    .width(screen_size[0] as u32)
                    .height(screen_size[1] as u32),
            );
        let viewports = &[viewport];
        let scissors = &[scissor];

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(viewports)
            .scissors(scissors);

        let attachments = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false); 3];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&self.shaders)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .color_blend_state(&color_blend_state)
            .multisample_state(&multisample_state)
            .layout(layout)
            .subpass(0)
            .render_pass(rpass);

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
                .unwrap()
        }[0];

        Material {
            info: self.clone(),
            pipeline,
        }
    }
}
