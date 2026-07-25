#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct CameraPacket {
    pub view_projection: math::Matrix4<f32>,
}

#[derive(Debug)]
pub enum Projection {
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
    Perspective {
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    },
}

impl Projection {
    fn matrix(&self) -> math::Matrix4<f32> {
        match self {
            &Self::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => math::Matrix4::orthographic(left, right, bottom, top, near, far),
            &Self::Perspective {
                fov,
                aspect_ratio,
                near,
                far,
            } => math::Matrix4::perspective(fov, aspect_ratio, near, far),
        }
    }

    /// Returns an orthographic projection typically used
    /// in 2d games, where the top left point of the window is (0, 0)
    /// and the bottom left point is (win.width, win.height)
    pub fn standard_2d(view: math::Size<u32>) -> Self {
        Self::Orthographic {
            left: 0.0,
            right: view.width as f32,
            bottom: view.height as f32,
            top: 0.0,
            near: -1.0,
            far: 1.0,
        }
    }

    /// Returns a perspective projection typically used in 3d games.
    ///
    /// FOV (Field of View) must be in degrees.
    pub fn standard_3d(view: math::Size<u32>, fov: f32, near: f32, far: f32) -> Self {
        Self::Perspective {
            fov: fov.to_radians(),
            aspect_ratio: view.as_f32().aspect_ratio(),
            near,
            far,
        }
    }
}

#[derive(Debug)]
pub struct Camera {
    projection: Projection,
    pub position: math::Vector3<f32>,
    pub target: math::Vector3<f32>,
    pub up: math::Vector3<f32>,
}

impl Camera {
    pub fn new(projection: Projection) -> Self {
        Self {
            projection,
            position: math::Vector3::new(0.0, 0.0, -5.0),
            target: math::Vector3::new(0.0, 0.0, 1.0),
            up: math::Vector3::new(0.0, 1.0, 0.0),
        }
    }

    fn view_matrix(&self) -> math::Matrix4<f32> {
        match self.projection {
            Projection::Orthographic { .. } => math::Matrix4::from_translation(math::Vector3::new(
                -self.position.x,
                -self.position.y,
                0.0,
            )),

            Projection::Perspective { .. } => {
                math::Matrix4::look_at(&self.position, &self.target, &self.up)
            }
        }
    }

    pub(crate) fn update(&mut self, viewport: math::Size<u32>) {
        let viewport = viewport.as_f32();

        match &mut self.projection {
            Projection::Orthographic { right, bottom, .. } => {
                *right = viewport.width;
                *bottom = viewport.height;
            }

            Projection::Perspective { aspect_ratio, .. } => {
                *aspect_ratio = viewport.aspect_ratio();
            }
        }
    }

    pub(crate) fn packet(&self) -> CameraPacket {
        CameraPacket {
            view_projection: self.projection.matrix().matmul(&self.view_matrix()),
        }
    }
}
