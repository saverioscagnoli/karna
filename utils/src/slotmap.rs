use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Index;
use std::ops::IndexMut;

#[derive(Debug)]
pub struct Handle<T> {
    index: u32,
    generation: u32,
    _d: PhantomData<T>,
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self::INVALID
    }
}

impl<T> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
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
    pub const INVALID: Self = Self {
        index: u32::MAX,
        generation: 0,
        _d: PhantomData,
    };

    fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _d: PhantomData,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.generation != 0
    }

    pub fn cast<U>(self) -> Handle<U> {
        Handle {
            index: self.index,
            generation: self.generation,
            _d: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Slot<T> {
    value: Option<T>,
    generation: u32,
}

#[derive(Debug, Clone)]
pub struct SlotMap<T> {
    slots: Vec<Slot<T>>,
    free_list: Vec<u32>,
    len: usize,
}

impl<T> Index<Handle<T>> for SlotMap<T> {
    type Output = T;

    fn index(&self, handle: Handle<T>) -> &T {
        self.get(handle)
            .expect("invalid slotmap handle (stale generation or empty slot)")
    }
}

impl<T> IndexMut<Handle<T>> for SlotMap<T> {
    fn index_mut(&mut self, handle: Handle<T>) -> &mut T {
        self.get_mut(handle)
            .expect("invalid slotmap handle (stale generation or empty slot)")
    }
}

impl<T> SlotMap<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            free_list: Vec::new(),
            len: 0,
        }
    }

    pub fn insert(&mut self, value: T) -> Handle<T> {
        self.len += 1;

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
        self.len += 1;

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
        self.len -= 1;

        Some(value)
    }

    /// Check if a handle is valid
    pub fn contains(&self, handle: Handle<T>) -> bool {
        self.slots.get(handle.index as usize).map_or(false, |slot| {
            slot.generation == handle.generation && slot.value.is_some()
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if slot.value.take().is_some() {
                slot.generation = slot.generation.wrapping_add(1);

                if slot.generation == 0 {
                    slot.generation = 1;
                }

                self.free_list.push(i as u32);
            }
        }

        self.len = 0;
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
