use math::{Mat4, Transform};

#[derive(Debug, Clone)]
pub struct Camera {
    pub transform: Transform,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    #[must_use]
    pub fn build_proj(&self) -> Mat4 {
        let view = Mat4::look_at_rh(
            self.transform.translation,
            self.transform.forward(),
            self.transform.down(),
        );

        let mut proj =
            Mat4::perspective_rh_gl(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);

        proj.x_axis.x *= -1.0;
        proj * view
    }
}
