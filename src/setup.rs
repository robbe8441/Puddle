use anyhow::{anyhow, Context, Result};
use std::{
    ffi::CStr,
    io::Cursor,
    sync::{Mutex, MutexGuard},
};

use ash::{
    khr::swapchain,
    vk::{self, Handle},
};
use raw_window_handle::HasDisplayHandle;

const DEBUG_LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";

use glam::{vec2, vec3, Vec2, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}

unsafe impl bytemuck::NoUninit for Vertex {}



impl Vertex {
    pub const VERTICES: [Vertex; 6] = [
        Vertex::new(vec2(-0.5, -0.5), vec3(1.0, 0.0, 0.0)),
        Vertex::new(vec2(0.5, 0.5), vec3(0.0, 1.0, 0.0)),
        Vertex::new(vec2(-0.5, 0.5), vec3(0.0, 0.0, 1.0)),

        Vertex::new(vec2(0.5, -0.5), vec3(0.0, 1.0, 0.0)), 
        Vertex::new(vec2(0.5, 0.5), vec3(1.0, 0.0, 0.0)), 
        Vertex::new(vec2(-0.5, -0.5), vec3(0.0, 0.0, 1.0)),
    ];

    pub const fn new(pos: Vec2, color: Vec3) -> Self {
        Self { pos: [pos.x, pos.y], color: [color.x, color.y, color.z] }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let pos = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0);

        let color = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<Vec2>() as u32);

        [pos, color]
    }
}

pub unsafe fn create_buffer(
    instance: &ash::Instance,
    device: &ash::Device,
    pdevice: vk::PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = device.create_buffer(&buffer_info, None)?;

    let requirements = device.get_buffer_memory_requirements(buffer);

    let memory_info = vk::MemoryAllocateInfo::default()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            pdevice,
            properties,
            requirements,
        )?);

    let buffer_memory = device.allocate_memory(&memory_info, None)?;

    device.bind_buffer_memory(buffer, buffer_memory, 0)?;

    Ok((buffer, buffer_memory))
}

unsafe fn get_memory_type_index(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Result<u32> {
    let memory = instance.get_physical_device_memory_properties(pdevice);

    (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];
            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow!("Failed to find suitable memory type."))
}

/// create a vulkan Instance and entry
/// the entry point is rust specific, we need it to interact with the C library,
/// the instance contains all the vulkan library data,
/// as vulkan doesn't use global variables for that
pub unsafe fn create_instance(
    display_handle: &impl HasDisplayHandle,
) -> Result<(ash::Instance, ash::Entry)> {
    let app_info = vk::ApplicationInfo {
        api_version: vk::API_VERSION_1_3,
        p_application_name: c"some vulkan renderer".as_ptr(),
        ..Default::default()
    };

    let raw_display_handle = display_handle.display_handle().unwrap().as_raw();
    let instance_extensions = ash_window::enumerate_required_extensions(raw_display_handle)?;

    let debug_layers = [DEBUG_LAYER.as_ptr()];

    let instance_create_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(instance_extensions)
        .enabled_layer_names(&debug_layers);

    let entry = ash::Entry::load()?;
    let instance = entry.create_instance(&instance_create_info, None)?;

    Ok((instance, entry))
}

/// normally, the less features a queue has,
/// the more specialized it is on the features it does support
/// means we want to find the queue that fits our needs, and has as less unneeded features as possible
fn get_best_queue_family(
    infos: &[vk::QueueFamilyProperties],
    flags: vk::QueueFlags,
) -> Option<(usize, &vk::QueueFamilyProperties)> {
    infos
        .iter()
        .enumerate()
        .filter(|(_, v)| v.queue_flags.contains(flags))
        .min_by_key(|(_, v)| v.queue_flags.as_raw().count_ones())
}

/// choose the best fitting GPU that supports our needs
/// this is just used to gather some information
/// and then create the logical device that's gonna be used for everything from then on
pub unsafe fn get_physical_device(
    instance: &ash::Instance,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<vk::PhysicalDevice> {
    let pdevices = instance.enumerate_physical_devices()?;

    let pdevice = pdevices
        .iter()
        .filter_map(|pdevice| {
            let queue_infos = instance.get_physical_device_queue_family_properties(*pdevice);

            // the device just needs to support rendering
            // that also means that it supports compute and transfer
            // we also need to check if its able to render to the canvas we want to render on
            queue_infos.iter().enumerate().find(|(i, v)| {
                v.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && surface_loader
                        .get_physical_device_surface_support(*pdevice, *i as u32, surface)
                        .unwrap()
            })?;

            Some(*pdevice)
        })
        .min_by_key(|pdevice| {
            let props = instance.get_physical_device_properties(*pdevice);

            match props.device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU => 0,
                _ => 1,
            }
        })
        .context("failed to find matching physical device")?;

    Ok(pdevice)
}

#[derive(Debug)]
#[allow(unused)]
pub struct DeviceQueues {
    pub graphics: (u32, Mutex<vk::Queue>),
    pub compute: (u32, Vec<Mutex<vk::Queue>>),
}

#[allow(unused)]
impl DeviceQueues {
    // search for an compute queue that isn't locked, and then lock it
    pub fn get_compute_queue(&self) -> (u32, Option<MutexGuard<vk::Queue>>) {
        (
            self.compute.0,
            self.compute.1.iter().find_map(|v| v.try_lock().ok()),
        )
    }

    pub fn get_graphics_queue(&self) -> Option<(u32, MutexGuard<vk::Queue>)> {
        let (family, queue) = &self.graphics;
        Some((*family, queue.try_lock().ok()?))
    }
}

/// create the logical device
/// this is our interaction point with our GPU and is used for basically everything
pub unsafe fn create_device(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
) -> Result<(ash::Device, DeviceQueues)> {
    let queue_props = instance.get_physical_device_queue_family_properties(pdevice);

    // use unwrap here because we already know that it supports all of them and should not error
    let (graphics_family, _) =
        get_best_queue_family(&queue_props, vk::QueueFlags::GRAPHICS).unwrap();

    let (compute_family, compute_queue_info) =
        get_best_queue_family(&queue_props, vk::QueueFlags::COMPUTE).unwrap();

    if graphics_family == compute_family {
        return Err(anyhow!(
            "queues having the same family is not yet supported"
        )); // TODO
    }

    let compute_priorities = vec![0.5; compute_queue_info.queue_count as usize];

    let queue_infos = [
        vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_family as u32)
            .queue_priorities(&[1.0]),
        vk::DeviceQueueCreateInfo::default()
            .queue_family_index(compute_family as u32)
            .queue_priorities(&compute_priorities),
    ];

    let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
    let device_create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extensions);

    let device = instance.create_device(pdevice, &device_create_info, None)?;

    let graphics_queue = (
        graphics_family as u32,
        Mutex::new(device.get_device_queue(graphics_family as u32, 0)),
    );

    let compute_queues: Vec<_> = compute_priorities
        .into_iter()
        .enumerate()
        .map(|(i, _)| Mutex::new(device.get_device_queue(compute_family as u32, i as u32)))
        .collect();

    Ok((
        device,
        DeviceQueues {
            graphics: graphics_queue,
            compute: (compute_family as u32, compute_queues),
        },
    ))
}

/// create the surface
/// this contains data about the window/canvas we want to render on
/// needed to create the swapchain
pub unsafe fn create_surface<T>(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &T,
) -> Result<(vk::SurfaceKHR, ash::khr::surface::Instance)>
where
    T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
{
    let surface_loader = ash::khr::surface::Instance::new(entry, instance);

    let window_handle = window.window_handle().unwrap().as_raw();
    let display_handle = window.display_handle().unwrap().as_raw();

    let surface = ash_window::create_surface(entry, instance, display_handle, window_handle, None)?;

    Ok((surface, surface_loader))
}

/// a queue of images queued to be shown on the screen
pub struct Swapchain {
    pub handle: vk::SwapchainKHR,
    pub loader: ash::khr::swapchain::Device,
    pub image_views: Vec<vk::ImageView>,
    pub create_info: vk::SwapchainCreateInfoKHR<'static>,
}

impl Swapchain {
    pub unsafe fn create_swapchain(
        pdevice: vk::PhysicalDevice,
        device: &ash::Device,
        instance: &ash::Instance,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
        image_extent: [u32; 2],
    ) -> Result<Swapchain> {
        let surface_capabilities =
            surface_loader.get_physical_device_surface_capabilities(pdevice, surface)?;

        let surface_format =
            surface_loader.get_physical_device_surface_formats(pdevice, surface)?[0];

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

        // let present_modes = surface_loader
        //     .get_physical_device_surface_present_modes(pdevice, surface)
        //     .unwrap();
        // let present_mode = present_modes
        //     .iter()
        //     .cloned()
        //     .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
        //     .unwrap_or(vk::PresentModeKHR::FIFO);

        let present_mode = vk::PresentModeKHR::MAILBOX; // always supported

        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
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

        let swapchain_loader = swapchain::Device::new(instance, device);

        let swapchain = swapchain_loader.create_swapchain(&swapchain_create_info, None)?;

        let image_views = Self::create_swapchain_image_views(
            device,
            &swapchain_loader,
            swapchain,
            surface_format.format,
        )?;

        Ok(Self {
            handle: swapchain,
            loader: swapchain_loader,
            create_info: swapchain_create_info,
            image_views,
        })
    }

    pub unsafe fn create_swapchain_image_views(
        device: &ash::Device,
        swapchain_loader: &ash::khr::swapchain::Device,
        swapchain: vk::SwapchainKHR,
        format: vk::Format,
    ) -> Result<Vec<vk::ImageView>> {
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

    pub unsafe fn recreate(&mut self, device: &ash::Device, new_extent: [u32; 2]) -> Result<()> {
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

        Ok(())
    }

    pub unsafe fn destroy(&self, device: &ash::Device) {
        for view in &self.image_views {
            device.destroy_image_view(*view, None);
        }

        self.loader.destroy_swapchain(self.handle, None);
    }
}

unsafe fn create_shader_module(device: &ash::Device, bytecode: &[u32]) -> Result<vk::ShaderModule> {
    let create_info = vk::ShaderModuleCreateInfo::default().code(bytecode);

    let module = device.create_shader_module(&create_info, None)?;

    Ok(module)
}

pub unsafe fn create_pipeline(
    device: &ash::Device,
    swapchain: &Swapchain,
    layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
) -> Result<vk::Pipeline> {
    use ash::util::read_spv;

    let mut vertex_spv_file = Cursor::new(&include_bytes!("../shaders/vertex.spv"));
    let mut frag_spv_file = Cursor::new(&include_bytes!("../shaders/fragment.spv"));

    let vertex_code =
        read_spv(&mut vertex_spv_file).expect("Failed to read vertex shader spv file");

    let frag_code = read_spv(&mut frag_spv_file).expect("Failed to read fragment shader spv file");

    let vertex_module = create_shader_module(device, &vertex_code)?;
    let fragment_module = create_shader_module(device, &frag_code)?;

    let vertex_stage = vk::PipelineShaderStageCreateInfo::default()
        .module(vertex_module)
        .stage(vk::ShaderStageFlags::VERTEX)
        .name(c"main");

    let fragment_stage = vk::PipelineShaderStageCreateInfo::default()
        .module(fragment_module)
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .name(c"main");

    let binding_descriptions = &[Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_descriptions();

    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

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
        .front_face(vk::FrontFace::CLOCKWISE)
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

    let dynamic_states = &[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(dynamic_states);

    let stages = &[vertex_stage, fragment_stage];
    let info = vk::GraphicsPipelineCreateInfo::default()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
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

    device.destroy_shader_module(vertex_module, None);
    device.destroy_shader_module(fragment_module, None);

    Ok(pipeline)
}

pub unsafe fn create_render_pass(
    device: &ash::Device,
    swapchain: &Swapchain,
) -> Result<vk::RenderPass> {
    let color_attachment = vk::AttachmentDescription::default()
        .format(swapchain.create_info.image_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_attachment_ref = vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachments = &[color_attachment_ref];
    let subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments);

    let dependency = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let attachments = &[color_attachment];
    let subpasses = &[subpass];
    let dependencies = &[dependency];

    let info = vk::RenderPassCreateInfo::default()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies);

    let render_pass = device.create_render_pass(&info, None)?;

    Ok(render_pass)
}

pub unsafe fn create_framebuffers(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: [u32; 2],
    image_views: &[vk::ImageView],
) -> Vec<vk::Framebuffer> {
    let frame_buffers = image_views
        .iter()
        .map(|i| {
            let attachments = &[*i];
            let create_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(extent[0])
                .height(extent[1])
                .layers(1);

            device.create_framebuffer(&create_info, None).unwrap()
        })
        .collect();

    frame_buffers
}

pub unsafe fn create_command_pool(device: &ash::Device, index: u32) -> Result<vk::CommandPool> {
    let create_info = vk::CommandPoolCreateInfo::default().queue_family_index(index);

    let command_pool = device.create_command_pool(&create_info, None)?;

    Ok(command_pool)
}

pub unsafe fn create_command_buffers(
    device: &ash::Device,
    pool: vk::CommandPool,
    count: u32,
) -> Result<Vec<vk::CommandBuffer>> {
    let create_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(count);

    let command_buffers = device.allocate_command_buffers(&create_info)?;

    Ok(command_buffers)
}

pub unsafe fn record_buffer(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    image_extent: [u32; 2],
    render_pass: vk::RenderPass,
    frame_buffer: vk::Framebuffer,
    pipeline: vk::Pipeline,
    vertex_buffer: vk::Buffer,
) -> Result<()> {
    let info = vk::CommandBufferBeginInfo::default();

    unsafe { device.begin_command_buffer(command_buffer, &info) }?;

    let render_area = vk::Rect2D::default()
        .offset(vk::Offset2D::default())
        .extent(
            vk::Extent2D::default()
                .width(image_extent[0])
                .height(image_extent[1]),
        );

    let color_clear_value = vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
        },
    };

    let clear_values = &[color_clear_value];
    let info = vk::RenderPassBeginInfo::default()
        .render_pass(render_pass)
        .framebuffer(frame_buffer)
        .render_area(render_area)
        .clear_values(clear_values);

    let viewports = [vk::Viewport::default()
        .x(0.0)
        .y(0.0)
        .width(image_extent[0] as f32)
        .height(image_extent[1] as f32)
        .min_depth(0.0)
        .max_depth(1.0)];

    let scissors = [vk::Rect2D::default()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(
            vk::Extent2D::default()
                .width(image_extent[0])
                .height(image_extent[1]),
        )];

    device.cmd_set_viewport(command_buffer, 0, &viewports);
    device.cmd_set_scissor(command_buffer, 0, &scissors);

    device.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE);

    device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);

    device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);

    device.cmd_draw(command_buffer, Vertex::VERTICES.len() as u32, 1, 0, 0);

    device.cmd_end_render_pass(command_buffer);
    device.end_command_buffer(command_buffer)?;

    Ok(())
}

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[allow(clippy::too_many_arguments)]
pub unsafe fn render(
    device: &ash::Device,
    queue: vk::Queue,
    swapchain: &Swapchain,
    command_buffers: &[vk::CommandBuffer],
    image_available_semaphores: &[vk::Semaphore],
    render_finished_semaphores: &[vk::Semaphore],
    in_flight_fences: &[vk::Fence],
    in_flight_images: &mut [vk::Fence],
    frame: usize,
) -> Result<()> {
    device.wait_for_fences(&[in_flight_fences[frame]], true, u64::MAX)?;

    let (image_index, _suboptimal) = swapchain.loader.acquire_next_image(
        swapchain.handle,
        u64::MAX,
        image_available_semaphores[frame],
        vk::Fence::null(),
    )?;

    if !in_flight_images[image_index as usize].is_null() {
        device.wait_for_fences(&[in_flight_images[image_index as usize]], true, u64::MAX)?;
    }

    in_flight_images[image_index as usize] = in_flight_fences[frame];

    let wait_semaphores = &[image_available_semaphores[frame]];
    let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    let command_buffers = &[command_buffers[image_index as usize]];
    let signal_semaphores = &[render_finished_semaphores[frame]];
    let submit_info = vk::SubmitInfo::default()
        .wait_semaphores(wait_semaphores)
        .wait_dst_stage_mask(wait_stages)
        .command_buffers(command_buffers)
        .signal_semaphores(signal_semaphores);

    device.reset_fences(&[in_flight_fences[frame]])?;
    device.queue_submit(queue, &[submit_info], in_flight_fences[frame])?;

    let swapchains = &[swapchain.handle];
    let image_indices = &[image_index];
    let present_info = vk::PresentInfoKHR::default()
        .wait_semaphores(signal_semaphores)
        .swapchains(swapchains)
        .image_indices(image_indices);

    swapchain.loader.queue_present(queue, &present_info)?;

    Ok(())
}
