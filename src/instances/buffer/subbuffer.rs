use anyhow::Result;
use ash::vk;
use std::sync::Arc;

use super::{raw_buffer::RawBuffer, BufferAllocation};
use crate::instances::Device;

pub struct Subbuffer<T> {
    raw_buffer: Arc<RawBuffer>,
    device: Arc<Device>,

    size: u64,
    offset: u64,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Subbuffer<T> {
    pub fn from_raw(raw_buffer: Arc<RawBuffer>) -> Result<Arc<Subbuffer<T>>> {
        let device = raw_buffer.device();

        Ok(Arc::new(Subbuffer {
            size: raw_buffer.size(),
            offset: 0,
            device,
            raw_buffer,
            _marker: std::marker::PhantomData,
        }))
    }

    pub fn empty(
        device: Arc<Device>,
        mut create_info: vk::BufferCreateInfo,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Arc<Subbuffer<T>>> {
        let raw_buffer = RawBuffer::new(device, create_info, property_flags)?;
        Ok(Subbuffer::from_raw(raw_buffer)?)
    }
}

impl<T: Copy> Subbuffer<T> {
    pub fn from_data(
        device: Arc<Device>,
        mut create_info: vk::BufferCreateInfo,
        property_flags: vk::MemoryPropertyFlags,
        data: &[T],
    ) -> Result<Arc<Subbuffer<T>>> {
        create_info.size = (std::mem::size_of::<T>() * data.len()) as u64;

        let raw_buffer = RawBuffer::new(device, create_info, property_flags)?;

        let subbuffer = Subbuffer::from_raw(raw_buffer)?;

        subbuffer.write(data)?;

        Ok(subbuffer)
    }

    pub fn read(&self) -> Result<&[T]> {
        let device_raw = self.device.as_raw();

        let ptr = unsafe {
            device_raw.map_memory(
                self.raw_buffer.memory().as_raw(),
                self.offset,
                self.size,
                vk::MemoryMapFlags::empty(),
            )
        }? as *mut T;

        let data = unsafe {
            std::slice::from_raw_parts(ptr, self.size as usize / std::mem::size_of::<T>())
        };

        // unsafe { device_raw.unmap_memory(self.raw_buffer.memory()) };

        Ok(data)
    }

    pub fn write(&self, data: &[T]) -> Result<()> {
        let device_raw = self.device.as_raw();

        let ptr = unsafe {
            device_raw.map_memory(
                self.raw_buffer.memory().as_raw(),
                self.offset,
                self.size,
                vk::MemoryMapFlags::empty(),
            )
        }?;

        let mut align =
            unsafe { ash::util::Align::new(ptr, std::mem::align_of::<T>() as u64, self.size) };

        align.copy_from_slice(&data);

        // unsafe { device_raw.unmap_memory(self.raw_buffer.memory()) };

        Ok(())
    }

    pub fn desc(&self) -> vk::DescriptorBufferInfo {
        vk::DescriptorBufferInfo {
            buffer: self.buffer_raw(),
            offset: self.offset,
            range: self.size,
        }
    }
}

impl<T> super::BufferAllocation for Subbuffer<T> {
    fn offset(&self) -> u64 {
        self.offset
    }
    fn size(&self) -> u64 {
        self.size
    }

    fn buffer_raw(&self) -> vk::Buffer {
        self.raw_buffer.as_raw()
    }
}
