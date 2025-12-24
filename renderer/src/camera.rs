use macros::{Get, Set};
use math::{Easing, rng};
use math::{Matrix4, Tween, Vector2, Vector3};
use std::time::Duration;

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
    fn matrix(&self, width: u32, height: u32) -> Matrix4 {
        match *self {
            Self::Orthographic {
                left,
                right,
                bottom,
                top,
                z_near,
                z_far,
            } => Matrix4::orthographic(left, right, bottom, top, z_near, z_far),
            Self::Perspective { fovy, near, far } => {
                Matrix4::perspective(fovy, width as f32 / height as f32, near, far)
            }
        }
    }
}

#[derive(Debug)]
#[derive(Get, Set)]
pub struct Camera {
    projection: Projection,
    view_projection_buffer: wgpu::Buffer,

    #[get(visibility = "pub(crate)")]
    view_projection_bind_group_layout: wgpu::BindGroupLayout,

    #[get(visibility = "pub(crate)")]
    view_projection_bind_group: wgpu::BindGroup,

    #[get]
    #[get(copied, prop = "x", ty = f32)]
    #[get(copied, prop = "y", ty = f32)]
    #[get(mut, also = self.mark())]
    #[get(mut, prop = "x", ty = &mut f32, also = self.mark())]
    #[get(mut, prop = "y", ty = &mut f32, also = self.mark())]
    #[set(into, also = self.mark())]
    #[set(prop = x, ty = f32, also = self.mark())]
    #[set(prop = y, ty = f32, also = self.mark())]
    position: Vector2,

    #[get(copied, visibility = "pub(crate)")]
    dirty: bool,

    target: Vector3,
    up: Vector3,

    shake_offset: Vector2,
    shake_tween: Option<Tween<Vector2>>,
}

impl Camera {
    pub(crate) fn new(projection: Projection) -> Self {
        let device = gpu::device();
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
            position: Vector2::new(0.0, 0.0),
            target: Vector3::z(),
            // No need to set as dirty initially,
            // as the winit loop sends a resize event at startup
            dirty: false,
            up: Vector3::y(),
            shake_offset: Vector2::zeros(),
            shake_tween: None,
        }
    }

    #[inline]
    fn view_matrix(&self) -> Matrix4 {
        match self.projection {
            Projection::Orthographic { .. } => Matrix4::from_translation(Vector3::new(
                -(self.position.x + self.shake_offset.x),
                -(self.position.y + self.shake_offset.y),
                0.0,
            )),
            Projection::Perspective { .. } => Matrix4::look_at(
                (self.position + self.shake_offset).extend(-5.0),
                self.target,
                self.up,
            ),
        }
    }

    #[inline]
    fn view_projection_matrix(&self, width: u32, height: u32) -> Matrix4 {
        self.projection.matrix(width, height) * self.view_matrix()
    }

    #[inline]
    fn mark(&mut self) {
        self.dirty = true;
    }

    #[inline]
    pub(crate) fn clean(&mut self) {
        self.dirty = false;
    }

    #[inline]
    pub(crate) fn update_shake(&mut self, dt: f32) {
        if let Some(ref mut tween) = self.shake_tween {
            tween.update(dt);
            self.shake_offset = tween.value();

            if tween.is_complete() {
                self.shake_offset = Vector2::zeros();
                self.shake_tween = None;
            }

            self.mark();
        }
    }

    #[inline]
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.projection = Projection::Orthographic {
            left: 0.0,
            right: width as f32,
            bottom: height as f32,
            top: 0.0,
            z_near: -1.0,
            z_far: 1.0,
        };

        gpu::queue().write_buffer(
            &self.view_projection_buffer,
            0,
            utils::as_u8_slice(&[self.view_projection_matrix(width, height)]),
        );
    }

    #[inline]
    pub fn shake(&mut self, intensity: f32, duration: Duration) {
        let angle = rng(0.0..std::f32::consts::TAU);
        let target_offset = Vector2::new(angle.cos() * intensity, angle.sin() * intensity);

        let easing = Easing::Custom(|t: f32| {
            let freq = 15.0;
            let decay = 3.0;
            (1.0 - t).powf(decay) * (t * freq * std::f32::consts::TAU).sin()
        });

        let mut tween = Tween::new(Vector2::zeros(), target_offset, easing, duration);

        tween.start();

        self.shake_offset = Vector2::zeros();
        self.shake_tween = Some(tween);
    }
}
