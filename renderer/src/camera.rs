use common::utils;
use macros::{Get, Set};
use math::{Matrix4, Size, Vector3};

#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        z_near: f32,
        z_far: f32,
    },
    #[allow(unused)]
    Perspective { fovy: f32, near: f32, far: f32 },
}

impl Projection {
    fn matrix(&self, window_size: &Size<u32>) -> Matrix4 {
        match *self {
            Self::Orthographic {
                left,
                right,
                bottom,
                top,
                z_near,
                z_far,
            } => Matrix4::orthographic(left, right, bottom, top, z_near, z_far),
            Self::Perspective { fovy, near, far } => Matrix4::perspective(
                fovy,
                window_size.width as f32 / window_size.height as f32,
                near,
                far,
            ),
        }
    }
}

#[derive(Debug)]
#[derive(Get, Set)]
pub struct Camera {
    projection: Projection,
    view_projection_buffer: wgpu::Buffer,
    pub(crate) view_projection_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) view_projection_bind_group: wgpu::BindGroup,

    #[get]
    #[get(copied, prop = x, ty = f32)]
    #[get(copied, prop = y, ty = f32)]
    #[set(into)]
    #[set(prop = x, ty = f32)]
    #[set(prop = y, ty = f32)]
    pub position: Vector3,
    target: Vector3,
    up: Vector3,
}

impl Camera {
    pub(crate) fn new(device: &wgpu::Device, projection: Projection) -> Self {
        let view_projection_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera vp buffer"),
            size: std::mem::size_of::<Matrix4>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let view_projection_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("vp bind group layout"),
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

        let view_projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vp bind group"),
            layout: &view_projection_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &view_projection_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Self {
            projection,
            view_projection_buffer,
            view_projection_bind_group_layout,
            view_projection_bind_group,
            position: Vector3::new(0.0, 0.0, -5.0),
            target: Vector3::z(),
            up: Vector3::y(),
        }
    }

    #[inline]
    fn view_matrix(&self) -> Matrix4 {
        match self.projection {
            Projection::Orthographic { .. } => {
                Matrix4::from_translation(Vector3::new(-self.position.x, -self.position.y, 0.0))
            }

            Projection::Perspective { .. } => Matrix4::look_at(self.position, self.target, self.up),
        }
    }
    #[inline]
    fn view_projection_matrix(&self, window_size: &Size<u32>) -> Matrix4 {
        self.projection.matrix(window_size) * self.view_matrix()
    }

    #[inline]
    pub(crate) fn update(&mut self, window_size: &Size<u32>, queue: &wgpu::Queue) {
        self.projection = Projection::Orthographic {
            left: 0.0,
            right: window_size.width as f32,
            bottom: window_size.height as f32,
            top: 0.0,
            z_near: -1.0,
            z_far: 1.0,
        };

        queue.write_buffer(
            &self.view_projection_buffer,
            0,
            utils::as_u8_slice(&[self.view_projection_matrix(window_size)]),
        );
    }
}
