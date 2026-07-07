use std::any::Any;

use utils::Handle;
use utils::SlotMap;

pub struct GenericStorage {
    items: SlotMap<Box<dyn Any>>,
}

impl GenericStorage {
    pub(crate) fn new() -> Self {
        Self {
            items: SlotMap::new(),
        }
    }

    pub fn insert<T: 'static>(&mut self, value: T) -> Handle<T> {
        let boxed: Box<dyn Any> = Box::new(value);
        let handle = self.items.insert(boxed);

        handle.cast::<T>()
    }

    pub fn remove<T: 'static>(&mut self, item: Handle<T>) -> T {
        let boxed = self
            .items
            .remove(item.cast::<Box<dyn Any>>())
            .expect("Failed to remove item from generic storage");

        *boxed
            .downcast::<T>()
            .expect("Failed to downcast removed item")
    }

    pub fn remove_any<T: 'static>(&mut self, item: Handle<T>) {
        self.items
            .remove(item.cast::<Box<dyn Any>>())
            .expect("Failed to remove item from generic storage");
    }

    pub fn get<T: 'static>(&self, item: Handle<T>) -> &T {
        self.items
            .get(item.cast::<Box<dyn Any>>())
            .expect("Failed to get item from generic storage")
            .downcast_ref::<T>()
            .expect("Failed to get item from generic storage")
    }

    pub fn get_mut<T: 'static>(&mut self, item: Handle<T>) -> &mut T {
        self.items
            .get_mut(item.cast::<Box<dyn Any>>())
            .expect("Failed to get item from generic storage")
            .downcast_mut::<T>()
            .expect("Failed to get item from generic storage")
    }
}
