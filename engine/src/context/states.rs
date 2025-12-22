use std::{
    any::{Any, TypeId},
    ops::{Deref, DerefMut},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use wgpu::naga::{FastHashMap, Statement};

pub type StateMap = FastHashMap<TypeId, Box<dyn Any + Send + Sync>>;

pub struct StateRef<'a, T: 'static> {
    _guard: RwLockReadGuard<'a, StateMap>,
    value: *const T,
}

impl<'a, T: 'static> Deref for StateRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

pub struct StateRefMut<'a, T: 'static> {
    _guard: RwLockWriteGuard<'a, StateMap>,
    value: *mut T,
}

impl<'a, T: 'static> Deref for StateRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

impl<'a, T: 'static> DerefMut for StateRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value }
    }
}

pub struct States(RwLock<StateMap>);

impl States {
    pub(crate) fn new() -> Self {
        Self(RwLock::new(StateMap::default()))
    }

    pub fn insert<T: Send + Sync + 'static>(&self, value: T) -> Option<T> {
        let mut lock = self.0.write().expect("State lock is poisoned");

        lock.insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    pub fn get<T: 'static>(&self) -> Option<StateRef<T>> {
        let lock = self.0.read().expect("State lock is poisoned");

        let ptr = lock
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
            .map(|r| r as *const T)?;

        Some(StateRef {
            _guard: lock,
            value: ptr,
        })
    }

    pub fn get_mut<T: 'static>(&self) -> Option<StateRefMut<T>> {
        let mut lock = self.0.write().expect("State lock is poisoned");

        let ptr = lock
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
            .map(|r| r as *mut T)?;

        Some(StateRefMut {
            _guard: lock,
            value: ptr,
        })
    }

    #[inline]
    pub fn remove<T: 'static>(&self) -> Option<T> {
        let mut lock = self.0.write().expect("States lock is poisoned");

        lock.remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    #[inline]
    pub fn has<T: 'static>(&self) -> bool {
        let lock = self.0.read().expect("States lock is poisoned");

        lock.contains_key(&TypeId::of::<T>())
    }
}
