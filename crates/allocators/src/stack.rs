use std::{alloc::Layout, ptr::null_mut};

/// the ``StackAllocator`` is good to use for algorithms
/// it eliminates memory fragmentation and improves cache locality
/// by managing memory allocations with popping old and pushing new data to a stack
/// the layout of the memory may look something like this
/// |    data1    | data2 | data3 |   free space |
///
/// data can only be allocated/freed at the end of the stack
/// what also means that you cant just drop data in any order
/// memory is freed using a ``StackMarker``
/// it marks a point in memory
/// |    data1    | data2 | data3 |   free space |
///               |
///             marker
///
/// once ``free_to_marker`` is called everything up until the marker is freed
/// and can now be reused
///
/// if an allocation is made without having enough space left,
/// the allocator tries to resize to the size needed by the allocation
/// this is slow and should be avoided, its main use is not to crash if no space if left
pub struct StackAllocator {
    /// the pointer to the memory
    memory: *mut i8,

    /// how big that memory is (bytes)
    mem_size: usize,

    /// how much of that memory is currently used (bytes)
    mem_used: usize,
}

/// marks a point in memory
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct StackMarker {
    marker: usize,
}

impl StackAllocator {
    #[must_use]
    /// # Panics
    /// if allocation fails
    /// # Safety
    /// the memory needs to be deallocated manually (to allow using custom allocators)
    pub fn new(mem: *mut i8, mem_size: usize) -> Self {
        Self {
            memory: mem,
            mem_size,
            mem_used: 0,
        }
    }

    unsafe fn allocate_unaligned(&mut self, size: usize) -> *mut i8 {
        let old_size = self.mem_used;
        self.mem_used += size;

        if self.mem_used > self.mem_size {
            return null_mut();
            // resizing invalidated all pointers!!
            // (needs smart pointers)
            //  self.resize(self.mem_used);
        }

        self.memory.add(old_size)
    }

    /// return null if there was an issue resizing the memory
    /// # Panics
    /// if align is bigger than 128
    pub fn allocate(&mut self, layout: Layout) -> *mut i8 {
        assert!(
            layout.align() <= 128,
            "align must be smaller than 128 bytes"
        );


        let ptr = unsafe { self.memory.add(self.mem_used) } as usize;
        let padding = (layout.align() - (ptr % layout.align())) % layout.align();

        let expanding_size = layout.size() + padding;

        let raw_mem = unsafe { self.allocate_unaligned(expanding_size) };

        if raw_mem.is_null() {
            return null_mut();
        }

        #[allow(clippy::cast_possible_truncation)]
        unsafe {
            // write the adjustment to the last byte before the allocation
            raw_mem.add(padding)
        }
    }

    /// gets the marker to the current end of the stack
    /// this will clear everything that has been allocated after this marker
    pub fn get_marker(&mut self) -> StackMarker {
        StackMarker {
            marker: self.mem_used,
        }
    }

    /// # Safety
    /// this invalidates every pointer allocated after this marker
    pub unsafe fn free_to_marker(&mut self, marker: StackMarker) {
        self.mem_used = marker.marker;
    }

    /// # Safety
    /// this invalidates every pointer
    pub unsafe fn clear(&mut self) {
        self.mem_used = 0;
    }
}
