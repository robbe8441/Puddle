use std::alloc::{alloc, dealloc, Layout};

use allocators::StackAllocator;

// TODO: fix ugly tests

#[test]
fn allocate() {
    unsafe {
        let stack_layout = Layout::new::<[u8; 100]>();
        let stack_memory = alloc(stack_layout);

        let mut allocator = StackAllocator::new(stack_memory.cast(), stack_layout.size());

        let layout1 = Layout::new::<u8>();
        let layout2 = Layout::new::<u128>();

        let data1 = allocator.allocate(layout1).cast::<u8>();
        *data1 = 10;

        let data2 = allocator.allocate(layout2).cast::<u128>();
        *data2 = 100;

        let marker = allocator.get_marker();
        allocator.free_to_marker(marker);

        let data3 = allocator.allocate(layout1).cast::<u8>();
        *data3 = 255;

        dealloc(stack_memory, stack_layout);
    }
}

#[test]
fn free() {
    unsafe {
        let stack_layout = Layout::new::<[u8; 100]>();
        let stack_memory = alloc(stack_layout);

        let mut allocator = StackAllocator::new(stack_memory.cast(), stack_layout.size());

        let data1 = allocator.allocate(Layout::new::<u128>()).cast::<u128>();
        *data1 = 10;

        let marker = allocator.get_marker();

        let data2 = allocator.allocate(Layout::new::<u8>()).cast::<u8>();
        *data2 = 50;

        allocator.free_to_marker(marker);

        let data3 = allocator.allocate(Layout::new::<u32>()).cast::<u32>();
        *data3 = 200;

        // data 1 shouldn't change
        assert_eq!(*data1, 10);
        // data2 should now have been overwritten by data3
        assert_eq!(*data2, 200);
        dealloc(stack_memory, stack_layout);
    }
}

#[test]
fn free_to_marker() {
    unsafe {
        let stack_layout = Layout::array::<usize>(100).unwrap();
        let stack_memory = alloc(stack_layout);

        let mut allocator = StackAllocator::new(stack_memory.cast(), stack_layout.size());
        let marker = allocator.get_marker();

        let data1 = allocator.allocate(Layout::new::<u128>()).cast::<u128>();
        *data1 = 10;

        allocator.free_to_marker(marker);

        let data2 = allocator.allocate(Layout::new::<u128>()).cast::<u128>();
        *data2 = 200;

        // both pointers should now point to the same data;
        assert_eq!(*data2, 200);
        assert_eq!(*data1, *data2);
        dealloc(stack_memory, stack_layout);
    }
}

#[test]
fn allocate_with_padding() {
    unsafe {
        let stack_layout = Layout::new::<[u8; 128]>();
        let stack_memory = alloc(stack_layout);

        let mut allocator = StackAllocator::new(stack_memory.cast(), stack_layout.size());

        let layout1 = Layout::new::<u8>();
        let layout2 = Layout::from_size_align(64, 16).unwrap(); // Alignment erfordert Padding

        let data1 = allocator.allocate(layout1).cast::<u8>();
        *data1 = 42;

        let data2 = allocator.allocate(layout2).cast::<[u8; 64]>();
        *data2 = [1; 64];

        assert_eq!(*data1, 42);
        assert_eq!(*data2, [1; 64]);

        dealloc(stack_memory, stack_layout);
    }
}

#[test]
fn out_of_memory() {
    unsafe {
        let stack_layout = Layout::new::<[u8; 32]>();
        let stack_memory = alloc(stack_layout);

        let mut allocator = StackAllocator::new(stack_memory.cast(), stack_layout.size());

        let layout = Layout::new::<[u8; 16]>();

        allocator.allocate(layout);
        allocator.allocate(layout);

        // Third allocation should fail
        let result = allocator.allocate(layout);
        assert!(result.is_null());

        dealloc(stack_memory, stack_layout);
    }
}

#[test]
fn multiple_free_to_markers() {
    unsafe {
        let stack_layout = Layout::new::<[u8; 64]>();
        let stack_memory = alloc(stack_layout);

        let mut allocator = StackAllocator::new(stack_memory.cast(), stack_layout.size());

        let marker1 = allocator.get_marker();
        let data1 = allocator.allocate(Layout::new::<u32>()).cast::<u32>();
        *data1 = 123;

        let marker2 = allocator.get_marker();
        let data2 = allocator.allocate(Layout::new::<u32>()).cast::<u32>();
        *data2 = 456;

        allocator.free_to_marker(marker2);

        let data3 = allocator.allocate(Layout::new::<u32>()).cast::<u32>();
        *data3 = 789;

        allocator.free_to_marker(marker1);

        let data4 = allocator.allocate(Layout::new::<u8>()).cast::<u8>();
        *data4 = 255;

        assert_eq!(*data3, 789);
        assert_eq!(*data4, 255);

        dealloc(stack_memory, stack_layout);
    }
}

#[test]
fn alignment_and_fragmentation() {
    unsafe {
        let stack_layout = Layout::new::<[u8; 128]>();
        let stack_memory = alloc(stack_layout);

        let mut allocator = StackAllocator::new(stack_memory.cast(), stack_layout.size());

        let layout1 = Layout::from_size_align(16, 8).unwrap();
        let layout2 = Layout::from_size_align(32, 16).unwrap();

        let data1 = allocator.allocate(layout1).cast::<[u8; 16]>();
        *data1 = [0xAA; 16];

        let data2 = allocator.allocate(layout2).cast::<[u8; 32]>();
        *data2 = [0xBB; 32];

        let maker = allocator.get_marker();
        allocator.free_to_marker(maker);

        let data3 = allocator.allocate(layout1).cast::<[u8; 16]>();
        *data3 = [0xCC; 16];

        assert_eq!(*data3, [0xCC; 16]);
        assert_ne!(data1, data3);

        dealloc(stack_memory, stack_layout);
    }
}

#[test]
fn double_free() {
    unsafe {
        let stack_layout = Layout::new::<[u8; 64]>();
        let stack_memory = alloc(stack_layout);

        let mut allocator = StackAllocator::new(stack_memory.cast(), stack_layout.size());

        let data1 = allocator.allocate(Layout::new::<u32>()).cast::<u32>();
        *data1 = 42;

        let marker = allocator.get_marker();

        allocator.free_to_marker(marker);
        allocator.free_to_marker(marker); // Should not cause undefined behavior

        let data2 = allocator.allocate(Layout::new::<u32>()).cast::<u32>();
        *data2 = 84;

        assert_eq!(*data2, 84);

        dealloc(stack_memory, stack_layout);
    }
}

#[test]
fn no_overwrite_beyond_stack_limit() {
    unsafe {
        let stack_layout = Layout::new::<[u8; 64]>();
        let stack_memory = alloc(stack_layout);

        let mut allocator = StackAllocator::new(stack_memory.cast(), stack_layout.size());

        let layout1 = Layout::new::<[u8; 32]>();
        let layout2 = Layout::new::<[u8; 40]>(); // Exceeds remaining space

        allocator.allocate(layout1);
        let result = allocator.allocate(layout2);

        assert!(result.is_null()); // Allocation should fail

        dealloc(stack_memory, stack_layout);
    }
}
