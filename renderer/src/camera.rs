use gpu::GpuState;
use math::Matrix4;
use math::Size;
use math::Vector3;

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
    fn matrix(&self) -> Matrix4<f32> {
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
            aspect_ratio: view.as_f32().aspect_ratio(),
            near,
            far,
        }
    }
}

pub struct Camera {
    uniform_buffer: gpu::Buffer<Matrix4<f32>>,

    pub(crate) bgl: wgpu::BindGroupLayout,
    pub(crate) bg: wgpu::BindGroup,

    pub projection: Projection,
    pub position: Vector3<f32>,
    pub target: Vector3<f32>,
    pub up: Vector3<f32>,
}

impl Camera {
    pub(crate) fn new(proj: Projection) -> Self {
        let gpu = GpuState::get();
        let uniform_buffer = gpu::Buffer::new_with_capacity(
            "camera uniform buffer",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            1,
        );

        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera projection buffer bind group layout"),
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

        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera projection buffer bind group"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: uniform_buffer.wgpu(),
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Self {
            uniform_buffer,
            bg,
            bgl,
            projection: proj,
            position: Vector3::new(0.0, 0.0, -5.0),
            target: Vector3::new(0.0, 0.0, 1.0),
            up: Vector3::new(0.0, 1.0, 0.0),
        }
    }

    fn view_matrix(&self) -> Matrix4<f32> {
        match self.projection {
            Projection::Orthographic { .. } => {
                Matrix4::from_translation(Vector3::new(-(self.position.x), -(self.position.y), 0.0))
            }
            Projection::Perspective { .. } => {
                Matrix4::look_at(&self.position, &self.target, &self.up)
            }
        }
    }

    pub(crate) fn update(&mut self, view: Size<u32>) {
        match &mut self.projection {
            Projection::Orthographic { right, bottom, .. } => {
                *right = view.width as f32;
                *bottom = view.height as f32;
            }

            Projection::Perspective { aspect_ratio, .. } => {
                *aspect_ratio = view.as_f32().aspect_ratio();
            }
        }

        let vp = self.projection.matrix().matmul(&self.view_matrix());

        self.uniform_buffer.write(0, &[vp]);
    }
}
