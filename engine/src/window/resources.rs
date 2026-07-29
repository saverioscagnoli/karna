use std::any::Any;
use std::any::TypeId;

use logging::fatal;
use utils::FastHashMap;

#[derive(Default)]
pub struct Resources {
    items: FastHashMap<TypeId, Box<dyn Any>>,
}

impl Resources {
    pub fn insert<T>(&mut self, item: T)
    where
        T: 'static,
    {
        self.items.insert(TypeId::of::<T>(), Box::new(item));
    }

    pub fn get<T>(&self) -> &T
    where
        T: 'static,
    {
        match self.items.get(&TypeId::of::<T>()) {
            Some(t) => match t.downcast_ref() {
                Some(t) => t,
                None => fatal!("Failed to get resource of type '{:?}'", TypeId::of::<T>()),
            },
            None => fatal!("Failed to get resource of type '{:?}'", TypeId::of::<T>()),
        }
    }

    pub fn get_mut<T>(&mut self) -> &mut T
    where
        T: 'static,
    {
        match self.items.get_mut(&TypeId::of::<T>()) {
            Some(t) => match t.downcast_mut() {
                Some(t) => t,
                None => fatal!("Failed to get resource of type '{:?}'", TypeId::of::<T>()),
            },
            None => fatal!("Failed to get resource of type '{:?}'", TypeId::of::<T>()),
        }
    }
}
