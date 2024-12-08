use crate::application::transform::Transform;
use glam::{vec4, Mat4, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct CameraUniformData {
    view_proj: Mat4,
    inverse_view_proj: Mat4,
    pos: Vec4,
}

#[derive(Debug, Default)]
pub struct Camera {
    transform: Transform,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn build_proj(&self) -> CameraUniformData {
        let view = Mat4::look_at_rh(
            self.transform.translation,
            self.transform.forward(),
            self.transform.down(),
        );

        let proj =
            Mat4::perspective_rh_gl(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);

        let pos = self.transform.translation;

        let view_proj = proj * view;

        CameraUniformData {
            view_proj,
            inverse_view_proj: view_proj.inverse(),
            pos: vec4(pos.x, pos.y, pos.z, 1.0),
        }
    }
}
