use std::ops::{Index, IndexMut, Mul};

use super::Vec3;

pub struct Mat4([[f32; 4]; 4]);

impl Index<(usize, usize)> for Mat4 {
    type Output = f32;

    fn index(&self, (row, col): (usize, usize)) -> &f32 {
        &self.0[row][col]
    }
}

impl IndexMut<(usize, usize)> for Mat4 {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut f32 {
        &mut self.0[row][col]
    }
}

impl Mat4 {
    pub const ZERO: Mat4 = Mat4([
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
    ]);

    pub fn as_ptr(&self) -> *const f32 {
        self.0.as_ptr() as *const f32
    }

    pub fn identity() -> Self {
        let mut m = Self::ZERO;
        m[(0, 0)] = 1.0;
        m[(1, 1)] = 1.0;
        m[(2, 2)] = 1.0;
        m[(3, 3)] = 1.0;
        m
    }

    pub fn translate(rhs: Vec3) -> Self {
        let mut m = Self::identity();

        m[(0, 3)] = rhs.x;
        m[(1, 3)] = rhs.y;
        m[(2, 3)] = rhs.z;
        m[(3, 3)] = 1.0;

        m
    }

    pub fn non_uniform_scale(rhs: Vec3) -> Self {
        let mut m = Self::identity();
        m[(0, 0)] *= rhs.x;
        m[(1, 1)] *= rhs.y;
        m[(2, 2)] *= rhs.z;
        m[(3, 3)] = 1.0;
        m
    }

    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let mut m = Self::identity();
        m[(0, 0)] = 2.0 / (right - left);
        m[(1, 1)] = 2.0 / (top - bottom);
        m[(2, 2)] = -2.0 / (far - near);
        m[(3, 0)] = -(right + left) / (right - left);
        m[(3, 1)] = -(top + bottom) / (top - bottom);
        m[(3, 2)] = -(far + near) / (far - near);
        m
    }
}

/// Matrix-Matrix multiplication
impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut m = Self::ZERO;

        for i in 0..4 {
            for j in 0..4 {
                m[(i, j)] = (0..4).map(|k| self[(i, k)] * rhs[(k, j)]).sum();
            }
        }

        m
    }
}

/// Matrix-Vector multiplication
impl Mul<Vec3> for Mat4 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        let mut v = Vec3::ZERO;

        for i in 0..3 {
            for j in 0..3 {
                v[i] += self[(i, j)] * rhs[j];
            }
            v[i] += self[(i, 3)]; // Add the translation component
        }

        v
    }
}
