use core::array;
use core::array::IntoIter;
use core::ops::Add;
use core::ops::AddAssign;
use core::ops::Div;
use core::ops::DivAssign;
use core::ops::Index;
use core::ops::IndexMut;
use core::ops::Mul;
use core::ops::MulAssign;
use core::ops::Neg;
use core::ops::Sub;
use core::ops::SubAssign;
use core::slice::Iter;
use core::slice::IterMut;

use num_traits::Num;
use num_traits::Signed;
use utils::impl_deref_to_generic;

use crate::SdlFloat;
use crate::point::Point2;
use crate::point::Point3;
use crate::point::Point4;

// Type

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vector<const N: usize, T: Num + Copy>([T; N]);

pub type Vector2<T> = Vector<2, T>;
pub type Vector3<T> = Vector<3, T>;
pub type Vector4<T> = Vector<4, T>;

// Construction

impl<const N: usize, T: Num + Copy> Vector<N, T> {
    pub fn zero() -> Self {
        Self([T::zero(); N])
    }

    pub fn one() -> Self {
        Self([T::one(); N])
    }

    pub fn splat(v: T) -> Self {
        Self([v; N])
    }

    pub fn from_slice(v: &[T]) -> Self {
        Self(array::from_fn(|i| v[i]))
    }
}

impl<const N: usize, T: Num + Copy> Default for Vector<N, T> {
    fn default() -> Self {
        Self::zero()
    }
}

// Access

impl<const N: usize, T: Num + Copy> Vector<N, T> {
    pub fn as_array(&self) -> [T; N] {
        self.0
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    pub fn set(&mut self, v: [T; N]) {
        self.0 = v;
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.0.iter_mut()
    }
}

impl<const N: usize, T: Num + Copy> Index<usize> for Vector<N, T> {
    type Output = T;

    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl<const N: usize, T: Num + Copy> IndexMut<usize> for Vector<N, T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

impl<const N: usize, T: Num + Copy> IntoIterator for Vector<N, T> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Operators

macro_rules! impl_bin_op {
    ($trait:ident, $method:ident) => {
        impl<const N: usize, T: Num + Copy> $trait for Vector<N, T> {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self {
                Self(array::from_fn(|i| self.0[i].$method(rhs.0[i])))
            }
        }

        impl<const N: usize, T: Num + Copy> $trait for &Vector<N, T> {
            type Output = Vector<N, T>;
            fn $method(self, rhs: Self) -> Vector<N, T> {
                Vector(array::from_fn(|i| self.0[i].$method(rhs.0[i])))
            }
        }

        impl<const N: usize, T: Num + Copy> $trait<Vector<N, T>> for &Vector<N, T> {
            type Output = Vector<N, T>;
            fn $method(self, rhs: Vector<N, T>) -> Vector<N, T> {
                Vector(array::from_fn(|i| self.0[i].$method(rhs.0[i])))
            }
        }

        impl<const N: usize, T: Num + Copy> $trait<&Vector<N, T>> for Vector<N, T> {
            type Output = Vector<N, T>;
            fn $method(self, rhs: &Vector<N, T>) -> Vector<N, T> {
                Vector(array::from_fn(|i| self.0[i].$method(rhs.0[i])))
            }
        }
    };
}

macro_rules! impl_assign_op {
    ($trait:ident, $method:ident, $bin_method:ident) => {
        impl<const N: usize, T: Num + Copy> $trait for Vector<N, T> {
            fn $method(&mut self, rhs: Self) {
                for i in 0..N {
                    self.0[i] = self.0[i].$bin_method(rhs.0[i]);
                }
            }
        }
    };
}

macro_rules! impl_scalar_op {
    ($trait:ident, $method:ident) => {
        impl<const N: usize, T: Num + Copy> $trait<T> for Vector<N, T> {
            type Output = Self;
            fn $method(self, rhs: T) -> Self {
                Self(array::from_fn(|i| self.0[i].$method(rhs)))
            }
        }
    };
}

macro_rules! impl_scalar_assign_op {
    ($trait:ident, $method:ident, $bin_method:ident) => {
        impl<const N: usize, T: Num + Copy> $trait<T> for Vector<N, T> {
            fn $method(&mut self, rhs: T) {
                for i in 0..N {
                    self.0[i] = self.0[i].$bin_method(rhs);
                }
            }
        }
    };
}

impl_bin_op!(Add, add);
impl_bin_op!(Sub, sub);
impl_bin_op!(Mul, mul);
impl_bin_op!(Div, div);

impl_assign_op!(AddAssign, add_assign, add);
impl_assign_op!(SubAssign, sub_assign, sub);
impl_assign_op!(MulAssign, mul_assign, mul);
impl_assign_op!(DivAssign, div_assign, div);

impl_scalar_op!(Add, add);
impl_scalar_op!(Sub, sub);
impl_scalar_op!(Mul, mul);
impl_scalar_op!(Div, div);

impl_scalar_assign_op!(AddAssign, add_assign, add);
impl_scalar_assign_op!(SubAssign, sub_assign, sub);
impl_scalar_assign_op!(MulAssign, mul_assign, mul);
impl_scalar_assign_op!(DivAssign, div_assign, div);

impl<const N: usize, T: Num + Copy + Neg<Output = T>> Neg for Vector<N, T> {
    type Output = Self;

    fn neg(self) -> Self {
        Self(array::from_fn(|i| -self.0[i]))
    }
}

// Num

impl<const N: usize, T: Num + Copy> Vector<N, T> {
    pub fn dot(&self, other: &Self) -> T {
        self.iter()
            .zip(other.iter())
            .map(|(a, b)| *a * *b)
            .fold(T::zero(), |acc, x| acc + x)
    }

    pub fn length_sq(&self) -> T {
        self.dot(self)
    }

    pub fn distance_sq(&self, other: &Self) -> T {
        (*self - *other).length_sq()
    }
}

// PartialOrd

impl<const N: usize, T: Num + Copy + PartialOrd> Vector<N, T> {
    pub fn clamp(&self, min: T, max: T) -> Self {
        Self(self.0.map(|x| num_traits::clamp(x, min, max)))
    }

    pub fn clamp_mut(&mut self, min: T, max: T) {
        self.0 = self.0.map(|x| num_traits::clamp(x, min, max))
    }

    pub fn min(&self, other: &Self) -> Self {
        Self(array::from_fn(|i| {
            if self[i] < other[i] {
                self[i]
            } else {
                other[i]
            }
        }))
    }

    pub fn max(&self, other: &Self) -> Self {
        Self(array::from_fn(|i| {
            if self[i] > other[i] {
                self[i]
            } else {
                other[i]
            }
        }))
    }
}

// Signed

impl<const N: usize, T: Num + Copy + Signed> Vector<N, T> {
    pub fn abs(&self) -> Self {
        Self(array::from_fn(|i| self[i].abs()))
    }

    pub fn reflect(&self, normal: &Self) -> Self {
        let dot = self.dot(normal);
        let two = T::one() + T::one();
        Self(array::from_fn(|i| self[i] - two * dot * normal[i]))
    }

    pub fn project(&self, onto: &Self) -> Self {
        let k = self.dot(onto) / onto.dot(onto);
        Self(array::from_fn(|i| k * onto[i]))
    }
}

// SdlFloat

impl<const N: usize, T: Num + SdlFloat> Vector<N, T> {
    pub fn length(&self) -> T {
        self.length_sq().sdl_sqrt()
    }

    pub fn distance(&self, other: &Self) -> T {
        self.distance_sq(other).sdl_sqrt()
    }

    pub fn normalize(&self) -> Self {
        let l = self.length();

        if l == T::zero() {
            return Self::zero();
        }

        *self / l
    }

    pub fn normalize_mut(&mut self) {
        let l = self.length();

        if l == T::zero() {
            return;
        }

        *self = *self / l
    }

    pub fn lerp(&self, other: &Self, t: T) -> Self {
        Self(array::from_fn(|i| self[i] + t * (other[i] - self[i])))
    }

    pub fn project_normalized(&self, onto: &Self) -> Self {
        let scalar = self.dot(onto);
        Self(array::from_fn(|i| scalar * onto[i]))
    }
}

// Vector2

impl_deref_to_generic!(Vector2<T> => Point2<T> where T: Num + Copy);

impl<T: Num + Copy> Vector2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self([x, y])
    }
}

impl<T: Num + Copy + Neg<Output = T>> Vector2<T> {
    pub fn perp(&self) -> Self {
        Self::new(-self.y, self.x)
    }

    pub fn perp_dot(&self, other: &Self) -> T {
        self.x * other.y - self.y * other.x
    }
}

impl<T: Num + SdlFloat> Vector2<T> {
    pub fn from_angle(angle: T) -> Self {
        Self::new(angle.sdl_cos(), angle.sdl_sin())
    }

    pub fn angle(&self) -> T {
        self.y.sdl_atan2(self.x)
    }

    pub fn rotate(&self, angle: T) -> Self {
        let sin = angle.sdl_sin();
        let cos = angle.sdl_cos();

        Self::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos)
    }
}

// Vector3

impl_deref_to_generic!(Vector3<T> => Point3<T> where T: Num + Copy);

impl<T: Num + Copy> Vector3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self([x, y, z])
    }

    pub fn xy(&self) -> Vector2<T> {
        Vector2::new(self.x, self.y)
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
}

impl<T: Num + SdlFloat> Vector3<T> {
    pub fn angle_between(&self, other: &Self) -> T {
        let d = self.dot(other) / (self.length() * other.length());
        d.sdl_clamp(-T::ONE, T::ONE).sdl_acos()
    }
}

// Vector4

impl_deref_to_generic!(Vector4<T> => Point4<T> where T: Num + Copy);

impl<T: Num + Copy> Vector4<T> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self([x, y, z, w])
    }

    pub fn xy(&self) -> Vector2<T> {
        Vector2::new(self.x, self.y)
    }

    pub fn xyz(&self) -> Vector3<T> {
        Vector3::new(self.x, self.y, self.z)
    }
}

impl<T: Num + SdlFloat> Vector4<T> {
    pub fn perspective_divide(&self) -> Vector3<T> {
        let inv = T::one() / self.w;

        Vector3::new(self.x * inv, self.y * inv, self.z * inv)
    }
}

// Conversion

impl<const N: usize, T: Num + Copy> From<[T; N]> for Vector<N, T> {
    fn from(arr: [T; N]) -> Self {
        Self(arr)
    }
}

impl<const N: usize, T: Num + Copy> From<Vector<N, T>> for [T; N] {
    fn from(v: Vector<N, T>) -> Self {
        v.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::f32::consts::PI;

    const EPS: f32 = 1e-5;

    fn approx(a: f32, b: f32) -> bool {
        let d = if a > b { a - b } else { b - a };
        d < EPS
    }

    fn approx_vec<const N: usize>(a: Vector<N, f32>, b: Vector<N, f32>) -> bool {
        (0..N).all(|i| approx(a[i], b[i]))
    }

    #[test]
    fn zero() {
        assert_eq!(Vector::<3, i32>::zero().as_array(), [0, 0, 0]);
    }

    #[test]
    fn one() {
        assert_eq!(Vector::<4, i32>::one().as_array(), [1, 1, 1, 1]);
    }

    #[test]
    fn splat() {
        assert_eq!(Vector::<3, i32>::splat(7).as_array(), [7, 7, 7]);
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(Vector::<2, i32>::default(), Vector::<2, i32>::zero());
    }

    #[test]
    fn from_slice() {
        assert_eq!(
            Vector::<3, i32>::from_slice(&[1, 2, 3]).as_array(),
            [1, 2, 3]
        );
    }

    #[test]
    fn from_slice_ignores_extra() {
        assert_eq!(
            Vector::<2, i32>::from_slice(&[1, 2, 3, 4]).as_array(),
            [1, 2]
        );
    }

    #[test]
    fn constructors() {
        assert_eq!(Vector2::new(1, 2).as_array(), [1, 2]);
        assert_eq!(Vector3::new(1, 2, 3).as_array(), [1, 2, 3]);
        assert_eq!(Vector4::new(1, 2, 3, 4).as_array(), [1, 2, 3, 4]);
    }

    #[test]
    fn set() {
        let mut v = Vector3::new(1, 2, 3);
        v.set([4, 5, 6]);
        assert_eq!(v.as_array(), [4, 5, 6]);
    }

    #[test]
    fn as_ptr() {
        let v = Vector3::new(9, 8, 7);
        unsafe {
            assert_eq!(*v.as_ptr(), 9);
            assert_eq!(*v.as_ptr().add(2), 7);
        }
    }

    #[test]
    fn index() {
        let v = Vector3::new(1, 2, 3);
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
        assert_eq!(v[2], 3);
    }

    #[test]
    fn index_mut() {
        let mut v = Vector3::new(1, 2, 3);
        v[1] = 20;
        assert_eq!(v.as_array(), [1, 20, 3]);
    }

    #[test]
    #[should_panic]
    fn index_out_of_bounds() {
        let v = Vector2::new(1, 2);
        let _ = v[2];
    }

    #[test]
    fn iter() {
        let v = Vector3::new(1, 2, 3);
        let sum: i32 = v.iter().sum();
        assert_eq!(sum, 6);
    }

    #[test]
    fn iter_mut() {
        let mut v = Vector3::new(1, 2, 3);
        for x in v.iter_mut() {
            *x *= 2;
        }
        assert_eq!(v.as_array(), [2, 4, 6]);
    }

    #[test]
    fn into_iter() {
        let v = Vector3::new(1, 2, 3);
        let collected: [i32; 3] = core::array::from_fn(|i| v[i]);
        let sum: i32 = v.into_iter().sum();
        assert_eq!(collected, [1, 2, 3]);
        assert_eq!(sum, 6);
    }

    #[test]
    fn add() {
        let a = Vector3::new(1, 2, 3);
        let b = Vector3::new(10, 20, 30);
        assert_eq!((a + b).as_array(), [11, 22, 33]);
    }

    #[test]
    fn sub() {
        let a = Vector3::new(10, 20, 30);
        let b = Vector3::new(1, 2, 3);
        assert_eq!((a - b).as_array(), [9, 18, 27]);
    }

    #[test]
    fn mul() {
        let a = Vector3::new(2, 3, 4);
        let b = Vector3::new(5, 6, 7);
        assert_eq!((a * b).as_array(), [10, 18, 28]);
    }

    #[test]
    fn div() {
        let a = Vector3::new(10, 20, 30);
        let b = Vector3::new(2, 4, 5);
        assert_eq!((a / b).as_array(), [5, 5, 6]);
    }

    #[test]
    fn ref_variants() {
        let a = Vector2::new(4, 6);
        let b = Vector2::new(1, 2);
        assert_eq!((&a + &b).as_array(), [5, 8]);
        assert_eq!((&a + b).as_array(), [5, 8]);
        assert_eq!((a + &b).as_array(), [5, 8]);
        assert_eq!((&a - &b).as_array(), [3, 4]);
        assert_eq!((&a * &b).as_array(), [4, 12]);
        assert_eq!((&a / &b).as_array(), [4, 3]);
    }

    #[test]
    fn add_assign() {
        let mut a = Vector2::new(1, 2);
        a += Vector2::new(3, 4);
        assert_eq!(a.as_array(), [4, 6]);
    }

    #[test]
    fn sub_assign() {
        let mut a = Vector2::new(5, 7);
        a -= Vector2::new(1, 2);
        assert_eq!(a.as_array(), [4, 5]);
    }

    #[test]
    fn mul_assign() {
        let mut a = Vector2::new(2, 3);
        a *= Vector2::new(4, 5);
        assert_eq!(a.as_array(), [8, 15]);
    }

    #[test]
    fn div_assign() {
        let mut a = Vector2::new(8, 15);
        a /= Vector2::new(4, 5);
        assert_eq!(a.as_array(), [2, 3]);
    }

    #[test]
    fn scalar_ops() {
        let v = Vector3::new(2, 4, 6);
        assert_eq!((v + 1).as_array(), [3, 5, 7]);
        assert_eq!((v - 1).as_array(), [1, 3, 5]);
        assert_eq!((v * 3).as_array(), [6, 12, 18]);
        assert_eq!((v / 2).as_array(), [1, 2, 3]);
    }

    #[test]
    fn scalar_assign_ops() {
        let mut v = Vector3::new(2, 4, 6);
        v += 1;
        assert_eq!(v.as_array(), [3, 5, 7]);
        v -= 1;
        assert_eq!(v.as_array(), [2, 4, 6]);
        v *= 2;
        assert_eq!(v.as_array(), [4, 8, 12]);
        v /= 4;
        assert_eq!(v.as_array(), [1, 2, 3]);
    }

    #[test]
    fn neg() {
        let v = Vector3::new(1, -2, 3);
        assert_eq!((-v).as_array(), [-1, 2, -3]);
    }

    #[test]
    fn neg_twice_is_identity() {
        let v = Vector3::new(1, -2, 3);
        assert_eq!(-(-v), v);
    }

    #[test]
    fn dot() {
        let a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 5, 6);
        assert_eq!(a.dot(&b), 32);
    }

    #[test]
    fn dot_orthogonal() {
        let a = Vector2::new(1, 0);
        let b = Vector2::new(0, 1);
        assert_eq!(a.dot(&b), 0);
    }

    #[test]
    fn dot_is_commutative() {
        let a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 5, 6);
        assert_eq!(a.dot(&b), b.dot(&a));
    }

    #[test]
    fn length_sq() {
        assert_eq!(Vector2::new(3, 4).length_sq(), 25);
    }

    #[test]
    fn distance_sq() {
        let a = Vector2::new(0, 0);
        let b = Vector2::new(3, 4);
        assert_eq!(a.distance_sq(&b), 25);
    }

    #[test]
    fn length() {
        assert!(approx(Vector2::new(3.0f32, 4.0).length(), 5.0));
    }

    #[test]
    fn length_of_zero() {
        assert!(approx(Vector3::<f32>::zero().length(), 0.0));
    }

    #[test]
    fn distance() {
        let a = Vector2::new(1.0f32, 1.0);
        let b = Vector2::new(4.0f32, 5.0);
        assert!(approx(a.distance(&b), 5.0));
    }

    #[test]
    fn distance_is_symmetric() {
        let a = Vector3::new(1.0f32, 2.0, 3.0);
        let b = Vector3::new(-4.0f32, 0.5, 2.0);
        assert!(approx(a.distance(&b), b.distance(&a)));
    }

    #[test]
    fn normalize() {
        let v = Vector2::new(3.0f32, 4.0);
        assert!(approx_vec(v.normalize(), Vector2::new(0.6, 0.8)));
    }

    #[test]
    fn normalize_has_unit_length() {
        let v = Vector3::new(1.0f32, -2.0, 3.5);
        assert!(approx(v.normalize().length(), 1.0));
    }

    #[test]
    fn normalize_zero() {
        let v = Vector3::<f32>::zero();
        assert_eq!(v.normalize(), Vector3::<f32>::zero());
    }

    #[test]
    fn normalize_mut() {
        let mut v = Vector2::new(3.0f32, 4.0);
        v.normalize_mut();
        assert!(approx_vec(v, Vector2::new(0.6, 0.8)));
    }

    #[test]
    fn normalize_mut_zero() {
        let mut v = Vector2::<f32>::zero();
        v.normalize_mut();
        assert_eq!(v, Vector2::<f32>::zero());
    }

    #[test]
    fn lerp() {
        let a = Vector2::new(0.0f32, 0.0);
        let b = Vector2::new(10.0f32, 20.0);
        assert!(approx_vec(a.lerp(&b, 0.0), a));
        assert!(approx_vec(a.lerp(&b, 1.0), b));
        assert!(approx_vec(a.lerp(&b, 0.5), Vector2::new(5.0, 10.0)));
    }

    #[test]
    fn lerp_extrapolates() {
        let a = Vector2::new(0.0f32, 0.0);
        let b = Vector2::new(10.0f32, 0.0);
        assert!(approx_vec(a.lerp(&b, 2.0), Vector2::new(20.0, 0.0)));
    }

    #[test]
    fn abs() {
        let v = Vector3::new(-1, 2, -3);
        assert_eq!(v.abs().as_array(), [1, 2, 3]);
    }

    #[test]
    fn reflect() {
        let v = Vector2::new(1.0f32, -1.0);
        let n = Vector2::new(0.0f32, 1.0);
        assert!(approx_vec(v.reflect(&n), Vector2::new(1.0, 1.0)));
    }

    #[test]
    fn reflect_along_normal() {
        let v = Vector2::new(0.0f32, -2.0);
        let n = Vector2::new(0.0f32, 1.0);
        assert!(approx_vec(v.reflect(&n), Vector2::new(0.0, 2.0)));
    }

    #[test]
    fn project() {
        let v = Vector2::new(3.0f32, 4.0);
        let onto = Vector2::new(2.0f32, 0.0);
        assert!(approx_vec(v.project(&onto), Vector2::new(3.0, 0.0)));
    }

    #[test]
    fn project_onto_self() {
        let v = Vector3::new(1.0f32, 2.0, 3.0);
        assert!(approx_vec(v.project(&v), v));
    }

    #[test]
    fn project_normalized() {
        let v = Vector2::new(3.0f32, 4.0);
        let onto = Vector2::new(1.0f32, 0.0);
        assert!(approx_vec(
            v.project_normalized(&onto),
            Vector2::new(3.0, 0.0)
        ));
    }

    #[test]
    fn clamp_mut() {
        let mut v = Vector3::new(-5, 5, 15);
        v.clamp_mut(0, 10);
        assert_eq!(v.as_array(), [0, 5, 10]);
    }

    #[test]
    fn min() {
        let a = Vector3::new(1, 5, 3);
        let b = Vector3::new(4, 2, 3);
        assert_eq!(a.min(b).as_array(), [1, 2, 3]);
    }

    #[test]
    fn max() {
        let a = Vector3::new(1, 5, 3);
        let b = Vector3::new(4, 2, 3);
        assert_eq!(a.max(b).as_array(), [4, 5, 3]);
    }

    #[test]
    fn deref_fields() {
        let v2 = Vector2::new(1, 2);
        let v3 = Vector3::new(1, 2, 3);
        let v4 = Vector4::new(1, 2, 3, 4);
        assert_eq!((v2.x, v2.y), (1, 2));
        assert_eq!((v3.x, v3.y, v3.z), (1, 2, 3));
        assert_eq!((v4.x, v4.y, v4.z, v4.w), (1, 2, 3, 4));
    }

    #[test]
    fn perp() {
        let v = Vector2::new(1, 0);
        assert_eq!(v.perp().as_array(), [0, 1]);
    }

    #[test]
    fn perp_is_orthogonal() {
        let v = Vector2::new(3, 4);
        assert_eq!(v.dot(&v.perp()), 0);
    }

    #[test]
    fn perp_dot() {
        let a = Vector2::new(1, 0);
        let b = Vector2::new(0, 1);
        assert_eq!(a.perp_dot(&b), 1);
        assert_eq!(b.perp_dot(&a), -1);
    }

    #[test]
    fn angle() {
        assert!(approx(Vector2::new(1.0f32, 0.0).angle(), 0.0));
        assert!(approx(Vector2::new(0.0f32, 1.0).angle(), PI / 2.0));
    }

    #[test]
    fn from_angle() {
        assert!(approx_vec(
            Vector2::from_angle(0.0f32),
            Vector2::new(1.0, 0.0)
        ));
        assert!(approx_vec(
            Vector2::from_angle(PI / 2.0),
            Vector2::new(0.0, 1.0)
        ));
    }

    #[test]
    fn from_angle_is_unit() {
        assert!(approx(Vector2::from_angle(1.234f32).length(), 1.0));
    }

    #[test]
    fn angle_round_trip() {
        let a = 0.75f32;
        assert!(approx(Vector2::from_angle(a).angle(), a));
    }

    #[test]
    fn rotate() {
        let v = Vector2::new(1.0f32, 0.0);
        assert!(approx_vec(v.rotate(PI / 2.0), Vector2::new(0.0, 1.0)));
        assert!(approx_vec(v.rotate(PI), Vector2::new(-1.0, 0.0)));
    }

    #[test]
    fn rotate_preserves_length() {
        let v = Vector2::new(3.0f32, 4.0);
        assert!(approx(v.rotate(1.1).length(), 5.0));
    }

    #[test]
    fn rotate_zero_is_identity() {
        let v = Vector2::new(3.0f32, 4.0);
        assert!(approx_vec(v.rotate(0.0), v));
    }

    #[test]
    fn vector3_xy() {
        assert_eq!(Vector3::new(1, 2, 3).xy().as_array(), [1, 2]);
    }

    #[test]
    fn cross_basis() {
        let x = Vector3::new(1, 0, 0);
        let y = Vector3::new(0, 1, 0);
        let z = Vector3::new(0, 0, 1);
        assert_eq!(x.cross(&y), z);
        assert_eq!(y.cross(&z), x);
        assert_eq!(z.cross(&x), y);
    }

    #[test]
    fn cross_is_anticommutative() {
        let a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 5, 6);
        assert_eq!(a.cross(&b), -b.cross(&a));
    }

    #[test]
    fn cross_with_self_is_zero() {
        let a = Vector3::new(1, 2, 3);
        assert_eq!(a.cross(&a), Vector3::zero());
    }

    #[test]
    fn cross_is_orthogonal() {
        let a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 5, 6);
        let c = a.cross(&b);
        assert_eq!(c.dot(&a), 0);
        assert_eq!(c.dot(&b), 0);
    }

    #[test]
    fn angle_between_orthogonal() {
        let a = Vector3::new(1.0f32, 0.0, 0.0);
        let b = Vector3::new(0.0f32, 1.0, 0.0);
        assert!(approx(a.angle_between(&b), PI / 2.0));
    }

    #[test]
    fn angle_between_parallel() {
        let a = Vector3::new(1.0f32, 2.0, 3.0);
        let b = Vector3::new(2.0f32, 4.0, 6.0);
        assert!(approx(a.angle_between(&b), 0.0));
    }

    #[test]
    fn angle_between_opposite() {
        let a = Vector3::new(1.0f32, 0.0, 0.0);
        let b = Vector3::new(-1.0f32, 0.0, 0.0);
        assert!(approx(a.angle_between(&b), PI));
    }

    #[test]
    fn angle_between_identical_is_not_nan() {
        let a = Vector3::new(0.3f32, 0.7, -0.2);
        assert!(!a.angle_between(&a).is_nan());
    }

    #[test]
    fn vector4_swizzles() {
        let v = Vector4::new(1, 2, 3, 4);
        assert_eq!(v.xy().as_array(), [1, 2]);
        assert_eq!(v.xyz().as_array(), [1, 2, 3]);
    }

    #[test]
    fn perspective_divide() {
        let v = Vector4::new(2.0f32, 4.0, 6.0, 2.0);
        assert!(approx_vec(
            v.perspective_divide(),
            Vector3::new(1.0, 2.0, 3.0)
        ));
    }

    #[test]
    fn perspective_divide_by_one() {
        let v = Vector4::new(1.0f32, 2.0, 3.0, 1.0);
        assert!(approx_vec(
            v.perspective_divide(),
            Vector3::new(1.0, 2.0, 3.0)
        ));
    }

    #[test]
    fn from_array() {
        let v: Vector3<i32> = [1, 2, 3].into();
        assert_eq!(v.as_array(), [1, 2, 3]);
    }

    #[test]
    fn into_array() {
        let a: [i32; 3] = Vector3::new(1, 2, 3).into();
        assert_eq!(a, [1, 2, 3]);
    }

    #[test]
    fn array_round_trip() {
        let v = Vector4::new(1, 2, 3, 4);
        let a: [i32; 4] = v.into();
        assert_eq!(Vector4::from(a), v);
    }

    #[test]
    fn equality() {
        assert_eq!(Vector3::new(1, 2, 3), Vector3::new(1, 2, 3));
        assert_ne!(Vector3::new(1, 2, 3), Vector3::new(1, 2, 4));
    }

    #[test]
    fn repr_matches_array() {
        assert_eq!(
            core::mem::size_of::<Vector4<f32>>(),
            core::mem::size_of::<[f32; 4]>()
        );
        assert_eq!(
            core::mem::align_of::<Vector4<f32>>(),
            core::mem::align_of::<[f32; 4]>()
        );
    }
}
