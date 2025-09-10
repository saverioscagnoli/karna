use crate::{
    color::Color,
    fundamentals::Vertex,
    mesh::{Mesh, MeshInstance},
};
use math::{Size, Vec2, Vec3};

pub struct Rect(MeshInstance);

impl Mesh for Rect {
    fn vertices() -> Vec<Vertex> {
        vec![
            Vertex {
                position: Vec3::new(-0.5, -0.5, 0.0),
                color: Color::WHITE.into(),
            },
            Vertex {
                position: Vec3::new(0.5, -0.5, 0.0),
                color: Color::WHITE.into(),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, 0.0),
                color: Color::WHITE.into(),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, 0.0),
                color: Color::WHITE.into(),
            },
        ]
    }

    fn indices() -> Vec<u16> {
        vec![0, 1, 2, 2, 3, 0]
    }

    fn instance(&self) -> &MeshInstance {
        &self.0
    }

    fn insance_mut(&mut self) -> &mut MeshInstance {
        &mut self.0
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self(MeshInstance::new(
            Vec3::zero(),
            Vec3::new(10.0, 10.0, 1.0),
            Vec3::zero(),
            Color::WHITE.into(),
        ))
    }
}

impl Rect {
    pub fn new<P: Into<Vec3>, S: Into<Size<f32>>>(pos: P, size: S) -> Self {
        Self(MeshInstance::new(
            pos.into(),
            Vec2::from(size.into()).extend(1.0),
            Vec3::zero(),
            Color::WHITE.into(),
        ))
    }

    pub fn with_position<P: Into<Vec3>>(mut self, pos: P) -> Self {
        self.0.translation = pos.into();
        self
    }

    pub fn with_size<S: Into<Size<f32>>>(mut self, size: S) -> Self {
        self.0.scale = Vec2::from(size.into()).extend(1.0);
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.0.color = color.into();
        self
    }
}
