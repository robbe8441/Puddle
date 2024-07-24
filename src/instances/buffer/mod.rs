use ash::{util::Align, vk};
use std::{mem, sync::Arc};

#[derive(Clone, Copy, Debug)]
struct Vertex {
    pos: [f32; 4],
}

unsafe fn temp(device: Arc<super::Device>) {
    let vertex_input_buffer_info = vk::BufferCreateInfo {
        size: 3 * mem::size_of::<Vertex>() as u64,
        usage: vk::BufferUsageFlags::VERTEX_BUFFER,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let device_raw = device.as_raw();

    let vertex_input_buffer = device_raw
        .create_buffer(&vertex_input_buffer_info, None)
        .unwrap();

    let vertex_input_buffer_memory_req =
        device_raw.get_buffer_memory_requirements(vertex_input_buffer);

    let vertex_input_buffer_memory_index = find_memorytype_index(
        &vertex_input_buffer_memory_req,
        &device.memory_priorities(),
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
    .expect("Unable to find suitable memorytype for the vertex buffer.");

    let vertex_buffer_allocate_info = vk::MemoryAllocateInfo {
        allocation_size: vertex_input_buffer_memory_req.size,
        memory_type_index: vertex_input_buffer_memory_index,
        ..Default::default()
    };

    let vertex_input_buffer_memory = 
        device_raw
        .allocate_memory(&vertex_buffer_allocate_info, None)
        .unwrap();

    let vertices = [
        Vertex {
            pos: [-1.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            pos: [1.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            pos: [0.0, -1.0, 0.0, 1.0],
        },
    ];

    let vert_ptr = 
        device_raw
        .map_memory(
            vertex_input_buffer_memory,
            0,
            vertex_input_buffer_memory_req.size,
            vk::MemoryMapFlags::empty(),
        )
        .unwrap();

    let mut vert_align = Align::new(
        vert_ptr,
        mem::align_of::<Vertex>() as u64,
        vertex_input_buffer_memory_req.size,
    );
    vert_align.copy_from_slice(&vertices);
    device_raw.unmap_memory(vertex_input_buffer_memory);

    device_raw
        .bind_buffer_memory(vertex_input_buffer, vertex_input_buffer_memory, 0)
        .unwrap();
}

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
