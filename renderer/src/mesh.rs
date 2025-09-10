use math::{Vec3, Vec4};
use std::sync::OnceLock;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

pub trait Mesh {
    const VERTICES: &'static [Vertex];
    const INDICES: &'static [u16];

    fn vertices() -> &'static [Vertex] {
        Self::VERTICES
    }

    fn indices() -> &'static [u16] {
        Self::INDICES
    }
}
