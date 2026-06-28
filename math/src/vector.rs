use std::array;
use std::array::IntoIter;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::DivAssign;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Mul;
use std::ops::MulAssign;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;
use std::slice::Iter;
use std::slice::IterMut;
use std::usize;

use num::Float;
use num::Num;
use num::Signed;
use utils::impl_deref_to_generic;

use crate::point::Point2;
use crate::point::Point3;
use crate::point::Point4;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vector<const N: usize, T: Num + Copy>([T; N]);

macro_rules! impl_bin_op {
    ($trait:ident, $method:ident) => {
        impl<const N: usize, T: Num + Copy> $trait for Vector<N, T> {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self {
                let mut result = [T::zero(); N];
                for i in 0..N {
                    result[i] = self.0[i].$method(rhs.0[i]);
                }
                Self(result)
            }
        }
    };
}

macro_rules! impl_bin_op_variants {
    ($trait:ident, $method:ident) => {
        // &owned op &owned
        impl<const N: usize, T: Num + Copy> $trait for &Vector<N, T> {
            type Output = Vector<N, T>;
            fn $method(self, rhs: Self) -> Vector<N, T> {
                let mut result = [T::zero(); N];
                for i in 0..N {
                    result[i] = self.0[i].$method(rhs.0[i]);
                }
                Vector(result)
            }
        }
        // &owned op owned
        impl<const N: usize, T: Num + Copy> $trait<Vector<N, T>> for &Vector<N, T> {
            type Output = Vector<N, T>;
            fn $method(self, rhs: Vector<N, T>) -> Vector<N, T> {
                let mut result = [T::zero(); N];
                for i in 0..N {
                    result[i] = self.0[i].$method(rhs.0[i]);
                }
                Vector(result)
            }
        }
        // owned op &owned
        impl<const N: usize, T: Num + Copy> $trait<&Vector<N, T>> for Vector<N, T> {
            type Output = Vector<N, T>;
            fn $method(self, rhs: &Vector<N, T>) -> Vector<N, T> {
                let mut result = [T::zero(); N];
                for i in 0..N {
                    result[i] = self.0[i].$method(rhs.0[i]);
                }
                Vector(result)
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
                let mut result = [T::zero(); N];
                for i in 0..N {
                    result[i] = self.0[i].$method(rhs);
                }
                Self(result)
            }
        }
    };
}

impl_bin_op!(Add, add);
impl_bin_op!(Sub, sub);
impl_bin_op!(Mul, mul);
impl_bin_op!(Div, div);

impl_bin_op_variants!(Add, add);
impl_bin_op_variants!(Sub, sub);
impl_bin_op_variants!(Mul, mul);
impl_bin_op_variants!(Div, div);

impl_assign_op!(AddAssign, add_assign, add);
impl_assign_op!(SubAssign, sub_assign, sub);
impl_assign_op!(MulAssign, mul_assign, mul);
impl_assign_op!(DivAssign, div_assign, div);

impl_scalar_op!(Add, add);
impl_scalar_op!(Sub, sub);
impl_scalar_op!(Mul, mul);
impl_scalar_op!(Div, div);

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

impl<const N: usize, T: Num + Copy> Vector<N, T> {
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.0.iter_mut()
    }
}

impl<const N: usize, T: Num + Copy> Vector<N, T> {
    pub fn zero() -> Self {
        Self([T::zero(); N])
    }

    pub fn one() -> Self {
        Self([T::one(); N])
    }

    pub fn from_slice(v: &[T]) -> Self {
        Self(array::from_fn(|i| v[i]))
    }

    pub fn as_array(&self) -> [T; N] {
        self.0
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    pub fn splat(v: T) -> Self {
        Self([v; N])
    }

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

/// Partial Ord Impl

impl<const N: usize, T: Num + Copy + PartialOrd> Vector<N, T> {
    pub fn clamp(&self, min: T, max: T) -> Self {
        Self(self.0.map(|x| num::clamp(x, min, max)))
    }

    pub fn clamp_mut(&mut self, min: T, max: T) {
        self.0 = self.0.map(|x| num::clamp(x, min, max))
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

/// Signed impl

impl<const N: usize, T: Num + Copy + Signed> Vector<N, T> {
    pub fn abs(&self) -> Self {
        Self(array::from_fn(|i| self[i].abs()))
    }

    pub fn reflect(&self, normal: &Self) -> Self {
        // r = v - 2(v·n)n
        let dot = self.dot(normal);
        let two = T::one() + T::one();
        Self(std::array::from_fn(|i| self[i] - two * dot * normal[i]))
    }

    pub fn project(&self, onto: &Self) -> Self {
        // proj = (v·onto / onto·onto) * onto
        let scalar = self.dot(onto);
        let onto_sq = onto.dot(onto);
        Self(std::array::from_fn(|i| scalar * onto[i] / onto_sq))
    }
}

/// Float impl

impl<const N: usize, T: Float> Vector<N, T> {
    pub fn length(&self) -> T {
        self.length_sq().sqrt()
    }

    pub fn normalize(&self) -> Self {
        let l = self.length();

        if l == T::zero() {
            return Self::zero();
        }

        return *self / l;
    }

    pub fn normalize_mut(&mut self) {
        let l = self.length();

        if l == T::zero() {
            return;
        }

        *self = *self / l
    }

    pub fn distance(&self, other: &Self) -> T {
        self.distance_sq(other).sqrt()
    }

    pub fn lerp(&self, other: &Self, t: T) -> Self {
        // result = self + t * (other - self)
        Self(array::from_fn(|i| self[i] + t * (other[i] - self[i])))
    }

    pub fn project_normalized(&self, onto: &Self) -> Self {
        let scalar = self.dot(onto);
        Self(array::from_fn(|i| scalar * onto[i]))
    }
}

pub type Vector2<T> = Vector<2, T>;

impl_deref_to_generic!(Vector2<T> => Point2<T> where T: Num + Copy);

impl<T: Num + Copy> Vector2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self([x, y])
    }
}

/// Vector2 where T can be negative
impl<T: Num + Copy + Neg<Output = T>> Vector2<T> {
    pub fn perp(&self) -> Self {
        Self::new(-self.y, self.x)
    }

    pub fn perp_dot(&self, other: &Self) -> T {
        self.x * other.y - self.y * other.x
    }
}

/// Vector2 where T is float
impl<T: Float> Vector2<T> {
    pub fn angle(&self) -> T {
        T::atan2(self.y, self.x)
    }

    pub fn rotate(&self, angle: T) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos)
    }

    pub fn from_angle(angle: T) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(cos, sin)
    }
}

pub type Vector3<T> = Vector<3, T>;

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

impl<T: Float> Vector3<T> {
    pub fn angle_between(&self, other: &Self) -> T {
        let cos = self.dot(other) / (self.length() * other.length());
        cos.acos()
    }
}

pub type Vector4<T> = Vector<4, T>;

impl_deref_to_generic!(Vector4<T> => Point4<T> where T: Num + Copy);

impl<T: Num + Copy> Vector4<T> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self([x, y, z, w])
    }

    pub fn xyz(&self) -> Vector3<T> {
        Vector([self.x, self.y, self.z])
    }
}

/// Vector4 where T is a float
impl<T: Float> Vector4<T> {
    pub fn perspective_divide(&self) -> Vector3<T> {
        let w = self.w;
        Vector3::new(self.x / w, self.y / w, self.z / w)
    }
}
