use std::error::Error;

use application::{
    world::{svo::OctreeNode, World},
    Application,
};
use ash::vk;
use math::dvec3;
use math::{Transform, Vec3};
use rand::{thread_rng, Rng};
use rendering::vulkan::Buffer;

fn update_camera(world: &mut World) {
    let t = world.start_time.elapsed().as_secs_f32() / 5.0;

    world.camera.transform =
        Transform::from_xyz(t.cos() * 1.2, 0.2, t.sin() * 1.2).looking_at(Vec3::ZERO, Vec3::Y);
}

fn create_octree(app: &mut Application) {
    let voxel_buffer = Buffer::new(
        app.renderer.device.clone(),
        8 * 1024 * 100,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE,
    )
    .unwrap();

    let handle = app.renderer.set_storage_buffer(voxel_buffer.clone(), 0);
    assert!(handle.index == 0);

    let mut octree = OctreeNode::default();
    octree.write(dvec3(1.0, 1.0, 1.0), 255, 1);
    octree.write(dvec3(1.0, 1.0, -1.0), 255, 1);
    octree.write(dvec3(1.0, -1.0, 1.0), 255, 1);
    octree.write(dvec3(1.0, -1.0, -1.0), 255, 1);
    octree.write(dvec3(-1.0, 1.0, 1.0), 255, 1);
    octree.write(dvec3(-1.0, 1.0, -1.0), 255, 1);
    octree.write(dvec3(-1.0, -1.0, 1.0), 255, 1);
    octree.write(dvec3(-1.0, -1.0, -1.0), 255, 1);

    octree.write(dvec3(-1.0, -1.0, -1.0), 60, 3);
    octree.write(dvec3(-1.0, -1.0, -1.0), 255, 4);

    // octree.write(dvec3(-1.0, -1.0, -1.0), 255, 3);

    let flatten = octree.flatten();
    let bytes = flatten.as_bytes();

    voxel_buffer.write(0, bytes);

    app.world.voxel_octrees.push(octree);
    app.world.voxel_buffers.push(voxel_buffer);
}

fn write_octree(world: &mut World) {
    let buffer = &world.voxel_buffers[0];
    let octree = &mut world.voxel_octrees[0];
    let t = world.start_time.elapsed().as_secs_f64() * 10.0;

    let mut rng = thread_rng();
    let x_pos = rng.gen_range(-1.0..1.0);
    let y_pos = rng.gen_range(-1.0..1.0);
    let z_pos = rng.gen_range(-1.0..1.0);
    // let color = (t / 10.0).cos() / 2.0 + 0.5;
    let color = if rng.gen_bool(0.0001) { 50 } else { 255 };

    octree.write(dvec3(x_pos, y_pos, z_pos) * (t / 51.0).cos(), 255, 4);

    let flatten = octree.flatten();
    let bytes = flatten.as_bytes();
    dbg!(bytes.len());

    buffer.write(0, bytes);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = Application::new()?;
    // std::thread::sleep(std::time::Duration::from_secs_f32(3.0));

    create_octree(&mut app);
    app.add_task(update_camera); //.add_task(write_octree);
    app.run();

    Ok(())
}

// fn main() {
//     let mut octree = OctreeNode::default();
//
//     octree.write(dvec3(-1.0, -1.0, -1.0), 255, 2);
//     octree.write(dvec3(0.0, 0.0, 0.0), 255, 2);
//
//     octree.write(dvec3(-1.0, 0.0, 0.0), 255, 2);
//     octree.write(dvec3(0.0, -1.0, 0.0), 255, 2);
//     octree.write(dvec3(0.0, 0.0, -1.0), 255, 2);
//
//     octree.write(dvec3(-1.0, 0.0, 0.0), 255, 2);
//     octree.write(dvec3(-1.0, -1.0, 0.0), 255, 2);
//     octree.write(dvec3(-1.0, 0.0, -1.0), 255, 2);
//     octree.write(dvec3(0.0, -1.0, -1.0), 255, 2);
//
//
//     let flat = octree.flatten();
//     dbg!(flat);
// }
