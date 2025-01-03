use ash::vk;

pub trait Material {
    fn shaders(&self) -> (&[vk::ShaderEXT], &[vk::ShaderStageFlags]);

    fn set_rasterizer_discard_enable(&self) -> bool {
        false
    }
    fn set_polygon_mode(&self) -> vk::PolygonMode {
        vk::PolygonMode::FILL
    }
    fn set_rasterization_samples(&self) -> vk::SampleCountFlags {
        vk::SampleCountFlags::TYPE_1
    }
    fn set_sample_mask(&self) -> (vk::SampleCountFlags, &[u32]) {
        (vk::SampleCountFlags::TYPE_1, &[1])
    }
    fn set_alpha_to_coverage_enable(&self) -> bool {
        false
    }
    fn set_cull_mode(&self) -> vk::CullModeFlags {
        vk::CullModeFlags::NONE
    }
    fn set_depth_test_enable(&self) -> bool {
        false
    }
    fn set_depth_write_enable(&self) -> bool {
        false
    }
    fn set_depth_bias_enable(&self) -> bool {
        false
    }
    fn set_stencil_test_enable(&self) -> bool {
        false
    }
    fn set_primitive_topology(&self) -> vk::PrimitiveTopology {
        vk::PrimitiveTopology::TRIANGLE_LIST
    }
    fn set_primitive_restart_enable(&self) -> bool {
        false
    }
    fn set_color_blend_enable(&self) -> (u32, &[u32]) {
        (0, &[0, 0])
    }
    fn set_front_face(&self) -> vk::FrontFace {
        vk::FrontFace::COUNTER_CLOCKWISE
    }

    fn set_color_blend_equation(&self) -> (u32, &[vk::ColorBlendEquationEXT]) {
        (
            0,
            &[vk::ColorBlendEquationEXT {
                src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ONE,
                dst_alpha_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
                alpha_blend_op: vk::BlendOp::ADD,
            }],
        )
    }
    fn set_color_write_mask(&self) -> (u32, &[vk::ColorComponentFlags]) {
        (
            0,
            &[vk::ColorComponentFlags::RGBA, vk::ColorComponentFlags::R],
        )
    }

    /// # Safety
    unsafe fn setup_material(
        &self,
        s_device: &ash::ext::shader_object::Device,
        cmd: vk::CommandBuffer,
    ) {
        s_device.cmd_set_rasterizer_discard_enable(cmd, self.set_rasterizer_discard_enable());
        s_device.cmd_set_polygon_mode(cmd, self.set_polygon_mode());
        s_device.cmd_set_rasterization_samples(cmd, self.set_rasterization_samples());
        s_device.cmd_set_alpha_to_coverage_enable(cmd, self.set_alpha_to_coverage_enable());
        s_device.cmd_set_front_face(cmd, self.set_front_face());
        s_device.cmd_set_cull_mode(cmd, self.set_cull_mode());
        s_device.cmd_set_depth_test_enable(cmd, self.set_depth_test_enable());
        s_device.cmd_set_depth_write_enable(cmd, self.set_depth_write_enable());
        s_device.cmd_set_depth_bias_enable(cmd, self.set_depth_bias_enable());
        s_device.cmd_set_stencil_test_enable(cmd, self.set_stencil_test_enable());
        s_device.cmd_set_primitive_topology(cmd, self.set_primitive_topology());
        s_device.cmd_set_primitive_restart_enable(cmd, self.set_primitive_restart_enable());

        let shaders = self.shaders();
        s_device.cmd_bind_shaders(cmd, shaders.1, shaders.0);

        let sample_mask = self.set_sample_mask();
        s_device.cmd_set_sample_mask(cmd, sample_mask.0, sample_mask.1);

        let color_blend = self.set_color_blend_enable();
        s_device.cmd_set_color_blend_enable(cmd, color_blend.0, color_blend.1);

        let blend_equation = self.set_color_blend_equation();
        s_device.cmd_set_color_blend_equation(cmd, blend_equation.0, blend_equation.1);

        let write_mask = self.set_color_write_mask();
        s_device.cmd_set_color_write_mask(cmd, write_mask.0, write_mask.1);
    }
}

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
}
