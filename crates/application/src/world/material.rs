use ash::vk;
use rendering::types::Material;

pub struct DefaultMaterial {
    pub shaders: [vk::ShaderEXT; 2],
}

impl Material for DefaultMaterial {
    fn shaders(&self) -> (&[vk::ShaderEXT], &[vk::ShaderStageFlags]) {
        (
            &self.shaders,
            &[vk::ShaderStageFlags::VERTEX, vk::ShaderStageFlags::FRAGMENT],
        )
    }

    fn set_cull_mode(&self) -> vk::CullModeFlags {
        vk::CullModeFlags::BACK
    }
}
