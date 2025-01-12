#![feature(debug_closure_helpers)]
#![feature(box_as_ptr)]
#![allow(clippy::cast_possible_truncation)]

use ash::prelude::VkResult;
use rendering::handler::RenderHandler;
use window::AppWindow;
use world::World;

mod window;
pub mod world;

type TaskFn = dyn Fn(&mut World);

pub struct Application {
    pub tasks: Vec<Box<TaskFn>>,
    pub world: World,
    pub renderer: RenderHandler,
    /// window should be dropped last as it invalidates the surface and so the swapchain
    pub window: AppWindow,
}

impl Application {
    /// # Errors
    /// if your gpu isn't supported by the renderer
    /// or something else causes vulkan to error (for example ``OutOfMemory``)
    pub fn new() -> VkResult<Self> {
        let window = AppWindow::new();

        let mut renderer = RenderHandler::new(&window.window, window.get_size())?;
        let world = World::new(&mut renderer);

        Ok(Self {
            window,
            renderer,
            world,
            tasks: vec![],
        })
    }

    pub fn add_task<F>(&mut self, task: F) -> &mut Self
    where
        F: Fn(&mut World) + 'static,
    {
        self.tasks.push(Box::new(task));
        self
    }

    pub fn run(&mut self) {
        let mut dt = std::time::Instant::now();

        while !self.window.window.should_close() {
            println!("fps: {}", 1.0 / dt.elapsed().as_secs_f64());
            dt = std::time::Instant::now();

            for task in &self.tasks {
                (task)(&mut self.world);
            }

            self.world.update();

            let _ = self
                .renderer
                .on_render()
                .inspect_err(|v| eprintln!("{v:?}"));

            self.window.glfw_ctx.poll_events();

            for (_, event) in glfw::flush_messages(&self.window.glfw_events) {
                match event {
                    // glfw::WindowEvent::Size(x, y) => {
                    //     let _ = self.renderer.on_window_resize([x as u32, y as u32]);
                    //     self.world.camera.aspect = x as f32 / y as f32;
                    // }
                    glfw::WindowEvent::Close => {
                        self.window.window.set_should_close(true);
                    }

                    _ => {}
                }
            }
        }
    }
}

impl Drop for AppWindow {
    fn drop(&mut self) {}
}
