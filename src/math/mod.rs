mod interpolation;
mod random;
mod size;
mod vec2;
mod vec3;
mod vec4;

mod mat4;

pub(crate) mod circles;

pub use interpolation::{Easing, Tween};
pub use mat4::Mat4;
pub use random::{coin_flip, pick, pick_mut, rng};
pub use size::Size;
pub use vec2::Vec2;
pub use vec3::Vec3;
pub use vec4::Vec4;

pub trait ToF32 {
    fn to_f32(&self) -> f32;
}

impl ToF32 for f32 {
    fn to_f32(&self) -> f32 {
        *self
    }
}

impl ToF32 for f64 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for i32 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for i64 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for u32 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for u64 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for usize {
    fn to_f32(&self) -> f32 {
        *self as f32
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

impl ToU32 for f64 {
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for i32 {
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for i64 {
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for u32 {
    fn to_u32(&self) -> u32 {
        *self
    }
}

impl ToU32 for u64 {
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for usize {
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}
