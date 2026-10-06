use core::ops::Range;
use core::time::Duration;

use nostd::alloc::rc::Rc;
use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use nostd::collections::HashMap;
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
