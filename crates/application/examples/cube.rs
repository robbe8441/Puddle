use application::{
    world::{svo::OctreeNode, World},
    Application,
};
use ash::vk;
use math::dvec3;
use math::{vec3, Transform, Vec3};
use rendering::vulkan::Buffer;
use std::error::Error;

fn update_camera(world: &mut World) {
    let t = 1.0f32;

    world.camera.transform =
        Transform::from_xyz(t.cos() * 2.0, 1.0, t.sin() * 2.0).looking_at(Vec3::ZERO, Vec3::Y);
}

fn create_octree(app: &mut Application) {
    let mut octree = OctreeNode::default();

    octree.write(dvec3(0.0, 0.0, 0.0), 255, 7);
    let flatten = octree.flatten();
    dbg!(&flatten);
    let bytes = flatten.as_bytes();

    let voxel_buffer = Buffer::new(
        app.renderer.device.clone(),
        bytes.len() as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE,
    )
    .unwrap();

    voxel_buffer.write(0, bytes);

    let _handle = app.renderer.set_storage_buffer(voxel_buffer.clone());

    app.world.voxel_octrees.push(octree);
    app.world.voxel_buffers.push(voxel_buffer);
}

fn write_octree(world: &mut World) {
    let buffer = &world.voxel_buffers[0];
    let t = world.start_time.elapsed().as_secs_f64() / 10.0;

    let mut octree = OctreeNode::default();
    octree.write(
        dvec3(
            t.sin() / 2.0 + 0.5,
            0.0,
            t.cos() / 2.0 + 0.5,
        ),
        255,
        7,
    );

    let flatten = octree.flatten();
    let bytes = flatten.as_bytes();

    buffer.write(0, bytes);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = Application::new()?;
    create_octree(&mut app);
    app.add_task(update_camera).add_task(write_octree);
    app.run();

    Ok(())
}
