use ash::vk;

#[allow(unused)]
pub enum CullMode {
    Front,
    Back,
    None,
}

impl Into<vk::PipelineRasterizationStateCreateInfo<'static>> for CullMode {
    fn into(self) -> vk::PipelineRasterizationStateCreateInfo<'static> {
        vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: match self {
                Self::Front => vk::CullModeFlags::FRONT,
                Self::Back => vk::CullModeFlags::BACK,
                Self::None => vk::CullModeFlags::NONE,
            },
            ..Default::default()
        }
    }
}

pub enum VertexInputBinding {
    Vertex { size: u32, binding: u32 },
    Insanced { size: u32, binding: u32 },
}

#[derive(Clone, Copy, Debug)]
#[allow(unused)]
pub enum ViewportMode {
    // viewpoer is relative to the window size 
    // 1.0 would fill the whole wndow
    // order : posx, posy, scalex, scaley
    Relative(f32, f32, f32, f32),

    // size is constant and needs to be changed manually
    Constant(u32, u32, u32, u32),
}

impl ViewportMode {
    pub fn get_size(&self, window_extent: vk::Extent2D) -> vk::Viewport {
        match self {
            Self::Relative(x, y, width, height) => vk::Viewport {
                x: x * window_extent.width as f32,
                y: y * window_extent.height as f32,
                width: width * window_extent.width as f32,
                height: height * window_extent.height as f32,
                max_depth: 1.0,
                min_depth: 0.0,
            },
            Self::Constant(x, y, width, height) => vk::Viewport {
                x: *x as f32,
                y: *y as f32,
                width: *width as f32,
                height: *height as f32,
                max_depth: 1.0,
                min_depth: 0.0,
            },
        }
    }
}

impl VertexInputBinding {
    pub fn instance<T>(binding: u32) -> Self {
        Self::Insanced {
            size: std::mem::size_of::<T>() as u32,
            binding,
        }
    }
    pub fn vertex<T>(binding: u32) -> Self {
        Self::Vertex {
            size: std::mem::size_of::<T>() as u32,
            binding,
        }
    }
}

impl Into<vk::VertexInputBindingDescription> for VertexInputBinding {
    fn into(self) -> vk::VertexInputBindingDescription {
        match self {
            Self::Vertex { size, binding } => vk::VertexInputBindingDescription {
                binding,
                stride: size,
                input_rate: vk::VertexInputRate::VERTEX,
            },
            Self::Insanced { size, binding } => vk::VertexInputBindingDescription {
                binding,
                stride: size,
                input_rate: vk::VertexInputRate::INSTANCE,
            },
        }
    }
}
