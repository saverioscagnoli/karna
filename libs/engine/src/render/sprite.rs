use nostd::collections::Handle;
use utils::Label;

use crate::assets::Image;
use crate::assets::SpriteAnimation;
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
    current: Option<Label>,
    elapsed: f32,
}

impl Sprite {
    pub fn new(spritesheet: Handle<Spritesheet>) -> Self {
        Self {
            spritesheet,
            current: None,
            elapsed: 0.0,
        }
    }

    #[inline]
    pub fn spritesheet(&self) -> Handle<Spritesheet> {
        self.spritesheet
    }

    #[inline]
    pub fn play(&mut self, name: Label) {
        if self.current != Some(name) {
            self.current = Some(name);
            self.elapsed = 0.0;
        }
    }

    #[inline]
    pub fn restart(&mut self) {
        self.elapsed = 0.0;
    }

    #[inline]
    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;
    }

    #[inline]
    pub fn current(&self) -> Option<Label> {
        self.current
    }

    #[inline]
    pub fn animation(&self, sheet: &Spritesheet) -> Option<SpriteAnimation> {
        sheet.animation(self.current)
    }

    #[inline]
    pub fn frame(&self, sheet: &Spritesheet) -> Option<u32> {
        self.animation(sheet)?.frame_at(self.elapsed)
    }

    #[inline]
    pub fn is_finished(&self, sheet: &Spritesheet) -> bool {
        self.animation(sheet)
            .is_some_and(|a| a.finished_at(self.elapsed))
    }
}
