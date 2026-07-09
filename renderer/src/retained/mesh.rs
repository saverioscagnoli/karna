use assets::Geometry;
use assets::Material;
use utils::Handle;

pub struct Mesh {
    pub geometry: Handle<Geometry>,
    pub material: Handle<Material>,
    pub transform: math::Matrix4<f32>,
}
