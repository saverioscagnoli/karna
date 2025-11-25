use crate::util;
use nalgebra::{Matrix4, Vector2, Vector3};

pub enum CameraKind {
    Orthographic,
    Perspective { fovy: f32, near: f32, far: f32 },
}

impl Default for CameraKind {
    fn default() -> Self {
        Self::Orthographic
    }
}

pub struct Camera {
    kind: CameraKind,
    view_projection_buffer: wgpu::Buffer,
    view_projection_bind_group_layout: wgpu::BindGroupLayout,
    view_projection_bind_group: wgpu::BindGroup,
    projection_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
    screen_size: Vector2<u32>,
    position: Vector3<f32>,
    target: Vector3<f32>,
    up: Vector3<f32>,
}

impl Camera {
    pub(crate) fn new(device: &wgpu::Device, screen_size: Vector2<u32>, kind: CameraKind) -> Self {
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

        let mut camera = Self {
            view_projection_buffer,
            view_projection_bind_group_layout,
            view_projection_bind_group,
            projection_matrix: Matrix4::identity(),
            view_matrix: Matrix4::identity(),
            kind,
            screen_size,
            position: Vector3::new(0.0, 0.0, 3.0),
            target: Vector3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
        };

        camera.update_view_matrix();
        camera.update_projection_matrix();
        camera
    }

    fn update_projection_buffer(&mut self, queue: &wgpu::Queue) {
        let mvp = self.projection_matrix * self.view_matrix;
        queue.write_buffer(&self.view_projection_buffer, 0, util::as_u8_slice(&[mvp]));
    }

    pub(crate) fn update_projection_matrix(&mut self) {
        self.projection_matrix = match self.kind {
            CameraKind::Orthographic => Matrix4::new_orthographic(
                0.0,
                self.screen_size.x as f32,
                self.screen_size.y as f32,
                0.0,
                -1.0,
                1.0,
            ),
            CameraKind::Perspective { fovy, near, far } => {
                let aspect = self.screen_size.x as f32 / self.screen_size.y as f32;
                Matrix4::new_perspective(aspect, fovy, near, far)
            }
        }
    }

    fn update_view_matrix(&mut self) {
        self.view_matrix = match self.kind {
            CameraKind::Orthographic => Matrix4::identity(),
            CameraKind::Perspective { .. } => {
                Matrix4::look_at_rh(&self.position.into(), &self.target.into(), &self.up)
            }
        };
    }

    pub(crate) fn update_projection(&mut self, screen_size: Vector2<u32>, queue: &wgpu::Queue) {
        self.screen_size = screen_size;
        self.update_projection_matrix();
        self.update_projection_buffer(queue);
    }

    pub fn set_position(&mut self, position: Vector3<f32>, queue: &wgpu::Queue) {
        self.position = position;
        self.update_view_matrix();
        self.update_projection_buffer(queue);
    }

    pub fn set_target(&mut self, target: Vector3<f32>, queue: &wgpu::Queue) {
        self.target = target;
        self.update_view_matrix();
        self.update_projection_buffer(queue);
    }

    pub fn look_at(&mut self, position: Vector3<f32>, target: Vector3<f32>, queue: &wgpu::Queue) {
        self.position = position;
        self.target = target;
        self.update_view_matrix();
        self.update_projection_buffer(queue);
    }

    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.view_projection_bind_group_layout
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.view_projection_bind_group
    }
}
