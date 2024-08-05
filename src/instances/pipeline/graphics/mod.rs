mod pipeline_create_info;
mod render_pass;

use crate::{
    instances::{BufferAllocation, CommandBuffer, Device, ShaderModule},
    types::Vertex,
};
use ash::vk;
use std::{ffi::CStr, sync::Arc};

pub struct PipelineGraphics {
    intern: vk::Pipeline,
    pub render_pass: Arc<render_pass::RenderPass>,
    layout: vk::PipelineLayout,
    device: Arc<Device>,
}

impl PipelineGraphics {
    // pub fn new(create_info: &pipeline_create_info::PipelineCreateInfo, layout: vk::PipelineLayout ) {
    //
    // }

    pub fn test(
        device: Arc<Device>,
        format: vk::Format,
        descriptors: Arc<crate::instances::descriptors::DescriptorSet>,
    ) -> Arc<PipelineGraphics> {
        let render_pass = render_pass::RenderPass::new_deafult(device.clone(), format).unwrap();

        let vertex_shader = ShaderModule::from_source(
            device.clone(),
            include_str!("./vertex.glsl"),
            crate::instances::ShaderKind::Vertex,
            "main",
        )
        .unwrap();

        let fragment_shader = ShaderModule::from_source(
            device.clone(),
            include_str!("./fragment.glsl"),
            crate::instances::ShaderKind::Fragment,
            "main",
        )
        .unwrap();

        let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: vertex_shader.as_raw(),
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: fragment_shader.as_raw(),
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let descriptor_layouts = [descriptors.layout()];
        let layout_create_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptor_layouts);

        let pipeline_layout = unsafe {
            device
                .as_raw()
                .create_pipeline_layout(&layout_create_info, None)
        }
        .unwrap();

        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];

        let vertex_input_attribute_descriptions = [Vertex::desc()];

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_input_binding_descriptions);
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::default()
            .scissors(&[])
            .viewports(&[]);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::FRONT,
            ..Default::default()
        };
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
            .render_pass(render_pass.as_raw());

        let graphics_pipelines = unsafe {
            device.as_raw().create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[graphic_pipeline_info],
                None,
            )
        }
        .expect("Unable to create graphics pipeline");

        let graphic_pipeline = graphics_pipelines[0];

        PipelineGraphics {
            device,
            render_pass,
            layout: pipeline_layout,
            intern: graphic_pipeline,
        }
        .into()
    }
}
pub unsafe fn draw(
    pipeline: Arc<PipelineGraphics>,
    command_buffer: &CommandBuffer,
    frame_buffer: Arc<crate::instances::Framebuffer>,
    vertex_buffers: &[Arc<dyn BufferAllocation>],
) {
    let clear_values = [
        vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.1, 0.1, 0.1, 0.0],
            },
        },
        vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        },
    ];

    let viewport = pipeline_create_info::ViewportMode::Relative(0.25, 0.25, 0.5, 0.5);

    let scissors = [frame_buffer.size().into()];
    let viewports = [viewport.get_size(frame_buffer.size().into())];

    let render_pass_begin_info = vk::RenderPassBeginInfo::default()
        .render_pass(pipeline.render_pass.as_raw())
        .framebuffer(frame_buffer.as_raw())
        .render_area(frame_buffer.size().into())
        .clear_values(&clear_values);

    command_buffer.begin_render_pass(&render_pass_begin_info, vk::SubpassContents::INLINE);
    command_buffer.bind_pipeline(pipeline);
    command_buffer.set_viewport(0, &viewports);
    command_buffer.set_scissor(0, &scissors);

    command_buffer.bind_vertex_buffers(0, vertex_buffers, &[0]);
    command_buffer.draw(
        (vertex_buffers[0].size() as usize / std::mem::size_of::<Vertex>()) as u32,
        1,
        0,
        0,
    );
    command_buffer.end_render_pass();
}

impl super::Pipeline for PipelineGraphics {
    fn bind_point(&self) -> vk::PipelineBindPoint {
        vk::PipelineBindPoint::GRAPHICS
    }
    fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    fn as_raw(&self) -> vk::Pipeline {
        self.intern
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
