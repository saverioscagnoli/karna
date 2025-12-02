use std::ops::{
    Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign,
};

use math::Vector;

#[derive(Debug, Clone, Copy)]
pub struct DirtyTracked<T> {
    pub value: T,
    pub dirty: bool,
}

impl<T> DirtyTracked<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            value,
            dirty: false,
        }
    }
}

impl<T> Deref for DirtyTracked<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for DirtyTracked<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty = true;
        &mut self.value
    }
}

impl<T> From<T> for DirtyTracked<T> {
    fn from(value: T) -> Self {
        DirtyTracked {
            value,
            dirty: false,
        }
    }
}

macro_rules! impl_dirty_vector_op {
    // Vector-Vector ops with DirtyTracked
    ($trait:ident, $method:ident, $op:tt) => {
        // DirtyTracked<Vector> op DirtyTracked<Vector>
        impl<const N: usize> $trait for DirtyTracked<Vector<N>> {
            type Output = Vector<N>;
            fn $method(self, rhs: Self) -> Self::Output {
                self.value $op rhs.value
            }
        }

        // DirtyTracked<Vector> op &DirtyTracked<Vector>
        impl<const N: usize> $trait<&DirtyTracked<Vector<N>>> for DirtyTracked<Vector<N>> {
            type Output = Vector<N>;
            fn $method(self, rhs: &DirtyTracked<Vector<N>>) -> Self::Output {
                self.value $op &rhs.value
            }
        }

        // &DirtyTracked<Vector> op DirtyTracked<Vector>
        impl<const N: usize> $trait<DirtyTracked<Vector<N>>> for &DirtyTracked<Vector<N>> {
            type Output = Vector<N>;
            fn $method(self, rhs: DirtyTracked<Vector<N>>) -> Self::Output {
                &self.value $op rhs.value
            }
        }

        // &DirtyTracked<Vector> op &DirtyTracked<Vector>
        impl<const N: usize> $trait for &DirtyTracked<Vector<N>> {
            type Output = Vector<N>;
            fn $method(self, rhs: Self) -> Self::Output {
                &self.value $op &rhs.value
            }
        }

        // DirtyTracked<Vector> op Vector
        impl<const N: usize> $trait<Vector<N>> for DirtyTracked<Vector<N>> {
            type Output = Vector<N>;
            fn $method(self, rhs: Vector<N>) -> Self::Output {
                self.value $op rhs
            }
        }

        // Vector op DirtyTracked<Vector>
        impl<const N: usize> $trait<DirtyTracked<Vector<N>>> for Vector<N> {
            type Output = Vector<N>;
            fn $method(self, rhs: DirtyTracked<Vector<N>>) -> Self::Output {
                self $op rhs.value
            }
        }
    };

    // With scalar operations
    ($trait:ident, $method:ident, $op:tt, scalar) => {
        impl_dirty_vector_op!($trait, $method, $op);

        // DirtyTracked<Vector> op f32
        impl<const N: usize> $trait<f32> for DirtyTracked<Vector<N>> {
            type Output = Vector<N>;
            fn $method(self, s: f32) -> Self::Output {
                self.value $op s
            }
        }

        // &DirtyTracked<Vector> op f32
        impl<const N: usize> $trait<f32> for &DirtyTracked<Vector<N>> {
            type Output = Vector<N>;
            fn $method(self, s: f32) -> Self::Output {
                &self.value $op s
            }
        }
    };

    // Commutative scalar operations
    ($trait:ident, $method:ident, $op:tt, scalar_commutative) => {
        impl_dirty_vector_op!($trait, $method, $op, scalar);

        // f32 op DirtyTracked<Vector>
        impl<const N: usize> $trait<DirtyTracked<Vector<N>>> for f32 {
            type Output = Vector<N>;
            fn $method(self, v: DirtyTracked<Vector<N>>) -> Self::Output {
                self $op v.value
            }
        }

        // f32 op &DirtyTracked<Vector>
        impl<const N: usize> $trait<&DirtyTracked<Vector<N>>> for f32 {
            type Output = Vector<N>;
            fn $method(self, v: &DirtyTracked<Vector<N>>) -> Self::Output {
                self $op &v.value
            }
        }
    };
}

// Op-assign versions for DirtyTracked (these will set dirty flag)
macro_rules! impl_dirty_op_assign {
    (vector: $trait:ident, $method:ident, $op:tt) => {
        impl<const N: usize> $trait for DirtyTracked<Vector<N>> {
            fn $method(&mut self, rhs: Self) {
                self.dirty = true;
                self.value $op rhs.value;
            }
        }

        impl<const N: usize> $trait<&DirtyTracked<Vector<N>>> for DirtyTracked<Vector<N>> {
            fn $method(&mut self, rhs: &DirtyTracked<Vector<N>>) {
                self.dirty = true;
                self.value $op &rhs.value;
            }
        }

        impl<const N: usize> $trait<Vector<N>> for DirtyTracked<Vector<N>> {
            fn $method(&mut self, rhs: Vector<N>) {
                self.dirty = true;
                self.value $op rhs;
            }
        }

        impl<const N: usize> $trait<&Vector<N>> for DirtyTracked<Vector<N>> {
            fn $method(&mut self, rhs: &Vector<N>) {
                self.dirty = true;
                self.value $op rhs;
            }
        }
    };

    (scalar: $trait:ident, $method:ident, $op:tt) => {
        impl<const N: usize> $trait<f32> for DirtyTracked<Vector<N>> {
            fn $method(&mut self, s: f32) {
                self.dirty = true;
                self.value $op s;
            }
        }
    };

    (both: $trait:ident, $method:ident, $op:tt) => {
        impl_dirty_op_assign!(vector: $trait, $method, $op);
        impl_dirty_op_assign!(scalar: $trait, $method, $op);
    };
}

impl_dirty_vector_op!(Add, add, +, scalar_commutative);
impl_dirty_vector_op!(Sub, sub, -, scalar);
impl_dirty_vector_op!(Mul, mul, *, scalar_commutative);
impl_dirty_vector_op!(Div, div, /, scalar);

impl_dirty_op_assign!(both: AddAssign, add_assign, +=);
impl_dirty_op_assign!(both: SubAssign, sub_assign, -=);
impl_dirty_op_assign!(both: MulAssign, mul_assign, *=);
impl_dirty_op_assign!(both: DivAssign, div_assign, /=);

impl<const N: usize> Neg for DirtyTracked<Vector<N>> {
    type Output = Vector<N>;

    fn neg(self) -> Self::Output {
        -self.value
    }
}
