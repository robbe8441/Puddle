#![feature(debug_closure_helpers)]

use ash::prelude::VkResult;
use rendering::handler::RenderHandler;
use window::AppWindow;
use world::World;

mod window;
pub mod world;

type TaskFn = dyn Fn(&mut World);

pub struct Application {
    pub window: AppWindow,
    pub renderer: RenderHandler,
    pub world: World,
    pub tasks: Vec<Box<TaskFn>>,
}

impl Application {
    /// # Errors
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
        while !self.window.window.should_close() {
            for task in &self.tasks {
                (task)(&mut self.world);
            }

            self.world.update();

            let _ = unsafe { self.renderer.draw() }.inspect_err(|v| eprintln!("{v:?}"));

            self.window.glfw_ctx.poll_events();

            for (_, event) in glfw::flush_messages(&self.window.glfw_events) {
                match event {
                    glfw::WindowEvent::Size(x, y) => {
                        let _ = unsafe { self.renderer.resize([x as u32, y as u32]) };
                        self.world.camera.aspect = x as f32 / y as f32;
                    }
                    glfw::WindowEvent::Close => {
                        self.window.window.set_should_close(true);
                    }

                    _ => {}
                }
            }
        }
    }
}
