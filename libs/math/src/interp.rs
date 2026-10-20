use core::ops::Add;
use core::ops::Mul;
use core::ops::Sub;

pub trait Lerp: Copy + Add<Output = Self> + Sub + Mul<f32, Output = Self> {
    fn lerp(self, to: Self, t: f32) -> Self {
        self * (1.0 - t) + to * t
    }
}

impl<T: Copy + Add<Output = Self> + Sub + Mul<f32, Output = Self>> Lerp for T {}
