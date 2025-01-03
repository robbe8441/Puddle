use application::{
    world::{svo::OctreeNode, World},
    Application,
};
use ash::vk;
use math::{vec3, Transform, Vec3};
use rendering::vulkan::Buffer;
use std::error::Error;

fn update_camera(world: &mut World) {
    let t = world.start_time.elapsed().as_secs_f32();

    world.camera.transform =
        Transform::from_xyz(t.cos(), 0.3, t.sin()).looking_at(Vec3::ZERO, Vec3::Y);
}

fn create_octree(app: &mut Application) {
    let mut octree = OctreeNode::default();

    octree.write(vec3(0.0, 0.0, 0.0), u8::MAX, 5);
    let flatten = octree.flatten();
    let bytes = flatten.as_bytes();

    let uniform_buffer = Buffer::new(
        app.renderer.device.clone(),
        bytes.len() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE,
    )
    .unwrap();

    uniform_buffer.write(0, bytes);

    let handle = app.renderer.set_uniform_buffer(uniform_buffer.clone());
    dbg!(handle);

    app.world.voxel_octrees.push(octree);
    app.world.voxel_buffers.push(uniform_buffer);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = Application::new()?;
    create_octree(&mut app);

    app.add_task(update_camera);
    app.run();

    Ok(())
}
