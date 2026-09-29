use nostd::collections::Handle;
use nostd::collections::SlotMap;

use crate::mesh::Material;

pub struct MaterialRegistry {
    materials: SlotMap<Material>,
    fallback: Material,
    generation: u64,
}

impl Default for MaterialRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialRegistry {
    pub fn new() -> Self {
        Self {
            materials: SlotMap::default(),
            fallback: Material::default(),
            generation: 0,
        }
    }

    pub fn add(&mut self, material: Material) -> Handle<Material> {
        self.generation += 1;
        self.materials.insert(material)
    }

    pub fn get(&self, handle: Handle<Material>) -> Option<&Material> {
        self.materials.get(handle)
    }

    pub fn get_mut(&mut self, handle: Handle<Material>) -> Option<&mut Material> {
        self.generation += 1;
        self.materials.get_mut(handle)
    }

    pub fn remove(&mut self, handle: Handle<Material>) -> Option<Material> {
        self.generation += 1;
        self.materials.remove(handle)
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (Handle<Material>, &Material)> {
        self.materials.iter()
    }

    pub(crate) fn resolve(&self, handle: Handle<Material>) -> &Material {
        self.materials.get(handle).unwrap_or(&self.fallback)
    }
}
