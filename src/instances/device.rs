use std::sync::Arc;

use anyhow::{Context, Result};
use ash::{
    khr::swapchain,
    vk::{self, PhysicalDevice},
};

pub struct Device {
    intern: ash::Device,
    instance: Arc<super::Instance>,
}

pub struct Queue {
    intern: vk::Queue,
    queue_family_index: u32,
    device: Arc<Device>,
}

impl Device {
    pub fn new_default(instance: Arc<super::Instance>) -> Result<(Arc<Self>, Arc<Queue>)> {
        let features = vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };

        let (pdevice, queue_index) = unsafe {
            Self::get_pdevice_with_queue_flags(instance.clone(), vk::QueueFlags::GRAPHICS)
        }
        .unwrap();

        unsafe { Self::from_features(instance, pdevice, queue_index as u32, &features) }
    }

    pub unsafe fn from_features(
        instance: Arc<super::Instance>,
        pdevice: PhysicalDevice,
        queue_family_index: u32,
        features: &vk::PhysicalDeviceFeatures,
    ) -> Result<(Arc<Self>, Arc<Queue>)> {
        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);

        let device_extension_names_raw = [
            swapchain::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ash::khr::portability_subset::NAME.as_ptr(),
        ];

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        let device_raw = instance.create_device(pdevice, &device_create_info)?;
        let queue_raw = device_raw.get_device_queue(queue_family_index, 0);

        let device = Arc::new(Device {
            intern: device_raw,
            instance: instance.clone(),
        });

        let queue = Arc::new(Queue {
            intern: queue_raw,
            queue_family_index,
            device: device.clone(),
        });

        Ok((device, queue))
    }

    pub fn instance(&self) -> Arc<super::Instance> {
        self.instance.clone()
    }

    unsafe fn get_pdevice_with_queue_flags(
        instance: Arc<super::Instance>,
        flags: vk::QueueFlags,
    ) -> Result<(PhysicalDevice, usize)> {
        Ok(unsafe { instance.enumerate_physical_devices() }?
            .iter()
            .find_map(|pdevice| {
                instance
                    .as_raw()
                    .get_physical_device_queue_family_properties(*pdevice)
                    .iter()
                    .enumerate()
                    .find_map(|(index, info)| {
                        if info.queue_flags.contains(flags) {
                            Some((*pdevice, index))
                        } else {
                            None
                        }
                    })
            })
            .context("failed to find device that supports these queue flags")?)
    }

    unsafe fn get_pdevice_from_surface(
        instance: Arc<super::Instance>,
        surface: super::Surface,
    ) -> (PhysicalDevice, usize) {
        let surface_loader = surface.loader();

        let pdevices = instance.enumerate_physical_devices().unwrap();

        pdevices
            .iter()
            .find_map(|pdevice| {
                instance
                    .as_raw()
                    .get_physical_device_queue_family_properties(*pdevice)
                    .iter()
                    .enumerate()
                    .find_map(|(index, info)| {
                        let supports_graphic_and_surface =
                            info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                && surface_loader
                                    .get_physical_device_surface_support(
                                        *pdevice,
                                        index as u32,
                                        surface.as_raw(),
                                    )
                                    .unwrap();
                        if supports_graphic_and_surface {
                            Some((*pdevice, index))
                        } else {
                            None
                        }
                    })
            })
            .unwrap()
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        unsafe { self.device.intern.destroy_device(None) };
    }
}
