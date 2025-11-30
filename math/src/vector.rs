use crate::point::{Point2, Point3, Point4};
use common::impl_deref_to;
use std::f32;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::{
    array::IntoIter,
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vector<const N: usize>([f32; N]);

impl<const N: usize> Default for Vector<N> {
    fn default() -> Self {
        Self::splat(0.0)
    }
}

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

impl<const N: usize> From<[f32; N]> for Vector<N> {
    fn from(value: [f32; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize> From<Vector<N>> for [f32; N] {
    fn from(value: Vector<N>) -> Self {
        let mut result = [0.0; N];

        for i in 0..N {
            result[i] = value[i]
        }

        result
    }
}

impl<const N: usize> Vector<N> {
    #[inline]
    pub fn iter(&self) -> Iter<'_, f32> {
        self.0.iter()
    }

    #[inline]
    pub fn inter_mut(&mut self) -> IterMut<'_, f32> {
        self.0.iter_mut()
    }

    #[inline]
    pub fn into_iter(self) -> IntoIter<f32, N> {
        self.0.into_iter()
    }

    #[inline]
    pub fn from_array(arr: [f32; N]) -> Self {
        Self(arr)
    }

    pub fn splat(n: f32) -> Self {
        Self([n; N])
    }

    #[inline]
    pub fn zeros() -> Self {
        Self::default()
    }

    #[inline]
    pub fn ones() -> Self {
        Self::splat(1.0)
    }

    #[inline]
    pub fn dot(&self, rhs: &Self) -> f32 {
        self.iter().zip(rhs.iter()).map(|(a, b)| a + b).sum()
    }

    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn normalize(&mut self) {
        *self = *self / self.length()
    }

    #[inline]
    pub fn normalized(&self) -> Self {
        *self / self.length()
    }

    #[inline]
    /// Angle between two vectors in radians (0 to PI)
    pub fn angle_to(&self, rhs: &Self) -> f32 {
        let dot = self.dot(rhs);
        let len = (self.length() * rhs.length()).max(f32::EPSILON);

        (dot / len).clamp(-1.0, 1.0).acos()
    }

    #[inline]
    /// Reflect across a normal
    pub fn reflected(&self, normal: &Self) -> Self {
        *self - *normal * 2.0 * self.dot(normal)
    }

    #[inline]
    /// Project onto another vector
    pub fn projected_onto(&self, rhs: &Self) -> Self {
        *rhs * (self.dot(rhs) / rhs.length_squared())
    }

    #[inline]
    /// Reject from another vector (perpendicular component)
    pub fn rejected_from(&self, rhs: &Self) -> Self {
        *self - self.projected_onto(rhs)
    }

    #[inline]
    /// Component-wise min
    pub fn min(&self, rhs: &Self) -> Self {
        let mut result = Self::zeros();

        for i in 0..N {
            result[i] = self[i].min(rhs[i])
        }

        result
    }

    #[inline]
    /// Component-wise max
    pub fn max(&self, rhs: &Self) -> Self {
        let mut result = Self::zeros();

        for i in 0..N {
            result[i] = self[i].max(rhs[i])
        }

        result
    }

    #[inline]
    /// Component-wise abs
    pub fn abs(&self) -> Self {
        let mut result = Self::zeros();

        for i in 0..N {
            result[i] = self[i].abs()
        }

        result
    }

    #[inline]
    /// Component-wise floor
    pub fn floor(&self) -> Self {
        let mut result = Self::zeros();

        for i in 0..N {
            result[i] = self[i].floor()
        }

        result
    }

    #[inline]
    /// Component-wise ceil
    pub fn ceil(&self) -> Self {
        let mut result = Self::zeros();

        for i in 0..N {
            result[i] = self[i].ceil()
        }

        result
    }

    #[inline]
    /// Component-wise round
    pub fn round(&self) -> Self {
        let mut result = Self::zeros();

        for i in 0..N {
            result[i] = self[i].round()
        }

        result
    }

    #[inline]
    /// Clamp components between min and max
    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        let mut result = Self::zeros();

        for i in 0..N {
            result[i] = self[i].clamp(min[i], max[i])
        }

        result
    }
}

macro_rules! impl_vector_op {
    // Base: Vector-Vector ops (all 4 ref combinations)
    ($trait:ident, $method:ident, $op:tt) => {
        impl<const N: usize> $trait for Vector<N> {
            type Output = Vector<N>;
            fn $method(self, rhs: Self) -> Self::Output {
                let mut r = [0.0; N];
                for i in 0..N { r[i] = self.0[i] $op rhs.0[i]; }
                Vector(r)
            }
        }

        impl<const N: usize> $trait<&Vector<N>> for Vector<N> {
            type Output = Vector<N>;
            fn $method(self, rhs: &Vector<N>) -> Self::Output {
                let mut r = [0.0; N];
                for i in 0..N { r[i] = self.0[i] $op rhs.0[i]; }
                Vector(r)
            }
        }

        impl<const N: usize> $trait<Vector<N>> for &Vector<N> {
            type Output = Vector<N>;
            fn $method(self, rhs: Vector<N>) -> Self::Output {
                let mut r = [0.0; N];
                for i in 0..N { r[i] = self.0[i] $op rhs.0[i]; }
                Vector(r)
            }
        }

        impl<const N: usize> $trait for &Vector<N> {
            type Output = Vector<N>;
            fn $method(self, rhs: Self) -> Self::Output {
                let mut r = [0.0; N];
                for i in 0..N { r[i] = self.0[i] $op rhs.0[i]; }
                Vector(r)
            }
        }
    };

    // With scalar (non-commutative): Vector op f32
    ($trait:ident, $method:ident, $op:tt, scalar) => {
        impl_vector_op!($trait, $method, $op);

        impl<const N: usize> $trait<f32> for Vector<N> {
            type Output = Vector<N>;
            fn $method(self, s: f32) -> Self::Output {
                let mut r = [0.0; N];
                for i in 0..N { r[i] = self.0[i] $op s; }
                Vector(r)
            }
        }

        impl<const N: usize> $trait<f32> for &Vector<N> {
            type Output = Vector<N>;
            fn $method(self, s: f32) -> Self::Output {
                let mut r = [0.0; N];
                for i in 0..N { r[i] = self.0[i] $op s; }
                Vector(r)
            }
        }
    };

    // Commutative scalar: Vector op f32 AND f32 op Vector
    ($trait:ident, $method:ident, $op:tt, scalar_commutative) => {
        impl_vector_op!($trait, $method, $op, scalar);

        impl<const N: usize> $trait<Vector<N>> for f32 {
            type Output = Vector<N>;
            fn $method(self, v: Vector<N>) -> Self::Output {
                let mut r = [0.0; N];
                for i in 0..N { r[i] = self $op v.0[i]; }
                Vector(r)
            }
        }

        impl<const N: usize> $trait<&Vector<N>> for f32 {
            type Output = Vector<N>;
            fn $method(self, v: &Vector<N>) -> Self::Output {
                let mut r = [0.0; N];
                for i in 0..N { r[i] = self $op v.0[i]; }
                Vector(r)
            }
        }
    };
}

macro_rules! impl_op_assign {
    // Vector op= Vector
    (vector: $trait:ident, $method:ident, $op:tt) => {
        impl<const N: usize> $trait for Vector<N> {
            fn $method(&mut self, rhs: Self) {
                for i in 0..N { self.0[i] $op rhs.0[i]; }
            }
        }

        impl<const N: usize> $trait<&Vector<N>> for Vector<N> {
            fn $method(&mut self, rhs: &Vector<N>) {
                for i in 0..N { self.0[i] $op rhs.0[i]; }
            }
        }
    };

    // Vector op= scalar
    (scalar: $trait:ident, $method:ident, $op:tt) => {
        impl<const N: usize> $trait<f32> for Vector<N> {
            fn $method(&mut self, s: f32) {
                for i in 0..N { self.0[i] $op s; }
            }
        }
    };

    // Both vector and scalar
    (both: $trait:ident, $method:ident, $op:tt) => {
        impl_op_assign!(vector: $trait, $method, $op);
        impl_op_assign!(scalar: $trait, $method, $op);
    };
}

impl_vector_op!(Add, add, +, scalar_commutative);
impl_vector_op!(Sub, sub, -, scalar);
impl_vector_op!(Mul, mul, *, scalar_commutative);
impl_vector_op!(Div, div, /, scalar);

impl_op_assign!(both: AddAssign, add_assign, +=);
impl_op_assign!(both: SubAssign, sub_assign, -=);
impl_op_assign!(both: MulAssign, mul_assign, *=);
impl_op_assign!(both: DivAssign, div_assign, /=);

impl<const N: usize> Neg for Vector<N> {
    type Output = Vector<N>;

    fn neg(mut self) -> Self::Output {
        for i in 0..N {
            self[i] = -self[i]
        }

        self
    }
}

pub type Vector2 = Vector<2>;
pub type Vector3 = Vector<3>;
pub type Vector4 = Vector<4>;

impl_deref_to!(Vector2 => Point2);
impl_deref_to!(Vector3 => Point3);
impl_deref_to!(Vector4 => Point4);

impl Vector2 {
    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self([x, y])
    }

    #[inline]
    pub fn x() -> Self {
        Self([1.0, 0.0])
    }

    #[inline]
    pub fn y() -> Self {
        Self([0.0, 1.0])
    }

    #[inline]
    pub fn with_x(mut self, x: f32) -> Self {
        self[0] = x;
        self
    }

    #[inline]
    pub fn with_y(mut self, y: f32) -> Self {
        self[1] = y;
        self
    }

    #[inline]
    /// Creates a counter-clockwise 90 deg perpendicular vector
    pub fn perp_ccw(&self) -> Self {
        Self([-self.y, self.x])
    }

    #[inline]
    /// Creates a clockwise 90 deg perpendicular vector
    pub fn perp_cw(&self) -> Self {
        Self([self.x, -self.y])
    }

    #[inline]
    /// 2D cross product — returns the z-component of the 3D cross
    /// Useful for determining winding order / signed area
    pub fn cross(&self, rhs: &Self) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }

    #[inline]
    // Angle in radians from positive X axis (-PI to PI)
    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    #[inline]
    /// Signed angle to another vector (-PI to PI)
    pub fn signed_angle_to(&self, rhs: &Self) -> f32 {
        self.cross(rhs).atan2(self.dot(rhs))
    }

    #[inline]
    /// Creates an unit vector from angle (radians)
    pub fn from_angle(rad: f32) -> Self {
        Self([rad.cos(), rad.sin()])
    }

    #[inline]
    /// Rotates by angle (radians)
    pub fn rotated(&self, rad: f32) -> Self {
        let (sin, cos) = rad.sin_cos();

        Self([self.x * cos - self.y * sin, self.x * sin - self.y * cos])
    }

    #[inline]
    /// Extend to Vector3
    pub fn extend(&self, z: f32) -> Vector3 {
        Vector([self.x, self.y, z])
    }
}

impl Vector3 {
    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self([x, y, z])
    }

    #[inline]
    pub fn x() -> Self {
        Self([1.0, 0.0, 0.0])
    }

    #[inline]
    pub fn y() -> Self {
        Self([0.0, 1.0, 0.0])
    }

    #[inline]
    pub fn z() -> Self {
        Self([0.0, 0.0, 1.0])
    }

    #[inline]
    pub fn with_x(mut self, x: f32) -> Self {
        self.x = x;
        self
    }

    #[inline]
    pub fn with_y(mut self, y: f32) -> Self {
        self.y = y;
        self
    }

    #[inline]
    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    #[inline]
    /// Cross product
    pub fn cross(&self, rhs: &Self) -> Self {
        Self([
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        ])
    }

    #[inline]
    /// Triple scalar product: self · (a × b)
    pub fn triple_scalar(&self, a: &Self, b: &Self) -> f32 {
        self.dot(&a.cross(b))
    }

    #[inline]
    /// Extend to Vector4
    pub fn extend(&self, w: f32) -> Vector4 {
        Vector([self.x, self.y, self.z, w])
    }

    #[inline]
    /// Truncate to Vector2
    pub fn truncate(&self) -> Vector2 {
        Vector([self.x, self.y])
    }

    /// Swizzle
    #[inline]
    pub fn xy(&self) -> Vector2 {
        Vector([self.x, self.y])
    }

    #[inline]
    pub fn xz(&self) -> Vector2 {
        Vector([self.x, self.z])
    }

    #[inline]
    pub fn yz(&self) -> Vector2 {
        Vector([self.y, self.z])
    }
}

impl Vector4 {
    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self([x, y, z, w])
    }

    #[inline]
    pub fn x() -> Self {
        Self([1.0, 0.0, 0.0, 0.0])
    }

    #[inline]
    pub fn y() -> Self {
        Self([0.0, 1.0, 0.0, 0.0])
    }

    #[inline]
    pub fn z() -> Self {
        Self([0.0, 0.0, 1.0, 0.0])
    }

    #[inline]
    pub fn w() -> Self {
        Self([0.0, 0.0, 0.0, 1.0])
    }

    #[inline]
    pub fn with_x(mut self, x: f32) -> Self {
        self.x = x;
        self
    }

    #[inline]
    pub fn with_y(mut self, y: f32) -> Self {
        self.y = y;
        self
    }

    #[inline]
    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    #[inline]
    pub fn with_w(mut self, w: f32) -> Self {
        self.w = w;
        self
    }

    #[inline]
    pub fn truncate2(&self) -> Vector2 {
        Vector([self.x, self.y])
    }

    #[inline]
    pub fn truncate(&self) -> Vector3 {
        Vector([self.x, self.y, self.z])
    }

    #[inline]
    pub fn perspective_divide(&self) -> Vector3 {
        let w = self.w;
        Vector([self.x / w, self.y / w, self.z / w])
    }

    #[inline]
    pub fn from_point(p: Vector3) -> Self {
        Self([p.x, p.y, p.z, 1.0])
    }

    #[inline]
    pub fn from_direction(d: Vector3) -> Self {
        Self([d.x, d.y, d.z, 0.0])
    }

    #[inline]
    pub fn is_point(&self) -> bool {
        self.w.abs() > f32::EPSILON
    }

    #[inline]
    pub fn is_direction(&self) -> bool {
        self.w.abs() <= f32::EPSILON
    }

    #[inline]
    pub fn xyz(&self) -> Vector3 {
        self.truncate()
    }

    #[inline]
    pub fn xy(&self) -> Vector2 {
        Vector([self.x, self.y])
    }

    #[inline]
    pub fn xz(&self) -> Vector2 {
        Vector([self.x, self.z])
    }

    #[inline]
    pub fn yz(&self) -> Vector2 {
        Vector([self.y, self.z])
    }

    #[inline]
    pub fn xw(&self) -> Vector2 {
        Vector([self.x, self.w])
    }

    #[inline]
    pub fn zw(&self) -> Vector2 {
        Vector([self.z, self.w])
    }
}
