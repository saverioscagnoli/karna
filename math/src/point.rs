use macros::{Get, Set};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Get, Set)]
pub struct Point2 {
    #[get(copied)]
    #[set]
    pub x: f32,

    #[get(copied)]
    #[set]
    pub y: f32,
}

impl Point2 {
    #[inline]
    pub fn set(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Get, Set)]
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

impl Point3 {
    #[inline]
    pub fn set(&mut self, x: f32, y: f32, z: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Get, Set)]
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

impl Point4 {
    #[inline]
    pub fn set(&mut self, x: f32, y: f32, z: f32, w: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
        self.w = w;
    }
}
