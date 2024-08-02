use super::pipeline::Pipeline;
use crate::instances::BufferAllocation;
use anyhow::{Context, Result};
use std::sync::Arc;

use ash::vk::{self, CommandPoolCreateFlags};

use super::Subbuffer;

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
    fn device_raw(&self) -> &ash::Device {
        self.pool.device.as_raw()
    }

    pub fn begin(&self, begin_info: vk::CommandBufferUsageFlags) {
        unsafe {
            self.device_raw().begin_command_buffer(
                self.intern,
                &vk::CommandBufferBeginInfo::default().flags(begin_info),
            )
        }
        .unwrap();
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
        unsafe {
            self.device_raw().cmd_update_buffer(
                self.intern,
                buffer.buffer_raw(),
                offset + buffer.offset(),
                data,
            )
        };
    }

    pub fn bind_pipeline(&self, pipeline: Arc<dyn Pipeline>) {
        unsafe {
            self.device_raw().cmd_bind_pipeline(
                self.intern,
                pipeline.bind_point(),
                pipeline.as_raw(),
            )
        };
    }

    pub fn bind_descriptor_set(
        &self,
        set: Arc<crate::instances::descriptors::DescriptorSet>,
        first_set: u32,
        pipeline: Arc<dyn Pipeline>,
        offsets: &[u32],
    ) {
        unsafe {
            self.device_raw().cmd_bind_descriptor_sets(
                self.intern,
                pipeline.bind_point(),
                pipeline.layout(),
                first_set,
                &set.as_raw(),
                offsets,
            )
        };
    }

    pub fn dispatch(&self, x: u32, y: u32, z: u32) {
        unsafe {
            self.device_raw().cmd_dispatch(self.intern, x, y, z);
        };
    }

    pub fn set_viewport(&self, first_viewport: u32, viewports: &[vk::Viewport]) {
        unsafe {
            self.device_raw()
                .cmd_set_viewport(self.intern, first_viewport, viewports)
        };
    }

    pub fn set_scissor(&self, first_scissor: u32, scissors: &[vk::Rect2D]) {
        unsafe {
            self.device_raw()
                .cmd_set_scissor(self.intern, first_scissor, scissors)
        };
    }

    pub fn bind_vertex_buffers(
        &self,
        first_binding: u32,
        buffers: &[Arc<dyn BufferAllocation>],
        offsets: &[u64],
    ) {
        let buffers: Vec<_> = buffers.iter().map(|v| v.buffer_raw()).collect();

        unsafe {
            self.device_raw()
                .cmd_bind_vertex_buffers(self.intern, first_binding, &buffers, offsets)
        };
    }

    pub fn draw(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.device_raw().cmd_draw(
                self.intern,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        }
    }

    pub fn begin_render_pass(
        &self,
        begin_info: &vk::RenderPassBeginInfo,
        contents: vk::SubpassContents,
    ) {
        unsafe {
            self.device_raw()
                .cmd_begin_render_pass(self.intern, begin_info, contents)
        }
    }
    pub fn end_render_pass(&self) {
        unsafe { self.device_raw().cmd_end_render_pass(self.intern) };
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
