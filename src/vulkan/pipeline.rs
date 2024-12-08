use ash::{util::read_spv, vk};
use std::io::Cursor;

use super::VulkanContext;

fn create_shader_module(device: &ash::Device) -> Result<vk::ShaderModule, vk::Result> {
    let mut shader_file = Cursor::new(&include_bytes!("../../shaders/shader.spv"));

    let vertex_code = read_spv(&mut shader_file).expect("Failed to read vertex shader spv file");

    let create_info = vk::ShaderModuleCreateInfo::default().code(&vertex_code);

    unsafe { device.create_shader_module(&create_info, None) }
}

pub fn create_pipeline(vk_ctx: &VulkanContext) -> Result<vk::Pipeline, vk::Result> {
    let shader_module = create_shader_module(&vk_ctx.device)?;

    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .name(c"main")
        .module(shader_module);

    let create_info = [vk::ComputePipelineCreateInfo::default()
        .stage(stage)
        .layout(vk_ctx.bindless_handler.pipeline_layout)];

    let pipeline = unsafe {
        vk_ctx
            .device
            .create_compute_pipelines(vk::PipelineCache::null(), &create_info, None)
    }
    .unwrap()[0];

    unsafe { vk_ctx.device.destroy_shader_module(shader_module, None) };

    Ok(pipeline)
}
