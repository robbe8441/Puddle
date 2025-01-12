use std::sync::Arc;

use ash::{prelude::VkResult, vk};

use crate::vulkan::{Buffer, VulkanDevice};

#[derive(Debug, Clone, Copy)]
pub struct BindlessResourceHandle {
    pub index: usize,
    pub ty: BindlessResourceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindlessResourceType {
    UniformBuffer,
    StorageBuffer,
    StorageImage,
}

impl BindlessResourceType {
    pub fn binding(self) -> u32 {
        match self {
            Self::UniformBuffer => BindlessHandler::UNIFORM_BUFFER_BINDING,
            Self::StorageBuffer => BindlessHandler::STORAGE_BUFFER_BINDING,
            Self::StorageImage => BindlessHandler::STORAGE_IMAGE_BINDING,
        }
    }

    pub fn desc_type(self) -> vk::DescriptorType {
        match self {
            Self::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
            Self::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
            Self::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
        }
    }
}

#[allow(unused)]
enum UpdateResourceTask {
    UpdateBuffer(Arc<Buffer>),
    UpdateImageView(vk::ImageView),
}

/// basically just an Option but with 3 states
pub enum ResourceSlot<T> {
    /// the resource is free to use
    Empty,
    /// an resource has already been submitted to be written here
    Submited,
    /// the resource is ready to be used
    Written(T),
}

impl<T> ResourceSlot<T> {
    pub fn take(&mut self) -> ResourceSlot<T> {
        match self {
            Self::Written(_) => std::mem::replace(self, Self::Empty),
            Self::Submited => Self::Submited,
            Self::Empty => Self::Empty,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn expect(self, err: &str) -> T {
        match self {
            Self::Written(v) => v,
            _ => panic!("{err}"),
        }
    }
}

#[allow(unused)]
pub struct BindlessHandler {
    descriptor_pool: vk::DescriptorPool,
    pub descriptor_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub descriptor_sets: [vk::DescriptorSet; super::FLYING_FRAMES],
    pub uniform_buffers: [ResourceSlot<Arc<Buffer>>; Self::POOL_SIZE],
    pub storage_buffers: [ResourceSlot<Arc<Buffer>>; Self::POOL_SIZE],
    pub storage_images: [ResourceSlot<vk::ImageView>; Self::POOL_SIZE],
    update_resource_queue: Vec<(usize, BindlessResourceHandle, UpdateResourceTask)>,
}

impl BindlessHandler {
    pub const UNIFORM_BUFFER_BINDING: u32 = 0;
    pub const STORAGE_BUFFER_BINDING: u32 = 1;
    pub const STORAGE_IMAGE_BINDING: u32 = 2;

    pub const POOL_SIZE: usize = 100;

    pub fn new(device: &VulkanDevice) -> VkResult<Self> {
        let descriptor_count = (Self::POOL_SIZE * super::FLYING_FRAMES) as u32;
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count,
            },
        ];

        let pool_create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(super::FLYING_FRAMES as u32);

        let pool = unsafe { device.create_descriptor_pool(&pool_create_info, None)? };

        let bindings: Vec<_> = pool_sizes
            .iter()
            .enumerate()
            .map(|(i, v)| {
                vk::DescriptorSetLayoutBinding::default()
                    .binding(i as u32)
                    .descriptor_type(v.ty)
                    .descriptor_count(Self::POOL_SIZE as u32)
                    .stage_flags(vk::ShaderStageFlags::ALL)
            })
            .collect();

        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

        let layout = unsafe { device.create_descriptor_set_layout(&layout_create_info, None)? };

        let layouts = [layout; super::FLYING_FRAMES];
        let set_allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);

        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&set_allocate_info)? }
            .try_into()
            .unwrap();

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
        // TODO: .push_constant_ranges(push_constant_ranges);

        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }?;

        Ok(Self {
            descriptor_pool: pool,
            descriptor_layout: layout,
            descriptor_sets,
            pipeline_layout,
            uniform_buffers: [const { ResourceSlot::Empty }; Self::POOL_SIZE],
            storage_images: [const { ResourceSlot::Empty }; Self::POOL_SIZE],
            storage_buffers: [const { ResourceSlot::Empty }; Self::POOL_SIZE],
            update_resource_queue: vec![],
        })
    }

    pub fn update_descriptor_set(&mut self, device: &VulkanDevice, frame_index: usize) {
        let mut i = 0;
        while i < self.update_resource_queue.len() {
            let (_, handle, resource) = &self.update_resource_queue[i];

            match resource {
                UpdateResourceTask::UpdateBuffer(b) => {
                    self.upload_buffer_intern(
                        device,
                        b.handle(),
                        handle.ty.desc_type(),
                        handle.ty.binding(),
                        handle.index as u32,
                        frame_index,
                    );
                }
                UpdateResourceTask::UpdateImageView(_) => unimplemented!(),
            }

            if self.update_resource_queue[i].0 == frame_index {
                let (_, handle, resource) = self.update_resource_queue.swap_remove(i);
                match resource {
                    UpdateResourceTask::UpdateBuffer(b) => {
                        if handle.ty == BindlessResourceType::UniformBuffer {
                            self.uniform_buffers[handle.index] = ResourceSlot::Written(b);
                        } else if handle.ty == BindlessResourceType::StorageBuffer {
                            self.storage_buffers[handle.index] = ResourceSlot::Written(b);
                        }
                    }
                    UpdateResourceTask::UpdateImageView(_) => unimplemented!(),
                }
            } else {
                i += 1;
            }
        }
    }

    pub fn upload_buffer(
        &mut self,
        buffer: Arc<Buffer>,
        handle: BindlessResourceHandle,
        set_index: usize,
    ) {
        self.update_resource_queue.push((
            set_index,
            handle,
            UpdateResourceTask::UpdateBuffer(buffer),
        ));
    }

    fn upload_buffer_intern(
        &self,
        device: &VulkanDevice,
        buffer: vk::Buffer,
        ty: vk::DescriptorType,
        binding: u32,
        arr_index: u32,
        set_index: usize,
    ) {
        let buffer_info = [vk::DescriptorBufferInfo::default()
            .buffer(buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE)];

        let write_set = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_sets[set_index])
            .dst_binding(binding)
            .descriptor_type(ty)
            .dst_array_element(arr_index)
            .buffer_info(&buffer_info)
            .descriptor_count(1);

        unsafe { device.update_descriptor_sets(&[write_set], &[]) };
    }

    #[allow(unused)]
    #[allow(clippy::too_many_arguments)]
    fn upload_image_intern(
        &self,
        device: &VulkanDevice,
        image_view: vk::ImageView,
        image_layout: vk::ImageLayout,
        sampler: vk::Sampler,
        ty: vk::DescriptorType,
        binding: u32,
        arr_index: u32,
        set_index: usize,
    ) {
        let image_info = [vk::DescriptorImageInfo::default()
            .image_view(image_view)
            .sampler(sampler)
            .image_layout(image_layout)];

        let write_set = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_sets[set_index])
            .dst_binding(binding)
            .descriptor_type(ty)
            .dst_array_element(arr_index)
            .image_info(&image_info)
            .descriptor_count(1);

        unsafe { device.update_descriptor_sets(&[write_set], &[]) };
    }

    pub unsafe fn destroy(&self, device: &VulkanDevice) {
        device.destroy_descriptor_pool(self.descriptor_pool, None);
        device.destroy_descriptor_set_layout(self.descriptor_layout, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
    }
}

/// gets the first value that is None in the array
/// used find a free slot in the bindless array
pub fn get_free_slot<T>(input: &[ResourceSlot<T>]) -> Option<usize> {
    input.iter().position(ResourceSlot::is_empty)
}
