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
    /// the offset to the next node (in bytes),
    next: u32,
    /// the size of this node (in bytes)
    size: u32,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_ptr_alignment)]
impl FreeList {
    /// # Safety
    /// ``memory`` and ``mem_size`` need to be valid
    /// # Panics
    /// if the size is bigger than ``u32::MAX``
    pub unsafe fn new(memory: *mut i8, mem_size: usize) -> Self {
        assert!(mem_size < u32::MAX as usize);

        *memory.cast::<Node>() = Node {
            next: INVALID,
            size: mem_size as u32,
        };

        Self { head: 0, memory }
    }

    /// # Safety
    pub unsafe fn allocate(&mut self, layout: Layout) -> Option<FreeListPtr<c_void>> {
        let alloc_align = layout.align() as u32;

        let mut node_ptr = self.head;
        let mut p_previous: *mut Node = null_mut();

        while node_ptr != INVALID {
            // we need to add some padding based on the required alignment
            // TODO: if padding > Node Size then add a new free node
            let padding = (alloc_align - (node_ptr % alloc_align)) % alloc_align;
            let alloc_size = layout.size() as u32 + padding;

            let node = &mut *self.memory.add(node_ptr as usize).cast::<Node>();

            let node_size = node.size;
            let mut return_full_node = || {
                let node_to_return;

                if p_previous.is_null() {
                    node_to_return = self.head;
                    self.head = node.next;
                } else {
                    (*p_previous).next = node.next;
                    node_to_return = node_ptr;
                }

                FreeListPtr {
                    ptr: self.memory.add((node_to_return + padding) as usize).cast(),
                    pad: padding,
                    size: node_size,
                }
            };

            match node.size.cmp(&alloc_size) {
                // if the size is equal, remove the node
                std::cmp::Ordering::Equal => {
                    return Some(return_full_node());
                }
                // if the size is greater, move the node back by the size
                std::cmp::Ordering::Greater => {
                    node.size -= alloc_size;
                    // if the left over space is to small to contain another node,
                    // then just use the full node
                    if node.size < size_of::<Node>() as u32 {
                        return Some(return_full_node());
                    }

                    let new_ptr = node_ptr + alloc_size;
                    *self.memory.add(new_ptr as usize).cast() = *node;

                    if p_previous.is_null() {
                        self.head = new_ptr;
                    } else {
                        (*p_previous).next = new_ptr;
                    }
                    return Some(FreeListPtr {
                        ptr: self.memory.add((node_ptr + padding) as usize).cast(),
                        size: alloc_size,
                        pad: padding,
                    });
                }
                std::cmp::Ordering::Less => {}
            }

            p_previous = node;
            node_ptr = node.next;
        }

        None
    }

    /// # Safety
    /// invalidates all pointers
    #[allow(clippy::needless_pass_by_value)]
    pub unsafe fn dealloc<T>(&mut self, ptr: FreeListPtr<T>) {
        let FreeListPtr { ptr, pad, size } = ptr;
        let search_node_ptr = (ptr as usize - self.memory as usize) as u32;

        let mut node_ptr = self.head;
        let mut p_previous: *mut Node = null_mut();

        // get the closest nodes to the deallocated one, to check if they touch each other
        while node_ptr != INVALID && node_ptr < search_node_ptr {
            let node = &mut *self.memory.add(node_ptr as usize).cast::<Node>();
            p_previous = node;
            node_ptr = node.next;
        }

        let p_next = if node_ptr == INVALID {
            null_mut()
        } else {
            self.memory.add(node_ptr as usize).cast::<Node>()
        };

        let new_node = ptr.offset(-(pad as isize)).cast::<Node>();
        *new_node = Node {
            next: INVALID,
            size,
        };

        // there is no free node before this one
        // so we need to replace the head with this

        if !p_next.is_null() {
            // check if the two nodes touch
            if ptr.add(size as usize).cast() == p_next {
                (*new_node).size += (*p_next).size;
                (*new_node).next = (*p_next).next;
                dbg!("tocuh 1");
            }
        }

        if !p_previous.is_null() {
            // check if the two nodes touch
            if p_previous.add((*p_previous).size as usize) == new_node {
                (*p_previous).size += (*new_node).size;
                (*p_previous).next = search_node_ptr - pad;
                dbg!("tocuh 2");
            }
        }

        // there is no free node after the deallocated one
        // if node_ptr == INVALID {}

        dbg!(node_ptr - pad);
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
