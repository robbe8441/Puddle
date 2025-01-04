use glfw::{Glfw, GlfwReceiver, PWindow, WindowEvent};

pub struct AppWindow {
    pub glfw_ctx: Glfw,
    pub window: PWindow,
    pub glfw_events: GlfwReceiver<(f64, WindowEvent)>,
}


impl AppWindow {
    pub fn new() -> Self {
        let mut glfw_ctx = glfw::init(glfw::fail_on_errors).unwrap();

        let (mut window, glfw_events) = glfw_ctx
            .create_window(800, 600, "Puddle triangle", glfw::WindowMode::Windowed)
            .unwrap();

        window.set_size_polling(true);

        Self {
            glfw_ctx,
            window,
            glfw_events,
        }
    }

    pub fn get_size(&self) -> [u32; 2] {
        let v = self.window.get_size();
        [v.0 as u32, v.1 as u32]
    }
}

impl Default for AppWindow {
    fn default() -> Self {
        Self::new()
    }
}
