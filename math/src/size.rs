use macros::{Get, Set, With};
use num::Num;
use winit::dpi::PhysicalSize;

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
