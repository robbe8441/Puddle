
use crate::{
    vulkan::{create_pipeline, Swapchain, VulkanContext},
    MAX_FRAMES_ON_FLY,
};

mod frame;
mod transform;
mod world;
use ash::vk;
use frame::FrameData;
pub use world::*;

#[allow(unused)]
pub struct Application {
    pub glfw_ctx: glfw::Glfw,
    pub window: glfw::PWindow,
    pub glfw_events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,

    vulkan_context: VulkanContext,
    swapchain: Swapchain,
    pipeline: vk::Pipeline,

    frames: [FrameData; MAX_FRAMES_ON_FLY],
}

impl Application {
    pub fn new() -> Self {
        let mut glfw_ctx = glfw::init(glfw::fail_on_errors).expect("failed to create glfw context");

        let (mut window, glfw_events) = glfw_ctx
            .create_window(600, 400, "vulkan window", glfw::WindowMode::Windowed)
            .expect("failed to create window");

        window.set_size_polling(true);
        window.set_key_polling(true);

        let image_extent = {
            let (x, y) = window.get_size();
            [x as u32, y as u32]
        };

        let vulkan_context = VulkanContext::new(&window).expect("failed to cretae vulkan context");
        let swapchain = Swapchain::new(&vulkan_context, image_extent).unwrap();

        let pipeline = create_pipeline(&vulkan_context).unwrap();

        let frames: [FrameData; MAX_FRAMES_ON_FLY] =
            std::array::from_fn(|_| FrameData::new(&vulkan_context).unwrap());

        Self {
            glfw_ctx,
            window,
            glfw_events,
            vulkan_context,
            swapchain,
            pipeline,
            frames,
        }
    }

    pub fn draw(&mut self) -> Result<(), vk::Result> {
        let frame = &mut self.frames[self.swapchain.current_frame];
        self.swapchain.current_frame = (self.swapchain.current_frame + 1) % MAX_FRAMES_ON_FLY;

        unsafe { frame.render(&mut self.swapchain, self.pipeline, &self.vulkan_context) }?;

        Ok(())
    }

    pub fn destroy(&self) {
        unsafe {
            for frame in &self.frames {
                frame.destroy();
            }

            self.vulkan_context
                .device
                .destroy_pipeline(self.pipeline, None);

            self.swapchain.destroy(&self.vulkan_context);
            self.vulkan_context.destroy();
        };
    }
}

// impl Drop for Application {
//     fn drop(&mut self) {
//     }
// }
