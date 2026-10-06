use nostd::collections::Handle;
use traccia::init;
use utils::Label;

use crate::assets::Image;
use crate::assets::Spritesheet;
use crate::render::Draw;

pub struct StaticSprite {
    image: Handle<Image>,
    portion: Option<(u32, u32, u32, u32)>,
}

impl StaticSprite {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            image,
            portion: None,
        }
    }

    pub fn new_portion(image: Handle<Image>, portion: (u32, u32, u32, u32)) -> Self {
        Self {
            image,
            portion: Some(portion),
        }
    }

    pub fn draw(&self, draw: &mut Draw, x: f32, y: f32) {
        if let Some((px, py, pw, ph)) = self.portion {
            draw.image_region(
                self.image, px as f32, py as f32, pw as f32, ph as f32, x, y, pw as f32, ph as f32,
            );
        } else {
            draw.image(self.image, x, y);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    spritesheet: Handle<Spritesheet>,
    current: Label,
    elapsed: f32,
}

impl Sprite {
    pub fn new(spritesheet: Handle<Spritesheet>, initial: Label) -> Self {
        Self {
            spritesheet,
            current: initial,
            elapsed: 0.0,
        }
    }

    pub fn play(&mut self, name: Label) {
        if name != self.current {
            self.current = name;
            self.elapsed = 0.0;
        }
    }

    pub fn restart(&mut self) {
        self.elapsed = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;
    }

    pub fn current(&self) -> Label {
        self.current
    }
}
