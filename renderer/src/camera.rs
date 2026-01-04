use gpu::core::{GpuBuffer, GpuBufferBuilder};
use macros::{Get, track_dirty};
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
    fn matrix(&self, view: Size<u32>) -> Matrix4 {
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
}

#[track_dirty]
#[derive(Debug)]
#[derive(Get)]
pub struct Camera {
    // WGPU
    uniform_buffer: GpuBuffer<Matrix4>,

    #[get(visibility = "pub(crate)")]
    bgl: wgpu::BindGroupLayout,

    #[get(visibility = "pub(crate)")]
    bg: wgpu::BindGroup,

    // Maths
    projection: Projection,
    position: Vector3,
    target: Vector3,
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
            position: Vector3::new(0.0, 0.0, -2.0),
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
    pub(crate) fn resize(&mut self, view: Size<u32>) {
        // FIXME: tf?
        self.projection = Projection::Orthographic {
            left: 0.0,
            right: view.width as f32,
            bottom: view.height as f32,
            top: 0.0,
            near: -1.0,
            far: 1.0,
        };

        let vp = self.projection.matrix(view) * self.view_matrix();

        self.uniform_buffer.write(0, &[vp]);
        self.set_dirty(Self::projection_f());
    }
}
