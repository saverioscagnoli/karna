use std::time::Duration;

use assets::Image;
use utils::Handle;

use crate::Draw;
use crate::sprite::animation::Animations;
use crate::sprite::animation::Frame;
use crate::sprite::animation::LoopMode;

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

    /// Flip the sprite without changing transforms (UV mirror).
    pub flip_x: bool,

    /// Flip the sprite without changing transforms (UV mirror).
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
    pub fn with_source(mut self, source: SpriteSource) -> Self {
        self.source = source;
        self
    }

    #[inline]
    pub fn with_region(mut self, x: u32, y: u32, w: u32, h: u32) -> Self {
        self.source = SpriteSource::Region { x, y, w, h };
        self
    }

    #[inline]
    pub fn with_flip_x(mut self, flip: bool) -> Self {
        self.flip_x = flip;
        self
    }

    #[inline]
    pub fn with_flip_y(mut self, flip: bool) -> Self {
        self.flip_y = flip;
        self
    }

    #[inline]
    pub fn draw(&self, draw: &mut Draw, x: f32, y: f32) {
        match self.source {
            SpriteSource::Whole => draw.image(self.image, x, y),
            SpriteSource::Region { x: sx, y: sy, w, h } => {
                if self.flip_x || self.flip_y {
                    draw.image_region_ex(
                        self.image,
                        x,
                        y,
                        w as f32,
                        h as f32,
                        sx,
                        sy,
                        w,
                        h,
                        self.flip_x,
                        self.flip_y,
                    );
                } else {
                    draw.image_region(self.image, x, y, sx, sy, w, h);
                }
            }
        }
    }
}

/// Runtime state for playing one animation out of an [`Animations`] set.
///
/// This is the "solid" part: it is frame-rate independent, handles large `dt`
/// by stepping multiple frames, and supports loop modes.
#[derive(Default)]
#[derive(Debug, Clone)]
pub struct SpriteAnimator {
    animations: Animations,

    current: Option<String>,
    frame_index: usize,
    frame_elapsed: Duration,

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
            frame_elapsed: Duration::ZERO,
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
    /// - If `restart` is `false` and the same animation is already playing, this
    ///   does nothing.
    /// - Returns `false` if the animation doesn't exist.
    pub fn play(&mut self, name: impl Into<String>, restart: bool) -> bool {
        let name: String = name.into();

        if !self.animations.contains(&name) {
            return false;
        }

        if !restart && self.current.as_deref() == Some(name.as_str()) {
            return true;
        }

        self.current = Some(name);
        self.frame_index = 0;
        self.frame_elapsed = Duration::ZERO;
        self.dir = 1;
        self.finished = false;

        true
    }

    #[inline]
    pub fn stop(&mut self) {
        self.current = None;
        self.frame_index = 0;
        self.frame_elapsed = Duration::ZERO;
        self.dir = 1;
        self.finished = false;
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Advance the animation by `dt`.
    pub fn update(&mut self, dt: Duration) {
        if dt.is_zero() {
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

        self.frame_elapsed = self.frame_elapsed.saturating_add(dt);

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
                self.frame_elapsed = Duration::ZERO;
                break;
            }

            let Some(frame) = anim.frame(self.frame_index) else {
                // Shouldn't happen, but keep it robust.
                self.frame_index = 0;
                self.frame_elapsed = Duration::ZERO;
                break;
            };

            if frame.duration.is_zero() {
                self.advance(loop_mode, len);
                continue;
            }

            if self.frame_elapsed < frame.duration {
                break;
            }

            // Consume the current frame duration and advance.
            self.frame_elapsed -= frame.duration;
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
    pub fn update(&mut self, dt: Duration) {
        self.animator.update(dt);
    }

    #[inline]
    pub fn draw(&self, draw: &mut Draw, x: f32, y: f32) {
        let Some(frame) = self.animator.frame() else {
            // If nothing is playing, fall back to drawing the full image.
            draw.image(self.image, x, y);
            return;
        };

        draw.image_region_ex(
            self.image,
            x,
            y,
            frame.width as f32,
            frame.height as f32,
            frame.x,
            frame.y,
            frame.width,
            frame.height,
            self.flip_x,
            self.flip_y,
        );
    }
}
