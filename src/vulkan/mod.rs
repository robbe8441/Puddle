use ash::vk;
use setup::DeviceQueues;

mod bindless;
mod buffer;
mod pipeline;
mod setup;
mod swapchain;

pub use bindless::*;
pub use buffer::Buffer;
pub use pipeline::*;
pub use swapchain::Swapchain;

#[repr(C)]
pub struct VulkanContext {
    // entry needs to be valid until the end of our program
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub pdevice: vk::PhysicalDevice,
    pub device: ash::Device,
    pub queues: DeviceQueues,
    pub surface_loader: ash::khr::surface::Instance,
    pub surface: vk::SurfaceKHR,
    pub bindless_handler: BindlessHandler,
}

unsafe impl Send for VulkanContext {}
unsafe impl Sync for VulkanContext {}

impl VulkanContext {
    pub unsafe fn destroy(&self) {
        let _ = self.device.device_wait_idle();
        self.bindless_handler.destroy(self);
        self.surface_loader.destroy_surface(self.surface, None);
        self.device.destroy_device(None);
        self.instance.destroy_instance(None);
    }
}
