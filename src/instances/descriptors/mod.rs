use ash::vk;
pub mod descriptor_pool;


#[allow(unused)]
pub enum DescriptorType {
    UniformBuffer,
    StorageBuffer,
    SampledImage,
    StorageTexelBuffer,
    UniformTexelBuffer,
}




impl Into<vk::DescriptorType> for DescriptorType {
    fn into(self) -> vk::DescriptorType {
        match self {
            Self::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
            Self::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
            Self::SampledImage => vk::DescriptorType::SAMPLED_IMAGE,
            Self::StorageTexelBuffer => vk::DescriptorType::STORAGE_TEXEL_BUFFER,
            Self::UniformTexelBuffer => vk::DescriptorType::UNIFORM_TEXEL_BUFFER,
        }
    }
}







