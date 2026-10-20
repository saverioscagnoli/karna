use core::mem;
use core::ops::Index;
use core::ops::IndexMut;
use core::ops::Neg;
use core::slice;

use num_traits::Num;

use crate::SdlFloat;
use crate::Vector;
use crate::Vector3;
use crate::Vector4;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
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
        unsafe { slice::from_raw_parts(self.0.as_ptr() as *const u8, mem::size_of::<Self>()) }
    }

    pub fn from_cols(cols: [[T; R]; C]) -> Self {
        Self(cols)
    }

    pub fn from_cols_slice(data: &[T]) -> Self {
        assert_eq!(data.len(), R * C, "expected {} elements", R * C);

        let mut m = Self::zero();
        for col in 0..C {
            for row in 0..R {
                m[col][row] = data[col * R + row];
            }
        }
        m
    }

    pub fn from_rows_slice(data: &[T]) -> Self {
        assert_eq!(data.len(), R * C, "expected {} elements", R * C);

        let mut m = Self::zero();
        for col in 0..C {
            for row in 0..R {
                m[col][row] = data[row * C + col];
            }
        }
        m
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
}

impl<T: Num + SdlFloat> Matrix2<T> {
    pub fn from_angle(angle: T) -> Self {
        let s = angle.sdl_sin();
        let c = angle.sdl_cos();

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

impl<T: Num + Copy + Neg<Output = T>> Matrix3<T> {
    pub fn det(&self) -> T {
        self[0][0] * (self[1][1] * self[2][2] - self[2][1] * self[1][2])
            - self[1][0] * (self[0][1] * self[2][2] - self[2][1] * self[0][2])
            + self[2][0] * (self[0][1] * self[1][2] - self[1][1] * self[0][2])
    }
}

impl<T: Num + SdlFloat> Matrix3<T> {
    pub fn from_axis_angle(axis: &Vector3<T>, angle: T) -> Self {
        let s = angle.sdl_sin();
        let c = angle.sdl_cos();
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

impl<T: Num + SdlFloat> Matrix4<T> {
    pub fn from_rotation_y(radians: T) -> Self {
        let sin = radians.sdl_sin();
        let cos = radians.sdl_cos();

        Self([
            [cos, T::zero(), -sin, T::zero()],
            [T::zero(), T::one(), T::zero(), T::zero()],
            [sin, T::zero(), cos, T::zero()],
            [T::zero(), T::zero(), T::zero(), T::one()],
        ])
    }

    pub fn from_rotation_z(radians: T) -> Self {
        let sin = radians.sdl_sin();
        let cos = radians.sdl_cos();

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
        let tan_half = (fov_y / two).sdl_tan();
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

impl<T: Num + Copy + Neg<Output = T>> Matrix4<T> {
    pub fn det(&self) -> T {
        let a = &self.0;
        let s0 = a[0][0] * a[1][1] - a[1][0] * a[0][1];
        let s1 = a[0][0] * a[1][2] - a[1][0] * a[0][2];
        let s2 = a[0][0] * a[1][3] - a[1][0] * a[0][3];
        let s3 = a[0][1] * a[1][2] - a[1][1] * a[0][2];
        let s4 = a[0][1] * a[1][3] - a[1][1] * a[0][3];
        let s5 = a[0][2] * a[1][3] - a[1][2] * a[0][3];
        let c5 = a[2][2] * a[3][3] - a[3][2] * a[2][3];
        let c4 = a[2][1] * a[3][3] - a[3][1] * a[2][3];
        let c3 = a[2][1] * a[3][2] - a[3][1] * a[2][2];
        let c2 = a[2][0] * a[3][3] - a[3][0] * a[2][3];
        let c1 = a[2][0] * a[3][2] - a[3][0] * a[2][2];
        let c0 = a[2][0] * a[3][1] - a[3][0] * a[2][1];
        s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0
    }

    /// Returns None if the matrix is singular.
    pub fn inverse(&self) -> Option<Self> {
        let a = &self.0;

        let s0 = a[0][0] * a[1][1] - a[1][0] * a[0][1];
        let s1 = a[0][0] * a[1][2] - a[1][0] * a[0][2];
        let s2 = a[0][0] * a[1][3] - a[1][0] * a[0][3];
        let s3 = a[0][1] * a[1][2] - a[1][1] * a[0][2];
        let s4 = a[0][1] * a[1][3] - a[1][1] * a[0][3];
        let s5 = a[0][2] * a[1][3] - a[1][2] * a[0][3];

        let c5 = a[2][2] * a[3][3] - a[3][2] * a[2][3];
        let c4 = a[2][1] * a[3][3] - a[3][1] * a[2][3];
        let c3 = a[2][1] * a[3][2] - a[3][1] * a[2][2];
        let c2 = a[2][0] * a[3][3] - a[3][0] * a[2][3];
        let c1 = a[2][0] * a[3][2] - a[3][0] * a[2][2];
        let c0 = a[2][0] * a[3][1] - a[3][0] * a[2][1];

        let det = s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0;
        if det == T::zero() {
            return None;
        }
        let inv = T::one() / det;

        let mut b = Self::zero();
        b[0][0] = (a[1][1] * c5 - a[1][2] * c4 + a[1][3] * c3) * inv;
        b[0][1] = (-a[0][1] * c5 + a[0][2] * c4 - a[0][3] * c3) * inv;
        b[0][2] = (a[3][1] * s5 - a[3][2] * s4 + a[3][3] * s3) * inv;
        b[0][3] = (-a[2][1] * s5 + a[2][2] * s4 - a[2][3] * s3) * inv;

        b[1][0] = (-a[1][0] * c5 + a[1][2] * c2 - a[1][3] * c1) * inv;
        b[1][1] = (a[0][0] * c5 - a[0][2] * c2 + a[0][3] * c1) * inv;
        b[1][2] = (-a[3][0] * s5 + a[3][2] * s2 - a[3][3] * s1) * inv;
        b[1][3] = (a[2][0] * s5 - a[2][2] * s2 + a[2][3] * s1) * inv;

        b[2][0] = (a[1][0] * c4 - a[1][1] * c2 + a[1][3] * c0) * inv;
        b[2][1] = (-a[0][0] * c4 + a[0][1] * c2 - a[0][3] * c0) * inv;
        b[2][2] = (a[3][0] * s4 - a[3][1] * s2 + a[3][3] * s0) * inv;
        b[2][3] = (-a[2][0] * s4 + a[2][1] * s2 - a[2][3] * s0) * inv;

        b[3][0] = (-a[1][0] * c3 + a[1][1] * c1 - a[1][2] * c0) * inv;
        b[3][1] = (a[0][0] * c3 - a[0][1] * c1 + a[0][2] * c0) * inv;
        b[3][2] = (-a[3][0] * s3 + a[3][1] * s1 - a[3][2] * s0) * inv;
        b[3][3] = (a[2][0] * s3 - a[2][1] * s1 + a[2][2] * s0) * inv;

        Some(b)
    }

    /// Multiplies by (x, y, z, w) and returns the raw 4D result (no divide).
    pub fn transform4(&self, v: Vector4<T>) -> Vector4<T> {
        let mut out = Vector4::zero();

        for row in 0..4 {
            let mut sum = T::zero();

            for col in 0..4 {
                sum = sum + self[col][row] * v[col];
            }

            out[row] = sum;
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Vector2;
    use core::f32::consts::FRAC_PI_2;

    const EPS: f32 = 1e-5;

    fn approx(a: f32, b: f32) -> bool {
        let d = if a > b { a - b } else { b - a };
        d < EPS
    }

    fn approx_mat4(a: Matrix4<f32>, b: Matrix4<f32>) -> bool {
        (0..4).all(|c| (0..4).all(|r| approx(a[c][r], b[c][r])))
    }

    fn approx_vec4(a: Vector4<f32>, b: Vector4<f32>) -> bool {
        (0..4).all(|i| approx(a[i], b[i]))
    }

    fn point(x: f32, y: f32, z: f32) -> Vector4<f32> {
        Vector4::new(x, y, z, 1.0)
    }

    #[test]
    fn default_is_identity() {
        assert_eq!(
            Matrix3::<i32>::default().as_array(),
            [[1, 0, 0], [0, 1, 0], [0, 0, 1]]
        );
        assert_eq!(Matrix4::<i32>::identity().trace(), 4);
    }

    #[test]
    fn identity_non_square() {
        let m = Matrix::<2, 3, i32>::identity();
        assert_eq!(m.as_array(), [[1, 0], [0, 1], [0, 0]]);
    }

    #[test]
    fn splat_zero_one() {
        assert_eq!(Matrix2::splat(7).as_array(), [[7, 7], [7, 7]]);
        assert_eq!(Matrix2::<i32>::zero().as_array(), [[0, 0], [0, 0]]);
        assert_eq!(Matrix2::<i32>::one().as_array(), [[1, 1], [1, 1]]);
    }

    #[test]
    fn index_is_column_major() {
        let mut m = Matrix2::new(1, 2, 3, 4);
        assert_eq!(m[0], [1, 2]);
        assert_eq!(m[1][0], 3);
        m[1][1] = 9;
        assert_eq!(m.as_array(), [[1, 2], [3, 9]]);
    }

    #[test]
    fn from_cols() {
        let m = Matrix2::from_cols([[1, 2], [3, 4]]);
        assert_eq!(m.as_array(), Matrix2::new(1, 2, 3, 4).as_array());
    }

    #[test]
    fn from_cols_slice() {
        let m = Matrix::<2, 3, i32>::from_cols_slice(&[1, 2, 3, 4, 5, 6]);
        assert_eq!(m.as_array(), [[1, 2], [3, 4], [5, 6]]);
    }

    #[test]
    fn from_rows_slice() {
        let m = Matrix::<2, 3, i32>::from_rows_slice(&[1, 2, 3, 4, 5, 6]);
        assert_eq!(m.as_array(), [[1, 4], [2, 5], [3, 6]]);
    }

    #[test]
    fn rows_slice_is_transpose_of_cols_slice() {
        let data = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let a = Matrix3::from_rows_slice(&data);
        let b = Matrix3::from_cols_slice(&data).transpose();
        assert_eq!(a.as_array(), b.as_array());
    }

    #[test]
    #[should_panic]
    fn from_cols_slice_wrong_len_panics() {
        Matrix2::<i32>::from_cols_slice(&[1, 2, 3]);
    }

    #[test]
    #[should_panic]
    fn from_rows_slice_wrong_len_panics() {
        Matrix2::<i32>::from_rows_slice(&[1, 2, 3, 4, 5]);
    }

    #[test]
    fn transpose_non_square() {
        let m = Matrix::<2, 3, i32>::from_rows_slice(&[1, 2, 3, 4, 5, 6]);
        let t: Matrix<3, 2, i32> = m.transpose();
        assert_eq!(t.as_array(), [[1, 2, 3], [4, 5, 6]]);
    }

    #[test]
    fn transpose_twice_is_identity() {
        let m = Matrix3::from_cols_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(m.transpose().transpose().as_array(), m.as_array());
    }

    #[test]
    fn map() {
        let m = Matrix2::new(1, 2, 3, 4).map(|x| x * 10);
        assert_eq!(m.as_array(), [[10, 20], [30, 40]]);
    }

    #[test]
    fn matmul_identity() {
        let m = Matrix3::from_cols_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(m.matmul(&Matrix3::identity()).as_array(), m.as_array());
        assert_eq!(Matrix3::identity().matmul(&m).as_array(), m.as_array());
    }

    #[test]
    fn matmul_non_square() {
        let a = Matrix::<2, 3, i32>::from_rows_slice(&[1, 2, 3, 4, 5, 6]);
        let b = Matrix::<3, 2, i32>::from_rows_slice(&[7, 8, 9, 10, 11, 12]);
        let c: Matrix2<i32> = a.matmul(&b);
        assert_eq!(
            c.as_array(),
            Matrix2::from_rows_slice(&[58, 64, 139, 154]).as_array()
        );
    }

    #[test]
    fn matmul_is_not_commutative() {
        let a = Matrix2::from_rows_slice(&[1, 2, 3, 4]);
        let b = Matrix2::from_rows_slice(&[0, 1, 1, 0]);
        assert_ne!(a.matmul(&b).as_array(), b.matmul(&a).as_array());
    }

    #[test]
    fn mul_vec() {
        let a = Matrix::<2, 3, i32>::from_rows_slice(&[1, 2, 3, 4, 5, 6]);
        let v = a.mul_vec(&Vector3::new(1, 1, 1));
        assert_eq!(v.as_array(), [6, 15]);
    }

    #[test]
    fn mul_vec_matches_matmul() {
        let a = Matrix3::from_cols_slice(&[2, 0, 1, 3, 1, 0, 0, 4, 5]);
        let b = Matrix3::from_cols_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let v = Vector3::new(1, -2, 3);
        let lhs = a.matmul(&b).mul_vec(&v);
        let rhs = a.mul_vec(&b.mul_vec(&v));
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn trace() {
        assert_eq!(
            Matrix3::from_cols_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9]).trace(),
            15
        );
    }

    #[test]
    fn as_bytes_and_ptr() {
        let m = Matrix4::<f32>::identity();
        assert_eq!(m.as_bytes().len(), 64);
        assert_eq!(&m.as_bytes()[0..4], &1.0f32.to_ne_bytes());
        assert_eq!(
            m.as_ptr() as *const f32,
            m.as_bytes().as_ptr() as *const f32
        );
    }

    #[test]
    fn det2() {
        assert_eq!(Matrix2::from_rows_slice(&[3, 8, 4, 6]).det(), -14);
        assert_eq!(Matrix2::<i32>::identity().det(), 1);
    }

    #[test]
    fn det3() {
        assert_eq!(
            Matrix3::from_rows_slice(&[6, 1, 1, 4, -2, 5, 2, 8, 7]).det(),
            -306
        );
        assert_eq!(
            Matrix3::from_rows_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9]).det(),
            0
        );
    }

    #[test]
    fn det4() {
        assert_eq!(Matrix4::<i32>::identity().det(), 1);
        assert_eq!(
            Matrix4::from_scale(Vector3::new(2.0f32, 3.0, 4.0)).det(),
            24.0
        );
        let m = Matrix4::from_rows_slice(&[1, 0, 2, -1, 3, 0, 0, 5, 2, 1, 4, -3, 1, 0, 5, 0]);
        assert_eq!(m.det(), 30);
    }

    #[test]
    fn det_of_product_is_product_of_dets() {
        let a = Matrix3::from_rows_slice(&[2, 0, 1, 1, 3, 2, 1, 1, 1]);
        let b = Matrix3::from_rows_slice(&[1, 2, 0, 0, 1, 4, 5, 6, 0]);
        assert_eq!(a.matmul(&b).det(), a.det() * b.det());
    }

    #[test]
    fn matrix2_from_angle() {
        let m = Matrix2::from_angle(FRAC_PI_2);
        let v = m.mul_vec(&Vector2::new(1.0, 0.0));
        assert!(approx(v[0], 0.0) && approx(v[1], 1.0));
        assert!(approx(m.det(), 1.0));
    }

    #[test]
    fn matrix3_axis_angle_z() {
        let m = Matrix3::from_axis_angle(&Vector3::new(0.0, 0.0, 1.0), FRAC_PI_2);
        let v = m.mul_vec(&Vector3::new(1.0, 0.0, 0.0));
        assert!(approx(v[0], 0.0) && approx(v[1], 1.0) && approx(v[2], 0.0));
        assert!(approx(m.det(), 1.0));
    }

    #[test]
    fn matrix3_2d_affine() {
        let t = Matrix3::from_translation(5.0, 6.0);
        let s = Matrix3::from_scale(2.0, 3.0);
        let v = t.matmul(&s).mul_vec(&Vector3::new(1.0, 1.0, 1.0));
        assert_eq!(v.as_array(), [7.0, 9.0, 1.0]);
    }

    #[test]
    fn matrix3_translation_ignores_directions() {
        let t = Matrix3::from_translation(5.0, 6.0);
        let v = t.mul_vec(&Vector3::new(1.0, 1.0, 0.0));
        assert_eq!(v.as_array(), [1.0, 1.0, 0.0]);
    }

    #[test]
    fn matrix4_translation() {
        let m = Matrix4::from_translation(Vector3::new(1.0, 2.0, 3.0));
        assert!(approx_vec4(
            m.transform4(point(1.0, 1.0, 1.0)),
            point(2.0, 3.0, 4.0)
        ));
        let dir = Vector4::new(1.0, 1.0, 1.0, 0.0);
        assert!(approx_vec4(m.transform4(dir), dir));
    }

    #[test]
    fn matrix4_scale() {
        let m = Matrix4::from_scale(Vector3::new(2.0, 3.0, 4.0));
        assert!(approx_vec4(
            m.transform4(point(1.0, 1.0, 1.0)),
            point(2.0, 3.0, 4.0)
        ));
    }

    #[test]
    fn matrix4_rotation_z() {
        let m = Matrix4::from_rotation_z(FRAC_PI_2);
        assert!(approx_vec4(
            m.transform4(point(1.0, 0.0, 0.0)),
            point(0.0, 1.0, 0.0)
        ));
    }

    #[test]
    fn matrix4_rotation_y() {
        let m = Matrix4::from_rotation_y(FRAC_PI_2);
        assert!(approx_vec4(
            m.transform4(point(1.0, 0.0, 0.0)),
            point(0.0, 0.0, -1.0)
        ));
        assert!(approx_vec4(
            m.transform4(point(0.0, 0.0, 1.0)),
            point(1.0, 0.0, 0.0)
        ));
    }

    #[test]
    fn matrix4_axis_angle_matches_rotation_helpers() {
        let a = 0.7;
        let z = Matrix4::from_axis_angle(&Vector3::new(0.0, 0.0, 1.0), a);
        let y = Matrix4::from_axis_angle(&Vector3::new(0.0, 1.0, 0.0), a);
        assert!(approx_mat4(z, Matrix4::from_rotation_z(a)));
        assert!(approx_mat4(y, Matrix4::from_rotation_y(a)));
    }

    #[test]
    fn matrix4_axis_angle_keeps_last_row_and_column() {
        let m = Matrix4::from_axis_angle(&Vector3::new(1.0, 2.0, 3.0).normalize(), 1.3);
        assert_eq!(
            [
                m[0][3], m[1][3], m[2][3], m[3][0], m[3][1], m[3][2], m[3][3]
            ],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        );
    }

    #[test]
    fn inverse_of_identity() {
        let m = Matrix4::<f32>::identity();
        assert!(approx_mat4(m.inverse().unwrap(), m));
    }

    #[test]
    fn inverse_round_trip() {
        let m = Matrix4::from_translation(Vector3::new(1.0, -2.0, 3.0))
            .matmul(&Matrix4::from_rotation_y(0.6))
            .matmul(&Matrix4::from_scale(Vector3::new(2.0, 0.5, 3.0)));
        let inv = m.inverse().unwrap();
        assert!(approx_mat4(m.matmul(&inv), Matrix4::identity()));
        assert!(approx_mat4(inv.matmul(&m), Matrix4::identity()));
    }

    #[test]
    fn inverse_of_translation_negates() {
        let m = Matrix4::from_translation(Vector3::new(1.0, 2.0, 3.0));
        let expected = Matrix4::from_translation(Vector3::new(-1.0, -2.0, -3.0));
        assert!(approx_mat4(m.inverse().unwrap(), expected));
    }

    #[test]
    fn inverse_of_singular_is_none() {
        assert!(Matrix4::<f32>::zero().inverse().is_none());
        assert!(
            Matrix4::from_scale(Vector3::new(1.0f32, 0.0, 1.0))
                .inverse()
                .is_none()
        );
    }

    #[test]
    fn perspective_maps_near_and_far() {
        let (near, far) = (0.1, 100.0);
        let m = Matrix4::perspective(FRAC_PI_2, 1.0, near, far);
        let n = m.transform4(point(0.0, 0.0, near)).perspective_divide();
        let f = m.transform4(point(0.0, 0.0, far)).perspective_divide();
        assert!(approx(n[2], 0.0));
        assert!(approx(f[2], 1.0));
    }

    #[test]
    fn perspective_fov_edge_maps_to_one() {
        let m = Matrix4::perspective(FRAC_PI_2, 2.0, 0.1, 100.0);
        let top = m.transform4(point(0.0, 5.0, 5.0)).perspective_divide();
        let right = m.transform4(point(10.0, 0.0, 5.0)).perspective_divide();
        assert!(approx(top[1], 1.0));
        assert!(approx(right[0], 1.0));
    }

    #[test]
    fn orthographic_maps_box_to_clip() {
        let m = Matrix4::orthographic(0.0, 800.0, 600.0, 0.0, 0.0, 1.0);
        let tl = m.transform4(point(0.0, 0.0, 0.0));
        let br = m.transform4(point(800.0, 600.0, 1.0));
        assert!(approx_vec4(tl, Vector4::new(-1.0, 1.0, 0.0, 1.0)));
        assert!(approx_vec4(br, Vector4::new(1.0, -1.0, 1.0, 1.0)));
    }

    #[test]
    fn look_at_moves_eye_to_origin() {
        let eye = Vector3::new(1.0, 2.0, 3.0);
        let m = Matrix4::look_at(
            &eye,
            &Vector3::new(0.0, 0.0, 0.0),
            &Vector3::new(0.0, 1.0, 0.0),
        );
        assert!(approx_vec4(
            m.transform4(point(1.0, 2.0, 3.0)),
            point(0.0, 0.0, 0.0)
        ));
    }

    #[test]
    fn look_at_puts_target_on_forward_axis() {
        let eye = Vector3::new(0.0, 0.0, -5.0);
        let up = Vector3::new(0.0, 1.0, 0.0);
        let m = Matrix4::look_at(&eye, &Vector3::new(0.0, 0.0, 0.0), &up);
        assert!(approx_vec4(
            m.transform4(point(0.0, 0.0, 0.0)),
            point(0.0, 0.0, 5.0)
        ));
        let above = m.transform4(point(0.0, 1.0, -5.0));
        assert!(approx(above[0], 0.0) && approx(above[1], 1.0) && approx(above[2], 0.0));
    }

    #[test]
    fn look_at_is_rigid() {
        let m = Matrix4::look_at(
            &Vector3::new(3.0, 1.0, -2.0),
            &Vector3::new(-1.0, 0.5, 4.0),
            &Vector3::new(0.0, 1.0, 0.0),
        );
        let a = m.transform4(point(1.0, 1.0, 1.0)).xyz();
        let b = m.transform4(point(2.0, -1.0, 3.0)).xyz();
        let d = Vector3::new(1.0, -2.0, 2.0).length();
        assert!(approx(a.distance(&b), d));
    }

    #[test]
    fn transform4_matches_mul_vec() {
        let m = Matrix4::from_rotation_y(0.4)
            .matmul(&Matrix4::from_translation(Vector3::new(1.0, 2.0, 3.0)));
        let v = Vector4::new(1.0, -1.0, 2.0, 1.0);
        assert!(approx_vec4(m.transform4(v), m.mul_vec(&v)));
    }

    #[test]
    fn equality() {
        assert_eq!(
            Matrix2::new(1, 2, 3, 4),
            Matrix2::from_cols([[1, 2], [3, 4]])
        );
        assert_ne!(Matrix2::new(1, 2, 3, 4), Matrix2::new(1, 2, 3, 5));
        assert_eq!(Matrix3::<f32>::identity(), Matrix3::default());
    }
}
