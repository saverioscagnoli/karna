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
