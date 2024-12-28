use std::io::Cursor;

use ash::vk;
use rendering::vulkan::{Swapchain, VulkanDevice};

pub struct Application {
    vk_device: VulkanDevice,
    swapchain: Swapchain,
    command_pool: vk::CommandPool,
    frame_buffers: Vec<vk::Framebuffer>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    renderpass: vk::RenderPass,
    queue: vk::Queue,
    queue_family: u32,
    window: glfw::PWindow,
    glfw_ctx: glfw::Glfw,
    glfw_events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
}

impl Application {
    pub fn new() -> Self {
        let mut glfw_ctx = glfw::init(glfw::fail_on_errors).unwrap();

        let (window, glfw_events) = glfw_ctx
            .create_window(800, 600, "Puddle triangle", glfw::WindowMode::Windowed)
            .unwrap();

        let vk_device = unsafe { VulkanDevice::new(&window) }.unwrap();

        #[allow(clippy::cast_sign_loss)]
        let window_size = {
            let v = window.get_size();
            [v.0 as u32, v.1 as u32]
        };

        let swapchain = unsafe { Swapchain::new(&vk_device, window_size) }.unwrap();

        let pipeline_layout = unsafe {
            vk_device.create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default(), None)
        }
        .unwrap();

        let renderpass = unsafe { create_renderpass(&vk_device, &swapchain) }.unwrap();
        let pipeline =
            unsafe { create_pipeline(&vk_device, &swapchain, pipeline_layout, renderpass) };

        let (queue_family, queue) = vk_device.queues.graphics;

        let command_pool = unsafe {
            let create_info = vk::CommandPoolCreateInfo::default().queue_family_index(queue_family);
            vk_device.create_command_pool(&create_info, None)
        }
        .unwrap();

        // let command_buffer = unsafe {
        //     let create_info = vk::CommandBufferAllocateInfo::default()
        //         .command_pool(command_pool)
        //         .command_buffer_count(1);
        //     vk_device.allocate_command_buffers(&create_info).unwrap()[0]
        // };

        let frame_buffers: Vec<vk::Framebuffer> = unsafe {
            (*swapchain.image_views.get())
                .iter()
                .map(|v| {
                    let attachments = [*v];
                    let create_info = vk::FramebufferCreateInfo::default()
                        .render_pass(renderpass)
                        .attachments(&attachments)
                        .width(window_size[0])
                        .height(window_size[1])
                        .layers(1);

                    vk_device.create_framebuffer(&create_info, None).unwrap()
                })
                .collect()
        };
        Self {
            vk_device,
            command_pool,
            frame_buffers,
            swapchain,
            pipeline_layout,
            pipeline,
            renderpass,
            queue_family,
            queue,
            window,
            glfw_ctx,
            glfw_events,
        }
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        let vk_device = &self.vk_device;

        unsafe {
            for v in &self.frame_buffers {
                vk_device.destroy_framebuffer(*v, None);
            }
            vk_device.destroy_render_pass(self.renderpass, None);
            vk_device.destroy_pipeline(self.pipeline, None);
            vk_device.destroy_pipeline_layout(self.pipeline_layout, None);
            vk_device.destroy_command_pool(self.command_pool, None);
            self.swapchain.destroy();
            vk_device.destroy();
        }
    }
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
}

unsafe fn create_renderpass(
    device: &VulkanDevice,
    swapchain: &Swapchain,
) -> Result<vk::RenderPass, vk::Result> {
    let attachments = [vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(),
        format: swapchain.image_format(),
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
    }];

    let color_attachments = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];

    let subpasses = [vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachments)];

    let dependencies = [vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        dst_subpass: 0,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        ..Default::default()
    }];

    let info = vk::RenderPassCreateInfo::default()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&dependencies);

    device.create_render_pass(&info, None)
}

unsafe fn create_pipeline(
    device: &ash::Device,
    swapchain: &Swapchain,
    layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
) -> vk::Pipeline {
    let mut code = Cursor::new(include_bytes!("../shaders/shader_opt.spv"));
    let byte_code = ash::util::read_spv(&mut code).unwrap();

    let shader_module = device
        .create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&byte_code),
            None,
        )
        .unwrap();

    let vertex_stage = vk::PipelineShaderStageCreateInfo::default()
        .module(shader_module)
        .stage(vk::ShaderStageFlags::VERTEX)
        .name(c"main");

    let fragment_stage = vk::PipelineShaderStageCreateInfo::default()
        .module(shader_module)
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .name(c"main");

    let image_size = swapchain.create_info.image_extent;

    let viewport = vk::Viewport::default()
        .x(0.0)
        .y(0.0)
        .width(image_size.width as f32)
        .height(image_size.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    let scissor = vk::Rect2D::default()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(image_size);

    let viewports = &[viewport];
    let scissors = &[scissor];
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewports(viewports)
        .scissors(scissors);

    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false);

    let attachments = &[attachment];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let dynamic_states = &[];

    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(dynamic_states);

    let stages = &[vertex_stage, fragment_stage];

    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    let info = vk::GraphicsPipelineCreateInfo::default()
        .stages(stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .color_blend_state(&color_blend_state)
        .dynamic_state(&dynamic_state)
        .layout(layout)
        .render_pass(render_pass)
        .subpass(0);

    let pipeline = device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)
        .unwrap()[0];

    device.destroy_shader_module(shader_module, None);

    pipeline
}
