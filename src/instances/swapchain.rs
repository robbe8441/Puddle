use std::sync::Arc;

use anyhow::Result;
use ash::{khr::swapchain, vk};

pub struct Swapchain {
    intern: vk::SwapchainKHR,
    loader: swapchain::Device,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub fn new(device: Arc<super::Device>, surface: super::Surface) -> Result<Arc<Self>> {
        let surface_loader = surface.loader();
        let pdevice = device.physical_device();

        let surface_capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(pdevice, surface.as_raw())
        }?;

        let surface_format = unsafe {
            surface_loader.get_physical_device_surface_formats(pdevice, surface.as_raw())
        }?[0];

        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }
        let surface_resolution = match surface_capabilities.current_extent.width {
            u32::MAX => vk::Extent2D {
                width: 600,
                height: 400,
            },
            _ => surface_capabilities.current_extent,
        };
        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };
        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(pdevice, surface.as_raw())
        }?;
        let present_mode = present_modes
            .iter()
            .cloned()
            .min_by_key(|v| match v {
                &vk::PresentModeKHR::MAILBOX => 1,
                &vk::PresentModeKHR::FIFO_RELAXED => 2,
                &vk::PresentModeKHR::FIFO => 3,
                _ => 4,
            })
            .unwrap();

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.as_raw())
            .min_image_count(desired_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain_loader =
            swapchain::Device::new(&device.instance().as_raw(), &device.as_raw());

        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }?;

        let present_images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;
        let present_image_views: Vec<vk::ImageView> = present_images
            .iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo::default()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(image);
                unsafe { device.as_raw().create_image_view(&create_view_info, None) }.unwrap()
            })
            .collect();

        Ok(Arc::new(Self {
            intern: swapchain,
            loader: swapchain_loader,
            images: present_images,
            image_views: present_image_views,
        }))
    }

    pub fn aquire_next_image(&self, fence: Arc<crate::instances::Fence>) -> (u32, bool){
        unsafe { self.loader.acquire_next_image(self.intern, u64::MAX, vk::Semaphore::null(), fence.as_raw()) }.unwrap()
    }

    pub fn present(&self, queue: Arc<crate::instances::queue::Queue>) -> bool {
        let present_info = vk::PresentInfoKHR::default();

        unsafe { self.loader.queue_present(queue.as_raw(), &present_info) }.unwrap()
    }
}






impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe { self.loader.destroy_swapchain(self.intern, None) };
    }
}
