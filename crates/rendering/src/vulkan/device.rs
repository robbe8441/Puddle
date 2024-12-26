use std::ops::Deref;

use ash::vk;

#[cfg(debug_assertions)]
const DEBUG_LAYER: &std::ffi::CStr = c"VK_LAYER_KHRONOS_validation";

#[allow(unused)]
pub struct VulkanDevice {
    pub entry: ash::Entry,
    pub instance: ash::Instance,

    pub pdevice: vk::PhysicalDevice,
    pub device: ash::Device,
    pub queues: DeviceQueues,

    pub surface: vk::SurfaceKHR,
    pub surface_loader: ash::khr::surface::Instance,

    #[cfg(debug_assertions)]
    debugger: debug::DebugHandler,
}

impl VulkanDevice {
    /// # Safety
    /// # Panics
    /// # Errors
    /// if the vulkan API isn't available
    pub unsafe fn new<T>(window: &T) -> Result<Self, vk::Result>
    where
        T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        let window_handle = window.window_handle().unwrap();
        let display_handle = window.display_handle().unwrap();

        let (instance, entry) = create_instance(&display_handle)?;

        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

        let surface = ash_window::create_surface(
            &entry,
            &instance,
            display_handle.as_raw(),
            window_handle.as_raw(),
            None,
        )?;

        let pdevice = get_physical_device(&instance, &surface_loader, surface)?;

        let (device, queues) = create_device(&instance, pdevice)?;

        Ok(Self {
            #[cfg(debug_assertions)]
            debugger: debug::setup_debugger(&instance, &entry),
            entry,
            instance,
            pdevice,
            device,
            queues,
            surface,
            surface_loader,
        })
    }

    pub unsafe fn destroy(&self) {
        let _ = self.device.device_wait_idle();
        #[cfg(debug_assertions)]
        self.debugger.destroy();
        self.surface_loader.destroy_surface(self.surface, None);
        self.device.destroy_device(None);
        self.instance.destroy_instance(None);
    }
}

impl Deref for VulkanDevice {
    type Target = ash::Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

/// create a vulkan Instance and entry
/// the entry point is rust specific, we need it to interact with the C library,
/// the instance contains all the vulkan library data,
/// as vulkan doesn't use global variables for that
unsafe fn create_instance(
    display_handle: &raw_window_handle::DisplayHandle,
) -> Result<(ash::Instance, ash::Entry), vk::Result> {
    let entry = ash::Entry::load().unwrap();

    let mut extensions =
        ash_window::enumerate_required_extensions(display_handle.as_raw())?.to_vec();

    #[cfg(debug_assertions)]
    extensions.push(ash::ext::debug_utils::NAME.as_ptr());

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());
        extension_names.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
    }

    let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } else {
        vk::InstanceCreateFlags::default()
    };

    let app_info = vk::ApplicationInfo::default()
        .engine_name(c"puddle")
        .engine_version(vk::API_VERSION_1_0)
        .api_version(vk::API_VERSION_1_3);

    let instance_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .flags(create_flags)
        .enabled_extension_names(&extensions);

    // handle debug stuff
    #[cfg(debug_assertions)]
    let debug_layers = [DEBUG_LAYER.as_ptr()];

    #[cfg(debug_assertions)]
    let mut sync_layers = vk::ValidationFeaturesEXT::default()
        .enabled_validation_features(&[vk::ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION]);

    #[cfg(debug_assertions)]
    let instance_info = instance_info
        .push_next(&mut sync_layers)
        .enabled_layer_names(&debug_layers);

    let instance = entry.create_instance(&instance_info, None)?;

    Ok((instance, entry))
}

/// choose the best fitting GPU that supports our needs
/// this is just used to gather some information
/// and then create the logical device that's gonna be used for everything from then on
unsafe fn get_physical_device(
    instance: &ash::Instance,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<vk::PhysicalDevice, vk::Result> {
    let pdevices = instance.enumerate_physical_devices()?;

    let pdevice = pdevices
        .iter()
        .filter_map(|pdevice| {
            let queue_infos = instance.get_physical_device_queue_family_properties(*pdevice);

            // the device just needs to support rendering
            // that also means that it supports compute and transfer
            // we also need to check if its able to render to the canvas we want to render on
            #[allow(clippy::cast_possible_truncation)]
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
        .expect("failed to find matching physical device");

    Ok(pdevice)
}

#[derive(Debug)]
#[allow(unused)]
pub struct DeviceQueues {
    pub graphics: (u32, vk::Queue),
    pub compute: (u32, vk::Queue),
}

/// create the logical device
/// this is our interaction point with our GPU and is used for basically everything
#[allow(clippy::cast_possible_truncation)]
unsafe fn create_device(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
) -> Result<(ash::Device, DeviceQueues), vk::Result> {
    let queue_props = instance.get_physical_device_queue_family_properties(pdevice);

    // use unwrap here because we already know that it supports all of them and should not error
    let (graphics_family, _) =
        get_best_queue_family(&queue_props, vk::QueueFlags::GRAPHICS).unwrap();

    let (compute_family, compute_queue_info) =
        get_best_queue_family(&queue_props, vk::QueueFlags::COMPUTE).unwrap();

    assert!(graphics_family != compute_family, "gpu not supported yet"); // TODO

    let compute_priorities = vec![0.5; compute_queue_info.queue_count as usize];

    let queue_infos = [
        vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_family as u32)
            .queue_priorities(&[1.0]),
        vk::DeviceQueueCreateInfo::default()
            .queue_family_index(compute_family as u32)
            .queue_priorities(&compute_priorities),
    ];

    let device_extensions = [
        ash::khr::swapchain::NAME.as_ptr(),
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        ash::khr::portability_subset::NAME.as_ptr(),
    ];
    let device_create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extensions);

    let device = instance.create_device(pdevice, &device_create_info, None)?;

    let graphics_queue = (
        graphics_family as u32,
        device.get_device_queue(graphics_family as u32, 0),
    );

    let compute_queue = (
        compute_family as u32,
        device.get_device_queue(compute_family as u32, 0),
    );

    Ok((
        device,
        DeviceQueues {
            graphics: graphics_queue,
            compute: compute_queue,
        },
    ))
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

#[cfg(debug_assertions)]
mod debug {
    use ash::{ext::debug_utils, vk};
    pub struct DebugHandler {
        debug_utils: debug_utils::Instance,
        debug_call_back: vk::DebugUtilsMessengerEXT,
    }

    impl DebugHandler {
        pub unsafe fn destroy(&self) {
            self.debug_utils
                .destroy_debug_utils_messenger(self.debug_call_back, None);
        }
    }

    pub fn setup_debugger(instance: &ash::Instance, entry: &ash::Entry) -> DebugHandler {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let debug_utils = debug_utils::Instance::new(entry, instance);

        let debug_call_back = unsafe {
            debug_utils
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        };

        DebugHandler {
            debug_utils,
            debug_call_back,
        }
    }

    unsafe extern "system" fn vulkan_debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
        _user_data: *mut std::os::raw::c_void,
    ) -> vk::Bool32 {
        let callback_data = *p_callback_data;
        let message_id_number = callback_data.message_id_number;

        let message_id_name = if callback_data.p_message_id_name.is_null() {
            std::borrow::Cow::from("")
        } else {
            std::ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
        };

        let message = if callback_data.p_message.is_null() {
            std::borrow::Cow::from("")
        } else {
            std::ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
        };

        if log::log_enabled!(log::Level::Error) {
            match message_type {
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => log::info!(
                "{message_severity:?}:\n[{message_id_name} ({message_id_number})] : {message}\n",
            ),
                vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => log::debug!(
                "{message_severity:?}:\n[{message_id_name} ({message_id_number})] : {message}\n",
            ),
                vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING => log::error!(
                "{message_severity:?}:\n[{message_id_name} ({message_id_number})] : {message}\n",
            ),
                _ => {}
            }
        } else {
            println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );
        }

        vk::FALSE
    }
}
