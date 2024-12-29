use std::cell::UnsafeCell;

use ash::vk;

use super::VulkanDevice;

pub struct Swapchain {
    pub handle: UnsafeCell<vk::SwapchainKHR>,
    pub loader: ash::khr::swapchain::Device,
    pub image_views: UnsafeCell<Vec<vk::ImageView>>,
    pub create_info: UnsafeCell<vk::SwapchainCreateInfoKHR<'static>>,
}

impl Swapchain {
    /// # Safety
    /// # Errors
    pub unsafe fn new(
        device: &VulkanDevice,
        image_extent: [u32; 2],
    ) -> Result<Swapchain, vk::Result> {
        let surface_capabilities = device
            .surface_loader
            .get_physical_device_surface_capabilities(device.pdevice, device.surface)?;

        let surface_format = device
            .surface_loader
            .get_physical_device_surface_formats(device.pdevice, device.surface)?[0];

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

        let present_modes = device
            .surface_loader
            .get_physical_device_surface_present_modes(device.pdevice, device.surface)?;

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
            .surface(device.surface)
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

        let swapchain_loader = ash::khr::swapchain::Device::new(&device.instance, device);

        let swapchain = swapchain_loader.create_swapchain(&swapchain_create_info, None)?;

        let image_views = Self::create_swapchain_image_views(
            device,
            &swapchain_loader,
            swapchain,
            surface_format.format,
        )?;

        Ok(Self {
            handle: UnsafeCell::new(swapchain),
            loader: swapchain_loader,
            create_info: UnsafeCell::new(swapchain_create_info),
            image_views: UnsafeCell::new(image_views),
        })
    }

    unsafe fn create_swapchain_image_views(
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

    /// # Safety
    /// there must not currently be written on to one of the swapchain images
    /// the pointer to the swapchain handle is now invalid
    /// # Errors
    /// if there was an issue allocating new images
    /// for example if no space if left
    pub unsafe fn recreate(
        &self,
        device: &ash::Device,
        new_extent: [u32; 2],
    ) -> Result<(), vk::Result> {
        let handle = self.handle.get();

        let image_extent = vk::Extent2D {
            width: new_extent[0],
            height: new_extent[1],
        };

        (*self.create_info.get()).image_extent = image_extent;

        let create_info = vk::SwapchainCreateInfoKHR {
            old_swapchain: *handle,
            ..*self.create_info.get()
        };

        *handle = self.loader.create_swapchain(&create_info, None)?;

        for view in &*self.image_views.get() {
            device.destroy_image_view(*view, None);
        }

        self.loader
            .destroy_swapchain(create_info.old_swapchain, None);

        *self.image_views.get() = Self::create_swapchain_image_views(
            device,
            &self.loader,
            *handle,
            create_info.image_format,
        )?;

        Ok(())
    }

    pub fn image_format(&self) -> vk::Format {
        unsafe { (*self.create_info.get()).image_format }
    }

    /// # Safety
    /// there must not currently be written on to one of the swapchain images
    pub unsafe fn destroy(&self, device: &ash::Device) {
        for view in &*self.image_views.get() {
            device.destroy_image_view(*view, None);
        }

        let handle = *self.handle.get();
        self.loader.destroy_swapchain(handle, None);
    }
}
