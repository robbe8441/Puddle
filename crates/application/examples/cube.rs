use std::error::Error;

use application::{
    world::{svo::OctreeNode, World},
    Application,
};
use ash::vk;
use math::dvec3;
use math::{Transform, Vec3};
use rendering::vulkan::Buffer;

fn update_camera(world: &mut World) {
    let t = world.start_time.elapsed().as_secs_f32() / 5.0;

    world.camera.transform =
        Transform::from_xyz(t.cos() * 2.0, 0.1, t.sin() * 0.1).looking_at(Vec3::ZERO, Vec3::Y);
}

fn create_octree(app: &mut Application) {
    let voxel_buffer = Buffer::new(
        app.renderer.device.clone(),
        1024 * 1024, // 1 Mib
        vk::BufferUsageFlags::STORAGE_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE,
    )
    .unwrap();

    let handle = app.renderer.set_storage_buffer(voxel_buffer.clone(), 0);
    assert!(handle.index == 0);

    let mut octree = OctreeNode::default();
    octree.write(dvec3(0.0, 0.0, 0.0), 255, 3);

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

    if t > 300.0 {
        return;
    }

    let orbit_x = t.cos();
    let orbit_z = t.sin();

    let t = t * 1.1;
    let x = orbit_x + (t / 10.1).cos() * 2.0;
    let z = orbit_z + (t / 10.1).sin() * 2.0;
    let y = x.powi(2) * z.powi(2);

    octree.write(dvec3(x / 3.0, y / 30.0, z / 3.0), 255, 9);

    let x = orbit_x + (t / 4.1).cos() * 2.0;
    let z = orbit_z + (t / 4.1).sin() * 2.0;

    octree.write(dvec3(-x / 3.0, -y / 100.0, -z / 3.0), 255, 9);

    let flatten = octree.flatten();
    let bytes = flatten.as_bytes();

    buffer.write(0, bytes);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = Application::new()?;
    // std::thread::sleep(std::time::Duration::from_secs_f32(3.0));

    create_octree(&mut app);
    app.add_task(update_camera).add_task(write_octree);
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
//     let flat = octree.flatten();
//     dbg!(flat);
// }
