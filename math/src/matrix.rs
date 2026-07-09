use std::ops::Index;
use std::ops::IndexMut;
use std::slice;

use num::Float;
use num::Num;

use crate::Vector;
use crate::Vector3;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Matrix<const R: usize, const C: usize, T: Num + Copy>([[T; R]; C]);

pub type Matrix2<T> = Matrix<2, 2, T>;
pub type Matrix3<T> = Matrix<3, 3, T>;
pub type Matrix4<T> = Matrix<4, 4, T>;

impl<const R: usize, const C: usize, T: Num + Copy> Default for Matrix<R, C, T> {
    fn default() -> Self {
        let mut result = Self::zero();

        for col in 0..C {
            for row in 0..R {
                if col == row {
                    result[col][row] = T::one()
                }
            }
        }

        result
    }
}

impl<const R: usize, const C: usize, T: Num + Copy> Index<usize> for Matrix<R, C, T> {
    type Output = [T; R];

    fn index(&self, col: usize) -> &Self::Output {
        &self.0[col]
    }
}

impl<const R: usize, const C: usize, T: Num + Copy> IndexMut<usize> for Matrix<R, C, T> {
    fn index_mut(&mut self, col: usize) -> &mut Self::Output {
        &mut self.0[col]
    }
}

impl<const R: usize, const C: usize, T: Num + Copy> Matrix<R, C, T> {
    pub fn splat(v: T) -> Self {
        Self([[v; R]; C])
    }

    pub fn zero() -> Self {
        Self::splat(T::zero())
    }

    pub fn one() -> Self {
        Self::splat(T::one())
    }

    pub fn identity() -> Self {
        Self::default()
    }

    pub fn as_array(&self) -> [[T; R]; C] {
        self.0
    }

    pub fn as_ptr(&self) -> *const [T] {
        self.0.as_ptr()
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.0.as_ptr() as *const u8, std::mem::size_of::<Self>()) }
    }

    pub fn transpose(&self) -> Matrix<C, R, T> {
        let mut result = Matrix::<C, R, T>::zero();

        for col in 0..C {
            for row in 0..R {
                result[row][col] = self[col][row];
            }
        }

        result
    }

    pub fn map<F: Fn(T) -> T>(&self, f: F) -> Self {
        let mut result = Self::zero();
        for col in 0..C {
            for row in 0..R {
                result[col][row] = f(self[col][row]);
            }
        }
        result
    }

    // multiply two matrices: self is R×C, rhs is C×K, result is R×K
    pub fn matmul<const K: usize>(&self, rhs: &Matrix<C, K, T>) -> Matrix<R, K, T> {
        let mut result = Matrix::<R, K, T>::zero();

        for col in 0..K {
            for row in 0..R {
                let mut sum = T::zero();

                for k in 0..C {
                    sum = sum + self[k][row] * rhs[col][k];
                }

                result[col][row] = sum;
            }
        }

        result
    }

    // multiply matrix by a vector: self is R×C, vec is C, result is R
    pub fn mul_vec(&self, rhs: &Vector<C, T>) -> Vector<R, T> {
        let mut result = Vector::<R, T>::zero();

        for row in 0..R {
            let mut sum = T::zero();

            for col in 0..C {
                sum = sum + self[col][row] * rhs[col];
            }

            result[row] = sum;
        }

        result
    }
}

// Square matrix
impl<const N: usize, T: Num + Copy> Matrix<N, N, T> {
    pub fn trace(&self) -> T {
        let mut sum = T::zero();

        for i in 0..N {
            sum = sum + self[i][i];
        }

        sum
    }
}

impl<T: Num + Copy> Matrix2<T> {
    pub fn new(c0r0: T, c0r1: T, c1r0: T, c1r1: T) -> Self {
        Self([[c0r0, c0r1], [c1r0, c1r1]])
    }

    pub fn det(&self) -> T {
        self[0][0] * self[1][1] - self[1][0] * self[0][1]
    }

    pub fn inverse(&self) -> Option<Self>
    where
        T: PartialEq,
    {
        let d = self.det();
        if d == T::zero() {
            return None;
        }
        // adjugate / det — needs Div
        todo!()
    }
}

impl<T: Float> Matrix2<T> {
    pub fn from_angle(angle: T) -> Self {
        let (s, c) = angle.sin_cos();
        Self::new(c, s, -s, c)
    }
}

impl<T: Num + Copy> Matrix3<T> {
    pub fn new(
        c0r0: T,
        c0r1: T,
        c0r2: T,
        c1r0: T,
        c1r1: T,
        c1r2: T,
        c2r0: T,
        c2r1: T,
        c2r2: T,
    ) -> Self {
        Self([[c0r0, c0r1, c0r2], [c1r0, c1r1, c1r2], [c2r0, c2r1, c2r2]])
    }
}

impl<T: Num + Copy + std::ops::Neg<Output = T>> Matrix3<T> {
    pub fn det(&self) -> T {
        self[0][0] * (self[1][1] * self[2][2] - self[2][1] * self[1][2])
            - self[1][0] * (self[0][1] * self[2][2] - self[2][1] * self[0][2])
            + self[2][0] * (self[0][1] * self[1][2] - self[1][1] * self[0][2])
    }
}

impl<T: Float> Matrix3<T> {
    pub fn from_axis_angle(axis: &Vector3<T>, angle: T) -> Self {
        let (s, c) = angle.sin_cos();
        let t = T::one() - c;
        let (x, y, z) = (axis[0], axis[1], axis[2]);

        Self::new(
            t * x * x + c,
            t * x * y + s * z,
            t * x * z - s * y,
            t * x * y - s * z,
            t * y * y + c,
            t * y * z + s * x,
            t * x * z + s * y,
            t * y * z - s * x,
            t * z * z + c,
        )
    }

    pub fn from_scale(x: T, y: T) -> Self {
        let mut m = Self::identity();
        m[0][0] = x;
        m[1][1] = y;
        m
    }

    pub fn from_translation(x: T, y: T) -> Self {
        let mut m = Self::identity();
        m[2][0] = x;
        m[2][1] = y;
        m
    }
}

impl<T: Num + Copy> Matrix4<T> {
    pub fn new(
        c0r0: T,
        c0r1: T,
        c0r2: T,
        c0r3: T,
        c1r0: T,
        c1r1: T,
        c1r2: T,
        c1r3: T,
        c2r0: T,
        c2r1: T,
        c2r2: T,
        c2r3: T,
        c3r0: T,
        c3r1: T,
        c3r2: T,
        c3r3: T,
    ) -> Self {
        Self([
            [c0r0, c0r1, c0r2, c0r3],
            [c1r0, c1r1, c1r2, c1r3],
            [c2r0, c2r1, c2r2, c2r3],
            [c3r0, c3r1, c3r2, c3r3],
        ])
    }
}

impl<T: Float> Matrix4<T> {
    pub fn from_rotation_y(radians: T) -> Self {
        let (sin, cos) = radians.sin_cos();

        Self([
            [cos, T::zero(), -sin, T::zero()],
            [T::zero(), T::one(), T::zero(), T::zero()],
            [sin, T::zero(), cos, T::zero()],
            [T::zero(), T::zero(), T::zero(), T::one()],
        ])
    }

    pub fn from_rotation_z(radians: T) -> Self {
        let (sin, cos) = radians.sin_cos();

        Self([
            [cos, sin, T::zero(), T::zero()],
            [-sin, cos, T::zero(), T::zero()],
            [T::zero(), T::zero(), T::one(), T::zero()],
            [T::zero(), T::zero(), T::zero(), T::one()],
        ])
    }

    pub fn from_translation(v: Vector3<T>) -> Self {
        let mut m = Self::identity();

        m[3][0] = v.x;
        m[3][1] = v.y;
        m[3][2] = v.z;
        m
    }

    pub fn from_scale(v: Vector3<T>) -> Self {
        let mut m = Self::identity();

        m[0][0] = v.x;
        m[1][1] = v.y;
        m[2][2] = v.z;
        m
    }

    pub fn from_axis_angle(axis: &Vector3<T>, angle: T) -> Self {
        // embed the Matrix3 rotation into a Matrix4
        let r = Matrix3::from_axis_angle(axis, angle);
        let mut m = Self::identity();

        for col in 0..3 {
            for row in 0..3 {
                m[col][row] = r[col][row];
            }
        }

        m
    }

    // right-handed, maps z into [0, 1] (Vulkan/Metal/DX convention)
    pub fn perspective(fov_y: T, aspect: T, near: T, far: T) -> Self {
        let two = T::one() + T::one();
        let tan_half = (fov_y / two).tan();
        let mut m = Self::zero();

        m[0][0] = T::one() / (aspect * tan_half);
        m[1][1] = T::one() / tan_half;
        m[2][2] = far / (far - near);
        m[2][3] = T::one();
        m[3][2] = -(far * near) / (far - near);
        m
    }

    pub fn orthographic(left: T, right: T, bottom: T, top: T, near: T, far: T) -> Self {
        let two = T::one() + T::one();
        let mut m = Self::identity();

        m[0][0] = two / (right - left);
        m[1][1] = two / (top - bottom);
        m[2][2] = T::one() / (far - near);
        m[3][0] = -(right + left) / (right - left);
        m[3][1] = -(top + bottom) / (top - bottom);
        m[3][2] = -near / (far - near);
        m
    }

    pub fn look_at(eye: &Vector3<T>, center: &Vector3<T>, up: &Vector3<T>) -> Self {
        let f = (center - eye).normalize();
        let r = f.cross(up).normalize();
        let u = r.cross(&f);
        let mut m = Self::identity();

        m[0][0] = r[0];
        m[1][0] = r[1];
        m[2][0] = r[2];
        m[0][1] = u[0];
        m[1][1] = u[1];
        m[2][1] = u[2];
        m[0][2] = f[0];
        m[1][2] = f[1];
        m[2][2] = f[2];
        m[3][0] = -r.dot(eye);
        m[3][1] = -u.dot(eye);
        m[3][2] = -f.dot(eye);
        m
    }
}
