use crate::instances::BufferAllocation;
use anyhow::{Context, Result};
use std::sync::Arc;

use ash::vk::{self, CommandPoolCreateFlags};

use super::{compute::PipelineCompute, Subbuffer};

pub struct CommandPool {
    intern: vk::CommandPool,
    queue_family: u32,
    device: Arc<super::Device>,
}

impl CommandPool {
    pub fn new(device: Arc<super::Device>, queue_family: u32) -> Result<Arc<Self>> {
        let device_raw = device.as_raw();

        let create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family)
            .flags(CommandPoolCreateFlags::TRANSIENT);

        let command_pool = unsafe { device_raw.create_command_pool(&create_info, None) }?;

        Ok(Arc::new(Self {
            intern: command_pool,
            queue_family,
            device,
        }))
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe { self.device.as_raw().destroy_command_pool(self.intern, None) };
    }
}

pub struct CommandBuffer {
    intern: vk::CommandBuffer,
    pool: Arc<CommandPool>,
}

impl CommandBuffer {
    pub fn new(pool: Arc<CommandPool>) -> Result<Self> {
        let device_raw = pool.device.as_raw();

        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool.intern)
            .command_buffer_count(1);

        let command_buffer = unsafe { device_raw.allocate_command_buffers(&allocate_info) }?
            .into_iter()
            .next()
            .context("failed to allocate command buffer")?;

        Ok(Self {
            intern: command_buffer,
            pool,
        })
    }

    pub fn as_raw(&self) -> vk::CommandBuffer {
        self.intern.clone()
    }
    fn device_raw(&self) -> ash::Device {
        self.pool.device.as_raw()
    }

    pub fn begin(&self, begin_info: vk::CommandBufferUsageFlags) -> Result<()> {
        unsafe {
            self.device_raw().begin_command_buffer(
                self.intern,
                &vk::CommandBufferBeginInfo::default().flags(begin_info),
            )
        }?;
        Ok(())
    }

    pub fn end(&self) {
        unsafe { self.device_raw().end_command_buffer(self.intern) }.unwrap();
    }

    pub fn update_buffer<T: bytemuck::Pod>(
        &self,
        buffer: Arc<Subbuffer<T>>,
        offset: u64,
        data: &[T],
    ) {
        let data = bytemuck::cast_slice(data);

        if std::mem::size_of_val(data) as u64 > buffer.size() - offset {
            panic!("the data exeeds the max limit of this buffer");
        }

        unsafe {
            self.device_raw().cmd_update_buffer(
                self.intern,
                buffer.buffer_raw(),
                offset + buffer.offset(),
                data,
            )
        };
    }

    pub fn bind_pipeline_compute(&self, pipeline: Arc<PipelineCompute>) {
        unsafe {
            self.device_raw().cmd_bind_pipeline(
                self.intern,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.as_raw(),
            )
        };
    }

    pub fn bind_descriptor_set(
        &self,
        set: Arc<crate::instances::descriptors::DescriptorSet>,
        pipeline: Arc<PipelineCompute>,
    ) {
        unsafe {
            self.device_raw().cmd_bind_descriptor_sets(
                self.intern,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.layout(),
                0,
                &set.as_raw(),
                &[],
            )
        };
    }

    pub fn dispatch(&self, x: u32, y: u32, z: u32) {
        unsafe {
            self.device_raw().cmd_dispatch(self.intern, x, y, z);
        };
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.pool
                .device
                .as_raw()
                .free_command_buffers(self.pool.intern, &[self.intern])
        };
    }
}
