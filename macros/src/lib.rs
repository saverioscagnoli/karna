/// For things to implement the `Mesh` trait (see renderer/src/mesh.rs)
/// They must implement Deref<Target = InstanceData>
#[macro_export]
macro_rules! impl_mesh_deref {
    ($type:ty) => {
        impl Deref for $type {
            type Target = MeshInstanceData;

            fn deref(&self) -> &Self::Target {
                &self.instance_data
            }
        }

        impl DerefMut for $type {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.instance_data
            }
        }
    };
}

/// Implements the `AsF32` (see `math/src/lib.rs`) for the given trait
#[macro_export]
macro_rules! impl_as_f32 {
    ($($t:ty)*) => {
        $(
            impl AsF32 for $t {
                fn as_f32(&self) -> f32 {
                    *self as f32
                }
            }
        )*
    };
}

/// Implement Deref + DerefMut by transmuting from one type to another.
///
/// IMPORTANT: Both types MUST have the same memory layout!!
#[macro_export]
macro_rules! impl_deref_to {
    ($from:ty => $to:ty) => {
        impl ::std::ops::Deref for $from {
            type Target = $to;

            #[inline]
            fn deref(&self) -> &Self::Target {
                unsafe { &*(self as *const Self as *const $to) }
            }
        }

        impl ::std::ops::DerefMut for $from {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { &mut *(self as *mut Self as *mut $to) }
            }
        }
    };
}

/// Implement binary operators (Add, Sub, Mul, Div) for Vector<N>
#[macro_export]
macro_rules! impl_vec_op {
    // With scalar commutative variant (for Add and Mul)
    ($trait:ident, $method:ident, $op:tt, commutative) => {
        // Vector op Vector
        impl<const N: usize> ::std::ops::$trait for Vector<N> {
            type Output = Self;

            #[inline]
            fn $method(self, rhs: Self) -> Self::Output {
                let mut result = Self::zero();
                for i in 0..N {
                    result[i] = self[i] $op rhs[i];
                }
                result
            }
        }

        // Vector op f32
        impl<const N: usize> ::std::ops::$trait<f32> for Vector<N> {
            type Output = Self;

            #[inline]
            fn $method(self, rhs: f32) -> Self::Output {
                let mut result = Self::zero();
                for i in 0..N {
                    result[i] = self[i] $op rhs;
                }
                result
            }
        }

        // f32 op Vector (only if commutative)
        impl<const N: usize> ::std::ops::$trait<Vector<N>> for f32 {
            type Output = Vector<N>;

            #[inline]
            fn $method(self, rhs: Vector<N>) -> Self::Output {
                let mut result = Vector::zero();
                for i in 0..N {
                    result[i] = self $op rhs[i];
                }
                result
            }
        }
    };

    // Without scalar commutative variant (for Sub and Div)
    ($trait:ident, $method:ident, $op:tt) => {
        // Vector op Vector
        impl<const N: usize> ::std::ops::$trait for Vector<N> {
            type Output = Self;

            #[inline]
            fn $method(self, rhs: Self) -> Self::Output {
                let mut result = Self::zero();
                for i in 0..N {
                    result[i] = self[i] $op rhs[i];
                }
                result
            }
        }

        // Vector op f32
        impl<const N: usize> ::std::ops::$trait<f32> for Vector<N> {
            type Output = Self;

            #[inline]
            fn $method(self, rhs: f32) -> Self::Output {
                let mut result = Self::zero();
                for i in 0..N {
                    result[i] = self[i] $op rhs;
                }
                result
            }
        }
    };
}

/// Implement assignment operators (AddAssign, SubAssign, MulAssign, DivAssign) for Vector<N>
#[macro_export]
macro_rules! impl_vec_op_assign {
    ($trait:ident, $method:ident, $op:tt) => {
        // Vector op= Vector
        impl<const N: usize> ::std::ops::$trait for Vector<N> {
            #[inline]
            fn $method(&mut self, rhs: Self) {
                for i in 0..N {
                    self[i] $op rhs[i];
                }
            }
        }

        // Vector op= f32
        impl<const N: usize> ::std::ops::$trait<f32> for Vector<N> {
            #[inline]
            fn $method(&mut self, rhs: f32) {
                for i in 0..N {
                    self[i] $op rhs;
                }
            }
        }
    };
}
