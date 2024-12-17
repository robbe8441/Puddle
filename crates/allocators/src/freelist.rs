use std::{
    alloc::Layout,
    ffi::c_void,
    fmt::Debug,
    ops::{Deref, DerefMut},
    ptr::null_mut,
};

const INVALID: u32 = u32::MAX;

#[derive(Clone, Copy)]
pub struct FreeListPtr<T> {
    ptr: *mut T,
    pad: u32,
    size: u32,
}

impl<T> FreeListPtr<T> {
    pub fn cast<B>(&self) -> FreeListPtr<B> {
        FreeListPtr {
            ptr: self.ptr.cast(),
            pad: self.pad,
            size: self.size,
        }
    }
}
impl<T> DerefMut for FreeListPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Deref for FreeListPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

pub struct FreeList {
    head: u32,
    memory: *mut i8,
}

#[derive(Debug, Clone, Copy)]
struct Node {
    /// the offset to the next node (in bytes)
    next: u32,
    /// the size of this node (in bytes)
    size: u32,
}

impl Node {
    unsafe fn touches(node: *const Node, rhs: *const Node) -> bool {
        let node_size = (*node).size as usize;
        let rhs_size = (*node).size as usize;
        node.add(node_size) == rhs || rhs.add(rhs_size) == node
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_ptr_alignment)]
impl FreeList {
    /// # Safety
    /// ``memory`` and ``mem_size`` need to be valid
    /// # Panics
    /// if the size is bigger than ``u32::MAX``
    pub unsafe fn new(memory: *mut i8, mem_size: usize) -> Self {
        assert!(mem_size < u32::MAX as usize);
        assert!(memory.is_aligned_to(align_of::<Node>()));

        *memory.cast::<Node>() = Node {
            next: INVALID,
            size: mem_size as u32,
        };

        Self { head: 0, memory }
    }

    /// # Safety
    /// # Panics
    pub unsafe fn allocate(&mut self, layout: Layout) -> Option<FreeListPtr<c_void>> {
        assert_eq!(layout.size() % size_of::<Node>(), 0);

        let mut node_index = self.head;
        let mut previous: *mut Node = null_mut();

        while node_index != INVALID {
            let node_addr = self.memory.add(node_index as usize).cast::<Node>();
            let padding = (layout.align() - (node_addr as usize % layout.align())) % layout.align();

            let alloc_size = (layout.size() + padding) as u32;

            let mut return_full_node = |size| {
                let node_to_return;

                if previous.is_null() {
                    node_to_return = self.head as usize;
                    self.head = (*node_addr).next;
                } else {
                    (*previous).next = (*node_addr).next;
                    node_to_return = node_index as usize;
                }

                return Some(FreeListPtr {
                    ptr: self.memory.add(node_to_return + padding).cast(),
                    pad: padding as u32,
                    size,
                });
            };

            match (*node_addr).size.cmp(&alloc_size) {
                std::cmp::Ordering::Equal => {
                    return return_full_node(alloc_size);
                }
                std::cmp::Ordering::Greater => {
                    let left_over_size = (*node_addr).size - alloc_size;
                    if left_over_size < size_of::<Node>() as u32 {
                        return return_full_node(alloc_size + left_over_size);
                    }

                    (*node_addr).size -= alloc_size;

                    let new_ptr = node_index + alloc_size;
                    *self.memory.add(new_ptr as usize).cast() = *node_addr;

                    if previous.is_null() {
                        self.head = new_ptr;
                    } else {
                        (*previous).next = new_ptr;
                    }

                    return Some(FreeListPtr {
                        ptr: self.memory.add(node_index as usize + padding).cast(),
                        size: alloc_size,
                        pad: padding as u32,
                    });
                }
                std::cmp::Ordering::Less => {}
            }

            node_index = (*node_addr).next;
            previous = node_addr;
        }

        None
    }

    unsafe fn allocate_full_node() {}

    /// # Safety
    /// invalidates all pointers
    #[allow(clippy::needless_pass_by_value, clippy::cast_possible_wrap)]
    pub unsafe fn dealloc<T>(&mut self, mem: FreeListPtr<T>) {
        // get the actual pointer without padding
        let real_ptr = mem.ptr.offset(-(mem.pad as isize)).cast();
        let mem_size = (mem.size + mem.pad) as usize;

        self.dealloc_intern(real_ptr, mem_size);
    }

    unsafe fn dealloc_intern(&mut self, ptr: *mut Node, size: usize) {
        let node_index = self.head;
    }
}

#[allow(clippy::cast_ptr_alignment)]
impl Debug for FreeList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();

        let mut node_ptr = self.head;

        while node_ptr != INVALID {
            let node = unsafe { &*self.memory.add(node_ptr as usize).cast::<Node>() };
            list.entry(&node.size);
            node_ptr = node.next;
        }

        list.finish()
    }
}
