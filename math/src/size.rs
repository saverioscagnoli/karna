use num::Num;
use winit::dpi::{LogicalSize, PhysicalSize};

use crate::{ToF32, ToU32};

/// Struct to represent a size with width and height.
/// Useful in many contexts, such as representing the size of a window or a rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size<T: Num + Copy + PartialOrd> {
    pub width: T,
    pub height: T,
}

impl<T: Num + Copy + PartialOrd> Size<T> {
    /// Creates a new Size with the given width and height.
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    /// Creates a new square Size with the given size.
    pub fn square(size: T) -> Self {
        Self {
            width: size,
            height: size,
        }
    }

    /// Returns the area of the Size.
    /// Basic rectangle stuff.
    pub fn area(&self) -> T {
        self.width * self.height
    }

    /// Returns true if the Size is square.
    pub fn is_square(&self) -> bool {
        self.width == self.height
    }

    /// Returns true if the Size is a rectangle
    /// where the width is greater than the height.
    pub fn is_landscape(&self) -> bool {
        self.width > self.height
    }

    /// Returns true if the Size is a rectangle
    /// where the height is greater than the width.
    pub fn is_portrait(&self) -> bool {
        self.width < self.height
    }

    /// Swaps the width and height of the Size.
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.width, &mut self.height);
    }
}

impl Size<f32> {
    /// Returns the aspect ratio of the Size.
    /// The aspect ratio is the width divided by the height.
    pub fn aspect_ratio(&self) -> f32 {
        self.width / self.height
    }
}

impl Size<u32> {
    /// Returns the aspect ratio of the Size.
    /// The aspect ratio is the width divided by the height.
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

impl Size<i32> {
    /// Returns the aspect ratio of the Size.
    /// The aspect ratio is the width divided by the height.
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

impl<T: Num + Copy + PartialOrd> Default for Size<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            width: T::default(),
            height: T::default(),
        }
    }
}

/// (f32, f32) -> Size<f32>
impl<F: ToF32> From<(F, F)> for Size<f32> {
    fn from((width, height): (F, F)) -> Self {
        Self {
            width: width.to_f32(),
            height: height.to_f32(),
        }
    }
}

/// Square size f32 -> Size<f32>
impl From<f32> for Size<f32> {
    fn from(size: f32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}

/// (u32, u32) -> Size<u32>
impl<U: ToU32> From<(U, U)> for Size<u32> {
    fn from((width, height): (U, U)) -> Self {
        Self {
            width: width.to_u32(),
            height: height.to_u32(),
        }
    }
}

/// Square size u32 -> Size<u32>
impl From<u32> for Size<u32> {
    fn from(size: u32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}

impl<T: Num + Copy + PartialOrd> From<Size<T>> for LogicalSize<T> {
    fn from(size: Size<T>) -> Self {
        LogicalSize::new(size.width, size.height)
    }
}

impl<T: Num + Copy + PartialOrd> From<LogicalSize<T>> for Size<T> {
    fn from(size: LogicalSize<T>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

impl<T: Num + Copy + PartialOrd> From<Size<T>> for PhysicalSize<T> {
    fn from(size: Size<T>) -> Self {
        PhysicalSize::new(size.width, size.height)
    }
}

impl<T: Num + Copy + PartialOrd> From<PhysicalSize<T>> for Size<T> {
    fn from(size: PhysicalSize<T>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size() {
        let size: Size<u32> = Size::new(10, 20);
        assert_eq!(size.width, 10);
        assert_eq!(size.height, 20);
        assert_eq!(size.area(), 200);
        assert_eq!(size.aspect_ratio(), 0.5);
        assert!(!size.is_square());
        assert!(!size.is_landscape());
        assert!(size.is_portrait());

        let mut size = Size::new(20, 10);
        size.swap();
        assert_eq!(size.width, 10);
        assert_eq!(size.height, 20);
    }

    #[test]
    fn test_size_square() {
        let size: Size<u32> = Size::square(10);
        assert_eq!(size.width, 10);
        assert_eq!(size.height, 10);
        assert_eq!(size.area(), 100);
        assert_eq!(size.aspect_ratio(), 1.0);
        assert!(size.is_square());
        assert!(!size.is_landscape());
        assert!(!size.is_portrait());
    }

    #[test]
    fn test_size_from_tuple() {
        let size: Size<f32> = (10.0, 20.0).into();
        assert_eq!(size.width, 10.0);
        assert_eq!(size.height, 20.0);

        let size: Size<u32> = (10, 20).into();
        assert_eq!(size.width, 10);
        assert_eq!(size.height, 20);
    }

    #[test]
    fn test_size_from_scalar() {
        let size: Size<f32> = 10.0.into();
        assert_eq!(size.width, 10.0);
        assert_eq!(size.height, 10.0);

        let size: Size<u32> = 10.into();
        assert_eq!(size.width, 10);
        assert_eq!(size.height, 10);
    }
}
