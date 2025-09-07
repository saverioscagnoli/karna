//! This module contains definitions for 2D, 3D, and 4D points.
//!
//! These are used to dereference the [`Vector`] derived types,
//! so they can be accessed using point.x, point.y, point.z, and point.w.

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Point4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
