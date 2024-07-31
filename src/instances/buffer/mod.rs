use ash::vk;

mod memory;
mod raw_buffer;
mod subbuffer;

pub use memory::{find_memorytype_index, DeviceMemory};
pub use raw_buffer::RawBuffer;
pub use subbuffer::Subbuffer;

pub trait BufferAllocation {
    fn size(&self) -> u64;
    fn offset(&self) -> u64;
    fn buffer_raw(&self) -> vk::Buffer;
}
