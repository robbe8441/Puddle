#![allow(
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::module_name_repetitions
)]

use std::{collections::VecDeque, fmt::Debug, sync::Arc};

use math::{dvec3, DVec3};

/// |  64 bit   |    8 bit      |    24 bit   |
///    colors      valid mask      child ptr
#[repr(C)]
#[derive(Default, Clone, PartialEq, Eq)]
pub struct FlatOctreeNode {
    colors: ColorData,
    child_descriptor: u32,
    _padding: u32,
}

#[allow(clippy::missing_fields_in_debug)]
impl Debug for FlatOctreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mask = self.get_valid_mask();
        let valid_mask: String = (0..8).map(|i| ((mask >> i) & 1).to_string()).collect();

        f.debug_struct("FlatOctreeNode")
            .field("valid_mask", &valid_mask)
            .field("child_ptr", &self.get_child_ptr())
            .field_with("colors", |f| {
                f.debug_list()
                    .entries((0..8).map(|i| self.colors.get_color(i)))
                    .finish()
            })
            .finish()
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct FlatOctree {
    data: Arc<[FlatOctreeNode]>,
}

#[allow(unused)]
impl FlatOctreeNode {
    /// valid mask tells how many and what child nodes there are
    /// each bit is one child node, this is only needed for flat octree's
    pub fn set_valid_mask(&mut self, mask: u8) {
        self.child_descriptor &= !(0xFF << 24);
        self.child_descriptor |= (mask as u32) << 24;
    }
    #[must_use]
    pub fn get_valid_mask(&self) -> u8 {
        (self.child_descriptor >> 24) as u8
    }

    /// sets the index of where the children are located in the buffer
    pub fn set_child_ptr(&mut self, val: u32) {
        self.child_descriptor &= !0xFFF_FFF;
        self.child_descriptor |= val & 0xFFF_FFF;
    }
    #[must_use]
    pub fn get_child_ptr(&self) -> u32 {
        self.child_descriptor & 0xFFF_FFF
    }
}

/// 64 bit of color data
/// every voxel has 8 bits for colors => 255 colors for every octree
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ColorData(u64);

impl ColorData {
    /// set the color at a given index
    /// the index shouldn't exceed 0-7 (3 bits)
    pub fn set_color(&mut self, index: u8, color: u8) {
        self.0 &= !(0xFF << (index * 8));
        self.0 |= (color as u64) << (index * 8);
    }

    pub fn set_all_colors(&mut self, color: u8) {
        self.0 = 0;
        for i in 0..8 {
            self.0 |= (color as u64) << (i * 8);
        }
    }

    #[must_use]
    pub fn get_color(&self, index: u8) -> u8 {
        (self.0 >> (index * 8)) as u8
    }

    #[must_use]
    pub fn are_equal(&self) -> bool {
        let color = self.get_color(0);
        (1..8).all(|i| self.get_color(i) == color)
    }
}

#[derive(Default)]
pub struct OctreeNode {
    colors: ColorData,
    children: [Option<Box<OctreeNode>>; 8],
}

impl OctreeNode {
    pub const NODE_POS: [DVec3; 8] = [
        dvec3(-1.0, -1.0, -1.0),
        dvec3(1.0, -1.0, -1.0),
        dvec3(-1.0, 1.0, -1.0),
        dvec3(1.0, 1.0, -1.0),
        dvec3(-1.0, -1.0, 1.0),
        dvec3(1.0, -1.0, 1.0),
        dvec3(-1.0, 1.0, 1.0),
        dvec3(1.0, 1.0, 1.0),
    ];

    /// get the valid mask of the node
    /// the valid mask is a u8 each bit tells if there is a child node at the given index
    #[must_use]
    pub fn get_valid_mask(&self) -> u8 {
        let mut valid_mask = 0u8;
        for i in 0..8 {
            valid_mask |= (self.children[i].is_some() as u8) << i;
        }
        valid_mask
    }
    /// write once to the octree
    /// position must contain values between -1 and 1
    /// ``total_layers`` is how deep it should go in to the tree
    pub fn write(&mut self, pos: DVec3, color: u8, layer: usize) {
        self.write_intern(pos, DVec3::ZERO, 1.0, color, layer);
    }

    fn write_intern(
        &mut self,
        pos: DVec3,
        mut center: DVec3,
        mut scale: f64,
        color: u8,
        layers: usize,
    ) -> bool {
        let index: u8 = (pos.x > center.x) as u8
            | ((pos.y > center.y) as u8) << 1
            | ((pos.z > center.z) as u8) << 2;

        scale *= 0.5;
        center += scale * Self::NODE_POS[index as usize];

        if layers > 1 {
            let child_node = self.children[index as usize].get_or_insert_with(|| {
                let mut colors = ColorData::default();
                colors.set_all_colors(self.colors.get_color(index));

                Box::new(OctreeNode {
                    colors,
                    ..Default::default()
                })
            });

            let res = child_node.write_intern(pos, center, scale, color, layers - 1);

            if res {
                self.children[index as usize] = None;
            }
        } else {
            self.children[index as usize] = None;
        }

        self.colors.set_color(index, color);
        self.colors.are_equal() && self.get_valid_mask() == 0
    }

    /// lookup one value in the octree
    /// position must contain values between -1 and 1
    /// ``total_layers`` is how deep it should go in to the tree, doesn't need to be the same when
    /// writing to the tree, this can be used for LOD's
    #[must_use]
    pub fn sample(&self, pos: DVec3, max_layers: usize) -> u8 {
        let mut node: &OctreeNode = self;
        let mut center = DVec3::splat(0.0);
        let mut scale = 1.0;

        for _ in 1..max_layers {
            let index: u8 = (pos.x > center.x) as u8
                | ((pos.y > center.y) as u8) << 1
                | ((pos.z > center.z) as u8) << 2;

            scale *= 0.5;
            let next_node = &node.children[index as usize];
            if let Some(next_node) = next_node {
                center += scale * Self::NODE_POS[index as usize];
                node = next_node.as_ref();
            } else {
                break;
            }
        }

        let index: u8 = (pos.x > center.x) as u8
            | ((pos.y > center.y) as u8) << 1
            | ((pos.z > center.z) as u8) << 2;

        node.colors.get_color(index)
    }

    /// flatten the octree
    /// compress the octree in to a linear format
    /// this is used to store it in a file or a buffer for the GPU
    #[must_use]
    pub fn flatten(&self) -> FlatOctree {
        let mut stack: VecDeque<&Box<OctreeNode>> = VecDeque::new();
        let mut flat_tree = vec![];

        let mut flat_root = FlatOctreeNode {
            colors: self.colors,
            ..Default::default()
        };
        flat_root.set_valid_mask(self.get_valid_mask());
        flat_root.set_child_ptr(1); // root always has child_ptr = 1
        flat_tree.push(flat_root);

        stack.extend(self.children.iter().filter_map(|v| v.as_ref()));

        while let Some(node) = stack.pop_front() {
            let mut flat_node = FlatOctreeNode {
                colors: node.colors,
                ..Default::default()
            };
            flat_node.set_valid_mask(node.get_valid_mask());

            let child_ptr = stack.len() + flat_tree.len() + 1;
            flat_node.set_child_ptr(child_ptr as u32);

            flat_tree.push(flat_node);

            stack.extend(node.children.iter().filter_map(|v| v.as_ref()));
        }

        FlatOctree {
            data: flat_tree.into(),
        }
    }
}

impl FlatOctree {
    /// convert a flat octree back to a normal octree
    /// for example after loading it from a file
    #[must_use]
    pub fn unflatten(&self) -> OctreeNode {
        struct StackNode {
            ptr: *mut OctreeNode,
            index: usize, // the index of this node in the flat array
        }

        let mut root = OctreeNode {
            colors: self.data[0].colors,
            ..Default::default()
        };

        let mut stack = vec![StackNode {
            ptr: &mut root,
            index: 0,
        }];

        while let Some(stack_node) = stack.pop() {
            let flat_node = &self.data[stack_node.index];
            let valid_mask = flat_node.get_valid_mask();

            for (i, j) in (0..8).filter(|i| valid_mask & (1 << i) != 0).enumerate() {
                let child_index = flat_node.get_child_ptr() as usize + i;
                let child = &self.data[child_index];

                let node = OctreeNode {
                    colors: child.colors,
                    ..Default::default()
                };

                let boxed_node = Box::new(node);
                unsafe { (*stack_node.ptr).children[j] = Some(boxed_node) };

                let mem_ptr = unsafe {
                    Box::as_mut_ptr((*stack_node.ptr).children[j].as_mut().unwrap_unchecked())
                };

                stack.push(StackNode {
                    index: child_index,
                    ptr: mem_ptr,
                });
            }
        }

        root
    }

    /// convert a flat octree to its raw unsafe format
    /// if this is edited, it can cause invalid data, so be careful
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        let ptr = self.data.as_ptr().cast();
        let len = self.data.len() * std::mem::size_of::<FlatOctreeNode>();
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }

    /// convert the raw data back to an flat octree
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let node_count = bytes.len() / std::mem::size_of::<FlatOctreeNode>();
        let ptr = bytes.as_ptr().cast();
        Self {
            data: unsafe { std::slice::from_raw_parts(ptr, node_count) }.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FlatOctree, FlatOctreeNode, OctreeNode};
    use math::dvec3;

    #[test]
    fn valid_mask() {
        let mut node = FlatOctreeNode::default();
        node.set_valid_mask(255);
        assert_eq!(node.get_valid_mask(), 255);
    }

    #[test]
    fn get_child_ptr() {
        let mut node = FlatOctreeNode::default();
        node.set_child_ptr(0xFFF_FFF);
        assert_eq!(node.get_child_ptr(), 0xFFF_FFF);
    }

    #[test]
    fn colors() {
        let mut node = FlatOctreeNode::default();
        for i in 0..8 {
            node.colors.set_color(i, i + (255 - 8));
        }
        for i in 0..8 {
            assert_eq!(node.colors.get_color(i), i + (255 - 8));
        }
    }

    #[test]
    fn flatten() {
        let mut node = OctreeNode::default();

        for x in 0..10 {
            let y = (x as f64 / 3.0).sin() / 2.0;
            node.write(dvec3(x as f64 / 10.0, y, 0.0), x, 10);
        }

        let flat1 = node.flatten();
        let node = flat1.unflatten();
        let flat2 = node.flatten();

        assert_eq!(flat1, flat2);
    }

    #[test]
    fn flatten_bytes() {
        let mut node = OctreeNode::default();

        for x in 0..10 {
            let y = (x as f64 / 3.0).sin() / 2.0;
            node.write(dvec3(x as f64 / 10.0, y, 0.0), x, 10);
        }

        let flat1 = node.flatten();

        let bytes = flat1.as_bytes();
        let flat2 = FlatOctree::from_bytes(bytes);

        let node = flat2.unflatten();

        for x in 0..10 {
            let y = (x as f64 / 3.0).sin() / 2.0;
            let v = node.sample(dvec3(x as f64 / 10.0, y, 0.0), 10);
            assert_eq!(v, x);
        }
    }
}
