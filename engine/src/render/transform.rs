#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: math::Vector3<f32>,
    pub rotation: math::Vector3<f32>,
    pub scale: math::Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: math::Vector3::new(0.0, 0.0, 0.0),
            rotation: math::Vector3::new(0.0, 0.0, 0.0),
            scale: math::Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn from_position(position: math::Vector3<f32>) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    pub fn with_position(mut self, position: math::Vector3<f32>) -> Self {
        self.position = position;
        self
    }

    pub fn with_rotation(mut self, rotation: math::Vector3<f32>) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: math::Vector3<f32>) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_uniform_scale(self, scale: f32) -> Self {
        self.with_scale(math::Vector3::new(scale, scale, scale))
    }

    fn basis(&self) -> [[f32; 3]; 3] {
        let (sp, cp) = self.rotation.x.sin_cos(); // pitch
        let (sy, cy) = self.rotation.y.sin_cos(); // yaw
        let (sr, cr) = self.rotation.z.sin_cos(); // roll

        [
            [cy * cr + sy * sp * sr, cp * sr, -sy * cr + cy * sp * sr],
            [-cy * sr + sy * sp * cr, cp * cr, sy * sr + cy * sp * cr],
            [sy * cp, -sp, cy * cp],
        ]
    }

    pub fn matrix(&self) -> math::Matrix4<f32> {
        let b = self.basis();
        let (sx, sy, sz) = (self.scale.x, self.scale.y, self.scale.z);
        let p = self.position;

        math::Matrix4::from_cols_slice(&[
            b[0][0] * sx,
            b[0][1] * sx,
            b[0][2] * sx,
            0.0,
            b[1][0] * sy,
            b[1][1] * sy,
            b[1][2] * sy,
            0.0,
            b[2][0] * sz,
            b[2][1] * sz,
            b[2][2] * sz,
            0.0,
            p.x,
            p.y,
            p.z,
            1.0,
        ])
    }

    pub fn normal_matrix(&self) -> math::Matrix4<f32> {
        let b = self.basis();
        let (ix, iy, iz) = (1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);

        math::Matrix4::from_cols_slice(&[
            b[0][0] * ix,
            b[0][1] * ix,
            b[0][2] * ix,
            0.0,
            b[1][0] * iy,
            b[1][1] * iy,
            b[1][2] * iy,
            0.0,
            b[2][0] * iz,
            b[2][1] * iz,
            b[2][2] * iz,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }
}
