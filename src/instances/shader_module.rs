use std::sync::Arc;

use crate::instances::Device;
use anyhow::{Context, Result};
use ash::vk;

pub struct ShaderModule {
    intern: vk::ShaderModule,
    kind: ShaderKind,
    entry: &'static str,
    device: Arc<Device>,
}

#[derive(Clone, Copy)]
pub enum ShaderKind {
    Compute,
    Fragment,
    Vertex,
}

impl Into<vk::ShaderStageFlags> for ShaderKind {
    fn into(self) -> vk::ShaderStageFlags {
        match self {
            Self::Compute => vk::ShaderStageFlags::COMPUTE,
            Self::Fragment => vk::ShaderStageFlags::FRAGMENT,
            Self::Vertex => vk::ShaderStageFlags::VERTEX,
        }
    }
}
impl Into<shaderc::ShaderKind> for ShaderKind {
    fn into(self) -> shaderc::ShaderKind {
        match self {
            Self::Compute => shaderc::ShaderKind::Compute,
            Self::Fragment => shaderc::ShaderKind::Fragment,
            Self::Vertex => shaderc::ShaderKind::Vertex,
        }
    }
}

impl ShaderModule {
    pub fn from_source(
        device: Arc<Device>,
        source: &str,
        shader_kind: ShaderKind,
        entry: &'static str,
    ) -> Result<Arc<Self>> {
        let compiler = shaderc::Compiler::new().context("failed to create compiler")?;

        let code =
            compiler.compile_into_spirv(source, shader_kind.into(), "shader", entry, None)?;

        let create_info = vk::ShaderModuleCreateInfo::default().code(code.as_binary());

        let module = unsafe { device.as_raw().create_shader_module(&create_info, None) }?;

        Ok(Arc::new(Self {
            intern: module,
            kind: shader_kind,
            entry,
            device,
        }))
    }

    pub fn as_raw(&self) -> vk::ShaderModule {
        self.intern.clone()
    }
    pub fn entry(&self) -> &str {
        self.entry
    }
    pub fn kind(&self) -> ShaderKind {
        self.kind
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device
                .as_raw()
                .destroy_shader_module(self.intern, None)
        };
    }
}
