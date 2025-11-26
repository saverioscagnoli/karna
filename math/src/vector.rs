use crate::{
    AsF32,
    point::{Point2, Point3, Point4},
};
use macros::{impl_deref_to, impl_vec_op, impl_vec_op_assign};
use std::ops::{Index, IndexMut, Neg};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vector<const N: usize>([f32; N]);

impl<const N: usize> Index<usize> for Vector<N> {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const N: usize> IndexMut<usize> for Vector<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<const N: usize> Vector<N> {
    #[inline]
    pub const fn from_array(arr: [f32; N]) -> Self {
        Self(arr)
    }

    #[inline]
    pub const fn zero() -> Self {
        Self([0.0; N])
    }

    #[inline]
    pub fn reset(&mut self) {
        for i in 0..N {
            self[i] = 0.0;
        }
    }

    #[inline]
    pub const fn one() -> Self {
        Self([1.0; N])
    }

    #[inline]
    pub const fn fill(n: f32) -> Self {
        Self([n; N])
    }

    #[inline]
    pub fn dot(&self, rhs: &Vector<N>) -> f32 {
        let mut sum = 0.0;

        for i in 0..N {
            sum += self[i] * rhs[i]
        }

        sum
    }

    #[inline]
    pub fn length_sq(&self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.length_sq().sqrt()
    }

    #[inline]
    pub fn normalize(&self) -> Self {
        let len = self.length();

        if len <= 0.0 {
            return Self::zero();
        }

        *self / len
    }

    #[inline]
    pub fn normalize_mut(&mut self) {
        let len = self.length();

        if len <= 0.0 {
            self.reset();
            return;
        }

        *self /= len;
    }
}

impl<F, const N: usize> From<[F; N]> for Vector<N>
where
    F: AsF32,
{
    fn from(value: [F; N]) -> Self {
        let mut v = Self::zero();

        for i in 0..N {
            v[i] = value[i].as_f32()
        }

        v
    }
}

// Operations
impl_vec_op!(Add, add, +, commutative);
impl_vec_op!(Sub, sub, -);
impl_vec_op!(Mul, mul, *, commutative);
impl_vec_op!(Div, div, /);

impl_vec_op_assign!(AddAssign, add_assign, +=);
impl_vec_op_assign!(SubAssign, sub_assign, -=);
impl_vec_op_assign!(MulAssign, mul_assign, *=);
impl_vec_op_assign!(DivAssign, div_assign, /=);

/// Vector negation
impl<const N: usize> Neg for Vector<N> {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        for i in 0..N {
            self[i] = -self[i]
        }

        self
    }
}

pub type Vector2 = Vector<2>;

impl Vector2 {
    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self([x, y])
    }

    #[inline]
    pub fn angle(&self) -> f32 {
        self[1].atan2(self[0])
    }

    #[inline]
    pub fn angle_deg(&self) -> f32 {
        self.angle().to_degrees()
    }
}

pub type Vector3 = Vector<3>;

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self([x, y, z])
    }

    #[inline]
    pub fn cross(&self, rhs: &Self) -> Self {
        Self::new(
            self[1] * rhs[2] - self[2] * rhs[1], // Yz - Zy
            self[2] * rhs[0] - self[0] * rhs[2], // Zx - Xz
            self[0] * rhs[1] - self[1] * rhs[0], // Xy - Yx
        )
    }
}

pub type Vector4 = Vector<4>;

impl Vector4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self([x, y, z, w])
    }
}

impl_deref_to!(Vector2 => Point2);
impl_deref_to!(Vector3 => Point3);
impl_deref_to!(Vector4 => Point4);

#[cfg(test)]
mod tests {
    use crate::vector::{Vector2, Vector3, Vector4};
    use nalgebra as na;
    const TOLERANCE: f32 = 1e-6;

    #[test]
    fn vec_dot() {
        let a = Vector2::new(10.0, 10.0);
        let b = Vector2::new(2.0, 3.0);
        let an = na::Vector2::<f32>::new(10.0, 10.0);
        let bn = na::Vector2::<f32>::new(2.0, 3.0);

        assert_eq!(a.dot(&b), an.dot(&bn));

        let a = Vector3::new(5.0, 6.0, 3.0);
        let b = Vector3::new(1.0, 7.0, 3.0);
        let an = na::Vector3::new(5.0, 6.0, 3.0);
        let bn = na::Vector3::new(1.0, 7.0, 3.0);

        assert_eq!(a.dot(&b), an.dot(&bn));

        let a = Vector4::new(3.0, 6.5, 10.0, 21.5);
        let b = Vector4::new(3.4, 6.7, 32.32, 0.5667);
        let an = na::Vector4::new(3.0, 6.5, 10.0, 21.5);
        let bn = na::Vector4::new(3.4, 6.7, 32.32, 0.5667);

        assert_eq!(a.dot(&b), an.dot(&bn));
    }

    #[test]
    fn vec_length_sq() {
        let a = Vector2::new(3.0, 4.0);
        let an = na::Vector2::<f32>::new(3.0, 4.0);

        assert!((a.length_sq() - an.norm_squared()).abs() < TOLERANCE);
        assert!((a.length_sq() - 25.0).abs() < TOLERANCE);

        let a = Vector3::new(1.0, 2.0, 2.0);
        let an = na::Vector3::<f32>::new(1.0, 2.0, 2.0);

        assert!((a.length_sq() - an.norm_squared()).abs() < TOLERANCE);
        assert!((a.length_sq() - 9.0).abs() < TOLERANCE);

        let a = Vector4::new(1.0, 1.0, 1.0, 1.0);
        let an = na::Vector4::<f32>::new(1.0, 1.0, 1.0, 1.0);

        assert!((a.length_sq() - an.norm_squared()).abs() < TOLERANCE);
        assert!((a.length_sq() - 4.0).abs() < TOLERANCE);
    }

    #[test]
    fn vec_length() {
        let a = Vector2::new(3.0, 4.0);
        let an = na::Vector2::<f32>::new(3.0, 4.0);

        assert!((a.length() - an.norm()).abs() < TOLERANCE);
        assert!((a.length() - 5.0).abs() < TOLERANCE);

        let a = Vector3::new(1.0, 2.0, 2.0);
        let an = na::Vector3::<f32>::new(1.0, 2.0, 2.0);

        assert!((a.length() - an.norm()).abs() < TOLERANCE);
        assert!((a.length() - 3.0).abs() < TOLERANCE);

        let a = Vector4::new(10.0, 0.0, 0.0, 0.0);
        let an = na::Vector4::<f32>::new(10.0, 0.0, 0.0, 0.0);
        assert!((a.length() - an.norm()).abs() < TOLERANCE);
        assert!((a.length() - 10.0).abs() < TOLERANCE);
    }

    #[test]
    fn vec_normalize() {
        let a = Vector2::new(3.0, 4.0);
        let b = a.normalize();
        let an = na::Vector2::<f32>::new(3.0, 4.0).normalize();

        assert!((b[0] - an[0]).abs() < TOLERANCE);
        assert!((b[1] - an[1]).abs() < TOLERANCE);

        assert!((b.length() - 1.0).abs() < TOLERANCE);

        let a = Vector3::new(10.0, 0.0, 0.0);
        let b = a.normalize();
        let an = na::Vector3::<f32>::new(10.0, 0.0, 0.0).normalize();

        assert!((b[0] - an[0]).abs() < TOLERANCE);
        assert!((b[1] - an[1]).abs() < TOLERANCE);
        assert!((b[2] - an[2]).abs() < TOLERANCE);
        assert!((b.length() - 1.0).abs() < TOLERANCE);

        let a = Vector4::new(0.0, 0.0, 0.0, 0.0);
        let b = a.normalize();

        assert_eq!(b, Vector4::zero());
        assert!((b.length() - 0.0).abs() < TOLERANCE);
    }

    #[test]
    fn vector3_cross() {
        // Define basis vectors
        let x = Vector3::new(1.0, 0.0, 0.0);
        let y = Vector3::new(0.0, 1.0, 0.0);
        let z = Vector3::new(0.0, 0.0, 1.0);

        // 1. Right-Hand Rule: X cross Y = Z
        let x_cross_y = x.cross(&y);
        let nx_cross_ny =
            na::Vector3::<f32>::new(1.0, 0.0, 0.0).cross(&na::Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(x_cross_y, z);
        assert!((x_cross_y[0] - nx_cross_ny[0]).abs() < TOLERANCE);

        // 2. Anti-Commutative: Y cross X = -Z
        assert_eq!(y.cross(&x), -z);

        // 3. Parallel vectors yield zero
        let a = Vector3::new(2.0, 4.0, 6.0);
        let b = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(a.cross(&b), Vector3::new(0.0, 0.0, 0.0));

        // 4. Test arbitrary vectors
        let a = Vector3::new(1.0, 2.0, 3.0);
        let b = Vector3::new(4.0, 5.0, 6.0);
        // Expected: (-3.0, 6.0, -3.0)
        let expected = Vector3::new(-3.0, 6.0, -3.0);
        let result = a.cross(&b);
        let n_result = na::Vector3::new(1.0, 2.0, 3.0).cross(&na::Vector3::new(4.0, 5.0, 6.0));

        assert_eq!(result, expected);
        assert!((result[0] - n_result[0]).abs() < TOLERANCE);
        assert!((result[1] - n_result[1]).abs() < TOLERANCE);
        assert!((result[2] - n_result[2]).abs() < TOLERANCE);
    }
}
