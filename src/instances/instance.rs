use std::{ffi::{c_char, CStr}, sync::Arc};

use anyhow::Result;
use ash::vk;

#[repr(C)]
pub struct Instance {
    intern: ash::Instance,

    // this the API entry and needs to be kept until the end of our vulkan program
    #[allow(unused)]
    entry: ash::Entry,
}

impl Instance {
    // Note: do not use for rendering, you need the surface extensions
    pub fn new_default() -> Result<Arc<Self>> {
        unsafe { Self::from_extensions(&[]) }
    }

    pub unsafe fn from_extensions(extensions: &[*const c_char]) -> Result<Arc<Self>> {
        // load the vulkan library
        let entry = ash::Entry::load()?;

        //  TODO: find an engine name
        let application = vk::ApplicationInfo::default()
            .api_version(vk::API_VERSION_1_0)
            .application_name(CStr::from_bytes_with_nul_unchecked(b"my app\n"));

        let create_info = vk::InstanceCreateInfo::default()
            .flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR)
            .enabled_extension_names(extensions)
            .application_info(&application);

        let instance = entry.create_instance(&create_info, None)?;

        Ok(Arc::new(Self {
            intern: instance,
            entry,
        }))
    }

    pub unsafe fn enumerate_physical_devices(&self) -> Result<Vec<vk::PhysicalDevice>> {
        Ok(self.intern.enumerate_physical_devices()?)
    }

    pub unsafe fn get_physical_device_properties(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> vk::PhysicalDeviceProperties {
        self.intern.get_physical_device_properties(physical_device)
    }

    pub unsafe fn create_device(
        &self,
        physical_device: vk::PhysicalDevice,
        create_info: &vk::DeviceCreateInfo,
    ) -> Result<ash::Device> {
        Ok(self
            .intern
            .create_device(physical_device, create_info, None)?)
    }

    pub fn as_raw(&self) -> ash::Instance {
        self.intern.clone()
    }

    pub fn entry(&self) -> ash::Entry {
        self.entry.clone()
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { self.intern.destroy_instance(None) };
    }
}

#[test]
fn create_insatce() {
    let instance = unsafe { Instance::from_extensions(&[]) };
    drop(instance);
}
