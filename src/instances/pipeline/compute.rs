use crate::instances::{Device, ShaderModule};
use anyhow::Result;
use std::{ffi::CStr, sync::Arc};

use ash::vk;

pub struct PipelineCompute {
    intern: vk::Pipeline,
    layout: vk::PipelineLayout,
    device: Arc<Device>,
    descriptor_layouts: Arc<[vk::DescriptorSetLayout]>,
}

impl PipelineCompute {
    pub fn new(
        device: Arc<Device>,
        shader: Arc<ShaderModule>,
        descriptor_layouts: Vec<vk::DescriptorSetLayout>,
    ) -> Result<Arc<Self>> {
        let device_raw = device.as_raw();

        let shader_stage = shader.shader_stage_info();

        let layout_create_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptor_layouts);

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
            descriptor_layouts: descriptor_layouts.into(),
        }))
    }
}

impl super::Pipeline for PipelineCompute {
    fn bind_point(&self) -> vk::PipelineBindPoint {
        vk::PipelineBindPoint::COMPUTE
    }
    fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    fn as_raw(&self) -> vk::Pipeline {
        self.intern
    }
    fn set_layouts(&self) -> Arc<[vk::DescriptorSetLayout]> {
        self.descriptor_layouts.clone()
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
