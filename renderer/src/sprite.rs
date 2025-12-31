use crate::{Geometry, Layer, Material, Mesh, TextureKind, Transform};
use macros::{Get, Set, With};
use math::{Vector2, Vector3};
use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};
use utils::{Handle, Label};

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
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
    #[get(mut, prop = "x", ty = &mut f32, also = self.update_scale())]
    #[get(mut, prop = "y", ty = &mut f32, also = self.update_scale())]
    #[set(into, also = self.update_scale())]
    #[set(prop = "x", ty = f32, also = self.update_scale())]
    #[set(prop = "y", ty = f32, also = self.update_scale())]
    #[with(into, also = self.update_scale())]
    #[with(prop = "x", ty = f32, also = self.update_scale())]
    #[with(prop = "y", ty = f32, also = self.update_scale())]
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
            Transform::default(),
        );

        let mut sprite = Self {
            texture_label: texture,
            mesh,
            frames,
            frame_current: 0,
            elapsed: 0.0,
            render_scale,
        };

        sprite.update_scale();
        sprite
    }

    #[inline]
    pub fn update(&mut self, dt: f32) {
        if self.frames.len() <= 1 {
            return;
        }

        self.elapsed += dt;

        let current_frame = self.frames[self.frame_current];

        if self.elapsed >= current_frame.duration.as_secs_f32() {
            self.elapsed -= current_frame.duration.as_secs_f32();
            self.frame_current = (self.frame_current + 1) % self.frames.len();

            self.refresh_mesh();
        }
    }

    /// Updates the underlying Mesh to reflect the current frame's data.
    /// This changes the Material (UV coords) and the Transform (Scale).
    fn refresh_mesh(&mut self) {
        let frame = self.frames[self.frame_current];
        let label = self.texture_label;

        // Update UV coordinates
        self.set_material(Material::new_texture(TextureKind::Partial(
            label,
            frame.x,
            frame.y,
            frame.width,
            frame.height,
        )));

        self.update_scale();
    }

    /// Calculates the final World Scale for the mesh.
    /// Intrinsic Frame Size (Pixels) * User Render Scale = Final Transform
    fn update_scale(&mut self) {
        let frame = self.frames[self.frame_current];
        let new_scale = Vector3::new(
            frame.width as f32 * self.render_scale.x,
            frame.height as f32 * self.render_scale.y,
            1.0,
        );

        self.set_scale(new_scale);
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

#[derive(Debug, Clone, Copy)]
#[derive(Get)]
pub struct SpriteHandle {
    #[get]
    pub(crate) layer: Layer,
    pub(crate) handle: Handle<Sprite>,
}

impl Deref for SpriteHandle {
    type Target = Handle<Sprite>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl DerefMut for SpriteHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

impl SpriteHandle {
    pub fn dummy() -> Self {
        Self {
            layer: Layer::World,
            handle: Handle::dummy(),
        }
    }
}
