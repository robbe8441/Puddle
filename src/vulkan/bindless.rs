#![allow(dead_code, unused)]

use std::sync::Mutex;

use super::{buffer::Buffer, VulkanContext};
use ash::vk;

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct PushConstant {
    pub camera: u32,
    pub swapchain_image: u32,
    pub insance_array: u32,
    pub instance_count: u32,
}

unsafe impl bytemuck::NoUninit for PushConstant {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u32);

impl TextureHandle {
    pub const INVALID: Self = Self(u32::MAX);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle(pub u32);

impl BufferHandle {
    pub const INVALID: Self = Self(u32::MAX);
}

pub struct BindlessHandler {
    pub pool: vk::DescriptorPool,
    pub descriptor_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub descriptor_set: vk::DescriptorSet,
    pub buffers: Mutex<Vec<vk::Buffer>>,
    pub textures: Mutex<Vec<vk::ImageView>>,
}

impl BindlessHandler {
    pub const UNIFORM_BINDING: u32 = 0;
    pub const STORAGE_BINDING: u32 = 1;
    pub const TEXTURE_BINDING: u32 = 2;
    pub const STORAGE_IMAGE_BINDING: u32 = 3;

    pub fn new(device: &ash::Device) -> Result<Self, vk::Result> {
        let pool = create_bindless_descriptor_pool(device)?;
        let descriptor_layout = create_bindless_descriptor_layout(device)?;
        let descriptor_set = create_bindless_descriptor_set(device, pool, descriptor_layout)?;

        let push_constant_ranges = [vk::PushConstantRange::default()
            .size(std::mem::size_of::<PushConstant>() as u32)
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .offset(0)];

        let layouts = [descriptor_layout];
        let create_info = vk::PipelineLayoutCreateInfo::default()
            .push_constant_ranges(&push_constant_ranges)
            .set_layouts(&layouts);

        let pipeline_layout = unsafe { device.create_pipeline_layout(&create_info, None) }?;

        Ok(Self {
            pool,
            descriptor_layout,
            pipeline_layout,
            descriptor_set,
            buffers: vec![].into(),
            textures: vec![].into(),
        })
    }

    pub unsafe fn destroy(&self, vk_ctx: &VulkanContext) {
        vk_ctx.device.destroy_descriptor_pool(self.pool, None);
        vk_ctx
            .device
            .destroy_descriptor_set_layout(self.descriptor_layout, None);
        vk_ctx
            .device
            .destroy_pipeline_layout(self.pipeline_layout, None);
    }

    pub fn store_image(&self, device: &ash::Device, image_view: vk::ImageView) -> TextureHandle {
        let mut textures = self.textures.lock().unwrap();
        let new_handle = textures.len();
        textures.push(image_view);

        let image_info = [vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(image_view)];

        let write = vk::WriteDescriptorSet::default()
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .dst_binding(Self::STORAGE_IMAGE_BINDING)
            .dst_set(self.descriptor_set)
            .dst_array_element(new_handle as u32)
            .image_info(&image_info);

        unsafe { device.update_descriptor_sets(&[write], &[]) };

        TextureHandle(new_handle as u32)
    }

    pub fn store_texture(
        &self,
        device: &ash::Device,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
    ) -> TextureHandle {
        let mut textures = self.textures.lock().unwrap();
        let new_handle = textures.len();
        textures.push(image_view);

        let image_info = [vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_view)
            .sampler(sampler)];

        let write = vk::WriteDescriptorSet::default()
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .dst_binding(Self::TEXTURE_BINDING)
            .dst_set(self.descriptor_set)
            .dst_array_element(new_handle as u32)
            .image_info(&image_info);

        unsafe { device.update_descriptor_sets(&[write], &[]) };

        TextureHandle(new_handle as u32)
    }

    pub fn store_buffer(&self, device: &ash::Device, buffer: &Buffer) -> BufferHandle {
        let mut buffers = self.buffers.lock().unwrap();
        let new_handle = buffers.len();
        buffers.push(buffer.as_raw());

        let buffer_info = [vk::DescriptorBufferInfo::default()
            .buffer(buffer.as_raw())
            .range(vk::WHOLE_SIZE)];

        let mut writes = [vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_array_element(new_handle as u32)
            .buffer_info(&buffer_info)];

        let mut index = 0;

        if buffer
            .usage()
            .contains(vk::BufferUsageFlags::UNIFORM_BUFFER)
        {
            writes[index].dst_binding = Self::UNIFORM_BINDING;
            writes[index].descriptor_type = vk::DescriptorType::UNIFORM_BUFFER;
            index += 1;
        }

        if buffer
            .usage()
            .contains(vk::BufferUsageFlags::STORAGE_BUFFER)
        {
            writes[index].dst_binding = Self::STORAGE_BINDING;
            writes[index].descriptor_type = vk::DescriptorType::STORAGE_BUFFER;
        }

        unsafe { device.update_descriptor_sets(&writes, &[]) };

        BufferHandle(new_handle as u32)
    }
}

fn create_bindless_descriptor_set(
    device: &ash::Device,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
) -> Result<vk::DescriptorSet, vk::Result> {
    let layouts = [layout];

    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);

    let set = unsafe { device.allocate_descriptor_sets(&allocate_info) }?[0];

    Ok(set)
}

fn create_bindless_descriptor_pool(device: &ash::Device) -> Result<vk::DescriptorPool, vk::Result> {
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1000),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1000),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1000),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(1000),
    ];

    let create_info = vk::DescriptorPoolCreateInfo::default()
        .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
        .pool_sizes(&sizes)
        .max_sets(1000);

    let pool = unsafe { device.create_descriptor_pool(&create_info, None)? };

    Ok(pool)
}

fn create_bindless_descriptor_layout(
    device: &ash::Device,
) -> Result<vk::DescriptorSetLayout, vk::Result> {
    let mut bindings = [vk::DescriptorSetLayoutBinding::default()
        .descriptor_count(1000) // use as upper bound
        .stage_flags(vk::ShaderStageFlags::ALL); 4];

    let flags = [vk::DescriptorBindingFlags::PARTIALLY_BOUND
        | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND; 4];

    let types = [
        vk::DescriptorType::UNIFORM_BUFFER,
        vk::DescriptorType::STORAGE_BUFFER,
        vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        vk::DescriptorType::STORAGE_IMAGE,
    ];

    for i in 0..bindings.len() {
        bindings[i].binding = i as u32;
        bindings[i].descriptor_type = types[i];
    }

    let mut bind_flags =
        vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&flags);

    let create_info = vk::DescriptorSetLayoutCreateInfo::default()
        .bindings(&bindings)
        .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
        .push_next(&mut bind_flags);

    let layout = unsafe { device.create_descriptor_set_layout(&create_info, None) }?;

    Ok(layout)
}
