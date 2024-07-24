use std::sync::Arc;
use ash::vk;

#[allow(unused)]
pub struct Queue {
    pub(super) intern: vk::Queue,
    pub(super) queue_family_index: u32,
    pub(super) device: Arc<super::Device>,
}

impl Queue {

    pub fn family_index(&self) -> u32 {
        self.queue_family_index
    }

}


