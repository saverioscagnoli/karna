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

const INSTANCE_ATTRS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
    3 => Float32x4,
    4 => Float32x4,
    5 => Float32x4,
    6 => Float32x4,
    7 => Float32x4,
];

const INSTANCE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: 80,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &INSTANCE_ATTRS,
};

pub struct Mesh {
    pub geometry: Handle<Geometry>,
    pub material: Handle<Material>,
    pub transform: Transform,
    pub color: Color,
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
