use std::ops::Deref;
use std::ops::DerefMut;

pub struct Lazy<T>(Option<T>);

impl<T> Lazy<T> {
    pub fn empty() -> Self {
        Self(None)
    }

    pub fn set(&mut self, value: T) {
        self.0 = Some(value);
    }

    pub fn unset(&mut self) {
        self.0 = None;
    }
}

impl<T> Deref for Lazy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
            .as_ref()
            .expect("trying to access an uninitialized lazy")
    }
}

impl<T> DerefMut for Lazy<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
            .as_mut()
            .expect("trying to access an uninitialized lazy")
    }
}
