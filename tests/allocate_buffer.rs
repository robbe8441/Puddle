use ash::vk;
use vk_render::instances::*;

#[test]
pub fn allocate_buffer() {
    let instance = Instance::new_default().unwrap();

    let (device, queue) = Device::new_default(instance.clone()).unwrap();

    let command_pool = CommandPool::new(device.clone(), queue.family_index()).unwrap();

    let command_buffer = CommandBuffer::new(command_pool).unwrap();

    let fence = Fence::new(device.clone()).unwrap();

    let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let subbuffer = Subbuffer::from_data(
        device,
        vk::BufferCreateInfo {
            size: std::mem::size_of_val(&data) as u64,
            usage: vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::STORAGE_BUFFER,
            ..Default::default()
        },
        vk::MemoryPropertyFlags::HOST_VISIBLE,
        &data,
    )
    .unwrap();

    assert_eq!(subbuffer.read().unwrap(), data);

    let data2 = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
    subbuffer.write(&data2).unwrap();

    assert_eq!(subbuffer.read().unwrap(), data2);

    command_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    let val = [1000, 254, 253];

    command_buffer.update_buffer(subbuffer.clone(), 0, &val);

    command_buffer.end();

    fence.submit_buffers(&[command_buffer], queue).unwrap();

    fence.wait_for_finished(u64::MAX).unwrap();

    assert_eq!(
        subbuffer.read().unwrap(),
        [1000, 254, 253, 7, 6, 5, 4, 3, 2, 1]
    );
}
