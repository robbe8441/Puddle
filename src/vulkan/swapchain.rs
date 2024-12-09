use ash::{
    khr::swapchain,
    vk::{self, Handle},
};

use super::VulkanContext;

pub struct Swapchain {
    pub handle: vk::SwapchainKHR,
    pub loader: ash::khr::swapchain::Device,
    pub image_views: Vec<vk::ImageView>,
    pub images: Vec<vk::Image>,
    pub create_info: vk::SwapchainCreateInfoKHR<'static>,
    // tracks whats the next semaphore/frame to be used
    pub image_use_fences: Vec<vk::Fence>,
    pub current_frame: usize,
    pub image_handles: Vec<u32>,
}

impl Swapchain {
    pub fn new(vk_ctx: &VulkanContext, image_extent: [u32; 2]) -> Result<Swapchain, vk::Result> {
        unsafe { Self::create_swapchain(vk_ctx, image_extent) }
    }

    unsafe fn create_swapchain(
        vk_ctx: &VulkanContext,
        image_extent: [u32; 2],
    ) -> Result<Swapchain, vk::Result> {
        let surface_capabilities = vk_ctx
            .surface_loader
            .get_physical_device_surface_capabilities(vk_ctx.pdevice, vk_ctx.surface)?;

        let surface_format = vk_ctx
            .surface_loader
            .get_physical_device_surface_formats(vk_ctx.pdevice, vk_ctx.surface)?[0];

        let surface_resolution = match surface_capabilities.current_extent.width {
            u32::MAX => vk::Extent2D {
                width: image_extent[0],
                height: image_extent[1],
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

        let present_modes = vk_ctx
            .surface_loader
            .get_physical_device_surface_present_modes(vk_ctx.pdevice, vk_ctx.surface)
            .unwrap();

        let present_mode = present_modes
            .iter()
            .copied()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(vk_ctx.surface)
            .min_image_count(desired_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_resolution)
            .image_usage(vk::ImageUsageFlags::STORAGE)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain_loader = swapchain::Device::new(&vk_ctx.instance, &vk_ctx.device);

        let swapchain = swapchain_loader.create_swapchain(&swapchain_create_info, None)?;

        let image_views = Self::create_swapchain_image_views(
            &vk_ctx.device,
            &swapchain_loader,
            swapchain,
            surface_format.format,
        )?;

        let image_handles = image_views
            .iter()
            .map(|v| vk_ctx.bindless_handler.store_image(&vk_ctx.device, *v).0)
            .collect();

        let images = swapchain_loader.get_swapchain_images(swapchain)?;

        Ok(Self {
            handle: swapchain,
            loader: swapchain_loader,
            create_info: swapchain_create_info,
            image_use_fences: vec![vk::Fence::null(); image_views.len()],
            image_views,
            current_frame: 0,
            image_handles,
            images,
        })
    }

    pub unsafe fn create_swapchain_image_views(
        device: &ash::Device,
        swapchain_loader: &ash::khr::swapchain::Device,
        swapchain: vk::SwapchainKHR,
        format: vk::Format,
    ) -> Result<Vec<vk::ImageView>, vk::Result> {
        let swapchain_images = swapchain_loader.get_swapchain_images(swapchain)?;

        Ok(swapchain_images
            .iter()
            .map(|image| {
                let components = vk::ComponentMapping::default()
                    .r(vk::ComponentSwizzle::IDENTITY)
                    .g(vk::ComponentSwizzle::IDENTITY)
                    .b(vk::ComponentSwizzle::IDENTITY)
                    .a(vk::ComponentSwizzle::IDENTITY);

                let subresource_range = vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1);

                let info = vk::ImageViewCreateInfo::default()
                    .image(*image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .components(components)
                    .subresource_range(subresource_range);

                device.create_image_view(&info, None).unwrap()
            })
            .collect())
    }

    pub unsafe fn recreate(
        &mut self,
        device: &ash::Device,
        new_extent: [u32; 2],
    ) -> Result<(), vk::Result> {
        for fence in self.image_use_fences.iter().filter(|v| !v.is_null()) {
            let _ = device.wait_for_fences(&[*fence], true, u64::MAX);
        }

        self.create_info.image_extent = vk::Extent2D {
            width: new_extent[0],
            height: new_extent[1],
        };

        let create_info = vk::SwapchainCreateInfoKHR {
            old_swapchain: self.handle,
            ..self.create_info
        };

        self.handle = self.loader.create_swapchain(&create_info, None)?;

        for view in &self.image_views {
            device.destroy_image_view(*view, None);
        }

        self.loader
            .destroy_swapchain(create_info.old_swapchain, None);

        self.image_views = Self::create_swapchain_image_views(
            device,
            &self.loader,
            self.handle,
            create_info.image_format,
        )?;

        self.images = self.loader.get_swapchain_images(self.handle)?;

        Ok(())
    }

    pub unsafe fn destroy(&self, vk_ctx: &VulkanContext) {
        for view in &self.image_views {
            vk_ctx.device.destroy_image_view(*view, None);
        }

        self.loader.destroy_swapchain(self.handle, None);
    }
}
