mod geometry;
mod material;
mod transform;

use std::ops::Deref;
use std::ops::DerefMut;

use utils::Handle;

use crate::Color;
pub use crate::mesh::geometry::Geometry;
pub use crate::mesh::material::Blend;
pub use crate::mesh::material::Material;
pub use crate::mesh::material::MaterialDesc;
pub use crate::mesh::material::MaterialKind;
pub use crate::mesh::material::Topology;
pub use crate::mesh::transform::Transform;

#[derive(Default)]
pub struct Mesh {
    pub(crate) geometry: Handle<Geometry>,
    pub(crate) material: Handle<Material>,

    // Only valid for mesh instances
    pub transform: Transform,
    pub tint: Color,
}

impl Mesh {
    pub fn new(geometry: Handle<Geometry>, material: Handle<Material>) -> Self {
        Self {
            geometry,
            material,
            ..Default::default()
        }
    }

    pub fn with_transform(mut self, t: Transform) -> Self {
        self.transform = t;
        self
    }

    pub fn with_tint(mut self, c: Color) -> Self {
        self.tint = c;
        self
    }

    pub fn geometry(&self) -> Handle<Geometry> {
        self.geometry
    }

    pub fn material(&self) -> Handle<Material> {
        self.material
    }
}

impl Deref for Mesh {
    type Target = Transform;

    fn deref(&self) -> &Self::Target {
        &self.transform
    }
}

impl DerefMut for Mesh {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MeshInstanceData {
    pub model: [[f32; 4]; 4],
    pub tint: [f32; 4],
}
