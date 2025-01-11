use std::{ffi::c_void, ptr::NonNull, sync::Arc, usize};

use ash::{prelude::VkResult, vk};

use crate::vulkan::VulkanDevice;

use super::MemoryBlock;

pub struct Buffer {
    memory: Arc<MemoryBlock>,
    handle: vk::Buffer,
    size: u64,
    offset: u64,
    ptr: Option<NonNull<c_void>>,
}

impl Buffer {
    /// # Errors
    /// if there is no space left to allocate
    pub fn new(
        device: Arc<VulkanDevice>,
        size: u64,
        usage: vk::BufferUsageFlags,
        property_flags: vk::MemoryPropertyFlags,
    ) -> VkResult<Arc<Self>> {
        let create_info = vk::BufferCreateInfo::default().size(size).usage(usage);

        let buffer = unsafe { device.create_buffer(&create_info, None) }?;
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        let memory = MemoryBlock::new(device.clone(), requirements, property_flags)?;
        unsafe { device.bind_buffer_memory(buffer, memory.memory, 0) }?;

        let ptr = if property_flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE) {
            let ptr = unsafe {
                device.map_memory(memory.handle(), 0, size, vk::MemoryMapFlags::empty())
            }?;
            NonNull::new(ptr)
        } else {
            None
        };

        Ok(Self {
            memory: Arc::new(memory),
            handle: buffer,
            size,
            offset: 0,
            ptr,
        }
        .into())
    }

    /// offset is in units of T, like an array index instead of Bytes
    /// # Panics
    /// if the buffer wasn't created with ``MemoryPropertyFlags::HOST_VISIBLE``
    pub fn write<T: Copy>(&self, offset: usize, data: &[T]) {
        let Some(ptr) = self.ptr else {
            panic!("trying to write to a buffer that isnt host visible");
        };

        assert!(ptr.as_ptr() as usize % align_of::<T>() == 0);

        let ptr = unsafe { ptr.as_ptr().cast::<T>().add(offset) };

        let len = data.len().min(self.size as usize / size_of::<T>());

        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
        slice.copy_from_slice(data);
    }

    /// # Panics
    /// if the buffer wasn't created with ``MemoryPropertyFlags::HOST_VISIBLE``
    /// # Safety
    /// this might contain uninitialized data
    #[must_use]
    pub fn read<T: Copy>(&self) -> &[T] {
        let Some(ptr) = self.ptr else {
            panic!("trying to write to a buffer that isnt host visible");
        };

        assert!(ptr.as_ptr() as usize % align_of::<T>() == 0);

        let ptr = ptr.as_ptr().cast::<T>();

        unsafe { std::slice::from_raw_parts(ptr, self.size as usize / size_of::<T>()) }
    }

    /// # Panics
    /// if the buffer wasn't created with ``MemoryPropertyFlags::HOST_VISIBLE``
    /// # Safety
    /// this might contain uninitialized data
    #[allow(clippy::mut_from_ref)] // we don't mutate the struct, just the data
    #[must_use]
    pub fn read_mut<T>(&self) -> &mut [T] {
        let Some(ptr) = self.ptr else {
            panic!("trying to read from a buffer that isnt devcie local");
        };

        assert!(ptr.as_ptr() as usize % align_of::<T>() == 0);

        let ptr = ptr.as_ptr().cast::<T>();

        unsafe { std::slice::from_raw_parts_mut(ptr, self.size as usize / size_of::<T>()) }
    }

    #[must_use]
    pub fn handle(&self) -> vk::Buffer {
        self.handle
    }
    #[must_use]
    pub fn mem_ref(&self) -> &MemoryBlock {
        &self.memory
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.memory.device.destroy_buffer(self.handle, None);
        }
    }
}
