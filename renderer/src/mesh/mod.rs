mod geometry;
mod material;
mod transform;

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
