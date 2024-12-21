use std::ops::Div;

use sdl2::rect::FRect;

use super::{ToF32, ToU32, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new<U: ToU32>(width: U, height: U) -> Self {
        Self {
            width: width.to_u32(),
            height: height.to_u32(),
        }
    }

    pub(crate) fn to_frect(&self, vec: Vec2) -> FRect {
        FRect::new(vec.x, vec.y, self.width as f32, self.height as f32)
    }

    pub fn center_x(&self) -> f32 {
        self.width as f32 / 2.0
    }

    pub fn center_y(&self) -> f32 {
        self.height as f32 / 2.0
    }

    pub fn center(&self) -> Vec2 {
        let x = self.center_x();
        let y = self.center_y();

        (x, y).into()
    }

    pub fn fit_center_x(&self, size: u32) -> f32 {
        (self.width as f32 - size as f32) / 2.0
    }

    pub fn fit_center_y(&self, size: u32) -> f32 {
        (self.height as f32 - size as f32) / 2.0
    }

    /// Returns the position to fit the size in the center of the size.
    pub fn fit_center(&self, size: Size) -> Vec2 {
        let x = self.fit_center_x(size.width);
        let y = self.fit_center_y(size.height);

        (x, y).into()
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}

impl Default for Size {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
}

impl<T: ToU32> From<(T, T)> for Size {
    fn from((width, height): (T, T)) -> Self {
        Self {
            width: width.to_u32(),
            height: height.to_u32(),
        }
    }
}

impl From<Size> for (u32, u32) {
    fn from(size: Size) -> Self {
        (size.width, size.height)
    }
}

impl<F: ToF32> Div<F> for Size {
    type Output = Self;

    fn div(self, rhs: F) -> Self::Output {
        Self {
            width: (self.width as f32 / rhs.to_f32()) as u32,
            height: (self.height as f32 / rhs.to_f32()) as u32,
        }
    }
}
