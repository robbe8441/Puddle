use std::alloc::{alloc, dealloc, Layout};

use allocators::PoolAllocator;

#[test]
fn pool_alloc() {
    let pool_layout = Layout::new::<[usize; 2]>();
    let pool_memory = unsafe { alloc(pool_layout) };

    let mut pool = unsafe { PoolAllocator::new(pool_memory.cast(), 2) };

    let mem1 = pool.allocate();
    unsafe { *mem1 = 10usize };

    let mem2 = pool.allocate();
    unsafe { *mem2 = 20usize };

    pool.free(mem2);

    let mem3 = pool.allocate();
    unsafe { *mem3 = 30usize };

    pool.free(mem3);

    let mem4 = pool.allocate();
    unsafe { *mem4 = 40usize };

    unsafe { assert_eq!(*mem1, 10) };

    unsafe { dealloc(pool_memory, pool_layout) };
}
