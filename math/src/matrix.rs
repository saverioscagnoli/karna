use crate::vector::{Vector, Vector3};
use std::ops::{Index, IndexMut, Mul};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Matrix<const R: usize, const C: usize>([[f32; R]; C]);

impl<const R: usize, const C: usize> Index<(usize, usize)> for Matrix<R, C> {
    type Output = f32;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        &self.0[col][row]
    }
}

impl<const R: usize, const C: usize> IndexMut<(usize, usize)> for Matrix<R, C> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        &mut self.0[col][row]
    }
}

impl<const R: usize, const C: usize> Matrix<R, C> {
    pub const fn from_array(arr: [[f32; R]; C]) -> Self {
        Self(arr)
    }

    pub const fn zero() -> Self {
        Self([[0.0; R]; C])
    }

    pub const fn one() -> Self {
        Self([[1.0; R]; C])
    }

    pub const fn fill(n: f32) -> Self {
        Self([[n; R]; C])
    }

    /// Creates a new identity matrix.
    /// The identity matrix is a square matrix with 1s on the diagonal
    /// and 0s elsewhere.
    ///
    /// If the matrix is not square, the largest square submatrix is filled
    pub fn identity() -> Self {
        let mut m = Self::zero();

        for i in 0..R.min(C) {
            m[(i, i)] = 1.0;
        }

        m
    }
}

// Matrix * Matrix multiplication: (R × K) * (K × C) = (R × C)
impl<const R: usize, const K: usize, const C: usize> Mul<Matrix<K, C>> for Matrix<R, K> {
    type Output = Matrix<R, C>;

    #[inline]
    fn mul(self, rhs: Matrix<K, C>) -> Self::Output {
        let mut result = Matrix::zero();

        // For column-major storage, iterate over columns of result
        for j in 0..C {
            for i in 0..R {
                let mut sum = 0.0;
                for k in 0..K {
                    sum += self[(i, k)] * rhs[(k, j)];
                }
                result[(i, j)] = sum;
            }
        }

        result
    }
}

// Matrix * Vector multiplication: (R × C) * Vector<C> = Vector<R>
impl<const R: usize, const C: usize> Mul<Vector<C>> for Matrix<R, C> {
    type Output = Vector<R>;

    #[inline]
    fn mul(self, rhs: Vector<C>) -> Self::Output {
        let mut result = Vector::zero();

        for i in 0..R {
            let mut sum = 0.0;
            for j in 0..C {
                sum += self[(i, j)] * rhs[j];
            }
            result[i] = sum;
        }

        result
    }
}

// Vector * Matrix multiplication: Vector<R> (as row vector) * Matrix<R, C> = Vector<C>
impl<const R: usize, const C: usize> Mul<Matrix<R, C>> for Vector<R> {
    type Output = Vector<C>;

    #[inline]
    fn mul(self, rhs: Matrix<R, C>) -> Self::Output {
        let mut result = Vector::zero();

        for j in 0..C {
            let mut sum = 0.0;
            for i in 0..R {
                sum += self[i] * rhs[(i, j)];
            }
            result[j] = sum;
        }

        result
    }
}

pub type Matrix4 = Matrix<4, 4>;

impl Matrix4 {
    #[inline]
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let mut m = Self::identity();

        m[(0, 0)] = 2.0 / (right - left);
        m[(1, 1)] = 2.0 / (top - bottom);
        m[(2, 2)] = -2.0 / (far - near);
        m[(0, 3)] = -(right + left) / (right - left);
        m[(1, 3)] = -(top + bottom) / (top - bottom);
        m[(2, 3)] = -(far + near) / (far - near);

        m
    }

    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y / 2.0).tan();
        let mut m = Self::zero();

        m[(0, 0)] = f / aspect;
        m[(1, 1)] = f;
        m[(2, 2)] = (far + near) / (near - far);
        m[(2, 3)] = (2.0 * far * near) / (near - far);
        m[(3, 2)] = -1.0;

        m
    }

    pub fn translate(v: Vector3) -> Self {
        let mut m = Self::identity();
        m[(0, 3)] = v.x;
        m[(1, 3)] = v.y;
        m[(2, 3)] = v.z;
        m
    }

    pub fn scale(v: Vector3) -> Self {
        let mut m = Self::identity();
        m[(0, 0)] = v.x;
        m[(1, 1)] = v.y;
        m[(2, 2)] = v.z;
        m
    }

    pub fn rotate_x(angle_rad: f32) -> Self {
        let mut m = Self::identity();
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();

        m[(1, 1)] = cos;
        m[(1, 2)] = -sin;
        m[(2, 1)] = sin;
        m[(2, 2)] = cos;

        m
    }

    pub fn rotate_y(angle_rad: f32) -> Self {
        let mut m = Self::identity();
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();

        m[(0, 0)] = cos;
        m[(0, 2)] = sin;
        m[(2, 0)] = -sin;
        m[(2, 2)] = cos;

        m
    }

    pub fn rotate_z(angle_rad: f32) -> Self {
        let mut m = Self::identity();
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();

        m[(0, 0)] = cos;
        m[(0, 1)] = -sin;
        m[(1, 0)] = sin;
        m[(1, 1)] = cos;

        m
    }

    pub fn look_at(eye: Vector3, target: Vector3, up: Vector3) -> Self {
        let f = (target - eye).normalize();
        let s = f.cross(&up).normalize();
        let u = s.cross(&f);

        let mut result = Self::identity();
        result[(0, 0)] = s.x;
        result[(1, 0)] = s.y;
        result[(2, 0)] = s.z;
        result[(0, 1)] = u.x;
        result[(1, 1)] = u.y;
        result[(2, 1)] = u.z;
        result[(0, 2)] = -f.x;
        result[(1, 2)] = -f.y;
        result[(2, 2)] = -f.z;
        result[(0, 3)] = -s.dot(&eye);
        result[(1, 3)] = -u.dot(&eye);
        result[(2, 3)] = f.dot(&eye);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra as na;
    use std::f32::consts::PI;

    const TOLERANCE: f32 = 1e-6;

    #[test]
    fn test_ortho() {
        let left = -1.0;
        let right = 1.0;
        let bottom = -2.0;
        let top = 2.0;
        let near = 0.1;
        let far = 100.0;

        let m = Matrix4::orthographic(left, right, bottom, top, near, far);
        let n = na::Orthographic3::new(left, right, bottom, top, near, far).to_homogeneous();

        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (m[(i, j)] - n[(i, j)]).abs() < TOLERANCE,
                    "m[{},{}]={}, n[{},{}]={}",
                    i,
                    j,
                    m[(i, j)],
                    i,
                    j,
                    n[(i, j)]
                );
            }
        }
    }

    #[test]
    fn test_perspective() {
        let fov_y = PI / 2.0; // 90 degrees
        let aspect = 16.0 / 9.0;
        let near = 0.1;
        let far = 100.0;

        let m = Matrix4::perspective(fov_y, aspect, near, far);
        let n = na::Perspective3::new(aspect, fov_y, near, far).to_homogeneous();

        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (m[(i, j)] - n[(i, j)]).abs() < TOLERANCE,
                    "m[{},{}]={}, n[{},{}]={}",
                    i,
                    j,
                    m[(i, j)],
                    i,
                    j,
                    n[(i, j)]
                );
            }
        }
    }

    #[test]
    fn test_translate() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let m = Matrix4::translate(v);
        let n = na::Matrix4::new_translation(&na::Vector3::new(1.0, 2.0, 3.0));

        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (m[(i, j)] - n[(i, j)]).abs() < TOLERANCE,
                    "m[{},{}]={}, n[{},{}]={}",
                    i,
                    j,
                    m[(i, j)],
                    i,
                    j,
                    n[(i, j)]
                );
            }
        }
    }

    #[test]
    fn test_scale() {
        let v = Vector3::new(2.0, 3.0, 4.0);
        let m = Matrix4::scale(v);
        let n = na::Matrix4::new_nonuniform_scaling(&na::Vector3::new(2.0, 3.0, 4.0));

        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (m[(i, j)] - n[(i, j)]).abs() < TOLERANCE,
                    "m[{},{}]={}, n[{},{}]={}",
                    i,
                    j,
                    m[(i, j)],
                    i,
                    j,
                    n[(i, j)]
                );
            }
        }
    }

    #[test]
    fn test_rotate_x() {
        let angle = PI / 4.0; // 45 degrees
        let m = Matrix4::rotate_x(angle);
        let n = na::Matrix4::from_axis_angle(&na::Vector3::x_axis(), angle);

        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (m[(i, j)] - n[(i, j)]).abs() < TOLERANCE,
                    "m[{},{}]={}, n[{},{}]={}",
                    i,
                    j,
                    m[(i, j)],
                    i,
                    j,
                    n[(i, j)]
                );
            }
        }
    }

    #[test]
    fn test_rotate_y() {
        let angle = PI / 3.0; // 60 degrees
        let m = Matrix4::rotate_y(angle);
        let n = na::Matrix4::from_axis_angle(&na::Vector3::y_axis(), angle);

        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (m[(i, j)] - n[(i, j)]).abs() < TOLERANCE,
                    "m[{},{}]={}, n[{},{}]={}",
                    i,
                    j,
                    m[(i, j)],
                    i,
                    j,
                    n[(i, j)]
                );
            }
        }
    }

    #[test]
    fn test_rotate_z() {
        let angle = PI / 6.0; // 30 degrees
        let m = Matrix4::rotate_z(angle);
        let n = na::Matrix4::from_axis_angle(&na::Vector3::z_axis(), angle);

        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (m[(i, j)] - n[(i, j)]).abs() < TOLERANCE,
                    "m[{},{}]={}, n[{},{}]={}",
                    i,
                    j,
                    m[(i, j)],
                    i,
                    j,
                    n[(i, j)]
                );
            }
        }
    }

    #[test]
    fn test_look_at() {
        let eye = Vector3::new(0.0, 0.0, 5.0);
        let target = Vector3::new(0.0, 0.0, 0.0);
        let up = Vector3::new(0.0, 1.0, 0.0);

        let m = Matrix4::look_at(eye, target, up);
        let n = na::Matrix4::look_at_rh(
            &na::Point3::new(0.0, 0.0, 5.0),
            &na::Point3::new(0.0, 0.0, 0.0),
            &na::Vector3::new(0.0, 1.0, 0.0),
        );

        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (m[(i, j)] - n[(i, j)]).abs() < TOLERANCE,
                    "m[{},{}]={}, n[{},{}]={}",
                    i,
                    j,
                    m[(i, j)],
                    i,
                    j,
                    n[(i, j)]
                );
            }
        }
    }

    #[test]
    fn test_matrix_multiplication() {
        let m1 = Matrix4::translate(Vector3::new(1.0, 2.0, 3.0));
        let m2 = Matrix4::scale(Vector3::new(2.0, 3.0, 4.0));
        let result = m1 * m2;

        let n1 = na::Matrix4::new_translation(&na::Vector3::new(1.0, 2.0, 3.0));
        let n2 = na::Matrix4::new_nonuniform_scaling(&na::Vector3::new(2.0, 3.0, 4.0));
        let expected = n1 * n2;

        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (result[(i, j)] - expected[(i, j)]).abs() < TOLERANCE,
                    "result[{},{}]={}, expected[{},{}]={}",
                    i,
                    j,
                    result[(i, j)],
                    i,
                    j,
                    expected[(i, j)]
                );
            }
        }
    }
}
