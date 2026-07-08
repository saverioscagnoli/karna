use std::any::Any;
use std::any::TypeId;

use utils::FastHashMap;

pub struct Resources {
    items: FastHashMap<TypeId, Box<dyn Any>>,
}

impl Resources {
    pub(crate) fn new() -> Self {
        Self {
            items: FastHashMap::default(),
        }
    }

    /// Insert a value, keyed by its concrete type.
    /// `T: 'static` is required because `Any` only works for owned,
    /// non-borrowed types (no lifetimes inside T).
    pub fn insert<T: 'static>(&mut self, item: T) {
        self.items.insert(TypeId::of::<T>(), Box::new(item));
    }

    /// Get a shared reference to the stored value of type T, if present.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.items
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable reference to the stored value of type T, if present.
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.items
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Remove and return the stored value of type T, if present.
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.items
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    /// Check whether a value of type T is stored.
    pub fn contains<T: 'static>(&self) -> bool {
        self.items.contains_key(&TypeId::of::<T>())
    }
}
