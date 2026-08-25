#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy> Point2<T> {
    pub fn tuple(&self) -> (T, T) {
        (self.x, self.y)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Copy> Point3<T> {
    pub fn tuple(&self) -> (T, T, T) {
        (self.x, self.y, self.z)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T: Copy> Point4<T> {
    pub fn tuple(&self) -> (T, T, T, T) {
        (self.x, self.y, self.z, self.w)
    }
}
