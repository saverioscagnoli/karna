use num::Num;
use winit::dpi::PhysicalPosition;

use crate::{
    matrix::Matrix,
    points::{Point2, Point3, Point4},
    ToF32,
};
use std::ops::{Deref, DerefMut, Index, IndexMut, Mul, MulAssign};

/// A vector with `N` elements.
/// The type is derived from the [`Matrix`] type.
pub type Vector<const N: usize> = Matrix<N, 1>;

/// Index helper implementation for vectors; so they can be accessed
/// using vector[index] syntax.
impl<const N: usize> Index<usize> for Vector<N> {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[0][index]
    }
}

/// Index helper implementation for vectors; so they can be accessed
/// using vector[index] syntax.
///
/// This is the mutable version.
impl<const N: usize> IndexMut<usize> for Vector<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[0][index]
    }
}

impl<F: ToF32, const N: usize> From<[F; N]> for Vector<N> {
    fn from(array: [F; N]) -> Self {
        let mut v = Self::zero();

        for i in 0..N {
            v[i] = array[i].to_f32();
        }

        v
    }
}

impl<const N: usize> Vector<N> {
    /// Returns `VecN(1, 0, ...)`
    pub fn unit_x() -> Self {
        let mut v = Self::zero();
        v[0] = 1.0;
        v
    }

    /// Returns `VecN(0, 1, ...)`
    pub fn unit_y() -> Self {
        let mut v = Self::zero();
        v[1] = 1.0;
        v
    }

    /// Calculates the dot product of two vectors.
    /// The dot product is the sum of the products of the corresponding elements.
    pub fn dot(&self, other: &Self) -> f32 {
        let mut sum = 0.0;

        for i in 0..N {
            sum += self[i] * other[i];
        }

        sum
    }

    /// Calculates the length of the vector.
    /// Uses sqrt, so not the fastest.
    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Normalizes the vector length to 1.
    pub fn normalize(&mut self) {
        let length = self.length();

        if length != 0.0 {
            for i in 0..N {
                self[i] /= length;
            }
        }
    }

    /// Returns a normalized version of the vector.
    pub fn normalized(&self) -> Self {
        let mut v = self.clone();
        v.normalize();
        v
    }
}

pub type Vec2 = Vector<2>;

/// Dereference implementation for Vec2.
/// This allows the Vec2 to be accessed using point.x and point.y.
impl Deref for Vec2 {
    type Target = Point2;

    fn deref(&self) -> &Point2 {
        unsafe { std::mem::transmute(self) }
    }
}

/// Mutable dereference implementation for Vec2.
/// This allows the Vec2 to be accessed and modified using point.x and point.y.
impl DerefMut for Vec2 {
    fn deref_mut(&mut self) -> &mut Point2 {
        unsafe { std::mem::transmute(self) }
    }
}

/// Hadamard product implementation for Vec2.
impl Mul<Vec2> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

/// Hadamard product assignment implementation for Vec2. (*=)
impl MulAssign<Vec2> for &mut Vec2 {
    fn mul_assign(&mut self, rhs: Vec2) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl Vec2 {
    /// Creates a new Vec2 from two values.
    pub fn new<F: ToF32>(x: F, y: F) -> Self {
        Self([[x.to_f32(), y.to_f32()]])
    }

    /// Sets both values of the Vec2.
    pub fn set<F: ToF32>(&mut self, x: F, y: F) {
        self.x = x.to_f32();
        self.y = y.to_f32();
    }
}

impl<T: Num + ToF32> From<PhysicalPosition<T>> for Vec2 {
    fn from(value: PhysicalPosition<T>) -> Self {
        Vec2::new(value.x.to_f32(), value.y.to_f32())
    }
}

pub type Vec3 = Vector<3>;

/// Dereference implementation for Vec3.
/// This allows the Vec3 to be accessed using point.x, point.y, and point.z.
impl Deref for Vec3 {
    type Target = Point3;

    fn deref(&self) -> &Point3 {
        unsafe { std::mem::transmute(self) }
    }
}

/// Mutable dereference implementation for Vec3.
/// This allows the Vec3 to be accessed and modified using point.x, point.y, and point.z.
impl DerefMut for Vec3 {
    fn deref_mut(&mut self) -> &mut Point3 {
        unsafe { std::mem::transmute(self) }
    }
}

/// Hadamard product implementation for Vec3.
impl Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

/// Hadamard product assignment implementation for Vec3. (*=)
impl MulAssign<Vec3> for &mut Vec3 {
    fn mul_assign(&mut self, rhs: Vec3) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl Vec3 {
    pub fn new<F: ToF32>(x: F, y: F, z: F) -> Self {
        Self([[x.to_f32(), y.to_f32(), z.to_f32()]])
    }

    pub fn set<F: ToF32>(&mut self, x: F, y: F, z: F) {
        self.x = x.to_f32();
        self.y = y.to_f32();
        self.z = z.to_f32();
    }
}

pub type Vec4 = Vector<4>;

/// Dereference implementation for Vec4.
/// This allows the Vec4 to be accessed using point.x, point.y, point.z, and point.w.
impl Deref for Vec4 {
    type Target = Point4;

    fn deref(&self) -> &Point4 {
        unsafe { std::mem::transmute(self) }
    }
}

/// Mutable dereference implementation for Vec4.
/// This allows the Vec4 to be accessed and modified using point.x, point.y, point.z, and point.w.
impl DerefMut for Vec4 {
    fn deref_mut(&mut self) -> &mut Point4 {
        unsafe { std::mem::transmute(self) }
    }
}

/// Hadamard product implementation for Vec4.
impl Mul<Vec4> for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        Vec4::new(
            self.x * rhs.x,
            self.y * rhs.y,
            self.z * rhs.z,
            self.w * rhs.w,
        )
    }
}

/// Hadamard product assignment implementation for Vec4. (*=)
impl MulAssign<Vec4> for &mut Vec4 {
    fn mul_assign(&mut self, rhs: Vec4) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
        self.w *= rhs.w;
    }
}

impl Vec4 {
    pub fn new<F: ToF32>(x: F, y: F, z: F, w: F) -> Self {
        Self([[x.to_f32(), y.to_f32(), z.to_f32(), w.to_f32()]])
    }

    pub fn set<F: ToF32>(&mut self, x: F, y: F, z: F, w: F) {
        self.x = x.to_f32();
        self.y = y.to_f32();
        self.z = z.to_f32();
        self.w = w.to_f32();
    }
}

/// Most of the stuff for vectors is already tested in the matrix tests,
/// since vectors are just a special case of matrices.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_hadamard() {
        let a = Vec2::new(1, 2);
        let b = Vec2::new(3, 4);

        let c = a * b;

        assert_eq!(c.x, 3.0);
        assert_eq!(c.y, 8.0);

        let a = Vec3::new(1, 2, 3);
        let b = Vec3::new(4, 5, 6);

        let c = a * b;

        assert_eq!(c.x, 4.0);
        assert_eq!(c.y, 10.0);
        assert_eq!(c.z, 18.0);

        let a = Vec4::new(1, 2, 3, 4);
        let b = Vec4::new(5, 6, 7, 8);

        let c = a * b;

        assert_eq!(c.x, 5.0);
        assert_eq!(c.y, 12.0);
        assert_eq!(c.z, 21.0);
        assert_eq!(c.w, 32.0);
    }

    #[test]
    fn vec_index() {
        let a = Vec2::new(1, 2);

        assert_eq!(a[0], 1.0);
        assert_eq!(a[1], 2.0);

        let mut a = Vec2::new(1, 2);

        a[0] = 3.0;
        a[1] = 4.0;

        assert_eq!(a[0], 3.0);
        assert_eq!(a[1], 4.0);
    }
}

#[cfg(test)]
mod fuzz_test {
    use const_random::const_random;

    use super::*;

    const fn const_random_range(min: u8, max: u8) -> usize {
        (min + (const_random!(u8) % (max - min))) as usize
    }

    const R: usize = const_random_range(2, 4);

    #[test]
    fn vec_hadamard() {
        for _ in 0..100 {
            let a = Vector::<R>::random(0.0..=255.0);
            let b = Vector::<R>::random(0.0..=255.0);

            let c = a * b;

            for i in 0..R {
                assert_eq!(c[i], a[i] * b[i]);
            }
        }
    }

    #[test]
    fn vec_index() {
        for _ in 0..100 {
            let mut a = Vector::<R>::random(0.0..=255.0);

            for i in 0..R {
                assert_eq!(a[i], a[i]);
            }

            for i in 0..R {
                a[i] = i as f32;
            }

            for i in 0..R {
                assert_eq!(a[i], i as f32);
            }
        }
    }
}
