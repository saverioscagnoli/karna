pub mod geometry;
pub mod material;
pub mod transform;

use utils::Handle;
use utils::SlotMap;

use crate::render::mesh::geometry::Geometry;
use crate::render::mesh::material::Material;
use crate::render::mesh::transform::Transform;

pub struct Mesh {
    geometry: Handle<Geometry>,
    material: Handle<Material>,
    transform: Transform,
}

impl Mesh {
    pub fn new(
        geometry: Handle<Geometry>,
        material: Handle<Material>,
        transform: Transform,
    ) -> Self {
        Self {
            geometry,
            material,
            transform,
        }
    }

    pub fn geometry(&self) -> Handle<Geometry> {
        self.geometry
    }

    pub fn material(&self) -> Handle<Material> {
        self.material
    }

    pub fn set_material(&mut self, material: Handle<Material>) {
        self.material = material;
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

#[derive(Default)]
pub struct MeshStorage {
    geometries: SlotMap<Geometry>,
    materials: SlotMap<Material>,
}

impl MeshStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_geometry(&mut self, geometry: Geometry) -> Handle<Geometry> {
        self.geometries.insert(geometry)
    }

    pub fn add_material(&mut self, material: Material) -> Handle<Material> {
        self.materials.insert(material)
    }

    pub fn geometry(&self, handle: Handle<Geometry>) -> &Geometry {
        &self.geometries[handle]
    }

    pub fn geometry_mut(&mut self, handle: Handle<Geometry>) -> &mut Geometry {
        &mut self.geometries[handle]
    }

    pub fn material(&self, handle: Handle<Material>) -> &Material {
        &self.materials[handle]
    }

    pub fn material_mut(&mut self, handle: Handle<Material>) -> &mut Material {
        &mut self.materials[handle]
    }
}

#[derive(Debug, Clone)]
pub struct MeshDraw {
    pub geometry: Geometry,
    pub material: Material,
    pub model: math::Matrix4<f32>,
    pub normal: math::Matrix4<f32>,
}
