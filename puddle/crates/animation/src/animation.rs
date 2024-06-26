use std::sync::Arc;
use asset_manager::RawTexture;

use bevy_ecs::{
    component::Component,
    system::Resource,
};

#[derive(Component, Resource)]
pub struct Animation {
    pub frames: Vec<RawTexture>,
    pub fps: f32,
}



impl Animation {
    pub fn new(frames: impl Into<Vec<RawTexture>>, fps: f32) -> Arc<Self> {
        Arc::new(Self {
            frames: frames.into(),
            fps,
        })
    }

}
