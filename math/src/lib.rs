mod matrix;
mod point;
mod quaternion;
mod size;
mod vector;

pub use matrix::*;
pub use point::*;
pub use quaternion::*;
pub use size::*;
pub use vector::*;

use macros::impl_as_f32;

pub trait AsF32 {
    fn as_f32(&self) -> f32;
}

impl_as_f32!(i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 usize isize);
