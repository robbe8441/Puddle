#![feature(pointer_is_aligned_to)]
// mod freelist TODO;
mod pool;
mod stack;

// pub use freelist::{FreeListPtr, FreeListAllocator};
pub use pool::{PoolAllocator, TypedPoolAllocator};
pub use stack::StackAllocator;
