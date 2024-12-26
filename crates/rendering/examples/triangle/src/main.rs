use std::io::Cursor;

use ash::vk;
use rendering::vulkan::{Swapchain, VulkanDevice};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    let (window, _window_events) = glfw
        .create_window(800, 600, "Puddle triangle", glfw::WindowMode::Windowed)
        .unwrap();

    let vk_device = unsafe { VulkanDevice::new(&window) }.unwrap();

    #[allow(clippy::cast_sign_loss)]
    let window_size = {
        let v = window.get_size();
        [v.0 as u32, v.1 as u32]
    };

    let swapchain = unsafe { Swapchain::new(&vk_device, window_size) }.unwrap();

    let (queue_family, queue) = vk_device.queues.graphics;

    let command_pool = unsafe {
        let create_info = vk::CommandPoolCreateInfo::default().queue_family_index(queue_family);
        vk_device.create_command_pool(&create_info, None)
    }
    .unwrap();

    let command_buffer = unsafe {
        let create_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .command_buffer_count(1);
        vk_device.allocate_command_buffers(&create_info).unwrap()[0]
    };

    // let frame_buffers: Vec<vk::Framebuffer> = unsafe {
    //     (*swapchain.image_views.get())
    //         .iter()
    //         .map(|v| {
    //             let attachments = [*v];
    //             let create_info = vk::FramebufferCreateInfo::default()
    //                 // TODO: RenderPass
    //                 .attachments(&attachments)
    //                 .width(window_size[0])
    //                 .height(window_size[1])
    //                 .layers(1);
    //
    //             vk_device.create_framebuffer(&create_info, None).unwrap()
    //         })
    //         .collect()
    // };

    let mut code = Cursor::new(include_bytes!("../shaders/shader_opt.spv"));
    let byte_code = ash::util::read_spv(&mut code).unwrap();

    let shader_module = unsafe {
        vk_device.create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&byte_code),
            None,
        )
    }
    .unwrap();

    unsafe {
        // for v in frame_buffers {
        //     vk_device.destroy_framebuffer(v, None);
        // }
        vk_device.destroy_shader_module(shader_module, None);
        vk_device.destroy_command_pool(command_pool, None);
        swapchain.destroy();
        vk_device.destroy();
    }
}
