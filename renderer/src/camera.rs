use crate::util;
use math::{Mat4, Size, Vec3};
use std::rc::Rc;

const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_array([
    [1.0, 0.0, 0.0, 0.0], //
    [0.0, 1.0, 0.0, 0.0], //
    [0.0, 0.0, 0.5, 0.0], //
    [0.0, 0.0, 0.5, 1.0], //
]);

#[derive(Clone, Copy, Debug)]
pub enum CameraType {
    Orthographic,
    Perspective {
        fov_y: f32, // Field of view in radians
        near: f32,
        far: f32,
    },
}

impl Default for CameraType {
    fn default() -> Self {
        CameraType::Orthographic
    }
}

pub struct Camera {
    queue: Rc<wgpu::Queue>,
    view_projection_buffer: wgpu::Buffer,
    view_projection_bind_group_layout: wgpu::BindGroupLayout,
    view_projection_bind_group: wgpu::BindGroup,
    projection_matrix: Mat4,
    view_matrix: Mat4,
    camera_type: CameraType,
    screen_size: Size<u32>,
}

impl Camera {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: Rc<wgpu::Queue>,
        screen_size: Size<u32>,
        camera_type: CameraType,
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

        let mut camera = Self {
            queue,
            view_projection_buffer,
            view_projection_bind_group_layout,
            view_projection_bind_group,
            projection_matrix: Mat4::identity(),
            view_matrix: Mat4::identity(),
            camera_type,
            screen_size,
        };

        camera.update_projection_matrix();
        camera
    }

    pub(crate) fn update_projection(&mut self, screen_size: Size<u32>) {
        self.screen_size = screen_size;
        self.update_projection_matrix();
    }

    pub fn set_view(&mut self, eye: Vec3, target: Vec3, up: Vec3) {
        self.view_matrix = Mat4::look_at(eye, target, up);
        self.update_view_projection_buffer();
    }

    // Add a method to position camera for 2D rendering with perspective
    pub fn set_view_2d(&mut self, distance: f32) {
        // Position camera back along Z axis, looking at origin
        let eye = Vec3::new(0.0, 0.0, distance);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);

        self.view_matrix = Mat4::look_at(eye, target, up);
        self.update_view_projection_buffer();
    }

    // Extract the buffer update logic into its own method
    fn update_view_projection_buffer(&self) {
        let view_projection = OPENGL_TO_WGPU_MATRIX * self.projection_matrix * self.view_matrix;

        self.queue.write_buffer(
            &self.view_projection_buffer,
            0,
            util::as_u8_slice(&[view_projection]),
        );
    }

    // Update the existing update_projection_matrix method
    fn update_projection_matrix(&mut self) {
        self.projection_matrix = match self.camera_type {
            CameraType::Orthographic => Mat4::orthographic(
                0.0,
                self.screen_size.width as f32,
                self.screen_size.height as f32,
                0.0,
                -1.0,
                1.0,
            ),
            CameraType::Perspective { fov_y, near, far } => {
                let aspect_ratio = self.screen_size.width as f32 / self.screen_size.height as f32;
                Mat4::perspective(fov_y, aspect_ratio, near, far)
            }
        };

        // Call the extracted method
        self.update_view_projection_buffer();
    }

    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.view_projection_bind_group_layout
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.view_projection_bind_group
    }
}
