use crate::util;
use nalgebra::{Matrix, Matrix4, Point3, Vector2, Vector3};
use winit::keyboard;

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
    OrthographicStandard2D {
        near: f32,
        far: f32,
    },
    Perspective {
        fovy: f32,
        near: f32,
        far: f32,
    },
}

impl Projection {
    fn matrix(&self, screen_size: Vector2<u32>) -> Matrix4<f32> {
        match *self {
            Self::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => Matrix4::new_orthographic(left, right, bottom, top, near, far),

            Self::OrthographicStandard2D { near, far } => Matrix4::new_orthographic(
                0.0,
                screen_size.x as f32,
                screen_size.y as f32,
                0.0,
                near,
                far,
            ),

            Self::Perspective { fovy, near, far } => Matrix::new_perspective(
                screen_size.x as f32 / screen_size.y as f32,
                fovy,
                near,
                far,
            ),
        }
    }
}

pub struct Camera {
    projection: Projection,

    view_projection_buffer: wgpu::Buffer,
    view_projection_bind_group_layout: wgpu::BindGroupLayout,
    view_projection_bind_group: wgpu::BindGroup,

    position: Vector3<f32>,
    target: Vector3<f32>,
    up: Vector3<f32>,
}

impl Camera {
    #[rustfmt::skip]
    const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.5,
        0.0, 0.0, 0.0, 1.0,
    );

    pub(crate) fn new(device: &wgpu::Device, projection: Projection) -> Self {
        let view_projection_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera mvp buffer"),
            size: std::mem::size_of::<Matrix4<f32>>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let view_projection_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("mvp bind group layout"),
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
            label: Some("mvp bind group"),
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
            position: Vector3::new(0.0, 0.0, 5.0),
            target: Vector3::new(0.0, 0.0, 0.0), // Look to -z
            up: Vector3::y(),
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(
            &Point3::from(self.position),
            &Point3::from(self.target),
            &self.up,
        )
    }

    pub fn view_projection_matrix(&self, screen_size: Vector2<u32>) -> Matrix4<f32> {
        Self::OPENGL_TO_WGPU_MATRIX * self.projection.matrix(screen_size) * self.view_matrix()
    }

    pub(crate) fn update(&mut self, screen_size: Vector2<u32>, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.view_projection_buffer,
            0,
            util::as_u8_slice(&[self.view_projection_matrix(screen_size)]),
        );
    }

    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.view_projection_bind_group_layout
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.view_projection_bind_group
    }
}
