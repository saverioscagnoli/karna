use std::rc::Rc;

use nalgebra::{Matrix4, Point3, Vector3};
use wgpu::BufferUsages;

use crate::util;

pub trait Camera {
    fn view_matrix(&self) -> Matrix4<f32>;
    fn projection_matrix(&self) -> Matrix4<f32>;
    fn view_projection_matrix(&self) -> Matrix4<f32>;

    fn position(&self) -> Vector3<f32>;
    fn forward(&self) -> Vector3<f32>;
    fn right(&self) -> Vector3<f32>;
    fn up(&self) -> Vector3<f32>;

    fn set_position(&mut self, position: Vector3<f32>);
    fn translate(&mut self, offset: Vector3<f32>);
    fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32);

    fn update(&mut self, dt: f32);
    fn handle_resize(&mut self, width: f32, height: f32);
}

pub trait WgpuCamera: Camera {
    fn view_projection_buffer(&self) -> &wgpu::Buffer;
    fn view_projection_bind_group(&self) -> &wgpu::BindGroup;
    fn view_projection_bind_group_layout(&self) -> &wgpu::BindGroupLayout;

    fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            self.view_projection_buffer(),
            0,
            util::as_u8_slice(&[self.view_projection_matrix()]),
        );
    }
}

pub struct PerspectiveCamera {
    position: Vector3<f32>,
    yaw: f32,
    pitch: f32,

    forward: Vector3<f32>,
    right: Vector3<f32>,
    up: Vector3<f32>,

    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,

    view_matrix: Matrix4<f32>,
    projection_matrix: Matrix4<f32>,
    view_projection_matrix: Matrix4<f32>,

    view_projection_buffer: wgpu::Buffer,
    view_projection_bind_group: wgpu::BindGroup,
    view_projection_bind_group_layout: wgpu::BindGroupLayout,

    dirty: bool,
}

impl PerspectiveCamera {
    pub fn new(
        device: Rc<wgpu::Device>,
        position: Vector3<f32>,
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let view_projection_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("perspective camera buffer"),
            size: std::mem::size_of::<Matrix4<f32>>() as wgpu::BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let view_projection_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
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
            label: Some("Camera Bind Group"),
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
            position,
            yaw: 0.0,
            pitch: 0.0,
            forward: Vector3::new(0.0, 0.0, -1.0),
            right: Vector3::new(1.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            fov,
            aspect_ratio,
            near,
            far,
            view_matrix: Matrix4::identity(),
            projection_matrix: Matrix4::identity(),
            view_projection_matrix: Matrix4::identity(),
            view_projection_buffer,
            view_projection_bind_group,
            view_projection_bind_group_layout,
            dirty: true,
        };

        camera.update_vectors();
        camera.update_matrices();

        camera
    }

    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch =
            (self.pitch + delta_pitch).clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.update_vectors();
        self.dirty = true;
    }

    pub fn set_rotation(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch.clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.update_vectors();
        self.dirty = true;
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    fn update_vectors(&mut self) {
        self.forward.x = self.yaw.cos() * self.pitch.cos();
        self.forward.y = self.pitch.sin();
        self.forward.z = self.yaw.sin() * self.pitch.cos();
        self.forward.normalize_mut();

        self.right = self.forward.cross(&Vector3::y());
        self.right.normalize_mut();

        self.up = self.right.cross(&self.forward);
        self.up.normalize_mut();
    }

    fn update_matrices(&mut self) {
        let position_point = Point3::from(self.position);
        let target = self.position + self.forward;
        let target_point = Point3::from(target);
        self.view_matrix = Matrix4::look_at_rh(&position_point, &target_point, &self.up);

        self.projection_matrix =
            Matrix4::new_perspective(self.aspect_ratio, self.fov, self.near, self.far);

        self.view_projection_matrix = self.projection_matrix * self.view_matrix;

        self.dirty = false;
    }
}

impl Camera for PerspectiveCamera {
    fn view_matrix(&self) -> Matrix4<f32> {
        self.view_matrix
    }

    fn projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix
    }

    fn view_projection_matrix(&self) -> Matrix4<f32> {
        self.view_projection_matrix
    }

    fn position(&self) -> Vector3<f32> {
        self.position
    }

    fn forward(&self) -> Vector3<f32> {
        self.forward
    }

    fn right(&self) -> Vector3<f32> {
        self.right
    }

    fn up(&self) -> Vector3<f32> {
        self.up
    }

    fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
        self.dirty = true;
    }

    fn translate(&mut self, offset: Vector3<f32>) {
        self.position += offset;
        self.dirty = true;
    }

    fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch =
            (self.pitch + delta_pitch).clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.update_vectors();
        self.dirty = true;
    }

    fn update(&mut self, _dt: f32) {
        if self.dirty {
            self.update_matrices();
        }
    }

    fn handle_resize(&mut self, width: f32, height: f32) {
        self.aspect_ratio = width / height;
        self.dirty = true;
    }
}

impl WgpuCamera for PerspectiveCamera {
    fn view_projection_buffer(&self) -> &wgpu::Buffer {
        &self.view_projection_buffer
    }

    fn view_projection_bind_group(&self) -> &wgpu::BindGroup {
        &self.view_projection_bind_group
    }

    fn view_projection_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.view_projection_bind_group_layout
    }
}

pub struct OrthographicCamera {
    position: Vector3<f32>,
    yaw: f32,
    pitch: f32,

    forward: Vector3<f32>,
    right: Vector3<f32>,
    up: Vector3<f32>,

    left: f32,
    right_bound: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,

    view_matrix: Matrix4<f32>,
    projection_matrix: Matrix4<f32>,
    view_projection_matrix: Matrix4<f32>,

    view_projection_buffer: wgpu::Buffer,
    view_projection_bind_group: wgpu::BindGroup,
    view_projection_bind_group_layout: wgpu::BindGroupLayout,

    dirty: bool,
}

impl OrthographicCamera {
    /// Create a new orthographic camera
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    /// * `position` - Initial camera position
    /// * `left` - Left bound of the view volume
    /// * `right` - Right bound of the view volume
    /// * `bottom` - Bottom bound of the view volume
    /// * `top` - Top bound of the view volume
    /// * `near` - Near clipping plane
    /// * `far` - Far clipping plane
    pub fn new(
        device: Rc<wgpu::Device>,
        position: Vector3<f32>,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let view_projection_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("orthographic camera buffer"),
            size: std::mem::size_of::<Matrix4<f32>>() as wgpu::BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let view_projection_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Orthographic Camera Bind Group Layout"),
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
            label: Some("Orthographic Camera Bind Group"),
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
            position,
            yaw: 0.0,
            pitch: 0.0,
            forward: Vector3::new(0.0, 0.0, -1.0),
            right: Vector3::new(1.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            left,
            right_bound: right,
            bottom,
            top,
            near,
            far,
            view_matrix: Matrix4::identity(),
            projection_matrix: Matrix4::identity(),
            view_projection_matrix: Matrix4::identity(),
            view_projection_buffer,
            view_projection_bind_group,
            view_projection_bind_group_layout,
            dirty: true,
        };

        camera.update_vectors();
        camera.update_matrices();

        camera
    }

    /// Create an orthographic camera that matches screen dimensions
    ///
    /// This is useful for 2D rendering where 1 unit = 1 pixel
    pub fn new_screen_size(
        device: Rc<wgpu::Device>,
        position: Vector3<f32>,
        width: f32,
        height: f32,
    ) -> Self {
        Self::new(device, position, 0.0, width, height, 0.0, -1.0, 1.0)
    }

    /// Create an orthographic camera centered at origin
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    /// * `position` - Initial camera position
    /// * `width` - Total width of the view volume
    /// * `height` - Total height of the view volume
    /// * `near` - Near clipping plane
    /// * `far` - Far clipping plane
    pub fn new_centered(
        device: Rc<wgpu::Device>,
        position: Vector3<f32>,
        width: f32,
        height: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        Self::new(
            device,
            position,
            -half_width,
            half_width,
            -half_height,
            half_height,
            near,
            far,
        )
    }

    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch =
            (self.pitch + delta_pitch).clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.update_vectors();
        self.dirty = true;
    }

    pub fn set_rotation(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch.clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.update_vectors();
        self.dirty = true;
    }

    pub fn set_bounds(&mut self, left: f32, right: f32, bottom: f32, top: f32) {
        self.left = left;
        self.right_bound = right;
        self.bottom = bottom;
        self.top = top;
        self.dirty = true;
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    fn update_vectors(&mut self) {
        self.forward.x = self.yaw.cos() * self.pitch.cos();
        self.forward.y = self.pitch.sin();
        self.forward.z = self.yaw.sin() * self.pitch.cos();
        self.forward.normalize_mut();

        self.right = self.forward.cross(&Vector3::y());
        self.right.normalize_mut();

        self.up = self.right.cross(&self.forward);
        self.up.normalize_mut();
    }

    fn update_matrices(&mut self) {
        let position_point = Point3::from(self.position);
        let target = self.position + self.forward;
        let target_point = Point3::from(target);
        self.view_matrix = Matrix4::look_at_rh(&position_point, &target_point, &self.up);

        self.projection_matrix = Matrix4::new_orthographic(
            self.left,
            self.right_bound,
            self.bottom,
            self.top,
            self.near,
            self.far,
        );

        self.view_projection_matrix = self.projection_matrix * self.view_matrix;

        self.dirty = false;
    }
}

impl Camera for OrthographicCamera {
    fn view_matrix(&self) -> Matrix4<f32> {
        self.view_matrix
    }

    fn projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix
    }

    fn view_projection_matrix(&self) -> Matrix4<f32> {
        self.view_projection_matrix
    }

    fn position(&self) -> Vector3<f32> {
        self.position
    }

    fn forward(&self) -> Vector3<f32> {
        self.forward
    }

    fn right(&self) -> Vector3<f32> {
        self.right
    }

    fn up(&self) -> Vector3<f32> {
        self.up
    }

    fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
        self.dirty = true;
    }

    fn translate(&mut self, offset: Vector3<f32>) {
        self.position += offset;
        self.dirty = true;
    }

    fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch =
            (self.pitch + delta_pitch).clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.update_vectors();
        self.dirty = true;
    }

    fn update(&mut self, _dt: f32) {
        if self.dirty {
            self.update_matrices();
        }
    }

    fn handle_resize(&mut self, width: f32, height: f32) {
        // For orthographic cameras, we can update bounds based on aspect ratio
        // This maintains the same scale but adjusts the view bounds
        let aspect_ratio = width / height;
        let height_half = (self.top - self.bottom) / 2.0;
        let width_half = height_half * aspect_ratio;

        self.left = -width_half;
        self.right_bound = width_half;
        self.bottom = -height_half;
        self.top = height_half;
        self.dirty = true;
    }
}

impl WgpuCamera for OrthographicCamera {
    fn view_projection_buffer(&self) -> &wgpu::Buffer {
        &self.view_projection_buffer
    }

    fn view_projection_bind_group(&self) -> &wgpu::BindGroup {
        &self.view_projection_bind_group
    }

    fn view_projection_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.view_projection_bind_group_layout
    }
}
