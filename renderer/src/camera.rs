use crate::util;
use math::{Mat4, Size};
use std::rc::Rc;

pub struct Camera {
    queue: Rc<wgpu::Queue>,
    view_projection_buffer: wgpu::Buffer,
    view_projection_bind_group_layout: wgpu::BindGroupLayout,
    view_projection_bind_group: wgpu::BindGroup,
    projection_matrix: Mat4,
    view_matrix: Mat4,
}

impl Camera {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: Rc<wgpu::Queue>,
        screen_size: math::Size<u32>,
    ) -> Self {
        let view_projection_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera View-Projection Buffer"),
            size: std::mem::size_of::<Mat4>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let view_projection_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("View-Projection Bind Group Layout"),
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
            label: Some("View-Projection Bind Group"),
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
            queue,
            view_projection_buffer,
            view_projection_bind_group_layout,
            view_projection_bind_group,
            projection_matrix: Mat4::identity(),
            view_matrix: Mat4::identity(),
        }
    }

    pub(crate) fn update_projection(&mut self, screen_size: Size<u32>) {
        self.projection_matrix = Mat4::orthographic(
            0.0,
            screen_size.width as f32,
            screen_size.height as f32,
            0.0,
            -1.0,
            1.0,
        );
        let view_projection = self.projection_matrix * self.view_matrix;

        self.queue.write_buffer(
            &self.view_projection_buffer,
            0,
            util::as_u8_slice(&[view_projection]),
        );
    }

    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.view_projection_bind_group_layout
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.view_projection_bind_group
    }
}
