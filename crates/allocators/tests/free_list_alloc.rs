use std::alloc::{alloc, dealloc, Layout};

use allocators::{FreeListAllocator, FreeListPtr};

// TODO: fix ugly tests

#[test]
fn list_allocate() {
    unsafe {
        const ITEMS: usize = 100 * size_of::<usize>();

        let mem_layout = Layout::from_size_align_unchecked(ITEMS, 8);
        let memory = alloc(mem_layout);

        let mut allocator = FreeListAllocator::new(memory.cast(), ITEMS);

        let item_layout = Layout::new::<usize>();
        let item_layout2 = Layout::new::<[u8; 8]>();

        let mem1 = allocator.allocate(item_layout).unwrap();
        *mem1.cast() = usize::MAX;

        // this has 2x the size
        let mem2 = allocator.allocate(item_layout2).unwrap();
        *mem2.cast() = [u8::MAX - 1; 8];

        let mem3 = allocator.allocate(item_layout).unwrap();
        *mem3.cast() = usize::MAX - 2;

        assert_eq!(*mem1.cast::<usize>(), usize::MAX);
        assert_eq!(*mem2.cast::<[u8; 8]>(), [u8::MAX - 1; 8]);
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

        let mut allocator = FreeListAllocator::new(memory.cast(), ITEMS);

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
        const ITEMS: usize = 2 * size_of::<usize>();

        let mem_layout = Layout::from_size_align_unchecked(ITEMS, 8);
        let memory = alloc(mem_layout);

        let mut allocator = FreeListAllocator::new(memory.cast(), ITEMS);

        let item_layout = Layout::new::<usize>();
        let item_layout2 = Layout::new::<[u8; 8]>();

        let mem1 = allocator.allocate(item_layout).unwrap();
        *mem1.cast() = usize::MAX;

        // this has 2x the size
        let mem2 = allocator.allocate(item_layout2).unwrap();
        *mem2.cast() = [u8::MAX - 1; 8];

        // the allocator is now full and needs to free memory before allocating new one
        allocator.dealloc(mem1);
        allocator.dealloc(mem2);

        let mem3 = allocator.allocate(item_layout).unwrap();
        *mem3.cast() = usize::MAX - 2;

        allocator.dealloc(mem3);

        dealloc(memory, mem_layout);
    }
}

#[test]
fn padding_test() {
    unsafe {
        const ITEMS: usize = 40;

        let mem_layout = Layout::from_size_align_unchecked(ITEMS, 8);
        let memory = alloc(mem_layout);

        let mut allocator = FreeListAllocator::new(memory.cast(), ITEMS);

        let item_layout = Layout::new::<u64>();
        let item_layout2 = Layout::new::<u128>();

        // this has 2x the size
        let mem1 = allocator.allocate(item_layout2).unwrap(); // 16 bytes
        *mem1.cast() = u128::MAX - 1;

        let mem2 = allocator.allocate(item_layout).unwrap(); // 8 bytes
        *mem2.cast() = u64::MAX - 1;

        let mem3 = allocator.allocate(item_layout2); // 16 bytes (needs padding)

        // 16 + 8 + 16 = 40
        // but because of padding this shouldn't work
        assert!(mem3.is_none());

        dealloc(memory, mem_layout);
    }
}

#[test]
fn dealloc_matching_nodes() {
    unsafe {
        const ITEMS: usize = 8 * size_of::<usize>();

        let mem_layout = Layout::from_size_align_unchecked(ITEMS, 8);
        let memory = alloc(mem_layout);

        let mut allocator = FreeListAllocator::new(memory.cast(), ITEMS);

        let item_layout2 = Layout::new::<[u8; 64]>();
        let item_layout = Layout::new::<usize>();

        // allocate 8 small memory blocks
        let mem_list: [FreeListPtr<usize>; 8] = std::array::from_fn(|_| {
            let mut mem = allocator.allocate(item_layout).unwrap().cast::<usize>();
            *mem = usize::MAX;
            mem
        });

        for mem in mem_list {
            allocator.dealloc(mem);
        }

        // then allocate a big one to test if they are all merged in to one big block
        let big_data = allocator.allocate(item_layout2).unwrap();
        *big_data.cast() = [u8::MAX - 1; 64];

        // the allocator is now full and needs to free memory before allocating new one
        allocator.dealloc(big_data);

        let mem3 = allocator.allocate(item_layout).unwrap();
        *mem3.cast() = usize::MAX - 2;

        allocator.dealloc(mem3);

        dealloc(memory, mem_layout);
    }
}

#[test]
fn allocate_exact_fit() {
    unsafe {
        const ITEMS: usize = 3 * size_of::<usize>();

        let mem_layout = Layout::from_size_align_unchecked(ITEMS, 8);
        let memory = alloc(mem_layout);

        let mut allocator = FreeListAllocator::new(memory.cast(), ITEMS);

        let item_layout = Layout::new::<usize>();

        // Allocate three blocks that exactly fit the memory
        let mem1 = allocator.allocate(item_layout).unwrap();
        let mem2 = allocator.allocate(item_layout).unwrap();
        let mem3 = allocator.allocate(item_layout).unwrap();

        *mem1.cast() = 1usize;
        *mem2.cast() = 2usize;
        *mem3.cast() = 3usize;

        assert_eq!(*mem1.cast::<usize>(), 1);
        assert_eq!(*mem2.cast::<usize>(), 2);
        assert_eq!(*mem3.cast::<usize>(), 3);

        // No more space available
        assert!(allocator.allocate(item_layout).is_none());

        allocator.dealloc(mem1);
        allocator.dealloc(mem2);
        allocator.dealloc(mem3);

        dealloc(memory, mem_layout);
    }
}

#[test]
fn fragmentation_test() {
    unsafe {
        const ITEMS: usize = 10 * size_of::<usize>();

        let mem_layout = Layout::from_size_align_unchecked(ITEMS, 8);
        let memory = alloc(mem_layout);

        let mut allocator = FreeListAllocator::new(memory.cast(), ITEMS);

        let item_layout = Layout::new::<usize>();

        let mem1 = allocator.allocate(item_layout).unwrap();
        let mem2 = allocator.allocate(item_layout).unwrap();
        let mem3 = allocator.allocate(item_layout).unwrap();

        *mem1.cast() = 10usize;
        *mem2.cast() = 20usize;
        *mem3.cast() = 30usize;

        allocator.dealloc(mem2);

        // Allocate a smaller block in the freed space
        let mem4 = allocator
            .allocate(Layout::new::<u64>())
            .unwrap();

        *mem4.cast() = 40u64;

        assert_eq!(*mem1.cast::<usize>(), 10);
        assert_eq!(*mem4.cast::<u32>(), 40);
        assert_eq!(*mem3.cast::<usize>(), 30);

        allocator.dealloc(mem1);
        allocator.dealloc(mem3);
        allocator.dealloc(mem4);

        dealloc(memory, mem_layout);
    }
}

#[test]
fn alignment_test() {
    unsafe {
        const ITEMS: usize = 32;

        let mem_layout = Layout::from_size_align_unchecked(ITEMS, 8);
        let memory = alloc(mem_layout);

        let mut allocator = FreeListAllocator::new(memory.cast(), ITEMS);

        let item_layout = Layout::from_size_align(16, 16).unwrap();

        // Allocate memory with stricter alignment
        let mem1 = allocator.allocate(item_layout).unwrap();
        assert_eq!(mem1.as_ptr() as usize % 16, 0);

        allocator.dealloc(mem1);

        dealloc(memory, mem_layout);
    }
}
