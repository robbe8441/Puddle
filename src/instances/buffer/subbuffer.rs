use anyhow::Result;
use ash::vk;
use std::sync::Arc;

use super::raw_buffer::RawBuffer;
use crate::instances::Device;

pub struct Subbuffer<T> {
    mem: vk::DeviceMemory,
    raw_buffer: Arc<RawBuffer>,
    device: Arc<Device>,

    size: u64,
    offset: u64,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Subbuffer<T> {
    pub fn from_raw(raw_buffer: Arc<RawBuffer>) -> Result<Arc<Subbuffer<T>>> {
        let device = raw_buffer.device();
        let device_raw = device.as_raw();

        let vertex_buffer_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: raw_buffer.memory_req().size,
            memory_type_index: raw_buffer.memory_index(),
            ..Default::default()
        };

        let mem = unsafe { device_raw.allocate_memory(&vertex_buffer_allocate_info, None) }?;

        unsafe { device_raw.bind_buffer_memory(raw_buffer.as_raw(), mem, 0) }.unwrap();

        Ok(Arc::new(Subbuffer {
            device,
            raw_buffer,
            mem,
            size: vertex_buffer_allocate_info.allocation_size,
            offset: 0,
            _marker: std::marker::PhantomData,
        }))
    }




    pub fn offset(&self) -> u64 {
        self.offset
    }
    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn raw_buffer(&self) -> vk::Buffer {
        self.raw_buffer.as_raw()
    }
}
impl<T:Copy> Subbuffer<T> {

    pub fn from_data(device: Arc<Device>, create_info: vk::BufferCreateInfo, property_flags: vk::MemoryPropertyFlags, data: &[T]) -> Result<Arc<Subbuffer<T>>> {
        let raw_buffer = RawBuffer::new(device, create_info, property_flags)?;

        let subbuffer = Subbuffer::from_raw(raw_buffer)?;

        subbuffer.write(data)?;

        Ok(subbuffer)
    }


    pub fn read(&self) -> Result<&[T]> {
        if !self
            .raw_buffer
            .properties()
            .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
        {
            panic!(
                "the host is not allowed to acces this buffer, the HOST_VISIBLE flag isnt given"
            );
        }

        let device_raw = self.device.as_raw();

        let ptr = unsafe {
            device_raw.map_memory(
                self.mem,
                self.offset,
                self.size,
                vk::MemoryMapFlags::empty(),
            )
        }? as *mut T;

        let data = unsafe { std::slice::from_raw_parts(ptr, self.size as usize / std::mem::size_of::<T>()) };

        // unsafe { device_raw.unmap_memory(self.mem) };

        Ok(data)
    }


    pub fn write(&self, data: &[T]) -> Result<()> {
        if !self
            .raw_buffer
            .properties()
            .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
        {
            panic!(
                "the host is not allowed to acces this buffer, the HOST_VISIBLE flag isnt given"
            );
        }

        let device_raw = self.device.as_raw();

        let ptr = unsafe {
            device_raw.map_memory(
                self.mem,
                self.offset,
                self.size,
                vk::MemoryMapFlags::empty(),
            )
        }?;

        let mut align =
            unsafe { ash::util::Align::new(ptr, std::mem::align_of::<T>() as u64, self.size) };

        align.copy_from_slice(&data);

        // unsafe { device_raw.unmap_memory(self.mem) };

        Ok(())
    }
}

impl<T> Drop for Subbuffer<T> {
    fn drop(&mut self) {
        unsafe { self.device.as_raw().free_memory(self.mem, None) };
    }
}
