use crate::instances::{Device, ShaderModule};
use std::sync::Arc;

pub fn load(device: Arc<Device>) -> (Arc<ShaderModule>, Arc<ShaderModule>){
    let vs = ShaderModule::from_source(
        device.clone(),
        include_str!("./vert.glsl"),
        crate::instances::ShaderKind::Vertex,
        "main",
    )
    .unwrap();
    let fs = ShaderModule::from_source(
        device.clone(),
        include_str!("./frag.glsl"),
        crate::instances::ShaderKind::Fragment,
        "main",
    )
    .unwrap();

    (vs, fs)
}
