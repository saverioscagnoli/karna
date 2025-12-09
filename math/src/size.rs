use macros::{Get, Set, With};
use num::{Num, cast::AsPrimitive};
use winit::dpi::PhysicalSize;

use crate::Vector2;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Get, Set, With)]
pub struct Size<T: Num + Copy> {
    #[get(copied)]
    #[set]
    #[with]
    pub width: T,

    #[get(copied)]
    #[set]
    #[with]
    pub height: T,
}

impl<T: Num + Copy + Default> Default for Size<T> {
    fn default() -> Self {
        Self {
            width: T::default(),
            height: T::default(),
        }
    }
}

impl<T: Num + Copy> From<(T, T)> for Size<T> {
    #[inline]
    fn from((width, height): (T, T)) -> Self {
        Self { width, height }
    }
}

impl<T: Num + Copy> From<Size<T>> for (T, T) {
    #[inline]
    fn from(value: Size<T>) -> Self {
        (value.width, value.height)
    }
}

impl<T: Num + Copy> Size<T> {
    #[inline]
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    #[inline]
    pub fn area(&self) -> T {
        self.width * self.height
    }
}

impl<T: Num + Copy> From<PhysicalSize<T>> for Size<T> {
    fn from(value: PhysicalSize<T>) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

impl<T: Num + Copy> From<Size<T>> for PhysicalSize<T> {
    fn from(value: Size<T>) -> Self {
        Self::new(value.width, value.height)
    }
}

impl From<winit::dpi::Size> for Size<u32> {
    fn from(value: winit::dpi::Size) -> Self {
        match value {
            winit::dpi::Size::Physical(size) => Self::new(size.width, size.height),
            winit::dpi::Size::Logical(size) => Self::new(size.width as u32, size.height as u32),
        }
    }
}

impl<T: Num + Copy + AsPrimitive<u32>> From<Size<T>> for winit::dpi::Size {
    fn from(value: Size<T>) -> Self {
        Self::Physical(PhysicalSize::new(value.width.as_(), value.height.as_()))
    }
}

impl<T: Num + Copy + AsPrimitive<f32>> From<Size<T>> for Vector2 {
    fn from(value: Size<T>) -> Self {
        Vector2::new(value.width.as_(), value.height.as_())
    }
}

impl From<Vector2> for Size<f32> {
    fn from(value: Vector2) -> Self {
        Size::new(value.x, value.y)
    }
}
