use std::fmt;

use num_traits::Float;
use num_traits::Num;
use winit::dpi::PhysicalSize;

use crate::Vector2;

#[derive(Default)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Size<T: Num + Copy> {
    pub width: T,
    pub height: T,
}

impl<T: fmt::Display + Num + Copy> fmt::Debug for Size<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl<T: Num + Copy> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn w(&self) -> T {
        self.width
    }

    pub fn h(&self) -> T {
        self.height
    }

    pub fn area(&self) -> T {
        self.width * self.height
    }

    pub fn scale(&self, factor: T) -> Self {
        Self::new(self.width * factor, self.height * factor)
    }

    pub fn scale_mut(&mut self, factor: T) {
        self.width = self.width * factor;
        self.height = self.height * factor;
    }

    pub fn contains(&self, other: &Self) -> bool
    where
        T: PartialOrd,
    {
        self.width >= other.width && self.height >= other.height
    }

    pub fn is_zero(&self) -> bool {
        self.width == T::zero() && self.height == T::zero()
    }
}

impl<T: Float> Size<T> {
    pub fn aspect_ratio(&self) -> T {
        self.width / self.height
    }

    pub fn fit_scale(&self, other: &Self) -> T {
        (self.width / other.width).min(self.height / other.height)
    }
}

impl Size<u32> {
    pub fn as_f32(&self) -> Size<f32> {
        Size::new(self.width as f32, self.height as f32)
    }
}

impl<T: Num + Copy> From<(T, T)> for Size<T> {
    fn from((width, height): (T, T)) -> Self {
        Self { width, height }
    }
}

impl<T: Num + Copy> Into<(T, T)> for Size<T> {
    fn into(self) -> (T, T) {
        (self.width, self.height)
    }
}

impl<T: Num + Copy> From<Vector2<T>> for Size<T> {
    fn from(v: Vector2<T>) -> Self {
        Self::new(v.x, v.y)
    }
}

impl<T: Num + Copy> Into<Vector2<T>> for Size<T> {
    fn into(self) -> Vector2<T> {
        Vector2::new(self.width, self.height)
    }
}

impl<T: Num + Copy> Into<[T; 2]> for Size<T> {
    fn into(self) -> [T; 2] {
        [self.width, self.height]
    }
}

impl<T: Num + Copy> From<PhysicalSize<T>> for Size<T> {
    fn from(value: PhysicalSize<T>) -> Self {
        Self::new(value.width, value.height)
    }
}

impl<T: Num + Copy> Into<PhysicalSize<T>> for Size<T> {
    fn into(self) -> PhysicalSize<T> {
        PhysicalSize::new(self.width, self.height)
    }
}
