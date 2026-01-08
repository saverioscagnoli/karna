use gpu::core::{GpuBuffer, GpuBufferBuilder};
use logging::warn;
use macros::{Get, Set, track_dirty};
use math::{Matrix4, Size, Vector3};

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
    fn matrix(&self) -> Matrix4 {
        match self {
            &Self::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => Matrix4::orthographic(left, right, bottom, top, near, far),
            &Self::Perspective {
                fov,
                aspect_ratio,
                near,
                far,
            } => Matrix4::perspective(fov, aspect_ratio, near, far),
        }
    }

    /// Returns an orthographic projection typically used
    /// in 2d games, where the top left point of the window is (0, 0)
    /// and the bottom left point is (win.width, win.height)
    pub fn standard_2d(view: Size<u32>) -> Self {
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
    pub fn standard_3d(view: Size<u32>, fov: f32, near: f32, far: f32) -> Self {
        Self::Perspective {
            fov: fov.to_radians(),
            aspect_ratio: view.to_f32().aspect_ratio(),
            near,
            far,
        }
    }
}

#[track_dirty(u16)]
#[derive(Debug)]
#[derive(Get, Set)]
pub struct Camera {
    // WGPU
    uniform_buffer: GpuBuffer<Matrix4>,

    #[get(visibility = "pub(crate)")]
    bgl: wgpu::BindGroupLayout,

    #[get(visibility = "pub(crate)")]
    bg: wgpu::BindGroup,

    // Maths
    projection: Projection,

    #[get]
    #[get(copied, prop = "x", ty = f32)]
    #[get(copied, prop = "y", ty = f32)]
    #[get(copied, prop = "z", ty = f32)]
    #[get(mut, also = self.tracker |= Self::position_f())]
    #[get(mut, prop = "x", ty = &mut f32, also = self.tracker |= Self::position_f())]
    #[get(mut, prop = "y", ty = &mut f32, also = self.tracker |= Self::position_f())]
    #[get(mut, prop = "z", ty = &mut f32, also = self.tracker |= Self::position_f())]
    #[set(into, also = self.tracker |= Self::position_f())]
    #[set(prop = "x", ty = f32, also = self.tracker |= Self::position_f())]
    #[set(prop = "y", ty = f32, also = self.tracker |= Self::position_f())]
    #[set(prop = "z", ty = f32, also = self.tracker |= Self::position_f())]
    position: Vector3,

    #[get]
    #[get(copied, prop = "x", ty = f32)]
    #[get(copied, prop = "y", ty = f32)]
    #[get(copied, prop = "z", ty = f32)]
    #[get(mut, also = self.tracker |= Self::target_f())]
    #[get(mut, prop = "x", ty = &mut f32, also = self.tracker |= Self::target_f())]
    #[get(mut, prop = "y", ty = &mut f32, also = self.tracker |= Self::target_f())]
    #[get(mut, prop = "z", ty = &mut f32, also = self.tracker |= Self::target_f())]
    #[set(name = "look_at", also = self.tracker |= Self::target_f())]
    #[set(prop = "x", ty = f32, name = "look_at_x", also = self.tracker |= Self::target_f())]
    #[set(prop = "y", ty = f32, name = "look_at_y", also = self.tracker |= Self::target_f())]
    #[set(prop = "z", ty = f32, name = "look_at_z", also = self.tracker |= Self::target_f())]
    target: Vector3,

    #[get]
    #[get(copied, prop = "x", ty = f32)]
    #[get(copied, prop = "y", ty = f32)]
    #[get(copied, prop = "z", ty = f32)]
    #[get(mut, also = self.tracker |= Self::up_f())]
    #[get(mut, prop = "x", ty = &mut f32, also = self.tracker |= Self::up_f())]
    #[get(mut, prop = "y", ty = &mut f32, also = self.tracker |= Self::up_f())]
    #[get(mut, prop = "z", ty = &mut f32, also = self.tracker |= Self::up_f())]
    up: Vector3,
}

impl Camera {
    pub(crate) fn new(projection: Projection) -> Self {
        let device = gpu::device();
        let uniform_buffer = GpuBufferBuilder::new()
            .label("Camera Projection Buffer")
            .uniform()
            .copy_dst()
            .build();

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Projection Buffer Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Projection Buffer Bind Group"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: uniform_buffer.inner(),
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Self {
            uniform_buffer,
            bg,
            bgl,
            projection,
            position: Vector3::new(0.0, 0.0, -5.0),
            target: Vector3::z(),
            up: Vector3::y(),
            tracker: 0,
        }
    }

    #[inline]
    fn view_matrix(&self) -> Matrix4 {
        match self.projection {
            Projection::Orthographic { .. } => {
                Matrix4::from_translation(Vector3::new(-(self.position.x), -(self.position.y), 0.0))
            }
            Projection::Perspective { .. } => Matrix4::look_at(self.position, self.target, self.up),
        }
    }

    #[inline]
    pub(crate) fn queue_resize(&mut self) {
        self.tracker |= Self::projection_f()
    }

    #[inline]
    pub(crate) fn update(&mut self, view: Size<u32>) {
        if !self.any_dirty() {
            return;
        }

        match &mut self.projection {
            Projection::Orthographic { right, bottom, .. } => {
                *right = view.width as f32;
                *bottom = view.height as f32;
            }
            Projection::Perspective { aspect_ratio, .. } => {
                *aspect_ratio = view.to_f32().aspect_ratio();
            }
        }

        let vp = self.projection.matrix() * self.view_matrix();

        self.uniform_buffer.write(0, &[vp]);
        self.clear_all_dirty();
    }

    #[inline]
    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
        self.tracker |= Self::projection_f();
    }

    /// Current camera FOV (field of view) in degrees
    ///
    /// If the camera is using an orthographic projection, it will just return 0.
    #[inline]
    pub fn fov(&self) -> f32 {
        if let Projection::Perspective { fov, .. } = self.projection {
            return fov.to_degrees();
        }

        warn!("Trying to get the fov, but the camera is using an orthographic projection!");

        0.0
    }

    /// Current camera FOV (field of view) in radians
    ///
    /// If the camera is using an orthographic projection, it will just return 0.
    #[inline]
    pub fn fov_rad(&self) -> f32 {
        if let Projection::Perspective { fov, .. } = self.projection {
            return fov;
        }

        warn!("Trying to get the fov, but the camera is using an orthographic projection!");

        0.0
    }

    /// Changes the FOV (field of view) of the camera.
    /// The value must be in degrees.
    ///
    /// If the camera is using an orthographic projection, it won't do anything.
    #[inline]
    pub fn set_fov(&mut self, fov: f32) {
        if let Projection::Perspective {
            fov: current_fov, ..
        } = &mut self.projection
        {
            *current_fov = fov.to_radians();
            self.tracker |= Self::projection_f();
            return;
        }

        warn!("Trying to update the fov, but the camera uses an orthographic projection!");
    }

    /// Changes the FOV (field of view) of the camera.
    /// The value must be in radians.
    ///
    /// If the camera is using an orthographic projection, it won't do anything.
    #[inline]
    pub fn set_fov_rad(&mut self, fov: f32) {
        if let Projection::Perspective {
            fov: current_fov, ..
        } = &mut self.projection
        {
            *current_fov = fov;
            self.tracker |= Self::projection_f();
            return;
        }

        warn!("Trying to update the fov, but the camera uses an orthographic projection!");
    }
}
