use std::any::{Any, TypeId};

use utils::FastHashMap;

#[derive(Default)]
#[derive(Debug)]
pub struct Resources {
    items: FastHashMap<TypeId, Box<dyn Any>>,
}

impl Resources {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: 'static>(&mut self, item: T) {
        self.items.insert(item.type_id(), Box::new(item));
    }

    pub fn get<T: 'static>(&self) -> &T {
        self.items
            .get(&TypeId::of::<T>())
            .expect("Failed to get item from resources")
            .downcast_ref()
            .expect("Failed to cast resource item")
    }

    pub fn get_mut<T: 'static>(&mut self) -> &mut T {
        self.items
            .get_mut(&TypeId::of::<T>())
            .expect("Failed to get item from resources")
            .downcast_mut()
            .expect("Failed to cast resource item")
    }
}
