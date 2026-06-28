#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point2<T> {
    pub x: T,
    pub y: T,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}
