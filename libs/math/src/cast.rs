use num_traits::Num;

use crate::Size;

pub trait CastFrom<T> {
    fn cast_from(value: T) -> Self;
}

#[macro_export]
macro_rules! cast_matrix {
    // Expand the destination list for a single source type.
    (@row $src:ty, [$($dst:ty),+]) => {
        $(
            impl CastFrom<$src> for $dst {
                #[inline]
                fn cast_from(value: $src) -> Self {
                    value as Self
                }
            }
        )+
    };

    // One row per source type. `$dsts` is a single `tt`, so it survives
    // the repetition intact instead of being zipped against `$src`.
    (@rows [$($src:ty),+], $dsts:tt) => {
        $( cast_matrix!(@row $src, $dsts); )+
    };

    // Entry point: hand the same list to both axes.
    ($($t:ty),+ $(,)?) => {
        cast_matrix!(@rows [$($t),+], [$($t),+]);
    };
}

cast_matrix!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

impl<T: Num + Copy> Size<T> {
    #[inline]
    pub fn map<U, F>(self, mut f: F) -> Size<U>
    where
        U: Num + Copy,
        F: FnMut(T) -> U,
    {
        Size::new(f(self.width), f(self.height))
    }

    #[inline]
    pub fn cast<U>(self) -> Size<U>
    where
        U: Num + Copy + CastFrom<T>,
    {
        self.map(U::cast_from)
    }

    #[inline]
    pub fn try_cast<U>(self) -> Result<Size<U>, U::Error>
    where
        U: Num + Copy + TryFrom<T>,
    {
        Ok(Size::new(
            U::try_from(self.width)?,
            U::try_from(self.height)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cast_from_behaves_like_as() {
        assert_eq!(i32::cast_from(3.9f32), 3);
        assert_eq!(i32::cast_from(-3.9f32), -3);
        assert_eq!(u8::cast_from(300i32), 44);
        assert_eq!(u8::cast_from(-1.0f32), 0);
        assert_eq!(f32::cast_from(2i64), 2.0);
        assert_eq!(f64::cast_from(0.5f32), 0.5);
        assert_eq!(usize::cast_from(7u8), 7);
        assert_eq!(i8::cast_from(i128::MAX), -1);
    }

    #[test]
    fn cast_identity() {
        assert_eq!(i32::cast_from(42i32), 42);
        assert_eq!(f32::cast_from(1.5f32), 1.5);
    }

    #[test]
    fn size_map() {
        assert_eq!(Size::new(2, 3).map(|v| v * 10), Size::new(20, 30));
        assert_eq!(Size::new(2, 3).map(|v| v as f32 + 0.5), Size::new(2.5, 3.5));
    }

    #[test]
    fn size_cast() {
        assert_eq!(
            Size::new(1280u32, 720).cast::<f32>(),
            Size::new(1280.0, 720.0)
        );
        assert_eq!(Size::new(2.9f32, -1.2).cast::<i32>(), Size::new(2, -1));
    }

    #[test]
    fn size_try_cast() {
        assert_eq!(
            Size::new(10i32, 20).try_cast::<u8>(),
            Ok(Size::new(10u8, 20))
        );
        assert!(Size::new(300i32, 1).try_cast::<u8>().is_err());
        assert!(Size::new(1i32, -1).try_cast::<u32>().is_err());
    }
}
