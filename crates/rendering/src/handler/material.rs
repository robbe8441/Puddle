use std::{default, io::Cursor, sync::Arc};

use ash::{
    prelude::VkResult,
    vk::{self, Framebuffer},
};

use crate::vulkan::{Swapchain, VulkanDevice};

pub(crate) struct MaterialHandler {
    device: Arc<VulkanDevice>,
    pub main_renderpass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub pipelines: Vec<vk::Pipeline>,
}

#[derive(Default)]
pub struct MaterialHandle {}

impl MaterialHandler {
    pub fn new(device: Arc<VulkanDevice>, swapchain: &Swapchain) -> VkResult<Self> {
        let attachment_desc = vk::AttachmentDescription::default()
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let attachments = [
            vk::AttachmentDescription {
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                format: unsafe { *swapchain.create_info.get() }.image_format,
                ..attachment_desc
            },
            vk::AttachmentDescription {
                initial_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                ..attachment_desc
            },
            vk::AttachmentDescription {
                initial_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                format: vk::Format::R32_SFLOAT,
                ..attachment_desc
            },
        ];

        let color_attachments_ref = [
            vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
            vk::AttachmentReference {
                attachment: 1,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
            vk::AttachmentReference {
                attachment: 2,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
        ];

        let subpasses = [vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments_ref)];

        let renderpass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses);

        let swapchain_res = swapchain.get_image_extent();

        let main_renderpass = unsafe { device.create_render_pass(&renderpass_info, None)? };

        let framebuffer_info = vk::FramebufferCreateInfo::default()
            .render_pass(main_renderpass)
            .width(swapchain_res.width)
            .height(swapchain_res.height)
            .layers(1);

        let framebuffers = unsafe {
            (*swapchain.images.get())
                .iter()
                .map(|v| {
                    let attachments = [v.main_view, v.normal_view, v.depth_view];
                    device
                        .create_framebuffer(
                            &vk::FramebufferCreateInfo {
                                p_attachments: attachments.as_ptr(),
                                attachment_count: attachments.len() as u32,
                                ..framebuffer_info
                            },
                            None,
                        )
                        .unwrap()
                })
                .collect()
        };

        Ok(Self {
            device,
            main_renderpass,
            framebuffers,
            pipelines: vec![],
        })
    }

    pub fn create_pipeline(&mut self, layout: vk::PipelineLayout, swapchain_extent: vk::Extent2D) {
        let mut binary = Cursor::new(include_bytes!("../../../application/shaders/shader.spv"));
        let byte_code = ash::util::read_spv(&mut binary).unwrap();

        let shader_info = vk::ShaderModuleCreateInfo::default().code(&byte_code);

        let shader = unsafe { self.device.create_shader_module(&shader_info, None) }.unwrap();

        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .module(shader)
                .stage(vk::ShaderStageFlags::VERTEX)
                .name(c"main"),
            vk::PipelineShaderStageCreateInfo::default()
                .module(shader)
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .name(c"main"),
        ];

        let vertex_binding = [vk::VertexInputBindingDescription::default()
            .stride(size_of::<[f32; 4]>() as u32)
            .binding(0)
            .input_rate(vk::VertexInputRate::VERTEX)];

        let vertex_attribute = [vk::VertexInputAttributeDescription::default()
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .location(0)
            .binding(0)
            .offset(0)];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_binding)
            .vertex_attribute_descriptions(&vertex_attribute);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let attachments = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let dynamic_states = vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);

        let viewport = vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(swapchain_extent.width as f32)
            .height(swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain_extent);
        let viewports = &[viewport];
        let scissors = &[scissor];

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(viewports)
            .scissors(scissors);

        let create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .color_blend_state(&color_blend_state)
            .multisample_state(&multisample_state)
            .layout(layout)
            .subpass(0)
            .render_pass(self.main_renderpass);
        // .dynamic_state(&dynamic_states);

        let pipeline = unsafe {
            self.device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
                .unwrap()[0]
        };

        unsafe { self.device.destroy_shader_module(shader, None) };

        self.pipelines.push(pipeline);
    }

    pub fn get_material(&self, _handle: &MaterialHandle) -> vk::Pipeline {
        self.pipelines[0]
    }
}

impl Drop for MaterialHandler {
    fn drop(&mut self) {
        unsafe {
            for pipeline in &self.pipelines {
                self.device.destroy_pipeline(*pipeline, None);
            }
            for frame in &self.framebuffers {
                self.device.destroy_framebuffer(*frame, None);
            }
            self.device.destroy_render_pass(self.main_renderpass, None);
        }
    }
}
