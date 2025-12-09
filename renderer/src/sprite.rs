use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use crate::{Color, Material, Mesh, MeshGeometry, Renderer, TextureKind, TextureRegion, Transform};
use common::utils::Label;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Frame {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub texture: Label,
    pub mesh: Mesh,

    frames: Vec<Frame>,
    frame_current: usize,
    frame_duration_original: f32,
    frame_duration_remaining: f32,
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
    pub fn new(texture: Label, frames: Vec<Frame>, frame_duration: Duration) -> Self {
        assert!(frames.len() > 0);
        let frame = frames[0];
        let duration_secs = frame_duration.as_secs_f32();

        Self {
            texture,
            mesh: Mesh {
                geometry: MeshGeometry::rect(),
                material: Material {
                    color: Some(Color::White),
                    texture: Some(TextureKind::Partial(
                        texture,
                        TextureRegion::new(frame.x, frame.y, frame.width, frame.height),
                    )),
                },
                transform: Transform::default()
                    .with_scale([frame.width as f32, frame.height as f32]),
            },
            frames,
            frame_current: 0,
            frame_duration_original: duration_secs,
            frame_duration_remaining: duration_secs,
        }
    }

    #[inline]
    pub fn update(&mut self, dt: f32) {
        if self.frames.len() <= 1 {
            return;
        }

        self.frame_duration_remaining -= dt;

        if self.frame_duration_remaining <= 0.0 {
            // Move to next frame
            self.frame_current = (self.frame_current + 1) % self.frames.len();

            // Update the mesh texture region with the new frame
            let frame = self.frames[self.frame_current];

            self.mesh.material.texture = Some(TextureKind::Partial(
                self.texture,
                TextureRegion::new(frame.x, frame.y, frame.width, frame.height),
            ));

            // Reset the frame timer
            self.frame_duration_remaining = self.frame_duration_original;
        }
    }

    #[inline]
    pub fn render(&self, renderer: &mut Renderer) {
        renderer.draw_mesh(&self.mesh);
    }
}
