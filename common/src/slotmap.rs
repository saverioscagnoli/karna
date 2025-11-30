use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    id: u32,
    generation: u32,
    _marker: PhantomData<T>,
}

struct Entry<T> {
    generation: u32,
    item: Option<T>,
}

pub struct Slotmap<T> {
    entries: Vec<Entry<T>>,
    free_list: Vec<u32>,
}

impl<T> Slotmap<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn insert(&mut self, item: T) -> Handle<T> {
        if let Some(id) = self.free_list.pop() {
            let entry = &mut self.entries[id as usize];

            entry.generation += 1;
            entry.item = Some(item);

            return Handle {
                id,
                generation: entry.generation,
                _marker: PhantomData,
            };
        }

        let id = self.entries.len() as u32;

        self.entries.push(Entry {
            generation: 0,
            item: Some(item),
        });

        Handle {
            id,
            generation: 0,
            _marker: PhantomData,
        }
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        self.entries
            .get(handle.id as usize)
            .filter(|e| e.generation == handle.generation)
            .and_then(|e| e.item.as_ref())
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.entries
            .get_mut(handle.id as usize)
            .filter(|e| e.generation == handle.generation)
            .and_then(|e| e.item.as_mut())
    }

    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let entry = self.entries.get_mut(handle.id as usize)?;

        if entry.generation != handle.generation {
            return None;
        }

        self.free_list.push(handle.id);
        entry.item.take()
    }
}
