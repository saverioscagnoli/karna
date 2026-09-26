use core::any::Any;
use core::any::TypeId;
use core::any::type_name;

use nostd::alloc::boxed::Box;
use nostd::collections::HashMap;

#[derive(Default)]
pub struct SharedStore {
    data: HashMap<TypeId, Box<dyn Any>>,
}

impl SharedStore {
    #[inline]
    pub fn try_get<T: Any>(&self) -> Option<&T> {
        self.data.get(&TypeId::of::<T>())?.downcast_ref()
    }

    #[inline]
    pub fn try_get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.data.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }

    #[inline]
    #[track_caller]
    pub fn get<T: Any>(&self) -> &T {
        self.try_get()
            .unwrap_or_else(|| panic!("Shared value '{}' not found.", type_name::<T>()))
    }

    #[inline]
    #[track_caller]
    pub fn get_mut<T: Any>(&mut self) -> &mut T {
        self.try_get_mut()
            .unwrap_or_else(|| panic!("Shared value '{}' not found.", type_name::<T>()))
    }

    #[inline]
    pub fn insert<T: Any>(&mut self, value: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(value));
    }

    #[inline]
    #[allow(clippy::unwrap_used)]
    pub fn remove<T: Any>(&mut self) -> Option<T> {
        self.data
            .remove(&TypeId::of::<T>())
            .map(|boxed| *boxed.downcast::<T>().unwrap())
    }
}
