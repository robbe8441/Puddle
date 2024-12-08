use std::sync::Mutex;
use std::{ffi::CStr, sync::MutexGuard};

use ash::vk;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use super::{BindlessHandler, VulkanContext};

const DEBUG_LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";

impl VulkanContext {
    pub fn new<W>(window: &W) -> Result<Self, vk::Result>
    where
        W: HasDisplayHandle + HasWindowHandle,
    {
        let (instance, entry) = unsafe { create_instance(&window) }?;

        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

        let display_handle = window.display_handle().unwrap().as_raw();
        let window_handle = window.window_handle().unwrap().as_raw();

        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
        }?;

        let pdevice = unsafe { get_physical_device(&instance, &surface_loader, surface) }?;

        let (device, queues) = unsafe { create_device(&instance, pdevice) }?;

        let bindless_handler = BindlessHandler::new(&device)?;

        Ok(Self {
            entry,
            instance,
            pdevice,
            device,
            queues,
            surface_loader,
            surface,
            bindless_handler,
        })
    }
}

/// create a vulkan Instance and entry
/// the entry point is rust specific, we need it to interact with the C library,
/// the instance contains all the vulkan library data,
/// as vulkan doesn't use global variables for that
pub unsafe fn create_instance(
    display_handle: &impl HasDisplayHandle,
) -> Result<(ash::Instance, ash::Entry), vk::Result> {
    let app_info = vk::ApplicationInfo {
        api_version: vk::API_VERSION_1_3,
        p_application_name: c"some vulkan voxel renderer".as_ptr(),
        ..Default::default()
    };

    let entry = ash::Entry::linked();

    let raw_display_handle = display_handle.display_handle().unwrap().as_raw();
    let instance_extensions = ash_window::enumerate_required_extensions(raw_display_handle)?;

    let supported_layers = entry.enumerate_instance_layer_properties()?;

    // check if device supports debug layers
    let supports_debug = supported_layers
        .iter()
        .any(|v| v.layer_name_as_c_str().unwrap() == DEBUG_LAYER);

    let debug_layers = if supports_debug {
        vec![DEBUG_LAYER.as_ptr()]
    } else {
        vec![]
    };

    let mut validation_features = vk::ValidationFeaturesEXT::default()
        .enabled_validation_features(&[vk::ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION]);

    let instance_create_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(instance_extensions)
        .enabled_layer_names(&debug_layers)
        .push_next(&mut validation_features);

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
) -> Result<vk::PhysicalDevice, vk::Result> {
    let pdevices = instance.enumerate_physical_devices()?;

    pdevices
        .iter()
        .filter_map(|pdevice| {
            let queue_infos = instance.get_physical_device_queue_family_properties(*pdevice);

            let mut indexing_features = vk::PhysicalDeviceDescriptorIndexingFeatures::default();
            let mut features2 =
                vk::PhysicalDeviceFeatures2::default().push_next(&mut indexing_features);

            instance.get_physical_device_features2(*pdevice, &mut features2);

            // needed to use bindless rendering
            if (indexing_features.shader_sampled_image_array_non_uniform_indexing
                & indexing_features.descriptor_binding_sampled_image_update_after_bind
                & indexing_features.shader_uniform_buffer_array_non_uniform_indexing
                & indexing_features.descriptor_binding_uniform_buffer_update_after_bind
                & indexing_features.shader_storage_buffer_array_non_uniform_indexing
                & indexing_features.descriptor_binding_storage_buffer_update_after_bind)
                != 1
            {
                return None;
            }

            // the device just needs to support rendering
            // that also means that it supports compute and transfer
            // we also need to check if its able to render to the canvas we want to render on
            queue_infos.iter().enumerate().find(|(i, v)| {
                v.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && surface_loader
                        .get_physical_device_surface_support(*pdevice, *i as u32, surface)
                        .unwrap_or(false)
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
        .ok_or_else(|| {
            println!("failed to find a gpu with supported features");
            vk::Result::ERROR_UNKNOWN
        })
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
    // !!! Returns 'None' if there is no queue without Blocking !!!
    pub fn get_compute_queue(&self) -> (u32, Option<MutexGuard<vk::Queue>>) {
        (
            self.compute.0,
            self.compute.1.iter().find_map(|v| {
                v.try_lock()
                    .map_err(|v| {
                        if matches!(v, std::sync::TryLockError::Poisoned(_)) {
                            println!("warining:, a compute queue has been Poisoned");
                        }
                    })
                    .ok()
            }),
        )
    }

    pub fn get_graphics_queue(&self) -> (u32, Option<MutexGuard<vk::Queue>>) {
        let (family, queue) = &self.graphics;
        (*family, queue.lock().ok())
    }
}

/// create the logical device
/// this is our interaction point with our GPU and is used for basically everything
pub unsafe fn create_device(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
) -> Result<(ash::Device, DeviceQueues), vk::Result> {
    let queue_props = instance.get_physical_device_queue_family_properties(pdevice);

    // use unwrap here because we already know that it supports all of them and should not error
    let (graphics_family, _) =
        get_best_queue_family(&queue_props, vk::QueueFlags::GRAPHICS).unwrap();

    let (compute_family, compute_queue_info) =
        get_best_queue_family(&queue_props, vk::QueueFlags::COMPUTE).unwrap();

    assert!(
        (graphics_family != compute_family),
        "failed to create device, not enough queues"
    );

    let compute_priorities = vec![0.5; compute_queue_info.queue_count as usize];

    let queue_infos = [
        vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_family as u32)
            .queue_priorities(&[1.0]),
        vk::DeviceQueueCreateInfo::default()
            .queue_family_index(compute_family as u32)
            .queue_priorities(&compute_priorities),
    ];

    let mut indexing_features = vk::PhysicalDeviceDescriptorIndexingFeatures::default();
    let mut features2 = vk::PhysicalDeviceFeatures2::default().push_next(&mut indexing_features);

    instance.get_physical_device_features2(pdevice, &mut features2);

    let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
    let device_create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_infos)
        .push_next(&mut features2)
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
