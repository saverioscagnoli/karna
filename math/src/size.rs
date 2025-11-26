use std::ops::Div;

use macros_derive::{Getters, Setters};
use num::Num;

#[rustfmt::skip]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Getters,Setters)]
pub struct Size<T: Num + Copy + PartialOrd> {
    #[get(copied)]
    #[set]
    pub width: T,

    #[get(copied)]
    #[set]
    pub height: T,
}

impl<T: Num + Copy + PartialOrd> Size<T> {
    #[inline]
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn area(&self) -> T {
        self.width * self.height
    }
}

impl<T: Num + Div + Copy + PartialOrd> Size<T> {
    /// Returns the aspect ratio of the Size.
    /// The aspect ratio is the width divided by the height.
    pub fn aspect_ratio(&self) -> T {
        self.width / self.height
    }
}

impl<T: Num + Copy + PartialOrd> From<(T, T)> for Size<T> {
    fn from((width, height): (T, T)) -> Self {
        Self { width, height }
    }
}

impl<T: Num + Copy + PartialOrd> From<Size<T>> for (T, T) {
    fn from(value: Size<T>) -> Self {
        (value.width, value.height)
    }
}
