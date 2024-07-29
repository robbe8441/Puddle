use anyhow::Result;
use ash::vk;
use graphics::PipelineGraphics;
use std::sync::Arc;

use vk_render::instances::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let window = Arc::new(WindowBuilder::new().build(&event_loop)?);

    let extensions = Surface::enumerate_required_extensions(&event_loop)?;

    let instance = unsafe { Instance::from_extensions(extensions) }?;

    let (device, queue) = Device::new_default(instance.clone())?;

    let surface = Surface::new(instance.clone(), window.clone())?;

    let swapchain = Swapchain::new(device.clone(), surface)?;

    let command_pool = CommandPool::new(device.clone(), queue.family_index())?;

    let pipeline = PipelineGraphics::test(device.clone(), swapchain.clone());

    event_loop.run(|event, target| match event {
        Event::WindowEvent {
            window_id: _,
            event,
        } => match event {
            WindowEvent::RedrawRequested => {
                let fence = Fence::new(device.clone()).unwrap();

                let (image_index, _suboptimal) = swapchain.aquire_next_image(fence.clone());

                let command_buffer = CommandBuffer::new(command_pool.clone()).unwrap();

                command_buffer
                    .begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                    .unwrap();

                unsafe { pipeline.draw(&command_buffer, image_index) };

                command_buffer.end();

                fence
                    .submit_buffers(&[command_buffer], queue.clone())
                    .unwrap();

                swapchain.present(queue.clone(), image_index);

                fence.wait_for_finished(u64::MAX).unwrap();
            }
            WindowEvent::CloseRequested => target.exit(),
            _ => {}
        },
        Event::AboutToWait => window.request_redraw(),
        _ => {}
    })?;

    Ok(())
}
