use core::ops::Range;

use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use nostd::collections::SlotMap;
use utils::Label;

use crate::assets::Image;

const DEFAULT_FPS: f32 = 10.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SpriteFrame {
    pub rect: [u32; 4],
    pub offset: [u32; 2],
}

impl SpriteFrame {
    pub fn new(rect: [u32; 4]) -> Self {
        Self {
            rect,
            offset: [0, 0],
        }
    }

    pub fn with_offset(mut self, x: u32, y: u32) -> Self {
        self.offset = [x, y];
        self
    }
}

impl From<[u32; 4]> for SpriteFrame {
    fn from(rect: [u32; 4]) -> Self {
        Self::new(rect)
    }
}

#[derive(Debug, Clone, Copy)]
enum Frames {
    Range { start: u32, len: u32 },
    Strip { count: u32 },
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteAnimation {
    image: Option<Handle<Image>>,
    frames: Frames,
    frame_time: f32,
    looping: bool,
}

impl SpriteAnimation {
    pub fn len(&self) -> u32 {
        match self.frames {
            Frames::Range { len, .. } => len,
            Frames::Strip { count } => count,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_looping(&self) -> bool {
        self.looping
    }

    pub fn frame_at(&self, elapsed: f32) -> Option<u32> {
        let len = self.len();

        if len == 0 {
            return None;
        }

        let n = (elapsed / self.frame_time) as u32;

        Some(if self.looping {
            n % len
        } else {
            n.min(len - 1)
        })
    }

    pub fn finished_at(&self, elapsed: f32) -> bool {
        !self.looping && elapsed >= self.len() as f32 * self.frame_time
    }

    pub(crate) fn image(&self, sheet: &Spritesheet) -> Handle<Image> {
        self.image.unwrap_or(sheet.image)
    }

    pub(crate) fn frame(
        &self,
        sheet: &Spritesheet,
        index: u32,
        width: u32,
        height: u32,
    ) -> Option<SpriteFrame> {
        match self.frames {
            Frames::Range { start, len } if index < len => {
                sheet.frames.get((start + index) as usize).copied()
            }
            Frames::Range { .. } => None,
            Frames::Strip { count } if index < count => {
                let [w, h] = strip_size(count, width, height);

                Some(SpriteFrame::new(if height > width {
                    [0, index * h, w, h]
                } else {
                    [index * w, 0, w, h]
                }))
            }
            Frames::Strip { .. } => None,
        }
    }

    pub(crate) fn size(&self, sheet: &Spritesheet, width: u32, height: u32) -> [u32; 2] {
        match self.frames {
            Frames::Range { start, len } => {
                let end = (start + len) as usize;

                sheet
                    .frames
                    .get(start as usize..end.min(sheet.frames.len()))
                    .unwrap_or_default()
                    .iter()
                    .fold([0, 0], |[w, h], f| {
                        [
                            w.max(f.offset[0] + f.rect[2]),
                            h.max(f.offset[1] + f.rect[3]),
                        ]
                    })
            }
            Frames::Strip { count } => strip_size(count, width, height),
        }
    }
}

fn strip_size(count: u32, width: u32, height: u32) -> [u32; 2] {
    let count = count.max(1);

    if height > width {
        [width, height / count]
    } else {
        [width / count, height]
    }
}

#[derive(Debug, Clone)]
pub struct Spritesheet {
    image: Handle<Image>,
    frames: Vec<SpriteFrame>,
    animations: Vec<(Label, SpriteAnimation)>,
    last: Option<usize>,
    fps: f32,
}

impl Default for Spritesheet {
    fn default() -> Self {
        Self::new()
    }
}

impl Spritesheet {
    pub fn new() -> Self {
        Self {
            image: Handle::INVALID,
            frames: Vec::new(),
            animations: Vec::new(),
            last: None,
            fps: DEFAULT_FPS,
        }
    }

    pub fn grid(image: Handle<Image>, width: u32, height: u32, columns: u32, rows: u32) -> Self {
        Self::new()
            .with_image(image)
            .with_grid(0, 0, width, height, columns, rows)
    }

    pub fn strip(image: Handle<Image>, count: u32, fps: f32) -> Self {
        Self::new().with_strip(Label::default(), image, count, fps)
    }

    pub fn with_image(mut self, image: Handle<Image>) -> Self {
        self.image = image;
        self
    }

    pub fn with_fps(mut self, fps: f32) -> Self {
        self.fps = fps;
        self
    }

    pub fn with_grid(
        mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        columns: u32,
        rows: u32,
    ) -> Self {
        self.frames.reserve((columns * rows) as usize);

        for row in 0..rows {
            for col in 0..columns {
                self.frames.push(SpriteFrame::new([
                    x + col * width,
                    y + row * height,
                    width,
                    height,
                ]));
            }
        }

        self
    }

    pub fn with_frame<F>(mut self, frame: F) -> Self
    where
        F: Into<SpriteFrame>,
    {
        self.frames.push(frame.into());
        self
    }

    pub fn with_frames<I>(mut self, frames: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<SpriteFrame>,
    {
        self.frames.extend(frames.into_iter().map(Into::into));
        self
    }

    pub fn with_animation(self, name: Label, frames: Range<u32>, fps: f32) -> Self {
        self.push(
            name,
            SpriteAnimation {
                image: None,
                frames: Frames::Range {
                    start: frames.start,
                    len: frames.end.saturating_sub(frames.start),
                },
                frame_time: 1.0 / fps,
                looping: true,
            },
        )
    }

    pub fn with_sequence<I>(
        mut self,
        name: Label,
        image: Handle<Image>,
        frames: I,
        fps: f32,
    ) -> Self
    where
        I: IntoIterator,
        I::Item: Into<SpriteFrame>,
    {
        let start = self.frames.len() as u32;

        self.frames.extend(frames.into_iter().map(Into::into));

        let len = self.frames.len() as u32 - start;

        self.push(
            name,
            SpriteAnimation {
                image: Some(image),
                frames: Frames::Range { start, len },
                frame_time: 1.0 / fps,
                looping: true,
            },
        )
    }

    pub fn with_strip(self, name: Label, image: Handle<Image>, count: u32, fps: f32) -> Self {
        self.push(
            name,
            SpriteAnimation {
                image: Some(image),
                frames: Frames::Strip { count },
                frame_time: 1.0 / fps,
                looping: true,
            },
        )
    }

    pub fn once(mut self) -> Self {
        if let Some(i) = self.last {
            self.animations[i].1.looping = false;
        }

        self
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn animation(&self, name: Option<Label>) -> Option<SpriteAnimation> {
        match name {
            Some(name) => self
                .animations
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, a)| *a),
            None => self.animations.first().map(|(_, a)| *a).or_else(|| {
                (!self.frames.is_empty()).then(|| SpriteAnimation {
                    image: None,
                    frames: Frames::Range {
                        start: 0,
                        len: self.frames.len() as u32,
                    },
                    frame_time: 1.0 / self.fps,
                    looping: true,
                })
            }),
        }
    }

    fn push(mut self, name: Label, animation: SpriteAnimation) -> Self {
        let i = match self.animations.iter().position(|(n, _)| *n == name) {
            Some(i) => {
                self.animations[i].1 = animation;
                i
            }
            None => {
                self.animations.push((name, animation));
                self.animations.len() - 1
            }
        };

        self.last = Some(i);
        self
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
