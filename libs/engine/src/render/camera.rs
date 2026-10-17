use math::Size;

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
    pub fn matrix(&self) -> math::Matrix4<f32> {
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

    pub fn topleft_ortho(view: math::Size<u32>) -> Self {
        let view = view.cast::<f32>();

        Self::Orthographic {
            left: 0.0,
            right: view.width,
            bottom: view.height,
            top: 0.0,
            near: -1.0,
            far: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    projection: Projection,
    position: math::Vector3<f32>,
    target: math::Vector3<f32>,
    up: math::Vector3<f32>,
    viewport: math::Size<f32>,
}

impl Camera {
    pub fn new(projection: Projection) -> Self {
        Self {
            projection,
            position: math::vec3!(0.0, 0.0, -5.0),
            target: math::vec3!(0.0, 0.0, 1.0),
            up: math::vec3!(0.0, 1.0, 0.0),
            viewport: Size::default(),
        }
    }

    pub fn projection(&self) -> Projection {
        self.projection
    }

    pub fn projection_mut(&mut self) -> &mut Projection {
        &mut self.projection
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

    pub fn translate<D>(&mut self, delta: D)
    where
        D: Into<math::Vector3<f32>>,
    {
        self.position = self.position + delta.into();
    }

    pub fn target(&self) -> math::Vector3<f32> {
        self.target
    }

    pub fn set_target<T>(&mut self, target: T)
    where
        T: Into<math::Vector3<f32>>,
    {
        self.target = target.into();
    }

    pub fn world_to_screen<P>(&self, world: P) -> Option<math::Vector2<f32>>
    where
        P: Into<math::Vector3<f32>>,
    {
        let p = world.into();
        let v = self.mvp().transform4(math::vec4!(p.x, p.y, p.z, 1.0));

        if v.w <= 0.0 {
            return None; // behind the camera
        }

        let (ndc_x, ndc_y) = (v.x / v.w, v.y / v.w);

        Some(math::vec2!(
            (ndc_x + 1.0) * 0.5 * self.viewport.width,
            (1.0 - ndc_y) * 0.5 * self.viewport.height,
        ))
    }

    fn unproject(&self, screen: math::Vector2<f32>, ndc_z: f32) -> Option<math::Vector3<f32>> {
        let ndc_x = screen.x / self.viewport.width * 2.0 - 1.0;
        let ndc_y = 1.0 - screen.y / self.viewport.height * 2.0;

        let v = self
            .mvp()
            .inverse()?
            .transform4(math::vec4!(ndc_x, ndc_y, ndc_z, 1.0));

        Some(math::vec3!(v.x / v.w, v.y / v.w, v.z / v.w))
    }

    pub fn screen_to_world<S>(&self, screen: S) -> Option<math::Vector3<f32>>
    where
        S: Into<math::Vector2<f32>>,
    {
        self.unproject(screen.into(), 0.0)
    }

    pub fn screen_to_ray<S>(&self, screen: S) -> Option<(math::Vector3<f32>, math::Vector3<f32>)>
    where
        S: Into<math::Vector2<f32>>,
    {
        let s = screen.into();
        let near = self.unproject(s, 0.0)?; // SDL_GPU: near = 0
        let far = self.unproject(s, 1.0)?; //          far  = 1
        Some((near, (far - near).normalize()))
    }

    pub fn view_matrix(&self) -> math::Matrix4<f32> {
        match self.projection {
            Projection::Orthographic { .. } => math::Matrix4::from_translation(math::vec3!(
                -self.position.x,
                -self.position.y,
                0.0
            )),
            Projection::Perspective { .. } => {
                math::Matrix4::look_at(&self.position, &self.target, &self.up)
            }
        }
    }

    pub fn mvp(&self) -> math::Matrix4<f32> {
        self.projection.matrix().matmul(&self.view_matrix())
    }

    pub(crate) fn update(&mut self, viewport: math::Size<u32>) {
        let viewport = viewport.cast::<f32>();

        self.viewport = viewport;

        match self.projection_mut() {
            Projection::Orthographic { right, bottom, .. } => {
                *right = viewport.width;
                *bottom = viewport.height;
            }
            Projection::Perspective { aspect_ratio, .. } => *aspect_ratio = viewport.aspect_ratio(),
        }
    }
}
