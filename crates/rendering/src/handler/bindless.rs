use std::sync::Arc;

use ash::{prelude::VkResult, vk};

use crate::vulkan::{Buffer, VulkanDevice};

#[derive(Debug)]
pub struct BindlessResourceHandle {
    binding: usize,
    ty: BindlessResourceType,
}

#[derive(Debug)]
pub enum BindlessResourceType {
    UniformBuffer,
    StorageBuffer,
    StorageImage,
}

pub struct BindlessHandler {
    descriptor_pool: vk::DescriptorPool,
    pub descriptor_layout: vk::DescriptorSetLayout,
    pub(crate) descriptor_set: vk::DescriptorSet,
    pub(crate) pipeline_layout: vk::PipelineLayout,
    uniform_buffers: Vec<Arc<Buffer>>,
    storage_buffers: Vec<Arc<Buffer>>,
    storage_images: Vec<vk::ImageView>,
}

impl BindlessHandler {
    pub const UNIFORM_BUFFER_BINDING: u32 = 0;
    pub const STORAGE_BUFFER_BINDING: u32 = 1;
    pub const STORAGE_IMAGE_BINDING: u32 = 2;

    pub const POOL_SIZE: u32 = 1000;

    pub fn new(device: &VulkanDevice) -> VkResult<Self> {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: Self::POOL_SIZE,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: Self::POOL_SIZE,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: Self::POOL_SIZE,
            },
        ];

        let pool_create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(Self::POOL_SIZE);

        let bindings: Vec<_> = pool_sizes
            .iter()
            .enumerate()
            .map(|(i, v)| {
                vk::DescriptorSetLayoutBinding::default()
                    .binding(i as u32)
                    .descriptor_type(v.ty)
                    .descriptor_count(v.descriptor_count)
                    .stage_flags(vk::ShaderStageFlags::ALL)
            })
            .collect();

        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

        let layout = unsafe { device.create_descriptor_set_layout(&layout_create_info, None)? };

        let pool = unsafe { device.create_descriptor_pool(&pool_create_info, None)? };

        let layouts = [layout];
        let set_allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);

        let set = unsafe { device.allocate_descriptor_sets(&set_allocate_info) }?[0];

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
        // TODO: .push_constant_ranges(push_constant_ranges);

        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }?;

        Ok(Self {
            descriptor_pool: pool,
            descriptor_layout: layout,
            descriptor_set: set,
            pipeline_layout,
            uniform_buffers: vec![],
            storage_images: vec![],
            storage_buffers: vec![],
        })
    }

    fn upload_buffer(
        &self,
        device: &VulkanDevice,
        buffer: vk::Buffer,
        ty: vk::DescriptorType,
        binding: u32,
        arr_index: u32,
    ) {
        let buffer_info = [vk::DescriptorBufferInfo::default()
            .buffer(buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE)];

        let write_set = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(binding)
            .descriptor_type(ty)
            .dst_array_element(arr_index)
            .buffer_info(&buffer_info)
            .descriptor_count(1);

        unsafe { device.update_descriptor_sets(&[write_set], &[]) };
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upload_image(
        &self,
        device: &VulkanDevice,
        image_view: vk::ImageView,
        image_layout: vk::ImageLayout,
        sampler: vk::Sampler,
        ty: vk::DescriptorType,
        binding: u32,
        arr_index: u32,
    ) {
        let image_info = [vk::DescriptorImageInfo::default()
            .image_view(image_view)
            .sampler(sampler)
            .image_layout(image_layout)];

        let write_set = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(binding)
            .descriptor_type(ty)
            .dst_array_element(arr_index)
            .image_info(&image_info)
            .descriptor_count(1);

        unsafe { device.update_descriptor_sets(&[write_set], &[]) };
    }

    pub fn set_uniform_buffer(
        &mut self,
        device: &VulkanDevice,
        buffer: Arc<Buffer>,
    ) -> BindlessResourceHandle {
        let arr_index = self.uniform_buffers.len();

        self.upload_buffer(
            device,
            buffer.handle(),
            vk::DescriptorType::UNIFORM_BUFFER,
            Self::UNIFORM_BUFFER_BINDING,
            arr_index as u32,
        );

        self.uniform_buffers.push(buffer);

        BindlessResourceHandle {
            binding: arr_index,
            ty: BindlessResourceType::UniformBuffer,
        }
    }

    pub fn set_storage_buffer(
        &mut self,
        device: &VulkanDevice,
        buffer: Arc<Buffer>,
    ) -> BindlessResourceHandle {
        let arr_index = self.storage_buffers.len();

        self.upload_buffer(
            device,
            buffer.handle(),
            vk::DescriptorType::STORAGE_BUFFER,
            Self::STORAGE_BUFFER_BINDING,
            arr_index as u32,
        );

        self.storage_buffers.push(buffer);

        BindlessResourceHandle {
            binding: arr_index,
            ty: BindlessResourceType::StorageBuffer,
        }
    }

    pub fn set_storage_image(
        &mut self,
        device: &VulkanDevice,
        image_view: vk::ImageView,
        image_layout: vk::ImageLayout,
    ) -> BindlessResourceHandle {
        let arr_index = self.storage_images.len();

        self.upload_image(
            device,
            image_view,
            image_layout,
            vk::Sampler::null(),
            vk::DescriptorType::STORAGE_IMAGE,
            Self::STORAGE_IMAGE_BINDING,
            arr_index as u32,
        );

        self.storage_images.push(image_view);

        BindlessResourceHandle {
            binding: arr_index,
            ty: BindlessResourceType::StorageImage,
        }
    }

    pub unsafe fn destroy(&self, device: &VulkanDevice) {
        device.destroy_descriptor_pool(self.descriptor_pool, None);
        device.destroy_descriptor_set_layout(self.descriptor_layout, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
    }
}
