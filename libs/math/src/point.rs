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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuples() {
        assert_eq!(Point2 { x: 1, y: 2 }.tuple(), (1, 2));
        assert_eq!(Point3 { x: 1, y: 2, z: 3 }.tuple(), (1, 2, 3));
        assert_eq!(
            Point4 {
                x: 1,
                y: 2,
                z: 3,
                w: 4
            }
            .tuple(),
            (1, 2, 3, 4)
        );
    }

    #[test]
    fn repr_matches_array() {
        assert_eq!(core::mem::size_of::<Point2<f32>>(), 8);
        assert_eq!(core::mem::size_of::<Point3<f32>>(), 12);
        assert_eq!(core::mem::size_of::<Point4<f32>>(), 16);
        let p = Point3 {
            x: 1.0f32,
            y: 2.0,
            z: 3.0,
        };
        let a: [f32; 3] = unsafe { core::mem::transmute(p) };
        assert_eq!(a, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn equality_and_order() {
        assert_eq!(Point2 { x: 1, y: 2 }, Point2 { x: 1, y: 2 });
        assert!(Point2 { x: 1, y: 9 } < Point2 { x: 2, y: 0 });
    }
}
