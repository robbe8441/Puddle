use std::{io::Cursor, sync::Arc};

use ash::{prelude::VkResult, vk};

use crate::{
    types::{Material, MaterialCreateInfo},
    vulkan::{Swapchain, VulkanDevice},
};

pub(crate) struct MaterialHandler {
    device: Arc<VulkanDevice>,
    pub main_renderpass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub materials: Vec<Arc<Material>>,
}

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
                format: swapchain.image_format(),
                ..attachment_desc
            },
            vk::AttachmentDescription {
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                ..attachment_desc
            },
            vk::AttachmentDescription {
                initial_layout: vk::ImageLayout::UNDEFINED,
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

        let subpass_dependencies = [vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::NONE)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)];

        let subpasses = [vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments_ref)];

        let renderpass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .dependencies(&subpass_dependencies)
            .subpasses(&subpasses);

        let swapchain_res = swapchain.get_image_extent();

        let main_renderpass = unsafe { device.create_render_pass(&renderpass_info, None)? };

        let framebuffer_info = vk::FramebufferCreateInfo::default()
            .render_pass(main_renderpass)
            .width(swapchain_res.width)
            .height(swapchain_res.height)
            .layers(1);

        let framebuffers = unsafe {
            swapchain
                .images
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
            materials: vec![],
        })
    }

    pub fn on_resize(&mut self, swapchain: &Swapchain, layout: vk::PipelineLayout) {
        let new_size = swapchain.create_info.image_extent;

        for buffer in self.framebuffers.drain(..) {
            unsafe { self.device.destroy_framebuffer(buffer, None) };
        }

        let framebuffer_info = vk::FramebufferCreateInfo::default()
            .render_pass(self.main_renderpass)
            .width(new_size.width)
            .height(new_size.height)
            .layers(1);

        self.framebuffers = unsafe {
            swapchain
                .images
                .iter()
                .map(|v| {
                    let attachments = [v.main_view, v.normal_view, v.depth_view];
                    self.device
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

        for p_material in &mut self.materials {
            // if the size is absolute then we don't need to recreate it
            if p_material.info.viewport.scale != [0.0, 0.0] {
                let material = unsafe { Arc::get_mut_unchecked(p_material) };
                unsafe { self.device.destroy_pipeline(material.pipeline, None) };

                let new = material.info.build(
                    &self.device,
                    self.main_renderpass,
                    layout,
                    [new_size.width, new_size.height],
                );

                *material = new;
            }
        }
    }
}

impl Drop for MaterialHandler {
    fn drop(&mut self) {
        unsafe {
            for mat in &self.materials {
                self.device.destroy_pipeline(mat.pipeline, None);
                self.device
                    .destroy_shader_module(mat.info.shaders[0].module, None);
            }
            for frame in &self.framebuffers {
                self.device.destroy_framebuffer(*frame, None);
            }
            self.device.destroy_render_pass(self.main_renderpass, None);
        }
    }
}
