use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::OnceLock,
};

use crate::{
    Descriptor, Renderer,
    color::Color,
    fundamentals::Vertex,
    mesh::{Mesh, MeshInstance},
};
use math::{Size, Vec2, Vec3, Vec4};

pub struct Rect {
    pub position: Vec2,
    pub size: Size<f32>,
    pub color: Color,
}

impl Mesh for Rect {
    fn id() -> u64 {
        static ID: OnceLock<u64> = OnceLock::new();

        *ID.get_or_init(|| {
            let type_name = std::any::type_name::<Self>();
            let mut hasher = DefaultHasher::new();
            type_name.hash(&mut hasher);
            hasher.finish()
        })
    }

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
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            position: Vec2::zero(),
            size: Size::new(10.0, 10.0),
            color: Color::WHITE,
        }
    }
}

impl Rect {
    pub fn new<P: Into<Vec2>, S: Into<Size<f32>>>(pos: P, size: S) -> Self {
        Self {
            position: pos.into(),
            size: size.into(),
            color: Color::WHITE,
        }
    }

    pub fn with_position<P: Into<Vec2>>(mut self, pos: P) -> Self {
        self.position = pos.into();
        self
    }

    pub fn with_size<S: Into<Size<f32>>>(mut self, size: S) -> Self {
        self.size = size.into();
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn render(&self, renderer: &mut Renderer) {
        renderer.add_mesh_instance::<Self>(MeshInstance {
            translation: Vec3::new(self.position.x, self.position.y, -1.0), // Move back in Z
            rotation: Vec3::zero(),
            scale: Vec3::new(self.size.width, self.size.height, 1.0),
            color: self.color.into(),
        });
    }
}
