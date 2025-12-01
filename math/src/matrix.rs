use crate::vector::{Vector, Vector2, Vector3, Vector4};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix<const R: usize, const C: usize>([[f32; R]; C]);

pub type Matrix2 = Matrix<2, 2>;
pub type Matrix3 = Matrix<3, 3>;
pub type Matrix4 = Matrix<4, 4>;

impl<const R: usize, const C: usize> Default for Matrix<R, C> {
    fn default() -> Self {
        Self::zeros()
    }
}

impl<const R: usize, const C: usize> Index<usize> for Matrix<R, C> {
    type Output = [f32; R];

    #[inline]
    fn index(&self, col: usize) -> &Self::Output {
        &self.0[col]
    }
}

impl<const R: usize, const C: usize> IndexMut<usize> for Matrix<R, C> {
    #[inline]
    fn index_mut(&mut self, col: usize) -> &mut Self::Output {
        &mut self.0[col]
    }
}

impl<const R: usize, const C: usize> Matrix<R, C> {
    #[inline]
    pub fn zeros() -> Self {
        Self([[0.0; R]; C])
    }

    #[inline]
    pub const fn from_cols(cols: [[f32; R]; C]) -> Self {
        Self(cols)
    }

    #[inline]
    pub fn col(&self, index: usize) -> Vector<R> {
        Vector::from_array(self.0[index])
    }

    #[inline]
    pub fn set_col(&mut self, index: usize, col: Vector<R>) {
        for r in 0..R {
            self.0[index][r] = col[r];
        }
    }

    #[inline]
    pub fn row(&self, index: usize) -> Vector<C> {
        let mut row = [0.0; C];
        for c in 0..C {
            row[c] = self.0[c][index];
        }
        Vector::from_array(row)
    }

    #[inline]
    pub fn set_row(&mut self, index: usize, row: Vector<C>) {
        for c in 0..C {
            self.0[c][index] = row[c];
        }
    }

    #[inline]
    pub fn transpose(&self) -> Matrix<C, R> {
        let mut result = [[0.0; C]; R];
        for c in 0..C {
            for r in 0..R {
                result[r][c] = self.0[c][r];
            }
        }
        Matrix(result)
    }

    #[inline]
    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.0[col][row]
    }

    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        self.0[col][row] = value;
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.0.as_ptr() as *const u8, std::mem::size_of::<Self>())
        }
    }
}

impl<const N: usize> Matrix<N, N> {
    #[inline]
    pub fn identity() -> Self {
        let mut m = Self::zeros();
        for i in 0..N {
            m.0[i][i] = 1.0;
        }
        m
    }

    #[inline]
    pub fn from_diagonal(diag: Vector<N>) -> Self {
        let mut m = Self::zeros();
        for i in 0..N {
            m.0[i][i] = diag[i];
        }
        m
    }

    #[inline]
    pub fn diagonal(&self) -> Vector<N> {
        let mut diag = [0.0; N];
        for i in 0..N {
            diag[i] = self.0[i][i];
        }
        Vector::from_array(diag)
    }

    #[inline]
    pub fn trace(&self) -> f32 {
        let mut sum = 0.0;
        for i in 0..N {
            sum += self.0[i][i];
        }
        sum
    }
}

impl<const R: usize, const C: usize> Neg for Matrix<R, C> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let mut r = [[0.0; R]; C];
        for c in 0..C {
            for row in 0..R {
                r[c][row] = -self.0[c][row];
            }
        }
        Matrix(r)
    }
}

macro_rules! impl_matrix_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<const R: usize, const C: usize> $trait for Matrix<R, C> {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self::Output {
                let mut r = [[0.0; R]; C];
                for c in 0..C {
                    for row in 0..R {
                        r[c][row] = self.0[c][row] $op rhs.0[c][row];
                    }
                }
                Matrix(r)
            }
        }

        impl<const R: usize, const C: usize> $trait<&Matrix<R, C>> for Matrix<R, C> {
            type Output = Self;
            fn $method(self, rhs: &Self) -> Self::Output {
                let mut r = [[0.0; R]; C];
                for c in 0..C {
                    for row in 0..R {
                        r[c][row] = self.0[c][row] $op rhs.0[c][row];
                    }
                }
                Matrix(r)
            }
        }

        impl<const R: usize, const C: usize> $trait for &Matrix<R, C> {
            type Output = Matrix<R, C>;
            fn $method(self, rhs: Self) -> Self::Output {
                let mut r = [[0.0; R]; C];
                for c in 0..C {
                    for row in 0..R {
                        r[c][row] = self.0[c][row] $op rhs.0[c][row];
                    }
                }
                Matrix(r)
            }
        }
    };

    ($trait:ident, $method:ident, $op:tt, scalar) => {
        impl_matrix_op!($trait, $method, $op);

        impl<const R: usize, const C: usize> $trait<f32> for Matrix<R, C> {
            type Output = Self;
            fn $method(self, s: f32) -> Self::Output {
                let mut r = [[0.0; R]; C];
                for c in 0..C {
                    for row in 0..R {
                        r[c][row] = self.0[c][row] $op s;
                    }
                }
                Matrix(r)
            }
        }

        impl<const R: usize, const C: usize> $trait<f32> for &Matrix<R, C> {
            type Output = Matrix<R, C>;
            fn $method(self, s: f32) -> Self::Output {
                let mut r = [[0.0; R]; C];
                for c in 0..C {
                    for row in 0..R {
                        r[c][row] = self.0[c][row] $op s;
                    }
                }
                Matrix(r)
            }
        }
    };

    ($trait:ident, $method:ident, $op:tt, scalar_commutative) => {
        impl_matrix_op!($trait, $method, $op, scalar);

        impl<const R: usize, const C: usize> $trait<Matrix<R, C>> for f32 {
            type Output = Matrix<R, C>;
            fn $method(self, m: Matrix<R, C>) -> Self::Output {
                let mut r = [[0.0; R]; C];
                for c in 0..C {
                    for row in 0..R {
                        r[c][row] = self $op m.0[c][row];
                    }
                }
                Matrix(r)
            }
        }
    };
}

macro_rules! impl_matrix_op_assign {
    (matrix: $trait:ident, $method:ident, $op:tt) => {
        impl<const R: usize, const C: usize> $trait for Matrix<R, C> {
            fn $method(&mut self, rhs: Self) {
                for c in 0..C {
                    for r in 0..R {
                        self.0[c][r] $op rhs.0[c][r];
                    }
                }
            }
        }
    };

    (scalar: $trait:ident, $method:ident, $op:tt) => {
        impl<const R: usize, const C: usize> $trait<f32> for Matrix<R, C> {
            fn $method(&mut self, s: f32) {
                for c in 0..C {
                    for r in 0..R {
                        self.0[c][r] $op s;
                    }
                }
            }
        }
    };

    (both: $trait:ident, $method:ident, $op:tt) => {
        impl_matrix_op_assign!(matrix: $trait, $method, $op);
        impl_matrix_op_assign!(scalar: $trait, $method, $op);
    };
}

impl_matrix_op!(Add, add, +);
impl_matrix_op!(Sub, sub, -);
impl_matrix_op!(Div, div, /, scalar);

impl_matrix_op_assign!(matrix: AddAssign, add_assign, +=);
impl_matrix_op_assign!(matrix: SubAssign, sub_assign, -=);
impl_matrix_op_assign!(scalar: DivAssign, div_assign, /=);

impl<const R: usize, const C: usize> Mul<f32> for Matrix<R, C> {
    type Output = Self;
    fn mul(self, s: f32) -> Self::Output {
        let mut r = [[0.0; R]; C];
        for c in 0..C {
            for row in 0..R {
                r[c][row] = self.0[c][row] * s;
            }
        }
        Matrix(r)
    }
}

impl<const R: usize, const C: usize> Mul<f32> for &Matrix<R, C> {
    type Output = Matrix<R, C>;
    fn mul(self, s: f32) -> Self::Output {
        let mut r = [[0.0; R]; C];
        for c in 0..C {
            for row in 0..R {
                r[c][row] = self.0[c][row] * s;
            }
        }
        Matrix(r)
    }
}

impl<const R: usize, const C: usize> Mul<Matrix<R, C>> for f32 {
    type Output = Matrix<R, C>;
    fn mul(self, m: Matrix<R, C>) -> Self::Output {
        m * self
    }
}

impl<const R: usize, const C: usize> MulAssign<f32> for Matrix<R, C> {
    fn mul_assign(&mut self, s: f32) {
        for c in 0..C {
            for r in 0..R {
                self.0[c][r] *= s;
            }
        }
    }
}

macro_rules! impl_matrix_mul_vector {
    ($r:literal, $c:literal) => {
        impl Mul<Vector<$c>> for Matrix<$r, $c> {
            type Output = Vector<$r>;

            #[inline]
            fn mul(self, v: Vector<$c>) -> Self::Output {
                let mut result = [0.0; $r];
                for c in 0..$c {
                    for r in 0..$r {
                        result[r] += self.0[c][r] * v[c];
                    }
                }
                Vector::from_array(result)
            }
        }

        impl Mul<Vector<$c>> for &Matrix<$r, $c> {
            type Output = Vector<$r>;

            #[inline]
            fn mul(self, v: Vector<$c>) -> Self::Output {
                let mut result = [0.0; $r];
                for c in 0..$c {
                    for r in 0..$r {
                        result[r] += self.0[c][r] * v[c];
                    }
                }
                Vector::from_array(result)
            }
        }

        impl Mul<&Vector<$c>> for Matrix<$r, $c> {
            type Output = Vector<$r>;

            #[inline]
            fn mul(self, v: &Vector<$c>) -> Self::Output {
                let mut result = [0.0; $r];
                for c in 0..$c {
                    for r in 0..$r {
                        result[r] += self.0[c][r] * v[c];
                    }
                }
                Vector::from_array(result)
            }
        }

        impl Mul<&Vector<$c>> for &Matrix<$r, $c> {
            type Output = Vector<$r>;

            #[inline]
            fn mul(self, v: &Vector<$c>) -> Self::Output {
                let mut result = [0.0; $r];
                for c in 0..$c {
                    for r in 0..$r {
                        result[r] += self.0[c][r] * v[c];
                    }
                }
                Vector::from_array(result)
            }
        }
    };
}

macro_rules! impl_matrix_mul_matrix {
    ($r:literal, $m:literal, $c:literal) => {
        impl Mul<Matrix<$m, $c>> for Matrix<$r, $m> {
            type Output = Matrix<$r, $c>;

            #[inline]
            fn mul(self, rhs: Matrix<$m, $c>) -> Self::Output {
                let mut result = [[0.0; $r]; $c];
                for c in 0..$c {
                    for m in 0..$m {
                        for r in 0..$r {
                            result[c][r] += self.0[m][r] * rhs.0[c][m];
                        }
                    }
                }
                Matrix(result)
            }
        }

        impl Mul<&Matrix<$m, $c>> for Matrix<$r, $m> {
            type Output = Matrix<$r, $c>;

            #[inline]
            fn mul(self, rhs: &Matrix<$m, $c>) -> Self::Output {
                let mut result = [[0.0; $r]; $c];
                for c in 0..$c {
                    for m in 0..$m {
                        for r in 0..$r {
                            result[c][r] += self.0[m][r] * rhs.0[c][m];
                        }
                    }
                }
                Matrix(result)
            }
        }

        impl Mul<Matrix<$m, $c>> for &Matrix<$r, $m> {
            type Output = Matrix<$r, $c>;

            #[inline]
            fn mul(self, rhs: Matrix<$m, $c>) -> Self::Output {
                let mut result = [[0.0; $r]; $c];
                for c in 0..$c {
                    for m in 0..$m {
                        for r in 0..$r {
                            result[c][r] += self.0[m][r] * rhs.0[c][m];
                        }
                    }
                }
                Matrix(result)
            }
        }

        impl Mul<&Matrix<$m, $c>> for &Matrix<$r, $m> {
            type Output = Matrix<$r, $c>;

            #[inline]
            fn mul(self, rhs: &Matrix<$m, $c>) -> Self::Output {
                let mut result = [[0.0; $r]; $c];
                for c in 0..$c {
                    for m in 0..$m {
                        for r in 0..$r {
                            result[c][r] += self.0[m][r] * rhs.0[c][m];
                        }
                    }
                }
                Matrix(result)
            }
        }
    };
}

impl_matrix_mul_vector!(2, 2);
impl_matrix_mul_vector!(3, 3);
impl_matrix_mul_vector!(4, 4);

impl_matrix_mul_matrix!(2, 2, 2);
impl_matrix_mul_matrix!(3, 3, 3);
impl_matrix_mul_matrix!(4, 4, 4);

impl Matrix2 {
    #[inline]
    pub fn new(m00: f32, m01: f32, m10: f32, m11: f32) -> Self {
        Self([[m00, m10], [m01, m11]])
    }

    #[inline]
    pub fn from_cols_vec(c0: Vector2, c1: Vector2) -> Self {
        Self([[c0[0], c0[1]], [c1[0], c1[1]]])
    }

    #[inline]
    pub fn from_scale(scale: Vector2) -> Self {
        Self([[scale[0], 0.0], [0.0, scale[1]]])
    }

    #[inline]
    pub fn from_angle(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self([[cos, sin], [-sin, cos]])
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        self.0[0][0] * self.0[1][1] - self.0[1][0] * self.0[0][1]
    }

    #[inline]
    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < f32::EPSILON {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Self([
            [self.0[1][1] * inv_det, -self.0[0][1] * inv_det],
            [-self.0[1][0] * inv_det, self.0[0][0] * inv_det],
        ]))
    }
}

impl Matrix3 {
    #[inline]
    pub fn from_cols_vec(c0: Vector3, c1: Vector3, c2: Vector3) -> Self {
        Self([
            [c0[0], c0[1], c0[2]],
            [c1[0], c1[1], c1[2]],
            [c2[0], c2[1], c2[2]],
        ])
    }

    #[inline]
    pub fn from_scale(scale: Vector2) -> Self {
        Self([[scale[0], 0.0, 0.0], [0.0, scale[1], 0.0], [0.0, 0.0, 1.0]])
    }

    #[inline]
    pub fn from_translation(translation: Vector2) -> Self {
        Self([
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [translation[0], translation[1], 1.0],
        ])
    }

    #[inline]
    pub fn from_angle(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self([[cos, sin, 0.0], [-sin, cos, 0.0], [0.0, 0.0, 1.0]])
    }

    #[inline]
    pub fn from_scale_angle_translation(scale: Vector2, angle: f32, translation: Vector2) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self([
            [cos * scale[0], sin * scale[0], 0.0],
            [-sin * scale[1], cos * scale[1], 0.0],
            [translation[0], translation[1], 1.0],
        ])
    }

    #[inline]
    pub fn to_matrix2(&self) -> Matrix2 {
        Matrix([[self.0[0][0], self.0[0][1]], [self.0[1][0], self.0[1][1]]])
    }

    #[inline]
    pub fn to_matrix4(&self) -> Matrix4 {
        Matrix([
            [self.0[0][0], self.0[0][1], self.0[0][2], 0.0],
            [self.0[1][0], self.0[1][1], self.0[1][2], 0.0],
            [self.0[2][0], self.0[2][1], self.0[2][2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        self.0[0][0] * (self.0[1][1] * self.0[2][2] - self.0[2][1] * self.0[1][2])
            - self.0[1][0] * (self.0[0][1] * self.0[2][2] - self.0[2][1] * self.0[0][2])
            + self.0[2][0] * (self.0[0][1] * self.0[1][2] - self.0[1][1] * self.0[0][2])
    }

    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < f32::EPSILON {
            return None;
        }
        let inv_det = 1.0 / det;

        let m = &self.0;
        Some(Self([
            [
                (m[1][1] * m[2][2] - m[2][1] * m[1][2]) * inv_det,
                (m[2][1] * m[0][2] - m[0][1] * m[2][2]) * inv_det,
                (m[0][1] * m[1][2] - m[1][1] * m[0][2]) * inv_det,
            ],
            [
                (m[2][0] * m[1][2] - m[1][0] * m[2][2]) * inv_det,
                (m[0][0] * m[2][2] - m[2][0] * m[0][2]) * inv_det,
                (m[1][0] * m[0][2] - m[0][0] * m[1][2]) * inv_det,
            ],
            [
                (m[1][0] * m[2][1] - m[2][0] * m[1][1]) * inv_det,
                (m[2][0] * m[0][1] - m[0][0] * m[2][1]) * inv_det,
                (m[0][0] * m[1][1] - m[1][0] * m[0][1]) * inv_det,
            ],
        ]))
    }
}

impl Matrix4 {
    #[inline]
    pub fn from_cols_vec(c0: Vector4, c1: Vector4, c2: Vector4, c3: Vector4) -> Self {
        Self([
            [c0[0], c0[1], c0[2], c0[3]],
            [c1[0], c1[1], c1[2], c1[3]],
            [c2[0], c2[1], c2[2], c2[3]],
            [c3[0], c3[1], c3[2], c3[3]],
        ])
    }

    #[inline]
    pub fn from_translation(translation: Vector3) -> Self {
        Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [translation[0], translation[1], translation[2], 1.0],
        ])
    }

    #[inline]
    pub fn from_scale(scale: Vector3) -> Self {
        Self([
            [scale[0], 0.0, 0.0, 0.0],
            [0.0, scale[1], 0.0, 0.0],
            [0.0, 0.0, scale[2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    #[inline]
    pub fn from_uniform_scale(scale: f32) -> Self {
        Self::from_scale(Vector3::splat(scale))
    }

    #[inline]
    pub fn from_rotation_x(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, cos, sin, 0.0],
            [0.0, -sin, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    #[inline]
    pub fn from_rotation_y(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self([
            [cos, 0.0, -sin, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [sin, 0.0, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    #[inline]
    pub fn from_rotation_z(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self([
            [cos, sin, 0.0, 0.0],
            [-sin, cos, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn from_axis_angle(axis: Vector3, radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        let t = 1.0 - cos;
        let x = axis[0];
        let y = axis[1];
        let z = axis[2];

        Self([
            [
                t * x * x + cos,
                t * x * y + sin * z,
                t * x * z - sin * y,
                0.0,
            ],
            [
                t * x * y - sin * z,
                t * y * y + cos,
                t * y * z + sin * x,
                0.0,
            ],
            [
                t * x * z + sin * y,
                t * y * z - sin * x,
                t * z * z + cos,
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn perspective(fov_y_radians: f32, aspect: f32, z_near: f32, z_far: f32) -> Self {
        let f = 1.0 / (fov_y_radians / 2.0).tan();
        let range = z_near - z_far;

        Self([
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, z_far / range, -1.0],
            [0.0, 0.0, (z_near * z_far) / range, 0.0],
        ])
    }

    pub fn perspective_infinite(fov_y_radians: f32, aspect: f32, z_near: f32) -> Self {
        let f = 1.0 / (fov_y_radians / 2.0).tan();

        Self([
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, -1.0, -1.0],
            [0.0, 0.0, -z_near, 0.0],
        ])
    }

    pub fn orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let rml = right - left;
        let tmb = top - bottom;
        let fmn = z_far - z_near;

        Self([
            [2.0 / rml, 0.0, 0.0, 0.0],
            [0.0, 2.0 / tmb, 0.0, 0.0],
            [0.0, 0.0, -1.0 / fmn, 0.0],
            [
                -(right + left) / rml,
                -(top + bottom) / tmb,
                -z_near / fmn,
                1.0,
            ],
        ])
    }

    pub fn orthographic_2d(width: f32, height: f32) -> Self {
        Self::orthographic(0.0, width, height, 0.0, -1.0, 1.0)
    }

    pub fn look_at(eye: Vector3, target: Vector3, up: Vector3) -> Self {
        let f = (target - eye).normalized();
        let r = f.cross(&up).normalized();
        let u = r.cross(&f);

        Self([
            [r[0], u[0], -f[0], 0.0],
            [r[1], u[1], -f[1], 0.0],
            [r[2], u[2], -f[2], 0.0],
            [-r.dot(&eye), -u.dot(&eye), f.dot(&eye), 1.0],
        ])
    }

    pub fn look_to(eye: Vector3, dir: Vector3, up: Vector3) -> Self {
        let f = dir.normalized();
        let r = f.cross(&up).normalized();
        let u = r.cross(&f);

        Self([
            [r[0], u[0], -f[0], 0.0],
            [r[1], u[1], -f[1], 0.0],
            [r[2], u[2], -f[2], 0.0],
            [-r.dot(&eye), -u.dot(&eye), f.dot(&eye), 1.0],
        ])
    }

    #[inline]
    pub fn translation(&self) -> Vector3 {
        Vector3::new(self.0[3][0], self.0[3][1], self.0[3][2])
    }

    #[inline]
    pub fn to_matrix3(&self) -> Matrix3 {
        Matrix([
            [self.0[0][0], self.0[0][1], self.0[0][2]],
            [self.0[1][0], self.0[1][1], self.0[1][2]],
            [self.0[2][0], self.0[2][1], self.0[2][2]],
        ])
    }

    #[inline]
    pub fn transform_point(&self, p: Vector3) -> Vector3 {
        let w = self.0[0][3] * p[0] + self.0[1][3] * p[1] + self.0[2][3] * p[2] + self.0[3][3];
        Vector3::new(
            (self.0[0][0] * p[0] + self.0[1][0] * p[1] + self.0[2][0] * p[2] + self.0[3][0]) / w,
            (self.0[0][1] * p[0] + self.0[1][1] * p[1] + self.0[2][1] * p[2] + self.0[3][1]) / w,
            (self.0[0][2] * p[0] + self.0[1][2] * p[1] + self.0[2][2] * p[2] + self.0[3][2]) / w,
        )
    }

    #[inline]
    pub fn transform_vector(&self, v: Vector3) -> Vector3 {
        Vector3::new(
            self.0[0][0] * v[0] + self.0[1][0] * v[1] + self.0[2][0] * v[2],
            self.0[0][1] * v[0] + self.0[1][1] * v[1] + self.0[2][1] * v[2],
            self.0[0][2] * v[0] + self.0[1][2] * v[1] + self.0[2][2] * v[2],
        )
    }

    pub fn determinant(&self) -> f32 {
        let m = &self.0;

        let s0 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
        let s1 = m[0][0] * m[2][1] - m[2][0] * m[0][1];
        let s2 = m[0][0] * m[3][1] - m[3][0] * m[0][1];
        let s3 = m[1][0] * m[2][1] - m[2][0] * m[1][1];
        let s4 = m[1][0] * m[3][1] - m[3][0] * m[1][1];
        let s5 = m[2][0] * m[3][1] - m[3][0] * m[2][1];

        let c5 = m[2][2] * m[3][3] - m[3][2] * m[2][3];
        let c4 = m[1][2] * m[3][3] - m[3][2] * m[1][3];
        let c3 = m[1][2] * m[2][3] - m[2][2] * m[1][3];
        let c2 = m[0][2] * m[3][3] - m[3][2] * m[0][3];
        let c1 = m[0][2] * m[2][3] - m[2][2] * m[0][3];
        let c0 = m[0][2] * m[1][3] - m[1][2] * m[0][3];

        s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0
    }

    pub fn inverse(&self) -> Option<Self> {
        let m = &self.0;

        let s0 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
        let s1 = m[0][0] * m[2][1] - m[2][0] * m[0][1];
        let s2 = m[0][0] * m[3][1] - m[3][0] * m[0][1];
        let s3 = m[1][0] * m[2][1] - m[2][0] * m[1][1];
        let s4 = m[1][0] * m[3][1] - m[3][0] * m[1][1];
        let s5 = m[2][0] * m[3][1] - m[3][0] * m[2][1];

        let c5 = m[2][2] * m[3][3] - m[3][2] * m[2][3];
        let c4 = m[1][2] * m[3][3] - m[3][2] * m[1][3];
        let c3 = m[1][2] * m[2][3] - m[2][2] * m[1][3];
        let c2 = m[0][2] * m[3][3] - m[3][2] * m[0][3];
        let c1 = m[0][2] * m[2][3] - m[2][2] * m[0][3];
        let c0 = m[0][2] * m[1][3] - m[1][2] * m[0][3];

        let det = s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0;

        if det.abs() < f32::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;

        Some(Self([
            [
                (m[1][1] * c5 - m[2][1] * c4 + m[3][1] * c3) * inv_det,
                (-m[0][1] * c5 + m[2][1] * c2 - m[3][1] * c1) * inv_det,
                (m[0][1] * c4 - m[1][1] * c2 + m[3][1] * c0) * inv_det,
                (-m[0][1] * c3 + m[1][1] * c1 - m[2][1] * c0) * inv_det,
            ],
            [
                (-m[1][0] * c5 + m[2][0] * c4 - m[3][0] * c3) * inv_det,
                (m[0][0] * c5 - m[2][0] * c2 + m[3][0] * c1) * inv_det,
                (-m[0][0] * c4 + m[1][0] * c2 - m[3][0] * c0) * inv_det,
                (m[0][0] * c3 - m[1][0] * c1 + m[2][0] * c0) * inv_det,
            ],
            [
                (m[1][3] * s5 - m[2][3] * s4 + m[3][3] * s3) * inv_det,
                (-m[0][3] * s5 + m[2][3] * s2 - m[3][3] * s1) * inv_det,
                (m[0][3] * s4 - m[1][3] * s2 + m[3][3] * s0) * inv_det,
                (-m[0][3] * s3 + m[1][3] * s1 - m[2][3] * s0) * inv_det,
            ],
            [
                (-m[1][2] * s5 + m[2][2] * s4 - m[3][2] * s3) * inv_det,
                (m[0][2] * s5 - m[2][2] * s2 + m[3][2] * s1) * inv_det,
                (-m[0][2] * s4 + m[1][2] * s2 - m[3][2] * s0) * inv_det,
                (m[0][2] * s3 - m[1][2] * s1 + m[2][2] * s0) * inv_det,
            ],
        ]))
    }

    pub fn inverse_affine(&self) -> Self {
        let m = self.to_matrix3();
        let inv_m = m.inverse().unwrap_or(Matrix3::identity());
        let t = self.translation();
        let inv_t = -(inv_m * t);

        Self([
            [inv_m.0[0][0], inv_m.0[0][1], inv_m.0[0][2], 0.0],
            [inv_m.0[1][0], inv_m.0[1][1], inv_m.0[1][2], 0.0],
            [inv_m.0[2][0], inv_m.0[2][1], inv_m.0[2][2], 0.0],
            [inv_t[0], inv_t[1], inv_t[2], 1.0],
        ])
    }
}
