#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use std::time::Instant;

use anyhow::{Context, Result};

mod setup;
mod frame;
mod transform;
mod application;


use application::Application;

fn main() -> Result<()> {
    let mut glfw_ctx = glfw::init(glfw::fail_on_errors)?;

    let (mut window, glfw_events) = glfw_ctx
        .create_window(600, 400, "vulkan window", glfw::WindowMode::Windowed)
        .context("failed to create window")?;

    window.set_size_polling(true);
    window.set_key_polling(true);

    let mut app = unsafe { Application::new(&window) }?;

    let mut dt = Instant::now();

    while !window.should_close() {
        glfw_ctx.poll_events();

        for (_, event) in glfw::flush_messages(&glfw_events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    window.set_should_close(true);
                }
                glfw::WindowEvent::Size(x, y) => unsafe { app.on_resize([x as u32, y as u32]) }?,

                _ => {}
            }
        }

        // println!("fps {}", 1.0 / dt.elapsed().as_secs_f64());
        dt = Instant::now();

        unsafe { app.on_render() }?;
    }

    unsafe { app.destroy() };

    Ok(())
}
