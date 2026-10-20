use core::ops::Add;

use num_traits::Num;

use crate::SdlFloat;
use crate::Vector2;

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Size<T: Num + Copy> {
    pub width: T,
    pub height: T,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_accessors() {
        let s = Size::new(3, 4);
        assert_eq!((s.width, s.height), (3, 4));
        assert_eq!((s.w(), s.h()), (3, 4));
        assert_eq!(s.tuple(), (3, 4));
    }

    #[test]
    fn zero_default_square() {
        assert_eq!(Size::<i32>::zero(), Size::new(0, 0));
        assert_eq!(Size::<i32>::default(), Size::new(0, 0));
        assert_eq!(Size::square(5), Size::new(5, 5));
    }

    #[test]
    fn macro_forms() {
        assert_eq!(crate::size!(2, 3), Size::new(2, 3));
        assert_eq!(crate::size!(4), Size::new(4, 4));
        assert_eq!(crate::size!(2, 3,), Size::new(2, 3));
    }

    #[test]
    fn area() {
        assert_eq!(Size::new(3, 4).area(), 12);
        assert_eq!(Size::new(0, 4).area(), 0);
    }

    #[test]
    fn scale() {
        assert_eq!(Size::new(3, 4).scale(2), Size::new(6, 8));
        let mut s = Size::new(3, 4);
        s.scale_mut(3);
        assert_eq!(s, Size::new(9, 12));
    }

    #[test]
    fn contains() {
        let big = Size::new(10, 10);
        assert!(big.contains(&Size::new(10, 5)));
        assert!(big.contains(&big));
        assert!(!big.contains(&Size::new(11, 1)));
        assert!(!big.contains(&Size::new(1, 11)));
    }

    #[test]
    fn contains_point_is_half_open() {
        let s = Size::new(10, 10);
        let pos = Vector2::new(5, 5);
        assert!(s.contains_point(pos, Vector2::new(5, 5)));
        assert!(s.contains_point(pos, Vector2::new(14, 14)));
        assert!(!s.contains_point(pos, Vector2::new(15, 10)));
        assert!(!s.contains_point(pos, Vector2::new(4, 10)));
    }

    #[test]
    fn is_zero() {
        assert!(Size::new(0, 0).is_zero());
        assert!(!Size::new(0, 1).is_zero());
        assert!(!Size::new(1, 0).is_zero());
    }

    #[test]
    fn aspect_ratio() {
        assert_eq!(Size::new(1920.0f32, 1080.0).aspect_ratio(), 16.0 / 9.0);
    }

    #[test]
    fn fit_scale() {
        let window = Size::new(1280.0f32, 720.0);
        assert_eq!(window.fit_scale(&Size::new(320.0, 180.0)), 4.0);
        assert_eq!(window.fit_scale(&Size::new(320.0, 360.0)), 2.0);
    }

    #[test]
    fn tuple_conversions() {
        let s: Size<i32> = (3, 4).into();
        assert_eq!(s, Size::new(3, 4));
        let t: (i32, i32) = s.into();
        assert_eq!(t, (3, 4));
    }

    #[test]
    fn vector_conversions() {
        let s = Size::from(Vector2::new(3, 4));
        assert_eq!(s, Size::new(3, 4));
        let v: Vector2<i32> = s.into();
        assert_eq!(v, Vector2::new(3, 4));
    }

    #[test]
    fn array_conversion() {
        let a: [i32; 2] = Size::new(3, 4).into();
        assert_eq!(a, [3, 4]);
    }

    #[test]
    fn partial_ord_is_lexicographic() {
        assert!(Size::new(1, 100) < Size::new(2, 0));
    }
}
