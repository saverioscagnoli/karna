use std::ops::{Index, IndexMut};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Matrix<const R: usize, const C: usize>([[f32; C]; R]);

impl<const R: usize, const C: usize> Index<(usize, usize)> for Matrix<R, C> {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.0[index.0][index.1]
    }
}

impl<const R: usize, const C: usize> IndexMut<(usize, usize)> for Matrix<R, C> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.0[index.0][index.1]
    }
}

impl<const R: usize, const C: usize> Matrix<R, C> {
    pub const fn new(data: [[f32; C]; R]) -> Self {
        Self(data)
    }

    pub const fn zero() -> Self {
        Self([[0.0; C]; R])
    }

    pub const fn fill(value: f32) -> Self {
        Self([[value; C]; R])
    }

    pub const fn identity() -> Self {
        let mut m = Self::zero();
        let min_dim = if R < C { R } else { C };
        let mut i = 0;

        while i < min_dim {
            m.0[i][i] = 1.0;
            i += 1;
        }

        m
    }

    pub const fn transpose(&self) -> Matrix<C, R> {
        let mut m = Matrix::<C, R>::zero();
        let mut r = 0;

        while r < R {
            let mut c = 0;
            while c < C {
                m.0[c][r] = self.0[r][c];
                c += 1;
            }
            r += 1;
        }

        m
    }
}
