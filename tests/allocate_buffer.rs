
use VkRender::instances::*;


#[test]
pub fn allocate_buffer() {

    let instance = Instance::new_default().unwrap();

    let (device, queue) = Device::new_default(instance.clone()).unwrap();

    let command_pool = CommandPool::new(device, queue.family_index()).unwrap();

    let command_buffer = CommandBuffer::new(command_pool).unwrap();

    drop(command_buffer);
}



