use anyhow::anyhow;
use sdl2::video::{FullscreenType, Window as SdlWindow, WindowPos};

use crate::{
    math::{Size, Vec2},
    traits::ToU32,
};

#[derive(Clone)]
pub struct Window(pub(crate) SdlWindow);

impl Window {
    pub(crate) fn new(
        title: String,
        width: u32,
        height: u32,
        video: &sdl2::VideoSubsystem,
    ) -> Self {
        // Sets up the inital window configuration.
        // It's hidden by default, and it's shown after the `load` function from
        // the `Load` trait is fired, so the user can set things up as they want before the window is shown.
        //
        // Example: if this wasnt the case, if the user wanted to have a borderless window,
        // it would first show the window with borders and then remove them, which would be a bad experience.
        let window = video
            .window(title.as_str(), width.to_u32(), height.to_u32())
            .position_centered()
            .hidden()
            .build()
            .unwrap();

        Self(window)
    }

    /// Returns the size of the window.
    /// Uses a [`Size`] struct to represent the size.
    pub fn size(&self) -> Size {
        self.0.size().into()
    }

    /// Sets the size of the window.
    /// Uses a [`Size`] struct to represent the size.
    pub fn set_size<S: Into<Size>>(&mut self, size: S) -> anyhow::Result<()> {
        let size = size.into();

        self.0
            .set_size(size.width, size.height)
            .map_err(|e| anyhow!(e))
    }

    /// Returns the minimum size of the window.
    /// Uses a [`Size`] struct to represent the size.
    ///
    /// The window cannot be resized to a size smaller than this.
    pub fn min_size(&self) -> Size {
        self.0.minimum_size().into()
    }

    /// Sets the minimum size of the window.
    /// Uses a [`Size`] struct to represent the size.
    pub fn set_min_size<S: Into<Size>>(&mut self, size: S) -> anyhow::Result<()> {
        let size = size.into();

        self.0
            .set_minimum_size(size.width, size.height)
            .map_err(|e| anyhow!(e))
    }

    /// Returns the maximum size of the window.
    /// Uses a [`Size`] struct to represent the size.
    ///
    /// The window cannot be resized to a size larger than this.
    pub fn max_size(&self) -> Size {
        self.0.maximum_size().into()
    }

    /// Sets the maximum size of the window.
    /// Uses a [`Size`] struct to represent the size.
    pub fn set_max_size<S: Into<Size>>(&mut self, size: S) -> anyhow::Result<()> {
        let size = size.into();

        self.0
            .set_maximum_size(size.width, size.height)
            .map_err(|e| anyhow!(e))
    }

    /// Returns the title of the window.
    pub fn title(&self) -> &str {
        self.0.title()
    }

    /// Sets the title of the window.
    pub fn set_title<T: ToString>(&mut self, title: T) -> anyhow::Result<()> {
        self.0.set_title(&title.to_string()).map_err(|e| anyhow!(e))
    }

    /// Returns the position of the window.
    /// The position is represented as a [`Vec2`].
    pub fn position(&self) -> Vec2 {
        self.0.position().into()
    }

    /// Sets the position of the window.
    /// Uses a [`Vec2`] to represent the position.
    pub fn set_position<P: Into<Vec2>>(&mut self, pos: P) {
        let pos = pos.into();

        self.0.set_position(
            WindowPos::Positioned(pos.x as i32),
            WindowPos::Positioned(pos.y as i32),
        );
    }

    /// Centers the window on the screen.
    pub fn center(&mut self) {
        self.0
            .set_position(WindowPos::Centered, WindowPos::Centered);
    }

    /// Sets whether the window can be resized by the user.
    pub fn set_resizable(&mut self, v: bool) {
        self.0.set_resizable(v);
    }

    /// Sets the window to fullscreen mode.
    pub fn fullscren(&mut self) -> anyhow::Result<()> {
        self.0
            .set_fullscreen(FullscreenType::Desktop)
            .map_err(|e| anyhow!(e))
    }

    /// Sets the window to windowed mode.
    pub fn windowed(&mut self) -> anyhow::Result<()> {
        self.0
            .set_fullscreen(FullscreenType::Off)
            .map_err(|e| anyhow!(e))
    }

    /// Returns a boolean indicating whether the window is in fullscreen mode.
    pub fn is_fullscreen(&self) -> bool {
        self.0.fullscreen_state() == FullscreenType::Desktop
    }

    /// Toggles between fullscreen and windowed mode.
    pub fn toggle_fullscreen(&mut self) -> anyhow::Result<()> {
        if self.is_fullscreen() {
            self.windowed()
        } else {
            self.fullscren()
        }
    }

    /// Shows the window.
    pub fn show(&mut self) {
        self.0.show();
    }

    /// Hides the window.
    pub fn hide(&mut self) {
        self.0.hide();
    }

    /// Returns a boolean indicating whether the window is always on top.
    pub fn is_always_on_top(&self) -> bool {
        self.0.is_always_on_top()
    }

    /// Sets the window to be always on top or not.
    pub fn set_always_on_top(&mut self, v: bool) {
        self.0.set_always_on_top(v);
    }

    /// Gets the opacity of the window.
    /// Only works on platforms that support it.
    pub fn opacity(&self) -> anyhow::Result<f32> {
        self.0.opacity().map_err(|e| anyhow!(e))
    }

    /// Sets the opacity of the window.
    /// Only works on platforms that support it.
    ///
    /// The value should be between 0.0 (fully transparent) and 1.0 (fully opaque).
    pub fn set_opacity(&mut self, v: f32) -> anyhow::Result<()> {
        self.0.set_opacity(v).map_err(|e| anyhow!(e))
    }

    /// Sets the window to be borderless or not.
    pub fn set_decorations(&mut self, v: bool) {
        self.0.set_bordered(v);
    }
}
