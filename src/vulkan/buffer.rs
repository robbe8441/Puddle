use ash::vk;
use std::{ffi::c_void, ptr::null_mut};

use super::VulkanContext;

#[allow(unused)]
pub struct Buffer {
    mem: vk::DeviceMemory,
    handle: vk::Buffer,
    usage: vk::BufferUsageFlags,
    ptr: *mut c_void,
    requirements: vk::MemoryRequirements,
    data_size: u64,
}

impl VulkanContext {
    pub fn create_buffer(
        &self,
        size: u64,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Buffer, vk::Result> {
        Buffer::new(
            &self.device,
            &self.instance,
            self.pdevice,
            size,
            usage,
            properties,
        )
    }
}

impl Buffer {
    pub fn new(
        device: &ash::Device,
        instance: &ash::Instance,
        pdevice: vk::PhysicalDevice,
        size: u64,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Self, vk::Result> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };

        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        let memory_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(
                get_memory_type_index(instance, pdevice, properties, requirements)
                    .ok_or(vk::Result::ERROR_FORMAT_NOT_SUPPORTED)?,
            );

        let mem = unsafe { device.allocate_memory(&memory_info, None)? };
        unsafe { device.bind_buffer_memory(buffer, mem, 0) }?;

        let ptr = if properties.contains(vk::MemoryPropertyFlags::HOST_VISIBLE) {
            unsafe { device.map_memory(mem, 0, size, vk::MemoryMapFlags::empty()) }?
        } else {
            null_mut()
        };

        Ok(Self {
            mem,
            handle: buffer,
            usage,
            ptr,
            requirements,
            data_size: size,
        })
    }

    pub fn write<T: Copy>(&self, data: &[T]) {
        assert!(!self.ptr.is_null());

        let mut align = unsafe {
            ash::util::Align::new(self.ptr, std::mem::align_of::<T>() as u64, self.data_size)
        };

        align.copy_from_slice(data);
    }

    pub fn read<T>(&self) -> &[T] {
        assert!(!self.ptr.is_null());

        unsafe {
            std::slice::from_raw_parts(
                self.ptr.cast(),
                self.data_size as usize / std::mem::size_of::<T>(),
            )
        }
    }

    pub unsafe fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_buffer(self.handle, None);
            device.free_memory(self.mem, None);
        }
    }

    pub fn memory(&self) -> vk::DeviceMemory {
        self.mem
    }

    pub fn usage(&self) -> vk::BufferUsageFlags {
        self.usage
    }

    pub fn as_raw(&self) -> vk::Buffer {
        self.handle
    }
}

fn get_memory_type_index(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Option<u32> {
    let memory = unsafe { instance.get_physical_device_memory_properties(pdevice) };

    (0..memory.memory_type_count).find(|i| {
        let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
        let memory_type = memory.memory_types[*i as usize];
        suitable && memory_type.property_flags.contains(properties)
    })
}
