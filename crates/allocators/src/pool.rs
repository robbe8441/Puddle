use std::{ffi::c_void, ptr::null_mut};

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
pub struct PoolAllocator<T> {
    head: *mut T,
}

impl<T> PoolAllocator<T> {
    /// # Panics
    /// if there was an issue allocating new memory
    /// if pool size is smaller than the size of a pointer
    /// # Safety
    /// the size of the memory needs to be at least ``size_of::<T>() * pool_count``
    /// the memory needs to be deallocated manually (to allow using custom allocators)
    #[must_use]
    pub unsafe fn new(memory: *mut c_void, pool_count: usize) -> Self {
        let pool_size = size_of::<T>();

        assert!(
            memory.cast::<T>().is_aligned(),
            "memory needs to have an alignment to fit a pointer : {}",
            align_of::<usize>()
        );
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
                node.cast::<*mut c_void>().write(next_node);
            };
        }

        // the last node needs to be a null ptr
        unsafe {
            memory
                .add((pool_count - 1) * pool_size)
                .cast::<*mut c_void>()
                .write(null_mut());
        }

        Self {
            head: memory.cast(),
        }
    }

    #[must_use]
    pub fn allocate(&mut self) -> *mut T {
        if self.head.is_null() {
            return null_mut();
        }
        let ptr = self.head;
        self.head = unsafe { *self.head.cast::<*mut T>() };

        ptr
    }

    pub fn free(&mut self, ptr: *mut T) {
        // write the current head pointer to this memory
        unsafe { ptr.cast::<*mut T>().write(self.head) };

        self.head = ptr;
    }
}
