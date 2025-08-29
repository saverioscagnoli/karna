use crate::{chance::rng, ToF32};
use rand::distr::uniform::SampleRange;
use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// Generic matrix representation.
/// Holds a 2D array of f32 values.
/// I choose to use f32 because it is the most common type
/// used in game development, so it can be simplified,
/// without adding an extra layer of generics.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Matrix<const R: usize, const C: usize>(pub(crate) [[f32; R]; C]);

/// Debug implementation for matrices.
impl<const R: usize, const C: usize> Debug for Matrix<R, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Strip the outer [] if the matrix is a vector
        if C <= 1 {
            f.debug_list().entries(self.0.iter().flatten()).finish()
        } else {
            f.debug_list().entries(self.0.iter()).finish()
        }
    }
}

/// PartialEq implementation for matrices.
impl<const R: usize, const C: usize> PartialEq for Matrix<R, C> {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..R {
            for j in 0..C {
                if self[(i, j)] != other[(i, j)] {
                    return false;
                }
            }
        }

        true
    }
}

/// Index helper implementation for matrices; so they can be accessed
/// using matrix[(row, col)] syntax.
impl<const R: usize, const C: usize> Index<(usize, usize)> for Matrix<R, C> {
    type Output = f32;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        &self.0[col][row]
    }
}

/// Index helper implementation for matrices; so they can be accessed
/// using matrix[(row, col)] syntax.
/// This is the mutable version.
impl<const R: usize, const C: usize> IndexMut<(usize, usize)> for Matrix<R, C> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        &mut self.0[col][row]
    }
}

impl<const R: usize, const C: usize> Matrix<R, C> {
    /// Creates a new matrix from a two-dimensional array.
    pub fn from_array(array: [[f32; R]; C]) -> Self {
        Self(array)
    }

    /// Creates a new matrix filled with random values.
    pub fn random<T: SampleRange<f32> + Clone>(range: T) -> Self {
        let mut m = Self::zero();
        for i in 0..R {
            for j in 0..C {
                m[(i, j)] = rng(range.clone());
            }
        }

        m
    }

    /// Creates a new matrix filled with a specific value.
    pub fn filled<F: ToF32>(value: F) -> Self {
        Self([[value.to_f32(); R]; C])
    }

    /// Creates a new matrix filled with zeros.
    pub fn zero() -> Self {
        Self::filled(0)
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

    /// Consumes self and returns the inner two-dimensional array.
    pub fn into_inner(self) -> [[f32; R]; C] {
        self.0
    }

    /// Returns a pointer to the first element of the matrix.
    pub fn as_ptr(&self) -> *const f32 {
        self.0.as_ptr() as *const f32
    }

    /// Converts this matrix to a different size matrix.
    /// If the new matrix is larger, the new elements are filled with the provided value.
    /// If the new matrix is smaller, the elements are truncated.
    pub fn resize<const NR: usize, const NC: usize, F: ToF32>(
        self,
        fill_value: F,
    ) -> Matrix<NR, NC> {
        let fill = fill_value.to_f32();
        let mut new_matrix = Matrix::<NR, NC>::filled(fill);

        let min_rows = R.min(NR);
        let min_cols = C.min(NC);

        for i in 0..min_rows {
            for j in 0..min_cols {
                new_matrix[(i, j)] = self[(i, j)];
            }
        }

        new_matrix
    }

    /// Converts this matrix to a different size matrix, filling new elements with zeros.
    pub fn resize_zeros<const NR: usize, const NC: usize>(self) -> Matrix<NR, NC> {
        self.resize(0.0)
    }

    /// Converts this matrix to a different size matrix, filling new elements with ones.
    pub fn resize_ones<const NR: usize, const NC: usize>(self) -> Matrix<NR, NC> {
        self.resize(1.0)
    }
}

impl<const R: usize, const C: usize> Default for Matrix<R, C> {
    fn default() -> Self {
        Self::zero()
    }
}

/// Matrix negation.
/// Negates all elements of the matrix.
impl<const R: usize, const C: usize> Neg for Matrix<R, C> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let mut m = self;

        for i in 0..R {
            for j in 0..C {
                m[(i, j)] = -m[(i, j)];
            }
        }

        m
    }
}

////////////////////////////////
///                          ///
/// Matrix-Matrix Operations ///
///                          ///
////////////////////////////////

/// Matrix addition.
/// Adds two matrices together.
/// The matrices must have the same dimensions.
///
/// # Example
/// ```no_run
/// let mat1 = Matrix::<2, 2>::filled(1);
/// let mat2 = Matrix::<2, 2>::filled(2);
///
/// let result = mat1 + mat2;
///
/// assert_eq!(result, Matrix::<2, 2>::filled(3));
/// ```
impl<const R: usize, const C: usize> Add for Matrix<R, C> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut m = self;

        for i in 0..R {
            for j in 0..C {
                m[(i, j)] += rhs[(i, j)];
            }
        }

        m
    }
}

/// Matrix assignment addition.
/// Adds one matrix to another.
/// The matrices must have the same dimensions.
/// The result is assigned to the first matrix. (+=)
impl<const R: usize, const C: usize> AddAssign for Matrix<R, C> {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..R {
            for j in 0..C {
                self[(i, j)] += rhs[(i, j)];
            }
        }
    }
}

/// Matrix subtraction.
/// Subtracts one matrix from another.
/// The matrices must have the same dimensions.
///
/// # Example
/// ```no_run
/// let mat1 = Matrix::<2, 2>::filled(1);
/// let mat2 = Matrix::<2, 2>::filled(2);
///
/// let result = mat1 - mat2;
///
/// assert_eq!(result, Matrix::<2, 2>::filled(-1));
/// ```
impl<const R: usize, const C: usize> Sub for Matrix<R, C> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut m = self;

        for i in 0..R {
            for j in 0..C {
                m[(i, j)] -= rhs[(i, j)];
            }
        }

        m
    }
}

/// Matrix assignment subtraction.
/// Subtracts one matrix from another.
/// The matrices must have the same dimensions.
/// The result is assigned to the first matrix. (-=)
impl<const R: usize, const C: usize> SubAssign for Matrix<R, C> {
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..R {
            for j in 0..C {
                self[(i, j)] -= rhs[(i, j)];
            }
        }
    }
}

/// Matrix multiplication.
/// Multiplies two matrices together.
/// The number of columns in the first matrix must be equal to the number of rows in the second matrix.
///
/// # Example
/// ```no_run
/// let mat1 = Matrix::<2, 3>::filled(1);
/// let mat2 = Matrix::<3, 2>::filled(2);
///
/// let result = mat1 * mat2;
///
/// assert_eq!(result, Matrix::<2, 2>::filled(6));
/// ```
impl<const R: usize, const C: usize, const N: usize> Mul<Matrix<C, N>> for Matrix<R, C> {
    type Output = Matrix<R, N>;

    fn mul(self, rhs: Matrix<C, N>) -> Self::Output {
        let mut m = Matrix::<R, N>::zero();

        for i in 0..R {
            for j in 0..N {
                for k in 0..C {
                    m[(i, j)] += self[(i, k)] * rhs[(k, j)];
                }
            }
        }

        m
    }
}

////////////////////////////////
///                          ///
/// Matrix-Scalar Operations ///
///                          ///
////////////////////////////////

/// Matrix scalar addition.
/// Adds a scalar to all elements of the matrix.
impl<F: ToF32, const R: usize, const C: usize> Add<F> for Matrix<R, C> {
    type Output = Self;

    fn add(self, rhs: F) -> Self::Output {
        let rhs = rhs.to_f32();
        let mut m = self;

        for i in 0..R {
            for j in 0..C {
                m[(i, j)] += rhs;
            }
        }

        m
    }
}

/// Matrix scalar addition assignment.
/// Adds a scalar to all elements of the matrix.
/// The result is assigned to the matrix. (+=)
impl<F: ToF32, const R: usize, const C: usize> AddAssign<F> for Matrix<R, C> {
    fn add_assign(&mut self, rhs: F) {
        let rhs = rhs.to_f32();

        for i in 0..R {
            for j in 0..C {
                self[(i, j)] += rhs;
            }
        }
    }
}

/// Matrix scalar subtraction.
/// Subtracts a scalar from all elements of the matrix.
impl<F: ToF32, const R: usize, const C: usize> Sub<F> for Matrix<R, C> {
    type Output = Self;

    fn sub(self, rhs: F) -> Self::Output {
        let rhs = rhs.to_f32();
        let mut m = self;

        for i in 0..R {
            for j in 0..C {
                m[(i, j)] -= rhs;
            }
        }

        m
    }
}

/// Matrix scalar subtraction assignment.
/// Subtracts a scalar from all elements of the matrix.
/// The result is assigned to the matrix. (-=)
impl<F: ToF32, const R: usize, const C: usize> SubAssign<F> for Matrix<R, C> {
    fn sub_assign(&mut self, rhs: F) {
        let rhs = rhs.to_f32();

        for i in 0..R {
            for j in 0..C {
                self[(i, j)] -= rhs;
            }
        }
    }
}

/// Matrix scalar multiplication.
/// Multiplies all elements of the matrix by a scalar.
impl<F: ToF32, const R: usize, const C: usize> Mul<F> for Matrix<R, C> {
    type Output = Self;

    fn mul(self, rhs: F) -> Self::Output {
        let rhs = rhs.to_f32();
        let mut m = self;

        for i in 0..R {
            for j in 0..C {
                m[(i, j)] *= rhs;
            }
        }

        m
    }
}

/// Matrix scalar multiplication assignment.
/// Multiplies all elements of the matrix by a scalar.
/// The result is assigned to the matrix. (*=)
impl<F: ToF32, const R: usize, const C: usize> MulAssign<F> for Matrix<R, C> {
    fn mul_assign(&mut self, rhs: F) {
        let rhs = rhs.to_f32();

        for i in 0..R {
            for j in 0..C {
                self[(i, j)] *= rhs;
            }
        }
    }
}

pub type Mat2 = Matrix<2, 2>;
pub type Mat3 = Matrix<3, 3>;
pub type Mat4 = Matrix<4, 4>;

impl Mat4 {
    /// Creates a new orthographic projection matrix.
    /// Useful for 2D rendering, where the dimensions of the objects do not change with depth.
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra as na;

    #[test]
    fn identity_matrix() {
        let m = Matrix::<4, 4>::identity();
        let na_m = na::SMatrix::<f32, 4, 4>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);
    }

    #[test]
    fn identity_matrix_nonsquare() {
        let m = Matrix::<4, 3>::identity();
        let na_m = na::SMatrix::<f32, 4, 3>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);

        let m = Matrix::<3, 4>::identity();
        let na_m = na::SMatrix::<f32, 3, 4>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);

        let m = Matrix::<3, 1>::identity();
        let na_m = na::SMatrix::<f32, 3, 1>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);

        let m = Matrix::<1, 3>::identity();
        let na_m = na::SMatrix::<f32, 1, 3>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);

        let m = Matrix::<1, 1>::identity();
        let na_m = na::SMatrix::<f32, 1, 1>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);

        let m = Matrix::<43, 13>::identity();
        let na_m = na::SMatrix::<f32, 43, 13>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);
    }

    #[test]
    fn matrix_negation() {
        let m = Matrix::<4, 4>::identity();
        let na_m = -na::SMatrix::<f32, 4, 4>::identity();

        assert_eq!((-m).into_inner(), na_m.data.0);
    }

    #[test]
    fn matrix_addition() {
        let m1 = Matrix::<2, 2>::filled(1);
        let m2 = Matrix::<2, 2>::filled(2);

        let m3 = na::SMatrix::<f32, 2, 2>::from_fn(|_, _| 1.0);
        let m4 = na::SMatrix::<f32, 2, 2>::from_fn(|_, _| 2.0);

        let result = m1 + m2;
        let na_result = m3 + m4;

        assert_eq!(result, Matrix::<2, 2>::filled(3));
        assert_eq!(result.into_inner(), na_result.data.0);

        let m1 = Matrix::<3, 4>::filled(24);
        let m2 = Matrix::<3, 4>::filled(1);

        let m3 = na::SMatrix::<f32, 3, 4>::from_fn(|_, _| 24.0);
        let m4 = na::SMatrix::<f32, 3, 4>::from_fn(|_, _| 1.0);

        let result = m1 + m2;
        let na_result = m3 + m4;

        assert_eq!(result, Matrix::<3, 4>::filled(25));
        assert_eq!(result.into_inner(), na_result.data.0);

        let m1 = Matrix::<1, 1>::filled(214);
        let m2 = Matrix::<1, 1>::filled(45982);

        let m3 = na::SMatrix::<f32, 1, 1>::from_fn(|_, _| 214.0);
        let m4 = na::SMatrix::<f32, 1, 1>::from_fn(|_, _| 45982.0);

        let result = m1 + m2;
        let na_result = m3 + m4;

        assert_eq!(result, Matrix::<1, 1>::filled(46196));
        assert_eq!(result.into_inner(), na_result.data.0);
    }

    #[test]
    fn matrix_subtraction() {
        let m1 = Matrix::<2, 2>::filled(1);
        let m2 = Matrix::<2, 2>::filled(2);

        let m3 = na::SMatrix::<f32, 2, 2>::from_fn(|_, _| 1.0);
        let m4 = na::SMatrix::<f32, 2, 2>::from_fn(|_, _| 2.0);

        let result = m1 - m2;
        let na_result = m3 - m4;

        assert_eq!(result, Matrix::<2, 2>::filled(-1));
        assert_eq!(result.into_inner(), na_result.data.0);

        let m1 = Matrix::<3, 4>::filled(24);
        let m2 = Matrix::<3, 4>::filled(1);

        let m3 = na::SMatrix::<f32, 3, 4>::from_fn(|_, _| 24.0);
        let m4 = na::SMatrix::<f32, 3, 4>::from_fn(|_, _| 1.0);

        let result = m1 - m2;
        let na_result = m3 - m4;

        assert_eq!(result, Matrix::<3, 4>::filled(23));
        assert_eq!(result.into_inner(), na_result.data.0);

        let m1 = Matrix::<1, 1>::filled(214);
        let m2 = Matrix::<1, 1>::filled(45982);

        let m3 = na::SMatrix::<f32, 1, 1>::from_fn(|_, _| 214.0);
        let m4 = na::SMatrix::<f32, 1, 1>::from_fn(|_, _| 45982.0);

        let result = m1 - m2;
        let na_result = m3 - m4;

        assert_eq!(result, Matrix::<1, 1>::filled(-45768));
        assert_eq!(result.into_inner(), na_result.data.0);
    }

    #[test]
    fn matrix_multiplication() {
        let m1 = Matrix::<2, 3>::filled(1);
        let m2 = Matrix::<3, 2>::filled(2);

        let m3 = na::SMatrix::<f32, 2, 3>::from_fn(|_, _| 1.0);
        let m4 = na::SMatrix::<f32, 3, 2>::from_fn(|_, _| 2.0);

        let result = m1 * m2;
        let na_result = m3 * m4;

        assert_eq!(result, Matrix::<2, 2>::filled(6));
        assert_eq!(result.into_inner(), na_result.data.0);

        let m1 = Matrix::<3, 3>::filled(12);
        let m2 = Matrix::<3, 3>::filled(42);

        let m3 = na::SMatrix::<f32, 3, 3>::from_fn(|_, _| 12.0);
        let m4 = na::SMatrix::<f32, 3, 3>::from_fn(|_, _| 42.0);

        let result = m1 * m2;
        let na_result = m3 * m4;

        assert_eq!(result, Matrix::<3, 3>::filled(1512));
        assert_eq!(result.into_inner(), na_result.data.0);

        let m1 = Matrix::<1, 1>::filled(214);
        let m2 = Matrix::<1, 1>::filled(45982);

        let m3 = na::SMatrix::<f32, 1, 1>::from_fn(|_, _| 214.0);
        let m4 = na::SMatrix::<f32, 1, 1>::from_fn(|_, _| 45982.0);

        let result = m1 * m2;
        let na_result = m3 * m4;

        assert_eq!(result, Matrix::<1, 1>::filled(214 * 45982));
        assert_eq!(result.into_inner(), na_result.data.0);
    }

    #[test]
    fn matrix_scalar_addition() {
        let m = Matrix::<2, 2>::filled(1);
        let result = m + 2;

        assert_eq!(result, Matrix::<2, 2>::filled(3));
    }

    #[test]
    fn matrix_scalar_subtraction() {
        let m = Matrix::<2, 2>::filled(1);
        let result = m - 2;

        assert_eq!(result, Matrix::<2, 2>::filled(-1));
    }

    #[test]
    fn matrix_scalar_multiplication() {
        let m = Matrix::<2, 2>::filled(1);
        let result = m * 2;

        assert_eq!(result, Matrix::<2, 2>::filled(2));
    }
}

#[cfg(test)]
mod fuzz_test {
    use super::*;
    use const_random::const_random;
    use nalgebra::{self as na};

    const fn const_random_range(min: u8, max: u8) -> usize {
        (min + (const_random!(u8) % (max - min))) as usize
    }

    const R: usize = const_random_range(1, 50);

    #[test]
    fn identity_matrix() {
        let m = Matrix::<R, R>::identity();
        let na_m = na::SMatrix::<f32, { R as usize }, { R as usize }>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);
    }

    #[test]
    fn identity_matrix_nonsquare() {
        let m = Matrix::<R, 3>::identity();
        let na_m = na::SMatrix::<f32, R, 3>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);

        let m = Matrix::<R, 4>::identity();
        let na_m = na::SMatrix::<f32, R, 4>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);

        let m = Matrix::<R, 1>::identity();
        let na_m = na::SMatrix::<f32, R, 1>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);

        let m = Matrix::<R, 3>::identity();
        let na_m = na::SMatrix::<f32, R, 3>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);

        let m = Matrix::<R, 1>::identity();
        let na_m = na::SMatrix::<f32, R, 1>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);

        let m = Matrix::<43, R>::identity();
        let na_m = na::SMatrix::<f32, 43, R>::identity();

        assert_eq!(m.into_inner(), na_m.data.0);
    }

    #[test]
    fn matrix_negation() {
        for _ in 0..100 {
            let m = Matrix::<R, R>::random(0.0..=255.0);
            let na_m =
                -na::SMatrix::<f32, R, R>::from_array_storage(na::ArrayStorage(m.into_inner()));

            assert_eq!((-m).into_inner(), na_m.data.0);
        }
    }

    #[test]
    fn matrix_addition() {
        for _ in 0..100 {
            let m1 = Matrix::<R, R>::random(0.0..=255.0);
            let m2 = Matrix::<R, R>::random(0.0..=255.0);

            let m3 =
                na::SMatrix::<f32, R, R>::from_array_storage(na::ArrayStorage(m1.into_inner()));
            let m4 =
                na::SMatrix::<f32, R, R>::from_array_storage(na::ArrayStorage(m2.into_inner()));

            let result = m1 + m2;
            let na_result = m3 + m4;

            assert_eq!(result.into_inner(), na_result.data.0);
        }
    }

    #[test]
    fn matrix_subtraction() {
        for _ in 0..100 {
            let m1 = Matrix::<R, R>::random(0.0..=255.0);
            let m2 = Matrix::<R, R>::random(0.0..=255.0);

            let m3 =
                na::SMatrix::<f32, R, R>::from_array_storage(na::ArrayStorage(m1.into_inner()));
            let m4 =
                na::SMatrix::<f32, R, R>::from_array_storage(na::ArrayStorage(m2.into_inner()));

            let result = m1 - m2;
            let na_result = m3 - m4;

            assert_eq!(result.into_inner(), na_result.data.0);
        }
    }

    #[test]
    fn matrix_multiplication() {
        for _ in 0..100 {
            let m1 = Matrix::<R, R>::random(0.0..=255.0);
            let m2 = Matrix::<R, R>::random(0.0..=255.0);

            let m3 =
                na::SMatrix::<f32, R, R>::from_array_storage(na::ArrayStorage(m1.into_inner()));
            let m4 =
                na::SMatrix::<f32, R, R>::from_array_storage(na::ArrayStorage(m2.into_inner()));

            let result = m1 * m2;
            let na_result = m3 * m4;

            assert_eq!(result.into_inner(), na_result.data.0);
        }
    }

    #[test]
    fn matrix_scalar_addition() {
        for _ in 0..100 {
            let m = Matrix::<R, R>::random(0.0..=255.0);
            let result = m + 2;

            assert_eq!(result.into_inner(), (m + 2).into_inner());
        }
    }

    #[test]
    fn matrix_scalar_subtraction() {
        for _ in 0..100 {
            let m = Matrix::<R, R>::random(0.0..=255.0);
            let result = m - 2;

            assert_eq!(result.into_inner(), (m - 2).into_inner());
        }
    }

    #[test]
    fn matrix_scalar_multiplication() {
        for _ in 0..100 {
            let m = Matrix::<R, R>::random(0.0..=255.0);
            let result = m * 2;

            assert_eq!(result.into_inner(), (m * 2).into_inner());
        }
    }
}
