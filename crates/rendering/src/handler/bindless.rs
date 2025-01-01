use ash::{prelude::VkResult, vk};

use crate::vulkan::VulkanDevice;

pub struct BindlessResourceHandle {
    binding: usize,
    ty: BindlessResourceType,
}

pub enum BindlessResourceType {
    UniformBuffer,
    StorageBuffer,
    StorageImage,
}

pub struct BindlessHandler {
    descriptor_pool: vk::DescriptorPool,
    descriptor_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
    uniform_buffers: Vec<vk::Buffer>,
    storage_buffers: Vec<vk::Buffer>,
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

        Ok(Self {
            descriptor_pool: pool,
            descriptor_layout: layout,
            descriptor_set: set,
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
        buffer: vk::Buffer,
    ) -> BindlessResourceHandle {
        let binding = self.uniform_buffers.len();

        self.upload_buffer(
            device,
            buffer,
            vk::DescriptorType::UNIFORM_BUFFER,
            binding as u32,
            Self::UNIFORM_BUFFER_BINDING,
        );

        self.uniform_buffers.push(buffer);

        BindlessResourceHandle {
            binding,
            ty: BindlessResourceType::UniformBuffer,
        }
    }

    pub fn set_storage_buffer(
        &mut self,
        device: &VulkanDevice,
        buffer: vk::Buffer,
    ) -> BindlessResourceHandle {
        let binding = self.storage_buffers.len();

        self.upload_buffer(
            device,
            buffer,
            vk::DescriptorType::STORAGE_BUFFER,
            binding as u32,
            Self::STORAGE_BUFFER_BINDING,
        );

        self.storage_buffers.push(buffer);

        BindlessResourceHandle {
            binding,
            ty: BindlessResourceType::StorageBuffer,
        }
    }

    pub fn set_storage_image(
        &mut self,
        device: &VulkanDevice,
        image_view: vk::ImageView,
        image_layout: vk::ImageLayout,
    ) -> BindlessResourceHandle {
        let binding = self.storage_images.len();

        self.upload_image(
            device,
            image_view,
            image_layout,
            vk::Sampler::null(),
            vk::DescriptorType::STORAGE_BUFFER,
            binding as u32,
            Self::STORAGE_BUFFER_BINDING,
        );

        self.storage_images.push(image_view);

        BindlessResourceHandle {
            binding,
            ty: BindlessResourceType::StorageImage,
        }
    }

    pub unsafe fn destroy(&self, device: &VulkanDevice) {
        device.destroy_descriptor_pool(self.descriptor_pool, None);
        device.destroy_descriptor_set_layout(self.descriptor_layout, None);
    }
}
