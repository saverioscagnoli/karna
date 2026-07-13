#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: math::Vector3<f32>,
    /// Euler angles in radians, applied yaw (Y), then pitch (X), then roll (Z).
    pub rotation: math::Vector3<f32>,
    pub scale: math::Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: math::Vector3::zero(),
            rotation: math::Vector3::zero(),
            scale: math::Vector3::one(),
        }
    }
}

impl Transform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_position<P>(position: P) -> Self
    where
        P: Into<math::Vector3<f32>>,
    {
        Self {
            position: position.into(),
            ..Self::default()
        }
    }

    pub fn position(&self) -> math::Vector3<f32> {
        self.position
    }

    pub fn position_mut(&mut self) -> &mut math::Vector3<f32> {
        &mut self.position
    }

    pub fn set_position<P>(&mut self, position: P)
    where
        P: Into<math::Vector3<f32>>,
    {
        self.position = position.into();
    }

    pub fn rotation(&self) -> math::Vector3<f32> {
        self.rotation
    }

    pub fn rotation_mut(&mut self) -> &mut math::Vector3<f32> {
        &mut self.rotation
    }

    pub fn set_rotation<R>(&mut self, rotation: R)
    where
        R: Into<math::Vector3<f32>>,
    {
        self.rotation = rotation.into();
    }

    pub fn scale(&self) -> math::Vector3<f32> {
        self.scale
    }

    pub fn scale_mut(&mut self) -> &mut math::Vector3<f32> {
        &mut self.scale
    }

    pub fn set_scale<S>(&mut self, scale: S)
    where
        S: Into<math::Vector3<f32>>,
    {
        self.scale = scale.into();
    }

    pub fn matrix(&self) -> math::Matrix4<f32> {
        let t = math::Matrix4::from_translation(self.position);
        let ry = math::Matrix4::from_rotation_y(self.rotation.y);
        let rx =
            math::Matrix4::from_axis_angle(&math::Vector3::new(1.0, 0.0, 0.0), self.rotation.x);
        let rz = math::Matrix4::from_rotation_z(self.rotation.z);
        let s = math::Matrix4::from_scale(self.scale);

        t.matmul(&ry).matmul(&rx).matmul(&rz).matmul(&s)
    }
}
