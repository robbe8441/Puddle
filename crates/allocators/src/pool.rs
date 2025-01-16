#![allow(clippy::cast_ptr_alignment)]
use std::{marker::PhantomData, ptr::null_mut};

/// a ``pool_allocator`` is used to allocate multiple allocations of the same size (pools)
/// this can be used for example to store game objects
/// as every pool has the same size, this also avoids memory fragmentation
/// other than the ``StackAllocator`` the order of witch the objects are deallocated doesn't matter
///
/// as the pool doesnt have access to data that is currently allocated data,
/// resizing isn't possible
///
/// allocating works like a linked list
/// every empty pool contains a pointer to the next empty pool (or null)
/// allocating is just replacing the head with the pointer of the next empty node
/// and returning the head
#[repr(transparent)]
pub struct PoolAllocator {
    head: *mut i8,
}

impl PoolAllocator {
    /// # Panics
    /// if there was an issue allocating new memory
    /// if pool size is smaller than the size of a pointer
    /// # Safety
    /// the size of the memory needs to be at least ``size_of::<T>() * pool_count``
    /// the memory needs to be deallocated manually (to allow using custom allocators)
    #[must_use]
    pub unsafe fn new(memory: *mut i8, pool_size: usize, pool_count: usize) -> Self {
        assert!(
            pool_size >= std::mem::size_of::<usize>(),
            "a pool needs to have at least the size of an pointer: {} bytes",
            size_of::<usize>()
        );

        for i in 0..pool_count - 1 {
            unsafe {
                let node = memory.add(i * pool_size);
                let next_node = node.add(pool_size);

                // store the pointer to the next node
                node.cast::<*mut i8>().write(next_node);
            };
        }

        // the last node needs to be a null ptr
        unsafe {
            memory
                .add((pool_count - 1) * pool_size)
                .cast::<*mut i8>()
                .write(null_mut());
        }

        Self {
            head: memory.cast(),
        }
    }

    #[must_use]
    pub fn allocate(&mut self) -> *mut i8 {
        if self.head.is_null() {
            return null_mut();
        }
        let ptr = self.head;
        self.head = unsafe { *self.head.cast::<*mut i8>() };

        ptr
    }

    pub fn free(&mut self, ptr: *mut i8) {
        // write the current head pointer to this memory
        unsafe { ptr.cast::<*mut i8>().write(self.head) };

        self.head = ptr;
    }
}

#[repr(transparent)]
pub struct TypedPoolAllocator<T> {
    pool: PoolAllocator,
    _maker: PhantomData<T>,
}

impl<T> TypedPoolAllocator<T> {
    /// # Safety 
    /// see ``PoolAllocator``
    pub unsafe fn new(memory: *mut i8, pool_count: usize) -> Self {
        Self {
            pool: PoolAllocator::new(memory, size_of::<T>(), pool_count),
            _maker: PhantomData,
        }
    }
    pub fn allocate(&mut self) -> *mut T {
        self.pool.allocate().cast()
    }
    pub fn free(&mut self, ptr: *mut T) {
        self.pool.free(ptr.cast());
    }
}
