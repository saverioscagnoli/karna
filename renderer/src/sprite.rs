use crate::{Geometry, Material, Mesh, TextureKind, Transform};
use macros::{Get, Set, With};
use math::Vector2;
use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};
use utils::map::Label;

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
#[derive(Get, Set, With)]
pub struct Sprite {
    texture_label: Label,
    mesh: Mesh,
    frames: Vec<Frame>,
    frame_current: usize,
    elapsed: f32,

    #[get]
    #[get(mut)]
    #[get(mut, prop = "x", ty = &mut f32)]
    #[get(mut, prop = "y", ty = &mut f32)]
    #[set(into)]
    #[set(prop = "x", ty = f32)]
    #[set(prop = "y", ty = f32)]
    #[with(into)]
    #[with(prop = "x", ty = f32)]
    #[with(prop = "y", ty = f32)]
    render_scale: Vector2,
}

impl Deref for Sprite {
    type Target = Mesh;
    fn deref(&self) -> &Self::Target {
        &self.mesh
    }
}

impl DerefMut for Sprite {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mesh
    }
}

impl Sprite {
    pub fn new(texture: Label, frames: Vec<Frame>) -> Self {
        assert!(!frames.is_empty());

        let frame = frames[0];
        let render_scale = Vector2::new(1.0, 1.0);

        let mesh = Mesh::new(
            Geometry::unit_rect(),
            Material::new_texture(TextureKind::Partial(
                texture,
                frame.x,
                frame.y,
                frame.width,
                frame.height,
            )),
            Transform::default()
                .with_scale([frame.width * render_scale.x, frame.height * render_scale.y]),
        );

        Self {
            texture_label: texture,
            mesh,
            frames,
            frame_current: 0,
            elapsed: 0.0,
            render_scale,
        }
    }

    #[inline]
    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;

        let current_frame = self.frames[self.frame_current];

        if self.elapsed >= current_frame.duration.as_secs_f32() {
            self.elapsed -= current_frame.duration.as_secs_f32();
            self.frame_current = (self.frame_current + 1) % self.frames.len();
            self.refresh_mesh();
        }
    }

    fn refresh_mesh(&mut self) {
        let frame = self.frames[self.frame_current];
        let label = self.texture_label;

        self.set_material(Material::new_texture(TextureKind::Partial(
            label,
            frame.x,
            frame.y,
            frame.width,
            frame.height,
        )));

        *self.scale_mut() = Vector2::new(
            frame.width * self.render_scale.x,
            frame.height * self.render_scale.y,
        );
    }

    #[inline]
    pub fn set_frame(&mut self, index: usize) {
        if index < self.frames.len() {
            self.frame_current = index;
            self.elapsed = 0.0;
            self.refresh_mesh();
        }
    }

    pub fn reset(&mut self) {
        self.set_frame(0);
    }
}
