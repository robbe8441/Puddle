
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryAccessFlags(u32);

impl MemoryAccessFlags {
    pub const READ: Self = Self(0b01);
    pub const WRITE: Self = Self(0b1);

    #[must_use]
    pub fn contains(&self, rhs: Self) -> bool {
        self.0 & rhs.0 == rhs.0
    }
}

pub enum ResourceType {
    Buffer,
    ImageView,
} 





