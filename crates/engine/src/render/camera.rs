use math as m;

#[derive(Debug, Clone, Copy)]
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
    pub fn matrix(&self) -> m::Matrix4<f32> {
        match self {
            &Self::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => m::Matrix4::orthographic(left, right, bottom, top, near, far),
            &Self::Perspective {
                fov,
                aspect_ratio,
                near,
                far,
            } => m::Matrix4::perspective(fov, aspect_ratio, near, far),
        }
    }

    pub fn topleft_2d(view: m::Size<u32>) -> Self {
        Self::Orthographic {
            left: 0.0,
            right: view.width as f32,
            bottom: view.height as f32,
            top: 0.0,
            near: -1.0,
            far: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    projection: Projection,
    position: m::Vector3<f32>,
    target: m::Vector3<f32>,
    up: m::Vector3<f32>,
}

impl Camera {
    pub fn new(projection: Projection) -> Self {
        Self {
            projection,
            position: m::Vector3::new(0.0, 0.0, -5.0),
            target: m::Vector3::new(0.0, 0.0, 1.0),
            up: m::Vector3::new(0.0, 1.0, 0.0),
        }
    }

    pub fn projection(&self) -> Projection {
        self.projection
    }

    pub fn projection_mut(&mut self) -> &mut Projection {
        &mut self.projection
    }

    pub fn view_matrix(&self) -> m::Matrix4<f32> {
        match self.projection {
            Projection::Orthographic { .. } => m::Matrix4::from_translation(m::Vector3::new(
                -self.position.x,
                -self.position.y,
                0.0,
            )),
            Projection::Perspective { .. } => {
                m::Matrix4::look_at(&self.position, &self.target, &self.up)
            }
        }
    }

    pub fn mvp(&self) -> m::Matrix4<f32> {
        self.projection().matrix().matmul(&self.view_matrix())
    }

    pub(crate) fn update(&mut self, viewport: m::Size<u32>) {
        let viewport = viewport.cast::<f32>();

        match &mut self.projection {
            Projection::Orthographic { right, bottom, .. } => {
                *right = viewport.width;
                *bottom = viewport.height
            }
            Projection::Perspective { aspect_ratio, .. } => *aspect_ratio = viewport.aspect_ratio(),
        }
    }
}
