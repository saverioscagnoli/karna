use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use nostd::collections::HashMap;
use nostd::collections::SlotMap;
use utils::Label;

use crate::assets::Image;

#[derive(Debug, Clone, Copy)]
pub struct SpriteAnimation {
    start: u32,
    len: u32,
    frame_time: f32,
    looping: bool,
}

impl SpriteAnimation {
    pub fn frame_at(&self, elapsed: f32) -> u32 {
        let n = (elapsed / self.frame_time) as u32;

        if self.looping {
            n % self.len
        } else {
            n.min(self.len - 1)
        }
    }

    pub fn finished_at(&self, elapsed: f32) -> bool {
        !self.looping && elapsed >= self.len as f32 * self.frame_time
    }
}

#[derive(Debug, Clone)]
pub struct Spritesheet {
    image: Handle<Image>,
    frames: Vec<[u32; 4]>,
    animations: HashMap<Label, SpriteAnimation>,
}

impl Spritesheet {
    pub(crate) fn image(&self) -> Handle<Image> {
        self.image
    }

    pub(crate) fn animation(&self, name: Label) -> Option<&SpriteAnimation> {
        self.animations.get(&name)
    }

    pub(crate) fn frame(&self, index: u32) -> Option<[u32; 4]> {
        self.frames.get(index as usize).copied()
    }
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct SpritesheetRegistry {
    spritesheets: SlotMap<Spritesheet>,
}

impl SpritesheetRegistry {
    pub fn add(&mut self, sheet: Spritesheet) -> Handle<Spritesheet> {
        self.spritesheets.insert(sheet)
    }

    pub fn get(&self, handle: Handle<Spritesheet>) -> Option<&Spritesheet> {
        self.spritesheets.get(handle)
    }

    pub fn get_mut(&mut self, handle: Handle<Spritesheet>) -> Option<&mut Spritesheet> {
        self.spritesheets.get_mut(handle)
    }

    pub fn remove(&mut self, handle: Handle<Spritesheet>) -> Option<Spritesheet> {
        self.spritesheets.remove(handle)
    }
}
