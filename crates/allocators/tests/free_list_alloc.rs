use std::alloc::{alloc, dealloc, Layout};

use allocators::FreeList;

// TODO: fix ugly tests

#[test]
fn list_allocate() {
    unsafe {
        const ITEMS: usize = 100 * size_of::<usize>();

        let mem_layout = Layout::from_size_align_unchecked(ITEMS, 8);
        let memory = alloc(mem_layout);

        let mut allocator = FreeList::new(memory.cast(), ITEMS);

        let item_layout = Layout::new::<usize>();
        let item_layout2 = Layout::new::<[u8; 4]>();

        let mem1 = allocator.allocate(item_layout).unwrap();
        *mem1.cast() = usize::MAX;

        // this has 2x the size
        let mem2 = allocator.allocate(item_layout2).unwrap();
        *mem2.cast() = [u8::MAX - 1; 4];

        let mem3 = allocator.allocate(item_layout).unwrap();
        *mem3.cast() = usize::MAX - 2;

        assert_eq!(*mem1.cast::<usize>(), usize::MAX);
        assert_eq!(*mem2.cast::<[u8; 4]>(), [u8::MAX - 1; 4]);
        assert_eq!(*mem3.cast::<usize>(), usize::MAX - 2);

        dealloc(memory, mem_layout);
    }
}

#[test]
fn out_of_space() {
    unsafe {
        const ITEMS: usize = 2 * size_of::<usize>();

        let mem_layout = Layout::from_size_align_unchecked(ITEMS, 8);
        let memory = alloc(mem_layout);

        let mut allocator = FreeList::new(memory.cast(), ITEMS);

        // allocate the full size
        allocator.allocate(mem_layout).unwrap();

        // this shouldn't as the full size has been allocated
        let res = allocator.allocate(mem_layout);
        assert!(res.is_none());

        dealloc(memory, mem_layout);
    }
}

#[test]
fn dealloc_test() {
    unsafe {
        const ITEMS: usize = 100 * size_of::<usize>();

        let mem_layout = Layout::from_size_align_unchecked(ITEMS, 8);
        let memory = alloc(mem_layout);

        let mut allocator = FreeList::new(memory.cast(), ITEMS);

        let item_layout = Layout::new::<usize>();
        let item_layout2 = Layout::new::<[u8; 4]>();

        let mem1 = allocator.allocate(item_layout).unwrap();
        *mem1.cast() = usize::MAX;

        // this has 2x the size
        let mem2 = allocator.allocate(item_layout2).unwrap();
        *mem2.cast() = [u8::MAX - 1; 4];

        allocator.dealloc(mem1);
        allocator.dealloc(mem2);

        let mem3 = allocator.allocate(item_layout).unwrap();
        *mem3.cast() = usize::MAX - 2;

        allocator.dealloc(mem3);

        dbg!(allocator);

        dealloc(memory, mem_layout);
    }
}
