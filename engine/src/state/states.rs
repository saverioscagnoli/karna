use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::{
    any::{Any, TypeId},
    sync::Arc,
};
use utils::FastHashMap;

type StateMap<T> = FastHashMap<TypeId, Box<T>>;

/// Map used to store states across all scenes of a single window.
/// Not thread safe.
pub struct States(StateMap<dyn Any>);

impl States {
    pub(crate) fn new() -> Self {
        Self(StateMap::default())
    }

    #[inline]
    pub fn insert<T: 'static>(&mut self, value: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(value));
    }

    #[inline]
    pub fn get<T: 'static>(&mut self) -> &T {
        self.0
            .get(&TypeId::of::<T>())
            .expect("Failed to find state")
            .downcast_ref::<T>()
            .expect("Failed to downcast")
    }

    #[inline]
    pub fn get_mut<T: 'static>(&mut self) -> &mut T {
        self.0
            .get_mut(&TypeId::of::<T>())
            .expect("Failed to find state")
            .downcast_mut::<T>()
            .expect("Failed to downcast")
    }

    #[inline]
    pub fn has<T: 'static>(&self) -> bool {
        self.0.contains_key(&TypeId::of::<T>())
    }
}

#[derive(Clone)]
pub struct GlobalStates(Arc<RwLock<StateMap<dyn Any + Send + Sync>>>);

impl GlobalStates {
    pub(crate) fn new() -> Self {
        Self(Arc::new(RwLock::new(StateMap::default())))
    }

    #[inline]
    pub fn insert<T: Send + Sync + 'static>(&self, value: T) {
        let mut lock = self.0.write();
        lock.insert(TypeId::of::<T>(), Box::new(value));
    }

    #[inline]
    pub fn get<T: 'static>(&self) -> MappedRwLockReadGuard<'_, T> {
        let lock = self.0.read();

        RwLockReadGuard::map(lock, |states| {
            states
                .get(&TypeId::of::<T>())
                .expect("Failed to find global state")
                .downcast_ref::<T>()
                .expect("Failed to downcast")
        })
    }

    #[inline]
    pub fn get_mut<T: 'static>(&self) -> MappedRwLockWriteGuard<'_, T> {
        let lock = self.0.write();

        RwLockWriteGuard::map(lock, |states| {
            states
                .get_mut(&TypeId::of::<T>())
                .expect("Failed to find global state")
                .downcast_mut::<T>()
                .expect("Failed to downcast")
        })
    }

    #[inline]
    pub fn has<T: 'static>(&self) -> bool {
        self.0.read().contains_key(&TypeId::of::<T>())
    }
}
