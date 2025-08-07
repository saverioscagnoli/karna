mod chance;
mod matrix;
mod points;
mod size;
mod vector;

pub use chance::*;
pub use matrix::*;
pub use points::*;
pub use size::*;
pub use vector::*;

pub trait ToF32 {
    fn to_f32(&self) -> f32;
}

pub trait ToU32 {
    fn to_u32(&self) -> u32;
}

macro_rules! impl_to_f32 {
    ($($t:ty),*) => {
        $(
            impl ToF32 for $t {
                fn to_f32(&self) -> f32 {
                    *self as f32
                }
            }
        )*
    };
}

impl_to_f32!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

macro_rules! impl_to_u32 {
    ($($t:ty),*) => {
        $(
            impl ToU32 for $t {
                fn to_u32(&self) -> u32 {
                    *self as u32
                }
            }
        )*
    };
}

impl_to_u32!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
