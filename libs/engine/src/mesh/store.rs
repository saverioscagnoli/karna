use core::mem;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;

use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use nostd::collections::SlotMap;

use crate::mesh::Mesh;

static NEXT_STORE: AtomicU64 = AtomicU64::new(1);

#[derive(Default)]
pub(crate) struct MeshChanges {
    pub reset: bool,
    pub spawned: Vec<Handle<Mesh>>,
    pub despawned: Vec<Handle<Mesh>>,
    pub dirty: Vec<Handle<Mesh>>,
}

struct MeshEntry {
    mesh: Mesh,
    dirty: bool,
}

pub struct MeshStore {
    id: u64,
    meshes: SlotMap<MeshEntry>,
    changes: MeshChanges,
}

impl Default for MeshStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshStore {
    pub fn new() -> Self {
        Self {
            id: NEXT_STORE.fetch_add(1, Ordering::Relaxed),
            meshes: SlotMap::default(),
            changes: MeshChanges::default(),
        }
    }

    pub fn spawn(&mut self, mesh: Mesh) -> Handle<Mesh> {
        let handle = self.meshes.insert(MeshEntry { mesh, dirty: false }).cast();

        if !self.changes.reset {
            self.changes.spawned.push(handle);
            self.collapse();
        }

        handle
    }

    pub fn get(&self, handle: Handle<Mesh>) -> Option<&Mesh> {
        self.meshes.get(handle.cast()).map(|e| &e.mesh)
    }

    pub fn get_mut(&mut self, handle: Handle<Mesh>) -> Option<&mut Mesh> {
        let entry = self.meshes.get_mut(handle.cast())?;

        if !entry.dirty && !self.changes.reset {
            entry.dirty = true;
            self.changes.dirty.push(handle);
        }

        Some(&mut entry.mesh)
    }

    pub fn despawn(&mut self, handle: Handle<Mesh>) -> Option<Mesh> {
        let entry = self.meshes.remove(handle.cast())?;

        if !self.changes.reset {
            self.changes.despawned.push(handle);
            self.collapse();
        }

        Some(entry.mesh)
    }

    pub fn contains(&self, handle: Handle<Mesh>) -> bool {
        self.meshes.contains(handle.cast())
    }

    pub fn len(&self) -> usize {
        self.meshes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.meshes.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Handle<Mesh>, &Mesh)> {
        self.meshes.iter().map(|(h, e)| (h.cast(), &e.mesh))
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    fn collapse(&mut self) {
        let pending = self.changes.spawned.len() + self.changes.despawned.len();

        if pending > self.meshes.len() * 2 + 64 {
            self.clear_dirty_flags();
            self.changes = MeshChanges {
                reset: true,
                ..MeshChanges::default()
            };
        }
    }

    fn clear_dirty_flags(&mut self) {
        for handle in &self.changes.dirty {
            if let Some(entry) = self.meshes.get_mut(handle.cast()) {
                entry.dirty = false;
            }
        }
    }

    pub(crate) fn take_changes(&mut self) -> MeshChanges {
        self.clear_dirty_flags();
        mem::take(&mut self.changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mesh() -> Mesh {
        Mesh::new(Handle::INVALID, Handle::INVALID)
    }

    #[test]
    fn get_mut_marks_once_per_frame() {
        let mut store = MeshStore::new();
        let a = store.spawn(mesh());
        let b = store.spawn(mesh());
        store.take_changes();

        store.get_mut(a);
        store.get_mut(a);
        store.get_mut(b);

        let changes = store.take_changes();
        assert!(changes.spawned.is_empty());
        assert_eq!(changes.dirty, [a, b]);

        store.get_mut(a);
        assert_eq!(store.take_changes().dirty, [a]);
    }

    #[test]
    fn get_does_not_mark() {
        let mut store = MeshStore::new();
        let a = store.spawn(mesh());
        store.take_changes();

        store.get(a);
        assert!(store.take_changes().dirty.is_empty());
    }

    #[test]
    fn reports_spawns_and_despawns() {
        let mut store = MeshStore::new();
        let a = store.spawn(mesh());
        let b = store.spawn(mesh());

        let changes = store.take_changes();
        assert_eq!(changes.spawned, [a, b]);
        assert!(changes.despawned.is_empty());

        assert!(store.despawn(a).is_some());
        assert!(store.despawn(a).is_none());

        let changes = store.take_changes();
        assert!(changes.spawned.is_empty());
        assert_eq!(changes.despawned, [a]);
        assert!(store.get(a).is_none());
    }

    #[test]
    fn collapses_into_reset_when_unconsumed() {
        let mut store = MeshStore::new();

        for _ in 0..200 {
            let h = store.spawn(mesh());
            store.despawn(h);
        }

        let changes = store.take_changes();
        assert!(changes.reset);
        assert!(changes.spawned.is_empty());
        assert!(changes.despawned.is_empty());
        assert!(!store.take_changes().reset);
    }
}
