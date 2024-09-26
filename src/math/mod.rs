pub trait ToF32 {
    fn to_f32(&self) -> f32;
}

impl ToF32 for u32 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for i32 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for f32 {
    fn to_f32(&self) -> f32 {
        *self
    }
}

pub trait ToU32 {
    fn to_u32(&self) -> u32;
}

impl ToU32 for f32 {
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for i32 {
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for u32 {
    fn to_u32(&self) -> u32 {
        *self
    }
}

mod rng;
mod size;
mod vector2;

pub(crate) mod utils;

pub use rng::{pick, rng};
pub use size::Size;
pub use vector2::Vector2;
