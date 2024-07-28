use ash::vk;
use vk_render::instances::*;

#[test]
fn compute_pipeline() {
    let instance =
        unsafe { Instance::from_extensions(&[ash::ext::debug_utils::NAME.as_ptr()]) }.unwrap();

    let (device, queue) = Device::new_default(instance.clone()).unwrap();

    let source = include_str!("./compute.glsl");

    let shader =
        ShaderModule::from_source(device.clone(), source, ShaderKind::Compute, "main").unwrap();

    let command_pool = CommandPool::new(device.clone(), queue.family_index()).unwrap();

    let command_buffer = CommandBuffer::new(command_pool).unwrap();

    let fence = Fence::new(device.clone()).unwrap();

    let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let subbuffer = Subbuffer::from_data(
        device.clone(),
        vk::BufferCreateInfo {
            size: std::mem::size_of_val(&data) as u64,
            usage: vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::UNIFORM_BUFFER,
            ..Default::default()
        },
        vk::MemoryPropertyFlags::HOST_VISIBLE,
        &data,
    )
    .unwrap();

    let subbuffer2 = Subbuffer::from_data(
        device.clone(),
        vk::BufferCreateInfo {
            size: std::mem::size_of_val(&data) as u64,
            usage: vk::BufferUsageFlags::STORAGE_BUFFER,
            ..Default::default()
        },
        vk::MemoryPropertyFlags::HOST_VISIBLE,
        &data,
    )
    .unwrap();

    subbuffer.write(&data).unwrap();

    let pool_sizes = [vk::DescriptorPoolSize {
        descriptor_count: 1,
        ty: vk::DescriptorType::STORAGE_BUFFER,
    }];

    let descriptor_pool = descriptors::DescriptorPool::new(device.clone(), &pool_sizes, 1).unwrap();

    let buffers = [subbuffer.desc()];

    let writes = [descriptors::WriteDescriptorSet {
        buffer_info: Some(&buffers),
        image_info: None,
        dst_binding: 0,
        descriptor_count: 1,
        dst_set: 0,
        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
    }];

    let descriptor_set = descriptors::DescriptorSet::new(descriptor_pool, &writes).unwrap();

    let pipeline =
        compute::PipelineCompute::new(device.clone(), shader, descriptor_set.clone()).unwrap();

    command_buffer
        .begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .unwrap();

    command_buffer.bind_pipeline_compute(pipeline.clone());

    command_buffer.bind_descriptor_set(descriptor_set, pipeline);

    command_buffer.dispatch(1, 1, 1);

    command_buffer.end();

    fence.submit_buffers(&[command_buffer], queue).unwrap();

    fence.wait_for_finished(u64::MAX).unwrap();

    assert_eq!(
        subbuffer.read().unwrap(),
        [300, 300, 300, 300, 5, 6, 7, 8, 9, 10]
    );
    assert_eq!(subbuffer2.read().unwrap(), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}
