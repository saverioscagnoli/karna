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
