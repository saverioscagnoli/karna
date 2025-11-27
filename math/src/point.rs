use crate::{Vector2, Vector3, Vector4};
use macros_derive::{Getters, Setters, With};

#[rustfmt::skip]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Getters, Setters, With)]
pub struct Point2 {
    #[get(copied)]
    #[set]
    #[with]
    pub x: f32,

    #[get(copied)]
    #[set]
    #[with]
    pub y: f32,
}

impl Point2 {
    pub fn to_vector(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }
}

#[rustfmt::skip]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Getters, Setters, With)]
pub struct Point3 {
    #[get(copied)]
    #[set]
    #[with]
    pub x: f32,

    #[get(copied)]
    #[set]
    #[with]
    pub y: f32,

    #[get(copied)]
    #[set]
    #[with]
    pub z: f32,
}

impl Point3 {
    pub fn to_vector(self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }
}

#[rustfmt::skip]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Getters, Setters, With)]
pub struct Point4 {
    #[get(copied)]
    #[set]
    #[with]
    pub x: f32,

    #[get(copied)]
    #[set]
    #[with]
    pub y: f32,

    #[get(copied)]
    #[set]
    #[with]
    pub z: f32,

    #[get(copied)]
    #[set]
    #[with]
    pub w: f32,
}

impl Point4 {
    pub fn to_vector(self) -> Vector4 {
        Vector4::new(self.x, self.y, self.z, self.w)
    }
}
