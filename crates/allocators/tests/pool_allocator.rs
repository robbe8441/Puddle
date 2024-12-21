use std::{alloc::{alloc, dealloc, Layout}, ptr::null_mut};

use allocators::TypedPoolAllocator as PoolAllocator;

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

#[test]
fn allocate_full_pool() {
    let pool_layout = Layout::new::<[usize; 3]>();
    let pool_memory = unsafe { alloc(pool_layout) };

    let mut pool = unsafe { PoolAllocator::new(pool_memory.cast(), 3) };

    let mem1 = pool.allocate();
    unsafe { *mem1 = 10usize };

    let mem2 = pool.allocate();
    unsafe { *mem2 = 20usize };

    let mem3 = pool.allocate();
    unsafe { *mem3 = 30usize };

    // Pool is now full, next allocation should fail
    let mem4 = pool.allocate();
    assert!(mem4.is_null());

    unsafe { dealloc(pool_memory, pool_layout) };
}

#[test]
fn free_and_reuse() {
    let pool_layout = Layout::new::<[usize; 4]>();
    let pool_memory = unsafe { alloc(pool_layout) };

    let mut pool = unsafe { PoolAllocator::new(pool_memory.cast(), 4) };

    let mem1 = pool.allocate();
    unsafe { *mem1 = 10usize };

    let mem2 = pool.allocate();
    unsafe { *mem2 = 20usize };

    pool.free(mem1);

    // Allocate again; should reuse the freed slot
    let mem3 = pool.allocate();
    unsafe { *mem3 = 30usize };

    assert_eq!(unsafe { *mem3 }, 30);
    assert_ne!(mem1, null_mut());

    unsafe { dealloc(pool_memory, pool_layout) };
}

#[test]
fn allocate_and_free_multiple() {
    let pool_layout = Layout::new::<[usize; 5]>();
    let pool_memory = unsafe { alloc(pool_layout) };

    let mut pool = unsafe { PoolAllocator::<usize>::new(pool_memory.cast(), 5) };

    let mem1 = pool.allocate();
    let mem2 = pool.allocate();
    let mem3 = pool.allocate();

    unsafe {
        *mem1 = 1;
        *mem2 = 2;
        *mem3 = 3;
    }

    pool.free(mem2);
    pool.free(mem1);

    let mem4 = pool.allocate();
    let mem5 = pool.allocate();

    unsafe {
        *mem4 = 4;
        *mem5 = 5;
    }

    assert_eq!(unsafe { *mem3 }, 3);
    assert_eq!(unsafe { *mem4 }, 4);
    assert_eq!(unsafe { *mem5 }, 5);

    unsafe { dealloc(pool_memory, pool_layout) };
}

#[test]
fn free_twice_error() {
    let pool_layout = Layout::new::<[usize; 2]>();
    let pool_memory = unsafe { alloc(pool_layout) };

    let mut pool = unsafe { PoolAllocator::<usize>::new(pool_memory.cast(), 2) };

    let mem1 = pool.allocate();

    pool.free(mem1);

    // Freeing the same memory again should not cause undefined behavior
    // Implementation should handle it gracefully (e.g., ignore it or panic)
    pool.free(mem1);

    unsafe { dealloc(pool_memory, pool_layout) };
}

#[test]
fn out_of_memory() {
    let pool_layout = Layout::new::<[usize; 1]>();
    let pool_memory = unsafe { alloc(pool_layout) };

    let mut pool = unsafe { PoolAllocator::new(pool_memory.cast(), 1) };

    let mem1 = pool.allocate();
    unsafe { *mem1 = 42usize };

    let mem2 = pool.allocate();
    assert!(mem2.is_null()); // Should fail as only one slot is available

    unsafe { dealloc(pool_memory, pool_layout) };
}

#[test]
fn pool_alignment_test() {
    let pool_layout = Layout::from_size_align(128, 16).unwrap();
    let pool_memory = unsafe { alloc(pool_layout) };

    let mut pool = unsafe { PoolAllocator::<usize>::new(pool_memory.cast(), 8) };

    let mem1 = pool.allocate();
    assert_eq!(mem1 as usize % 16, 0); // Ensure alignment is correct

    pool.free(mem1);

    unsafe { dealloc(pool_memory, pool_layout) };
}
