use crate::instances::{Device, ShaderModule};
use anyhow::Result;
use std::{ffi::CStr, sync::Arc};

use ash::vk;

pub struct PipelineCompute {
    intern: vk::Pipeline,
    layout: vk::PipelineLayout,
    device: Arc<Device>,
}

impl PipelineCompute {
    pub fn new(device: Arc<Device>, shader: Arc<ShaderModule>, descriptors: Arc<crate::instances::descriptors::descriptor_pool::DescriptorSet>) -> Result<Arc<Self>> {
        let device_raw = device.as_raw();

        let mut entry = shader.entry().to_string();
        entry.push_str("\0");

        let shader_stage = vk::PipelineShaderStageCreateInfo::default()
            .module(shader.as_raw())
            .stage(shader.kind().into())
            .name(unsafe { CStr::from_bytes_with_nul_unchecked(entry.as_bytes()) });

        let descriptor_layouts = descriptors.layout();
        let layout_create_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptor_layouts);

        let layout = unsafe { device_raw.create_pipeline_layout(&layout_create_info, None) }?;

        let create_info = [vk::ComputePipelineCreateInfo::default()
            .stage(shader_stage)
            .layout(layout)];

        let pipeline = unsafe {
            device_raw.create_compute_pipelines(vk::PipelineCache::default(), &create_info, None)
        }
        .unwrap()[0];

        Ok(Arc::new(Self {
            intern: pipeline,
            layout,
            device,
        }))
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout.clone()
    }

    pub fn as_raw(&self) -> vk::Pipeline {
        self.intern.clone()
    }
}

impl Drop for PipelineCompute {
    fn drop(&mut self) {
        unsafe {
            self.device
                .as_raw()
                .destroy_pipeline_layout(self.layout, None);
            self.device.as_raw().destroy_pipeline(self.intern, None);
        }
    }
}
