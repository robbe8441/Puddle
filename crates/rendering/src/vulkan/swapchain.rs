use super::{MemoryBlock, VulkanDevice};
use ash::prelude::VkResult;
use ash::vk;
use std::cell::UnsafeCell;
use std::sync::Arc;

pub struct SwapchainImage {
    pub main_image: vk::Image, // does not need to be destroyed manually
    pub main_view: vk::ImageView,

    pub depth_image: vk::Image,
    pub depth_memory: MemoryBlock,
    pub depth_view: vk::ImageView,

    pub normal_image: vk::Image,
    pub normal_memory: MemoryBlock,
    pub normal_view: vk::ImageView,

    pub available: vk::Fence, // also does not need to be destroyed
}

impl SwapchainImage {
    unsafe fn destroy(&self, device: &VulkanDevice) {
        device.destroy_image_view(self.main_view, None);

        device.destroy_image_view(self.depth_view, None);
        device.destroy_image(self.depth_image, None);

        device.destroy_image_view(self.normal_view, None);
        device.destroy_image(self.normal_image, None);
    }
}

pub struct Swapchain {
    device: Arc<VulkanDevice>,
    pub handle: vk::SwapchainKHR,
    pub loader: ash::khr::swapchain::Device,
    pub images: Vec<SwapchainImage>,
    pub create_info: vk::SwapchainCreateInfoKHR<'static>,
}

impl Swapchain {
    /// # Safety
    /// # Errors
    pub unsafe fn new(device: Arc<VulkanDevice>, image_extent: [u32; 2]) -> VkResult<Self> {
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

        let mut desired_image_count = surface_capabilities.min_image_count.max(3);
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        };

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

        let swapchain_loader = ash::khr::swapchain::Device::new(&device.instance, &device);

        let swapchain = swapchain_loader.create_swapchain(&swapchain_create_info, None)?;

        let images = Self::create_swapchain_images(
            device.clone(),
            &swapchain_loader,
            swapchain,
            surface_format.format,
            image_extent,
        )?;

        Ok(Self {
            device,
            handle: swapchain,
            loader: swapchain_loader,
            create_info: swapchain_create_info,
            images,
        })
    }

    unsafe fn create_swapchain_images(
        device: Arc<VulkanDevice>,
        swapchain_loader: &ash::khr::swapchain::Device,
        swapchain: vk::SwapchainKHR,
        format: vk::Format,
        image_extent: [u32; 2],
    ) -> VkResult<Vec<SwapchainImage>> {
        let swapchain_images = swapchain_loader.get_swapchain_images(swapchain)?;

        Ok(swapchain_images
            .iter()
            .map(|&main_image| {
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
                    .image(main_image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .components(components)
                    .subresource_range(subresource_range);

                let main_view = device.create_image_view(&info, None).unwrap();

                let (normal_memory, normal_image, normal_view) =
                    create_texture(&device, image_extent, vk::Format::R32G32B32A32_SFLOAT).unwrap();

                let (depth_memory, depth_image, depth_view) =
                    create_texture(&device, image_extent, vk::Format::R32_SFLOAT).unwrap();

                SwapchainImage {
                    main_image,
                    main_view,
                    depth_image,
                    depth_memory,
                    depth_view,
                    normal_image,
                    normal_memory,
                    normal_view,
                    available: vk::Fence::null(),
                }
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
        &mut self,
        device: Arc<VulkanDevice>,
        new_extent: [u32; 2],
    ) -> VkResult<()> {
        let image_extent = vk::Extent2D {
            width: new_extent[0],
            height: new_extent[1],
        };

        self.create_info.image_extent = image_extent;

        let create_info = vk::SwapchainCreateInfoKHR {
            old_swapchain: self.handle,
            ..self.create_info
        };

        self.handle = self.loader.create_swapchain(&create_info, None)?;

        for image in &self.images {
            image.destroy(&device);
        }

        self.loader
            .destroy_swapchain(create_info.old_swapchain, None);

        self.images = Self::create_swapchain_images(
            device,
            &self.loader,
            self.handle,
            create_info.image_format,
            new_extent,
        )?;

        Ok(())
    }

    pub fn image_format(&self) -> vk::Format {
        self.create_info.image_format
    }

    pub fn get_image_extent(&self) -> vk::Extent2D {
        self.create_info.image_extent
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for image in &self.images {
                image.destroy(&self.device);
            }

            self.loader.destroy_swapchain(self.handle, None);
        }
    }
}

unsafe fn create_texture(
    device: &Arc<VulkanDevice>,
    image_extent: [u32; 2],
    format: vk::Format,
) -> VkResult<(MemoryBlock, vk::Image, vk::ImageView)> {
    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width: image_extent[0],
            height: image_extent[1],
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

    let image = device.create_image(&image_info, None)?;

    let memory_requirements = device.get_image_memory_requirements(image);
    let memory = MemoryBlock::new(
        device.clone(),
        memory_requirements,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    device.bind_image_memory(image, memory.handle(), 0)?;

    let subresource = vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(subresource);

    let view = device.create_image_view(&view_info, None)?;

    Ok((memory, image, view))
}
