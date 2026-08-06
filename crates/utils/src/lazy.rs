use std::ops::Deref;
use std::ops::DerefMut;

pub struct Lazy<T>(Option<T>);

impl<T> Deref for Lazy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("Dereferencing an unset lazy")
    }
}

impl<T> DerefMut for Lazy<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect("Dereferencing an unset lazy")
    }
}

impl<T> Lazy<T> {
    pub fn unset() -> Self {
        Self(None)
    }

    pub fn set(&mut self, item: T) {
        self.0 = Some(item)
    }
}
