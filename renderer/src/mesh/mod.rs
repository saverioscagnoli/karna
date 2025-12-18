mod geometry;
mod material;
mod transform;

use crate::Color;
use assets::AssetManager;
use macros::{Get, Set};
use math::{Vector2, Vector3, Vector4};
use std::{cell::Cell, sync::Arc};

// Re-exports
pub use crate::mesh::{geometry::Geometry, material::Material, transform::Transform};

pub trait Descriptor {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vector3,
    pub color: Vector4,
    pub uv_coords: Vector2,
}

impl Descriptor for Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position attribute at location 0
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color attribute at location 1
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vector3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() + std::mem::size_of::<Vector4>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuMesh {
    position: Vector3,
    scale: Vector3,
    rotation: Vector3,
    color: Vector4,
    uv_offset: Vector2,
    uv_scale: Vector2,
}

impl Descriptor for GpuMesh {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuMesh>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Position attribute at location 3
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Scale attribute at location 4
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vector3>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Rotation attribute at location 5
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() * 2) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color attribute at location 6
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() * 3) as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // UV offset attribute at location 7
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() * 3 + std::mem::size_of::<Vector4>())
                        as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // UV scale attribute at location 8
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() * 3
                        + std::mem::size_of::<Vector4>()
                        + std::mem::size_of::<Vector2>())
                        as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub instance_buffer: wgpu::Buffer,
    pub instances: Vec<GpuMesh>,
    pub topology: wgpu::PrimitiveTopology,
    pub dirty_indices: Vec<usize>,
    pub instance_count: usize,
}

#[derive(Debug, Clone)]
#[derive(Get, Set)]
pub struct Mesh {
    #[get]
    geometry: Arc<Geometry>,

    #[get]
    #[get(prop = "color", ty = &Color, name = "color")]
    #[get(copied, prop = "color.r", ty = f32, name = "color_r")]
    #[get(copied, prop = "color.g", ty = f32, name = "color_g")]
    #[get(copied, prop = "color.b", ty = f32, name = "color_b")]
    #[get(copied, prop = "color.a", ty = f32, name = "color_a")]
    #[get(mut, also = self.mark())]
    #[get(mut, prop = "color", ty = &mut Color, name = "color_mut", also = self.mark())]
    #[get(mut, prop = "color.r", ty = &mut f32, name = "color_r_mut", also = self.mark())]
    #[get(mut, prop = "color.g", ty = &mut f32, name = "color_g_mut", also = self.mark())]
    #[get(mut, prop = "color.b", ty = &mut f32, name = "color_b_mut", also = self.mark())]
    #[get(mut, prop = "color.a", ty = &mut f32, name = "color_a_mut", also = self.mark())]
    #[set(also = self.mark())]
    #[set(into, prop = "color", ty = Color, name = "set_color", also = self.mark())]
    #[set(prop = "color.r", ty = f32, name = "set_color_r", also = self.mark())]
    #[set(prop = "color.g", ty = f32, name = "set_color_g", also = self.mark())]
    #[set(prop = "color.b", ty = f32, name = "set_color_b", also = self.mark())]
    #[set(prop = "color.a", ty = f32, name = "set_color_a", also = self.mark())]
    material: Material,

    #[get]
    #[get(prop = "position", ty = &Vector2, name = "position")]
    #[get(copied, prop = "position.x", ty = f32, name = "position_x")]
    #[get(copied, prop = "position.y", ty = f32, name = "position_y")]
    #[get(prop = "scale", ty = &Vector2, name = "scale")]
    #[get(copied, prop = "scale.x", ty = f32, name = "scale_x")]
    #[get(copied, prop = "scale.y", ty = f32, name = "scale_y")]
    #[get(copied, prop = "rotation", ty = f32, name = "rotation")]
    #[get(mut, also = self.mark())]
    #[get(mut, prop = "position", ty = &mut Vector2, name = "position_mut", also = self.mark())]
    #[get(mut, prop = "position.x", ty = &mut f32, name = "position_x_mut", also = self.mark())]
    #[get(mut, prop = "position.y", ty = &mut f32, name = "position_y_mut", also = self.mark())]
    #[get(mut, prop = "scale", ty = &mut Vector2, name = "scale_mut", also = self.mark())]
    #[get(mut, prop = "scale.x", ty = &mut f32, name = "scale_x_mut", also = self.mark())]
    #[get(mut, prop = "scale.y", ty = &mut f32, name = "scale_y_mut", also = self.mark())]
    #[get(mut, prop = "rotation", ty = &mut f32, name = "rotation_mut", also = self.mark())]
    #[set(into, also = self.mark())]
    #[set(into, prop = "position", ty = Vector2, name = "set_position", also = self.mark())]
    #[set(prop = "position.x", ty = f32, name = "set_position_x", also = self.mark())]
    #[set(prop = "position.y", ty = f32, name = "set_position_y", also = self.mark())]
    #[set(into, prop = "scale", ty = Vector2, name = "set_scale", also = self.mark())]
    #[set(prop = "scale.x", ty = f32, name = "set_scale_x", also = self.mark())]
    #[set(prop = "scale.y", ty = f32, name = "set_scale_y", also = self.mark())]
    #[set(prop = "rotation", ty = f32, name = "set_rotation", also = self.mark())]
    transform: Transform,

    #[get(copied)]
    #[set]
    visible: bool,

    dirty: Cell<bool>,
    instance_index: Cell<Option<usize>>,
}

impl Mesh {
    pub(crate) const INITIAL_INSTANCE_CAPACITY: usize = 1024;

    #[inline]
    pub fn new(geometry: Arc<Geometry>, material: Material, transform: Transform) -> Self {
        Self {
            geometry,
            material,
            transform,
            visible: true,
            dirty: Cell::new(true),
            instance_index: Cell::new(None),
        }
    }

    #[inline]
    pub(crate) fn mark(&self) {
        self.dirty.set(true);
    }

    #[inline]
    pub(crate) fn clean(&self) {
        self.dirty.set(false);
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.get()
    }

    #[inline]
    pub(crate) fn instance_index(&self) -> Option<usize> {
        self.instance_index.get()
    }

    #[inline]
    pub(crate) fn set_instance_index(&self, index: usize) {
        self.instance_index.set(Some(index));
    }

    #[inline]
    pub fn update_position<F: Fn(&mut Vector2)>(&mut self, f: F) {
        self.mark();
        f(&mut self.transform.position);
    }

    #[inline]
    pub fn update_scale<F: Fn(&mut Vector2)>(&mut self, f: F) {
        self.mark();
        f(&mut self.transform.position);
    }

    #[inline]
    pub(crate) fn for_gpu(&self, assets: &AssetManager) -> GpuMesh {
        // Get UV coordinates from the texture atlas if a texture is specified
        let (uv_offset, uv_scale) = if let Some(texture_label) = self.material.texture {
            let coords = assets.get_texture_coords(texture_label);
            (
                Vector2::new(coords.0, coords.1),
                Vector2::new(coords.2, coords.3),
            )
        } else {
            let coords = assets.get_white_texture_coords();
            (
                Vector2::new(coords.0, coords.1),
                Vector2::new(coords.2, coords.3),
            )
        };

        GpuMesh {
            position: self.position().extend(0.0),
            scale: self.scale().extend(0.0),
            rotation: Vector3::new(0.0, 0.0, self.rotation()),
            color: self.material.color.into(),
            uv_offset,
            uv_scale,
        }
    }
}
