use std::rc::Rc;

use math::{Mat4, Size};

use crate::util;

pub struct Camera {
    queue: Rc<wgpu::Queue>,
    projection_buffer: wgpu::Buffer,
    projection_bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(
        device: &wgpu::Device,
        queue: Rc<wgpu::Queue>,
        bind_group_layout: &wgpu::BindGroupLayout,
        screen_size: Size<u32>,
    ) -> Self {
        let projection_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Projection Buffer"),
            size: std::mem::size_of::<Mat4>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Projection Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &projection_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let mut camera = Self {
            queue,
            projection_buffer,
            projection_bind_group,
        };

        camera.update_projection(screen_size);

        camera
    }

    fn orthographic_matrix(screen_size: Size<u32>) -> Mat4 {
        Mat4::orthographic(
            0.0,
            screen_size.width as f32,
            screen_size.height as f32,
            0.0,
            -1.0,
            1.0,
        )
    }

    pub(crate) fn update_projection(&mut self, screen_size: Size<u32>) {
        let projection_matrix = Self::orthographic_matrix(screen_size);

        self.queue.write_buffer(
            &self.projection_buffer,
            0,
            util::as_u8_slice(&[projection_matrix]),
        );
    }

    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.projection_buffer
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.projection_bind_group
    }
}
