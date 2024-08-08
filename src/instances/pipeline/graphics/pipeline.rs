use anyhow::Result;
use ash::vk;
use std::sync::Arc;

use crate::{
    instances::Device,
    types::{Vertex, VertexInput},
};

use super::RenderPass;

pub struct PipelineGraphics {
    intern: vk::Pipeline,
    layout: vk::PipelineLayout,
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
    descriptor_layouts: Arc<[vk::DescriptorSetLayout]>,
}

impl PipelineGraphics {
    pub fn new<T: VertexInput>(info: super::PipelineCreateInfo<T>) -> Result<Arc<Self>> {
        let shader_stage_create_infos = [
            info.vertex_shader.shader_stage_info(),
            info.fragment_shader.shader_stage_info(),
        ];

        let layout_create_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&info.descriptor_layouts);

        let pipeline_layout = unsafe {
            info.device
                .as_raw()
                .create_pipeline_layout(&layout_create_info, None)
        }
        .unwrap();

        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<T>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];

        let vertex_input_attribute_descriptions = T::desc();

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_input_binding_descriptions);
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::default();

        let rasterization_info = info.cull_mode.into();

        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil_state,
            back: noop_stencil_state,
            max_depth_bounds: 1.0,
            ..Default::default()
        };

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::TRUE,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&color_blend_attachment_states);
        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_state);

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(info.render_pass.as_raw());

        let graphics_pipelines = unsafe {
            info.device.as_raw().create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[graphic_pipeline_info],
                None,
            )
        }
        .expect("Unable to create graphics pipeline");

        let graphic_pipeline = graphics_pipelines[0];

        Ok(Self {
            intern: graphic_pipeline,
            layout: pipeline_layout,
            device: info.device,
            render_pass: info.render_pass,
            descriptor_layouts: info.descriptor_layouts.into(),
        }
        .into())
    }

    pub fn render_pass(&self) -> Arc<super::RenderPass> {
        self.render_pass.clone()
    }
}

impl crate::instances::Pipeline for PipelineGraphics {
    fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }
    fn as_raw(&self) -> vk::Pipeline {
        self.intern
    }
    fn bind_point(&self) -> vk::PipelineBindPoint {
        vk::PipelineBindPoint::GRAPHICS
    }
    fn set_layouts(&self) -> Arc<[vk::DescriptorSetLayout]> {
        self.descriptor_layouts.clone()
    }
}

impl Drop for PipelineGraphics {
    fn drop(&mut self) {
        unsafe {
            self.device
                .as_raw()
                .destroy_pipeline_layout(self.layout, None);
            self.device.as_raw().destroy_pipeline(self.intern, None);
        }
    }
}
