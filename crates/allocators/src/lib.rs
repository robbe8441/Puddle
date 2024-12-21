#![feature(pointer_is_aligned_to)]
mod freelist;
mod pool;
mod stack;

pub use freelist::{FreeListPtr, FreeListAllocator};
pub use pool::{PoolAllocator, TypedPoolAllocator};
pub use stack::StackAllocator;
