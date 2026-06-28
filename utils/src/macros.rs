/// Implement Deref + DerefMut by transmuting from one type to another.k
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

#[macro_export]
macro_rules! impl_deref_to_generic {
    ($from:ident<$t:ident> => $to:ty where $t2:ident: $($bound:tt)+) => {
        impl<$t: $($bound)+> ::std::ops::Deref for $from<$t> {
            type Target = $to;
            #[inline]
            fn deref(&self) -> &Self::Target {
                unsafe { &*(self as *const Self as *const $to) }
            }
        }
        impl<$t: $($bound)+> ::std::ops::DerefMut for $from<$t> {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { &mut *(self as *mut Self as *mut $to) }
            }
        }
    };
}
