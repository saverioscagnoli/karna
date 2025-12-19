use std::ops::{Deref, DerefMut};

pub mod map;

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

pub fn as_u8_slice<T: Sized>(slice: &[T]) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * ::core::mem::size_of::<T>(),
        )
    }
}

#[derive(Debug)]
pub struct Lazy<T>(Option<T>);

impl<T> Lazy<T> {
    #[inline]
    pub fn new() -> Self {
        Self(None)
    }

    #[inline]
    pub fn init(&mut self, value: T) {
        self.0 = Some(value)
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl<T> Deref for Lazy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("Not initialized")
    }
}

impl<T> DerefMut for Lazy<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect("Not initialized")
    }
}
