use ash::vk;

mod raw_buffer;
mod subbuffer;
mod memory;

pub use subbuffer::Subbuffer;
pub use raw_buffer::RawBuffer;
pub use memory::DeviceMemory;

pub trait BufferAllocation {
    fn size(&self) -> u64;
    fn offset(&self) -> u64;
    fn buffer_raw(&self) -> vk::Buffer;
}



