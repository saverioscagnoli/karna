use std::{
    any::{Any, TypeId},
    ops::{Deref, DerefMut},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use utils::{Label, LabelMap};
use wgpu::naga::FastHashMap;

/// Map used to store states across all scenes of a single window.
/// Not thread safe.
///
/// This differs from [`GlobalStates`], as here more than one instance of the same
/// type can exist, at the cost of assigning each instance a unique label.
///
/// On the other hand, [`GlobalStates`]  is slower and should not be used for frequent access.
/// So that's why it supports only one instance of each type.
pub struct ScopedStates(FastHashMap<TypeId, LabelMap<Box<dyn Any>>>);

impl ScopedStates {
    pub fn new() -> Self {
        ScopedStates(FastHashMap::default())
    }

    #[inline]
    pub fn insert<T: Send + Sync + 'static>(&mut self, label: Label, value: T) -> Option<T> {
        let state_map = self
            .0
            .entry(TypeId::of::<T>())
            .or_insert_with(LabelMap::default);

        state_map
            .insert(label, Box::new(value))
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    #[inline]
    pub fn get<T: 'static>(&self, label: Label) -> Option<&T> {
        let state_map = self.0.get(&TypeId::of::<T>())?;

        state_map
            .get(&label)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    #[inline]
    pub fn get_mut<T: Send + Sync + 'static>(&mut self, label: Label) -> Option<&mut T> {
        let state_map = self.0.get_mut(&TypeId::of::<T>())?;

        state_map
            .get_mut(&label)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    #[inline]
    pub fn remove<T: 'static>(&mut self, label: Label) -> Option<T> {
        let state_map = self.0.get_mut(&TypeId::of::<T>())?;

        state_map
            .remove(&label)
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    #[inline]
    pub fn has<T: 'static>(&self, label: Label) -> bool {
        self.0
            .get(&TypeId::of::<T>())
            .is_some_and(|state_map| state_map.contains_key(&label))
    }
}

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

/// Map to store states across all scenes of all windows.
/// Thread safe.
///
/// This map shall not be used for frequent write access, as it's slower than [`ScopedStates`],
/// and also not meant for that.
///
/// As an example, it can be used for storing save states once in a while.
pub struct GlobalStates(RwLock<StateMap>);

impl GlobalStates {
    pub(crate) fn new() -> Self {
        Self(RwLock::new(StateMap::default()))
    }

    pub fn insert<T: Send + Sync + 'static>(&self, value: T) -> Option<T> {
        let mut lock = self.0.write().expect("State lock is poisoned");

        lock.insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    #[inline]
    pub fn get<'a, T: 'static>(&'a self) -> Option<StateRef<'a, T>> {
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

    #[inline]
    pub fn get_mut<'a, T: 'static>(&'a self) -> Option<StateRefMut<'a, T>> {
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
