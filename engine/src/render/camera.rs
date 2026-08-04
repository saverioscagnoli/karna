use std::matches;

use logging::warn;

#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct CameraPacket {
    pub view_projection: math::Matrix4<f32>,
    /// World-space eye position, for view-dependent shading (specular).
    /// `w` is unused padding, kept at 1.0.
    pub position: math::Vector4<f32>,
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
    pub(crate) fn matrix(&self) -> math::Matrix4<f32> {
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

    pub fn is_perspective(&self) -> bool {
        matches!(self, Self::Perspective { .. })
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

// How a camera's extents respond to the window being resized
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub enum Fit {
    #[default]
    Viewport,
    Virtual(math::Size<f32>),
    Manual,
}

#[derive(Debug)]
pub struct Camera {
    projection: Projection,
    fit: Fit,
    zoom: f32,
    position: math::Vector3<f32>,
    target: math::Vector3<f32>,
    up: math::Vector3<f32>,
}

impl Camera {
    pub fn new(projection: Projection) -> Self {
        Self {
            projection,
            fit: Fit::default(),
            zoom: 1.0,
            position: math::Vector3::new(0.0, 0.0, -5.0),
            target: math::Vector3::new(0.0, 0.0, 1.0),
            up: math::Vector3::new(0.0, 1.0, 0.0),
        }
    }

    pub fn orthographic(view: math::Size<u32>) -> Self {
        Self::new(Projection::standard_2d(view))
    }

    pub fn virtual_2d(size: math::Size<f32>) -> Self {
        let mut cam = Self::new(Projection::Orthographic {
            left: 0.0,
            right: size.width,
            bottom: size.height,
            top: 0.0,
            near: -1.0,
            far: 1.0,
        });

        cam.fit = Fit::Virtual(size);
        cam
    }

    pub fn perspective(view: math::Size<u32>, fov: f32, near: f32, far: f32) -> Self {
        Self::new(Projection::standard_3d(view, fov, near, far))
    }

    pub fn fit(&self) -> Fit {
        self.fit
    }

    pub fn set_fit(&mut self, fit: Fit) {
        self.fit = fit;
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn zoom_mut(&mut self) -> &mut f32 {
        &mut self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    pub fn projection(&self) -> &Projection {
        &self.projection
    }

    pub fn projection_mut(&mut self) -> &mut Projection {
        &mut self.projection
    }

    pub fn set_projection(&mut self, proj: Projection) {
        self.projection = proj;
    }

    pub fn is_perspective(&self) -> bool {
        self.projection.is_perspective()
    }

    pub fn position(&self) -> &math::Vector3<f32> {
        &self.position
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

    pub fn target(&self) -> &math::Vector3<f32> {
        &self.target
    }

    pub fn target_mut(&mut self) -> &mut math::Vector3<f32> {
        &mut self.target
    }

    pub fn set_target<T>(&mut self, target: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        self.target = target.into();
    }

    pub fn up(&self) -> &math::Vector3<f32> {
        &self.up
    }

    pub fn up_mut(&mut self) -> &mut math::Vector3<f32> {
        &mut self.up
    }

    pub fn set_up<U>(&mut self, up: U)
    where
        U: Into<math::Vector3<f32>>,
    {
        self.up = up.into();
    }

    pub fn fov(&self) -> Option<f32> {
        match self.projection {
            Projection::Perspective { fov, .. } => Some(fov),
            _ => None,
        }
    }

    pub fn fov_degrees(&self) -> Option<f32> {
        self.fov().map(f32::to_degrees)
    }

    pub fn fov_mut(&mut self) -> Option<&mut f32> {
        match &mut self.projection {
            Projection::Perspective { fov, .. } => Some(fov),
            _ => None,
        }
    }

    pub fn set_fov(&mut self, radians: f32) {
        match &mut self.projection {
            Projection::Perspective { fov, .. } => {
                *fov = radians.clamp(1.0, 120.0);
            }

            _ => warn!("set_fov called on an orthographic camera, ignoring"),
        }
    }

    pub fn set_fov_degrees(&mut self, degrees: f32) {
        self.set_fov(degrees.to_radians());
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
        let zoom = self.zoom;

        match (&mut self.projection, self.fit) {
            (Projection::Orthographic { right, bottom, .. }, Fit::Viewport) => {
                *right = viewport.width / zoom;
                *bottom = viewport.height / zoom;
            }

            (Projection::Orthographic { right, bottom, .. }, Fit::Virtual(size)) => {
                *right = size.width / zoom;
                *bottom = size.height / zoom;
            }

            (Projection::Orthographic { .. }, Fit::Manual) => {}

            (Projection::Perspective { aspect_ratio, .. }, Fit::Manual) => {
                *aspect_ratio = viewport.aspect_ratio();
            }

            (Projection::Perspective { aspect_ratio, .. }, _) => {
                *aspect_ratio = viewport.aspect_ratio();
            }
        }
    }

    pub(crate) fn packet(&self) -> CameraPacket {
        CameraPacket {
            view_projection: self.projection.matrix().matmul(&self.view_matrix()),
            position: math::Vector4::new(self.position.x, self.position.y, self.position.z, 1.0),
        }
    }
}
