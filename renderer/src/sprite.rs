use std::time::Duration;

use assets::Image;
use logging::warn;
use utils::FastHashMap;
use utils::Handle;
use utils::Timer;

use crate::Draw;
use crate::Flip;
use crate::SrcRect;

#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub enum LoopMode {
    Once,
    #[default]
    Loop,
    PingPong,
}

#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
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

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct Animation {
    frames: Vec<Frame>,
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

/// Pixel-space source for a sprite.
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteSource {
    /// Draw the full image.
    #[default]
    Whole,

    /// Draw a sub-rectangle of the image (in pixels, relative to the image).
    Region { x: u32, y: u32, w: u32, h: u32 },
}

impl SpriteSource {
    #[inline]
    pub fn region(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self::Region { x, y, w, h }
    }
}

/// Simple render-time sprite: an atlas image handle + an optional source rect.
///
/// This is intentionally **data-only**. Put movement/transform in your game
/// objects/components; call [`Sprite::draw`] to push geometry.
#[derive(Default)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sprite {
    pub image: Handle<Image>,
    pub source: SpriteSource,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl Sprite {
    #[inline]
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            image,
            source: SpriteSource::Whole,
            flip_x: false,
            flip_y: false,
        }
    }

    #[inline]
    pub fn with_region(mut self, x: u32, y: u32, w: u32, h: u32) -> Self {
        self.source = SpriteSource::Region { x, y, w, h };
        self
    }

    #[inline]
    pub fn draw(&self, draw: &mut Draw, x: f32, y: f32) {
        match self.source {
            SpriteSource::Whole => draw.image(self.image, x, y),
            SpriteSource::Region { x: sx, y: sy, w, h } => {
                if self.flip_x || self.flip_y {
                    draw.image_ex(
                        self.image,
                        [x, y],
                        (w as f32, h as f32),
                        SrcRect { x: sx, y: sy, w, h },
                        Flip {
                            x: self.flip_x,
                            y: self.flip_y,
                        },
                    );
                } else {
                    draw.image_region(self.image, x, y, sx, sy, w, h);
                }
            }
        }
    }
}

/// Runtime state for playing one animation out of an [`Animations`] set.
#[derive(Default)]
#[derive(Debug, Clone)]
pub struct SpriteAnimator {
    animations: Animations,

    current: Option<String>,
    frame_index: usize,
    frame_elapsed: Timer,

    // For ping-pong.
    dir: i32,

    finished: bool,
}

impl SpriteAnimator {
    #[inline]
    pub fn new(animations: Animations) -> Self {
        Self {
            animations,
            current: None,
            frame_index: 0,
            frame_elapsed: Timer::default(),
            dir: 1,
            finished: false,
        }
    }

    /// Returns the name of the currently playing animation, if any.
    #[inline]
    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    /// Start playing `name`.
    ///
    /// - If `restart` is `false` and the same animation is already playing,
    ///   this will **resume** it if it was paused.
    /// - Returns `false` if the animation doesn't exist.
    pub fn play<N: Into<String>>(&mut self, name: N, restart: bool) -> bool {
        let name: String = name.into();

        if !self.animations.contains(&name) {
            return false;
        }

        let same = self.current.as_deref() == Some(name.as_str());

        // Common case: "keep playing" every frame while moving.
        // If we were paused (e.g. idle state), calling `play(..., false)` should
        // resume instead of doing nothing.
        if same && !restart && !self.finished {
            self.frame_elapsed.resume();
            return true;
        }

        // Otherwise (different animation, explicit restart, or we previously
        // finished a `LoopMode::Once` animation), restart from frame 0.
        self.current = Some(name);
        self.frame_index = 0;
        self.frame_elapsed = Timer::default();
        self.dir = 1;
        self.finished = false;

        true
    }

    #[inline]
    pub fn pause(&mut self) {
        self.frame_elapsed.pause();
    }

    #[inline]
    pub fn resume(&mut self) {
        self.frame_elapsed.resume();
    }

    #[inline]
    pub fn stop(&mut self) {
        self.current = None;
        self.frame_index = 0;
        self.frame_elapsed = Timer::default();
        self.dir = 1;
        self.finished = false;
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Advance the animation by `dt`.
    pub fn update(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        if self.current.is_none() {
            return;
        }

        if self.finished {
            return;
        }

        let Some(anim) = self
            .current
            .as_deref()
            .and_then(|name| self.animations.get(name))
            .cloned()
        else {
            // Animation removed/renamed; stop cleanly.
            self.stop();
            return;
        };

        if anim.is_empty() {
            return;
        }

        let len = anim.len();
        let loop_mode = anim.loop_mode();

        // We use `Timer` as an accumulator here.
        // `Timer::tick()` stops updating once it is "finished", and `Timer::default()`
        // has a 0 duration (always finished), so we use `tick_unbounded()`.
        self.frame_elapsed.tick_unbounded(dt);

        // Handle big dt: step multiple frames if needed.
        //
        // Important guard: a frame duration of ZERO would otherwise cause an
        // infinite loop (we'd subtract 0 forever).
        let mut steps = 0usize;
        let max_steps = (len.saturating_mul(8)).max(32).min(2048);

        while !self.finished {
            steps += 1;
            if steps > max_steps {
                // Don't get stuck; drop any remaining accumulated time.
                self.frame_elapsed.reset();
                break;
            }

            let Some(frame) = anim.frame(self.frame_index) else {
                // Shouldn't happen, but keep it robust.
                self.frame_index = 0;
                self.frame_elapsed.reset();
                break;
            };

            if frame.duration.is_zero() {
                self.advance(loop_mode, len);
                continue;
            }

            if self.frame_elapsed.elapsed() < frame.duration.as_secs_f32() {
                break;
            }

            // Consume the current frame duration and advance.
            self.frame_elapsed.subtract(frame.duration.as_secs_f32());
            self.advance(loop_mode, len);
        }
    }

    fn advance(&mut self, loop_mode: LoopMode, len: usize) {
        if len <= 1 {
            self.frame_index = 0;
            return;
        }

        match loop_mode {
            LoopMode::Loop => {
                self.frame_index = (self.frame_index + 1) % len;
            }
            LoopMode::Once => {
                if self.frame_index + 1 >= len {
                    self.frame_index = len - 1;
                    self.finished = true;
                } else {
                    self.frame_index += 1;
                }
            }
            LoopMode::PingPong => {
                // dir: +1 forward, -1 backward
                let next = self.frame_index as i32 + self.dir;
                if next >= len as i32 {
                    self.dir = -1;
                    self.frame_index = (len - 2).max(0);
                } else if next < 0 {
                    self.dir = 1;
                    self.frame_index = 1;
                } else {
                    self.frame_index = next as usize;
                }
            }
        }
    }

    /// The current animation frame.
    #[inline]
    pub fn frame(&self) -> Option<Frame> {
        let anim = self
            .current
            .as_deref()
            .and_then(|name| self.animations.get(name))?;
        anim.frame(self.frame_index)
    }

    #[inline]
    pub fn set_frame(&mut self, frame_index: usize) -> bool {
        let Some(name) = self.current.as_deref() else {
            warn!(
                "set_frame({}) called but no animation is currently selected",
                frame_index
            );
            return false;
        };

        let Some(anim) = self.animations.get(name) else {
            warn!(
                "set_frame({}) called but current animation '{}' no longer exists",
                frame_index, name
            );
            return false;
        };

        if frame_index >= anim.len() {
            warn!(
                "frame_index {} is out of bounds for animation '{}' of length {}",
                frame_index,
                name,
                anim.len()
            );
            return false;
        }

        self.frame_index = frame_index;
        self.frame_elapsed.reset();
        self.finished = false;

        true
    }
}

/// Convenience wrapper: an image handle + animator + flips.
#[derive(Default)]
#[derive(Debug, Clone)]
pub struct AnimatedSprite {
    pub image: Handle<Image>,
    pub animator: SpriteAnimator,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl AnimatedSprite {
    #[inline]
    pub fn new(image: Handle<Image>, animations: Animations) -> Self {
        Self {
            image,
            animator: SpriteAnimator::new(animations),
            flip_x: false,
            flip_y: false,
        }
    }

    #[inline]
    pub fn update(&mut self, dt: f32) {
        self.animator.update(dt);
    }

    #[inline]
    pub fn draw(&self, draw: &mut Draw, x: f32, y: f32) {
        let Some(frame) = self.animator.frame() else {
            // If nothing is playing, fall back to drawing the full image.
            draw.image(self.image, x, y);
            return;
        };

        draw.image_ex(
            self.image,
            [x, y],
            (frame.width as f32, frame.height as f32),
            SrcRect {
                x: frame.x,
                y: frame.y,
                w: frame.width,
                h: frame.height,
            },
            Flip {
                x: self.flip_x,
                y: self.flip_y,
            },
        );
    }

    /// Draw the sprite using an anchor point inside the current frame.
    ///
    /// `anchor_x` / `anchor_y` are in normalized frame space:
    /// - (0, 0) = top-left
    /// - (0.5, 1.0) = bottom-center
    ///
    /// This helps reduce jitter when frames have different sizes, because you
    /// can keep e.g. the "feet" locked to a world position.
    #[inline]
    pub fn draw_aligned(&self, draw: &mut Draw, x: f32, y: f32, anchor_x: f32, anchor_y: f32) {
        let Some(frame) = self.animator.frame() else {
            draw.image(self.image, x, y);
            return;
        };

        let w = frame.width as f32;
        let h = frame.height as f32;

        let dx = x - w * anchor_x;
        let dy = y - h * anchor_y;

        draw.image_ex(
            self.image,
            [dx, dy],
            (w, h),
            SrcRect {
                x: frame.x,
                y: frame.y,
                w: frame.width,
                h: frame.height,
            },
            Flip {
                x: self.flip_x,
                y: self.flip_y,
            },
        );
    }
}
