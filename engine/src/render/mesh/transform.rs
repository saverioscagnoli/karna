#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Transform {
    pub position: math::Vector3<f32>,
    pub rotation: math::Vector3<f32>,
    pub scale: math::Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: math::Vector3::zero(),
            rotation: math::Vector3::zero(),
            scale: math::Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn at(position: math::Vector3<f32>) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    pub fn matrix(&self) -> math::Matrix4<f32> {
        math::Matrix4::from_translation(self.position)
            .matmul(&self.rotation_matrix())
            .matmul(&math::Matrix4::from_scale(self.scale))
    }

    pub fn rotation_matrix(&self) -> math::Matrix4<f32> {
        math::Matrix4::from_rotation_z(self.rotation.z)
            .matmul(&math::Matrix4::from_rotation_y(self.rotation.y))
            .matmul(&math::Matrix4::from_axis_angle(
                &math::Vector3::new(1.0, 0.0, 0.0),
                self.rotation.x,
            ))
    }
}
