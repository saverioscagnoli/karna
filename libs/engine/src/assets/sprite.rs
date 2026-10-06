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

#[derive(Debug, Clone)]
pub struct Spritesheet {
    image: Handle<Image>,
    frames: Vec<[u32; 4]>,
    animations: HashMap<Label, SpriteAnimation>,
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct SpritesheetRegistry {
    items: SlotMap<Spritesheet>,
}
