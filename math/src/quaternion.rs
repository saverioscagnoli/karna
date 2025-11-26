use macros_derive::{Getters, Setters};

#[rustfmt::skip]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Getters, Setters)]
pub struct Quaternion {
    #[get(copied)]
    #[set]
    pub x: f32,
    #[get(copied)]
    #[set]
    pub y: f32,
    #[get(copied)]
    #[set]
    pub z: f32,
    #[get(copied)]
    #[set]
    pub w: f32,
}

impl Quaternion {
    pub const fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z, w } // Constructor still takes w first for convenience
    }

    pub fn from_euler_angles(x: f32, y: f32, z: f32) -> Self {
        let (cx, cy, cz) = ((x / 2.0).cos(), (y / 2.0).cos(), (z / 2.0).cos());
        let (sx, sy, sz) = ((x / 2.0).sin(), (y / 2.0).sin(), (z / 2.0).sin());

        Self {
            w: cx * cy * cz + sx * sy * sz,
            x: sx * cy * cz - cx * sy * sz,
            y: cx * sy * cz + sx * cy * sz,
            z: cx * cy * sz - sx * sy * cz,
        }
    }
}
