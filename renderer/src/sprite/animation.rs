use std::time::Duration;

use macros::With;
use utils::FastHashMap;

#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub enum LoopMode {
    Once,
    #[default]
    Loop,
    PingPong,
}

#[derive(With)]
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    #[with]
    pub x: u32,

    #[with]
    pub y: u32,

    #[with]
    pub width: u32,

    #[with]
    pub height: u32,

    #[with]
    pub duration: Duration,
}

impl Frame {
    pub fn new(x: u32, y: u32, width: u32, height: u32, duration: Duration) -> Self {
        Self {
            x,
            y,
            width,
            height,
            duration,
        }
    }
}

#[derive(With)]
#[derive(Default)]
#[derive(Debug, Clone)]
pub struct Animation {
    #[with]
    frames: Vec<Frame>,
    #[with]
    loop_mode: LoopMode,
}

impl Animation {
    pub fn new(frames: Vec<Frame>, loop_mode: LoopMode) -> Self {
        Self { frames, loop_mode }
    }

    #[inline]
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    #[inline]
    pub fn loop_mode(&self) -> LoopMode {
        self.loop_mode
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    #[inline]
    pub fn frame(&self, index: usize) -> Option<Frame> {
        self.frames.get(index).copied()
    }

    pub fn add_frame(mut self, frame: Frame) -> Self {
        self.frames.push(frame);
        self
    }

    pub fn add_frame_d(
        mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        duration: Duration,
    ) -> Self {
        self.frames.push(Frame::new(x, y, width, height, duration));
        self
    }
}

#[derive(Default, Debug, Clone)]
pub struct Animations(FastHashMap<String, Animation>);

impl Animations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_animation<N: Into<String>>(mut self, name: N, animation: Animation) -> Self {
        self.0.insert(name.into(), animation);
        self
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<&Animation> {
        self.0.get(name)
    }

    #[inline]
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Animation> {
        self.0.get_mut(name)
    }

    #[inline]
    pub fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Animation)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v))
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
