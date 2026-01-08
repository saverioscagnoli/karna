use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Lazy<T>(Option<T>);

impl<T> Lazy<T> {
    #[inline]
    pub fn new() -> Self {
        Self(None)
    }

    #[inline]
    pub fn set(&mut self, value: T) {
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
