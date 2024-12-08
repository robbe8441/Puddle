#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use application::Application;

mod application;
mod vulkan;

const MAX_FRAMES_ON_FLY: usize = 3;

fn main() {
    let mut app = Application::new();

    while !app.window.should_close() {
        app.draw().unwrap();
    }

    app.destroy();
}
