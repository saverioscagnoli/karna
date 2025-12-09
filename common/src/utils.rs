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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(u32);

impl Label {
    const FNV_OFFSET: u32 = 2166136261;
    const FNV_PRIME: u32 = 16777619;

    /// Create a label from a string slice at compile time
    pub const fn new(s: &str) -> Self {
        Self(Self::hash(s))
    }

    /// Get the raw hash value
    pub const fn raw(&self) -> u32 {
        self.0
    }

    pub const fn hash(s: &str) -> u32 {
        let bytes = s.as_bytes();
        let mut hash = Self::FNV_OFFSET;
        let mut i = 0;

        while i < bytes.len() {
            hash ^= bytes[i] as u32;
            hash = hash.wrapping_mul(Self::FNV_PRIME);
            i += 1;
        }

        hash
    }
}

#[macro_export]
macro_rules! label {
    ($s:expr) => {
        $crate::utils::Label::new($s)
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
