use super::resource::Pipeline;
use std::{alloc::Layout, cell::UnsafeCell, marker::PhantomData};

pub struct TaskGraph {
    memory: *mut u8,
    mem_size: usize,
    allocator: UnsafeCell<allocators::FreeListAllocator>,
}

pub struct RenderTaskWriter<O> {
    _marker: PhantomData<O>,
}

const TASK_MEM_LAYOUT: Layout =
    unsafe { Layout::from_size_align_unchecked(u32::MAX as usize, align_of::<usize>()) };

impl TaskGraph {
    #[must_use]
    pub fn new() -> Self {
        let mem_size = TASK_MEM_LAYOUT.size();

        let memory = unsafe { std::alloc::alloc(TASK_MEM_LAYOUT) };
        let allocator = unsafe { allocators::FreeListAllocator::new(memory.cast(), mem_size) };

        Self {
            memory,
            mem_size,
            allocator: UnsafeCell::new(allocator),
        }
    }

    pub fn set_render_pipeline<M, O>(
        &self,
        pipeline: &Pipeline<M, O>,
        material: &M,
    ) -> RenderTaskWriter<O> {
        unimplemented!()
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TaskGraph {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.memory, TASK_MEM_LAYOUT);
        }
    }
}

impl<O> RenderTaskWriter<O> {
    pub fn draw(&self, obj: &O) {
        unimplemented!()
    }
}
