use std::sync::Arc;

use super::{BufferAllocation, Subbuffer};

pub struct BufferSlice<T> {
    buffer: Arc<Subbuffer<T>>,
    offset: u64,
    size: u64,
}

impl<T> BufferSlice<T> {
    pub fn new(buffer: Arc<Subbuffer<T>>, offset: u64, size: u64) -> Arc<Self> {
        Arc::new(Self {
            buffer,
            offset,
            size,
        })
    }
}

impl<T> BufferAllocation for BufferSlice<T> {
    fn size(&self) -> u64 {
        self.size
    }
    fn offset(&self) -> u64 {
        self.offset
    }
    fn buffer_raw(&self) -> ash::vk::Buffer {
        self.buffer.buffer_raw()
    }
}
