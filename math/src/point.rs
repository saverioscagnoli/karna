use macros_derive::{Getters, Setters};

#[rustfmt::skip]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Getters, Setters)]
pub struct Point2 {
    #[get(copied)]
    #[set]
    pub x: f32,

    #[get(copied)]
    #[set]
    pub y: f32,
}

#[rustfmt::skip]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Getters, Setters)]
pub struct Point3 {
    #[get(copied)]
    #[set]
    pub x: f32,

    #[get(copied)]
    #[set]
    pub y: f32,

    #[get(copied)]
    #[set]
    pub z: f32,
}

#[rustfmt::skip]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Getters, Setters)]
pub struct Point4 {
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
