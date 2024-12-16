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

// #[test]
// fn resize() {
//     unsafe {
//         let mut allocator = StackAllocator::new(10);
//
//         let data1 = allocator.allocate(Layout::new::<u128>()).cast::<u128>();
//         *data1 = 10;
//
//         let data2 = allocator.allocate(Layout::new::<u64>()).cast::<u64>();
//         *data2 = 100;
//     }
// }

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
