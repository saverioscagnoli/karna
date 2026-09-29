use math::Quaternion;
use nostd::collections::Handle;
use sdl3::render::Color;

use crate::mesh::Geometry;
use crate::mesh::material::Material;
use crate::mesh::transform::Transform;
use crate::render::Layer;

pub struct Mesh {
    geometry: Handle<Geometry>,
    material: Handle<Material>,
    transform: Transform,
    tint: Color,
    layer: Layer,
}

impl Mesh {
    pub fn new(geometry: Handle<Geometry>, material: Handle<Material>) -> Self {
        Self {
            geometry,
            material,
            transform: Transform::default(),
            tint: Color::WHITE,
            layer: Layer::WORLD,
        }
    }

    pub fn with_layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    #[inline]
    pub fn layer(&self) -> Layer {
        self.layer
    }

    #[inline]
    pub fn set_layer(&mut self, layer: Layer) {
        self.layer = layer;
    }

    #[inline]
    pub fn geometry(&self) -> Handle<Geometry> {
        self.geometry
    }

    #[inline]
    pub fn material(&self) -> Handle<Material> {
        self.material
    }

    #[inline]
    pub fn tint(&self) -> Color {
        self.tint
    }

    #[inline]
    pub fn set_tint<C>(&mut self, tint: C)
    where
        C: Into<Color>,
    {
        self.tint = tint.into();
    }

    #[inline]
    pub fn transform(&self) -> Transform {
        self.transform
    }

    #[inline]
    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    #[inline]
    pub fn position(&self) -> math::Vector3<f32> {
        self.transform.position
    }

    #[inline]
    pub fn position_mut(&mut self) -> &mut math::Vector3<f32> {
        &mut self.transform.position
    }

    #[inline]
    pub fn set_position(&mut self, pos: math::Vector3<f32>) {
        self.transform.position = pos;
    }

    #[inline]
    pub fn rotation(&self) -> math::Quaternion<f32> {
        self.transform.rotation
    }

    #[inline]
    pub fn rotation_mut(&mut self) -> &mut math::Quaternion<f32> {
        &mut self.transform.rotation
    }

    #[inline]
    pub fn set_rotation(&mut self, quat: math::Quaternion<f32>) {
        self.transform.rotation = quat;
    }

    #[inline]
    pub fn set_rotation_euler(&mut self, rot: math::Vector3<f32>) {
        self.transform.rotation = Quaternion::from_euler(rot.x, rot.y, rot.z);
    }

    #[inline]
    pub fn scale(&self) -> math::Vector3<f32> {
        self.transform.scale
    }

    #[inline]
    pub fn scale_mut(&mut self) -> &mut math::Vector3<f32> {
        &mut self.transform.scale
    }

    #[inline]
    pub fn set_scale(&mut self, scale: math::Vector3<f32>) {
        self.transform.scale = scale;
    }
}
