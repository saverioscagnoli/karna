mod geometry;
mod material;
mod transform;

use std::ops::Deref;
use std::ops::DerefMut;

use utils::Handle;

pub use crate::mesh::geometry::Geometry;
pub use crate::mesh::material::Blend;
pub use crate::mesh::material::Material;
pub use crate::mesh::material::MaterialDesc;
pub use crate::mesh::material::MaterialKind;
pub use crate::mesh::material::Topology;
pub use crate::mesh::transform::Transform;

pub struct Mesh {
    pub geometry: Handle<Geometry>,
    pub material: Handle<Material>,
    pub transform: Transform,
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
