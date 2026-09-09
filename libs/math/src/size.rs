use core::fmt;
use core::ops::Add;

use num_traits::Num;

use crate::SdlFloat;
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
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn zero() -> Self {
        Self::new(T::zero(), T::zero())
    }

    pub const fn square(size: T) -> Self {
        Self::new(size, size)
    }

    pub const fn w(&self) -> T {
        self.width
    }

    pub const fn h(&self) -> T {
        self.height
    }

    pub const fn tuple(&self) -> (T, T) {
        (self.width, self.height)
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

    pub fn contains_point(&self, size_pos: Vector2<T>, p: Vector2<T>) -> bool
    where
        T: PartialOrd + Copy + Add<Output = T>,
    {
        p.x >= size_pos.x
            && p.x < size_pos.x + self.width
            && p.y >= size_pos.y
            && p.y < size_pos.y + self.height
    }

    pub fn is_zero(&self) -> bool {
        self.width == T::zero() && self.height == T::zero()
    }
}

impl<T: Num + SdlFloat> Size<T> {
    pub fn aspect_ratio(&self) -> T {
        self.width / self.height
    }

    pub fn fit_scale(&self, other: &Self) -> T {
        (self.width / other.width).sdl_min(self.height / other.height)
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

#[macro_export]
macro_rules! size {
    ($w:expr, $h:expr $(,)?) => {
        $crate::Size::new($w, $h)
    };
    ($s:expr $(,)?) => {
        $crate::Size::square($s)
    };
}
