use std::{
    collections::HashMap,
    hash::{BuildHasher, BuildHasherDefault, Hasher},
    marker::PhantomData,
};

use macros::Get;

#[derive(Debug, Hash)]
#[derive(Get)]
pub struct Handle<T> {
    #[get(copied)]
    index: u32,
    #[get(copied)]
    generation: u32,
    _d: PhantomData<T>,
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for Handle<T> {}

impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index
            .cmp(&other.index)
            .then(self.generation.cmp(&other.generation))
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Handle<T> {
    fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _d: PhantomData,
        }
    }

    pub fn dummy() -> Self {
        Self {
            index: 0,
            generation: 0,
            _d: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Slot<T> {
    value: Option<T>,
    generation: u32,
}

#[derive(Debug, Clone)]
pub struct SlotMap<T> {
    slots: Vec<Slot<T>>,
    free_list: Vec<u32>,
}

impl<T> SlotMap<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            free_list: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> Handle<T> {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index as usize];

            slot.value = Some(value);

            Handle::new(index, slot.generation)
        } else {
            let index = self.slots.len() as u32;

            self.slots.push(Slot {
                value: Some(value),
                generation: 1,
            });

            Handle::new(index, 1)
        }
    }

    /// Insert with access to the handle that will be created
    /// Useful when you need to derive data from the handle (like a label)
    pub fn insert_with_key<F>(&mut self, f: F) -> Handle<T>
    where
        F: FnOnce(Handle<T>) -> T,
    {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index as usize];
            let handle = Handle::new(index, slot.generation);

            slot.value = Some(f(handle));

            handle
        } else {
            let index = self.slots.len() as u32;
            let handle = Handle::new(index, 1);

            self.slots.push(Slot {
                value: Some(f(handle)),
                generation: 1,
            });

            handle
        }
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let slot = self.slots.get(handle.index as usize)?;

        if slot.generation != handle.generation {
            return None;
        }

        slot.value.as_ref()
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.index as usize)?;

        if slot.generation != handle.generation {
            return None;
        }

        slot.value.as_mut()
    }

    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let slot = self.slots.get_mut(handle.index as usize)?;

        if slot.generation != handle.generation {
            return None;
        }

        let value = slot.value.take()?;

        slot.generation = slot.generation.wrapping_add(1);
        self.free_list.push(handle.index);

        Some(value)
    }

    /// Check if a handle is valid
    pub fn contains(&self, handle: Handle<T>) -> bool {
        self.slots.get(handle.index as usize).map_or(false, |slot| {
            slot.generation == handle.generation && slot.value.is_some()
        })
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.slots.iter().filter(|s| s.value.is_some()).count()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.slots.clear();
        self.free_list.clear();
    }

    /// Iterator over references
    pub fn iter(&self) -> impl Iterator<Item = (Handle<T>, &T)> {
        self.slots.iter().enumerate().filter_map(|(idx, slot)| {
            slot.value
                .as_ref()
                .map(|v| (Handle::new(idx as u32, slot.generation), v))
        })
    }

    /// Iterator over mutable references
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Handle<T>, &mut T)> {
        self.slots.iter_mut().enumerate().filter_map(|(idx, slot)| {
            slot.value
                .as_mut()
                .map(|v| (Handle::new(idx as u32, slot.generation), v))
        })
    }

    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.slots.iter().filter_map(|slot| slot.value.as_ref())
    }

    #[inline]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slots.iter_mut().filter_map(|slot| slot.value.as_mut())
    }
}

impl<T> Default for SlotMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        panic!("IdentityHasher only works with u32")
    }

    fn write_u32(&mut self, i: u32) {
        self.0 = i as u64;
    }
}

#[derive(Default)]
pub struct IdentityHasherBuilder;

impl BuildHasher for IdentityHasherBuilder {
    type Hasher = IdentityHasher;
    fn build_hasher(&self) -> Self::Hasher {
        IdentityHasher(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(u32);

impl Label {
    const FNV_OFFSET: u32 = 2166136261;
    const FNV_PRIME: u32 = 16777619;

    /// Create a label from a string slice at compile time
    pub const fn new(s: &str) -> Self {
        Self(Self::hash(s))
    }

    /// Get the raw hash value
    pub const fn raw(&self) -> u32 {
        self.0
    }

    pub const fn hash(s: &str) -> u32 {
        let bytes = s.as_bytes();
        let mut hash = Self::FNV_OFFSET;
        let mut i = 0;

        while i < bytes.len() {
            hash ^= bytes[i] as u32;
            hash = hash.wrapping_mul(Self::FNV_PRIME);
            i += 1;
        }

        hash
    }
}

#[macro_export]
macro_rules! label {
    ($s:literal) => {{
        const LABEL: $crate::Label = $crate::Label::new($s);
        LABEL
    }};
}

/// Only works with u32
pub type UMap<K, V> = HashMap<K, V, IdentityHasherBuilder>;
pub type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<rustc_hash::FxHasher>>;
