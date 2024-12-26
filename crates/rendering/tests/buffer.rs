use ash::vk;
use rendering::vulkan::{Buffer, VulkanDevice};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    let (window, _window_events) = glfw
        .create_window(800, 600, "Some window", glfw::WindowMode::Windowed)
        .unwrap();

    let vk_device = unsafe { VulkanDevice::new(&window) }.unwrap();

    let data = [1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    vk_device.destroy();
}
