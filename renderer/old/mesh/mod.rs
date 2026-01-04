pub mod geometry;
pub mod group;
pub mod material;
pub mod transform;

use std::ops::{Deref, DerefMut};

use crate::{
    Color, Descriptor, Layer,
    mesh::{
        geometry::Geometry,
        material::{Material, TextureKind},
        transform::Transform,
    },
};
use assets::AssetManager;
use gpu::core::GpuBuffer;
use macros::{Get, Set, With, track_dirty};
use math::{Vector2, Vector3, Vector4};
use utils::Handle;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv_coords: [f32; 2],
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
#[derive(Default, Debug, Clone, Copy)]
pub struct MeshInstanceGpu {
    position: Vector3,
    scale: Vector3,
    rotation: Vector3,
    color: Vector4,
    uv_offset: Vector2,
    uv_scale: Vector2,
}

impl Descriptor for MeshInstanceGpu {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
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

#[derive(Debug)]
pub struct GeometryBuffer {
    pub vertex_buffer: GpuBuffer<Vertex>,
    pub vertex_count: i32,
    pub index_buffer: GpuBuffer<u32>,
    pub index_count: i32,
    pub topology: wgpu::PrimitiveTopology,
}

#[track_dirty]
#[derive(Debug, Clone)]
#[derive(Get, Set, With)]
pub struct Mesh {
    #[get]
    geometry: Geometry,

    #[get(prop = "color", ty = &Color, name = "color")]
    #[get(copied, prop = "color.r", ty = f32, name = "color_r")]
    #[get(copied, prop = "color.g", ty = f32, name = "color_g")]
    #[get(copied, prop = "color.b", ty = f32, name = "color_b")]
    #[get(copied, prop = "color.a", ty = f32, name = "color_a")]
    #[get(mut, prop = "color", ty = &mut Color, name = "color_mut", also = self.tracker |= Self::material_f())]
    #[get(mut, prop = "color.r", ty = &mut f32, name = "color_r_mut", also = self.tracker |= Self::material_f())]
    #[get(mut, prop = "color.g", ty = &mut f32, name = "color_g_mut", also = self.tracker |= Self::material_f())]
    #[get(mut, prop = "color.b", ty = &mut f32, name = "color_b_mut", also = self.tracker |= Self::material_f())]
    #[get(mut, prop = "color.a", ty = &mut f32, name = "color_a_mut", also = self.tracker |= Self::material_f())]
    #[set(also = self.tracker |= Self::material_f())]
    #[set(into, prop = "color", ty = Color, name = "set_color", also = self.tracker |= Self::material_f())]
    #[set(prop = "color.r", ty = f32, name = "set_color_r", also = self.tracker |= Self::material_f())]
    #[set(prop = "color.g", ty = f32, name = "set_color_g", also = self.tracker |= Self::material_f())]
    #[set(prop = "color.b", ty = f32, name = "set_color_b", also = self.tracker |= Self::material_f())]
    #[set(prop = "color.a", ty = f32, name = "set_color_a", also = self.tracker |= Self::material_f())]
    material: Material,

    #[get]
    #[get(prop = "position", ty = &Vector3, name = "position")]
    #[get(copied, prop = "position", name = "position_2d", pre = truncate, ty = Vector2)]
    #[get(copied, prop = "position.x", ty = f32, name = "position_x")]
    #[get(copied, prop = "position.y", ty = f32, name = "position_y")]
    #[get(copied, prop = "position.z", ty = f32, name = "position_z")]
    #[get(prop = "rotation", ty = &Vector3, name = "rotation")]
    #[get(copied, prop = "rotation.z", ty = f32, name = "rotation_2d")]
    #[get(copied, prop = "rotation.x", ty = f32, name = "rotation_x")]
    #[get(copied, prop = "rotation.y", ty = f32, name = "rotation_y")]
    #[get(copied, prop = "rotation.z", ty = f32, name = "rotation_z")]
    #[get(prop = "scale", ty = &Vector3, name = "scale")]
    #[get(copied, prop = "scale", name = "scale_2d", pre = truncate, ty = Vector2)]
    #[get(copied, prop = "scale.x", ty = f32, name = "scale_x")]
    #[get(copied, prop = "scale.y", ty = f32, name = "scale_y")]
    #[get(copied, prop = "scale.z", ty = f32, name = "scale_z")]
    #[get(mut, also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "position", ty = &mut Vector3, name = "position_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "position.x", ty = &mut f32, name = "position_x_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "position.y", ty = &mut f32, name = "position_y_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "position.z", ty = &mut f32, name = "position_z_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "rotation", ty = &mut Vector3, name = "rotation_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "rotation.x", ty = &mut f32, name = "rotation_x_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "rotation.y", ty = &mut f32, name = "rotation_y_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "rotation.z", ty = &mut f32, name = "rotation_z_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "scale", ty = &mut Vector3, name = "scale_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "scale.x", ty = &mut f32, name = "scale_x_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "scale.y", ty = &mut f32, name = "scale_y_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "scale.z", ty = &mut f32, name = "scale_z_mut", also = self.tracker |= Self::transform_f())]
    #[set(also = self.tracker |= Self::transform_f())]
    #[set(into, prop = "position", ty = Vector3, name = "set_position", also = self.tracker |= Self::transform_f())]
    #[set(prop = "position.x", ty = f32, name = "set_position_x", also = self.tracker |= Self::transform_f())]
    #[set(prop = "position.y", ty = f32, name = "set_position_y", also = self.tracker |= Self::transform_f())]
    #[set(prop = "position.z", ty = f32, name = "set_position_z", also = self.tracker |= Self::transform_f())]
    #[set(into, prop = "rotation", ty = Vector3, name = "set_rotation", also = self.tracker |= Self::transform_f())]
    #[set(prop = "rotation.z", ty = f32, name = "set_rotation_2d", also = self.tracker |= Self::transform_f())]
    #[set(prop = "rotation.x", ty = f32, name = "set_rotation_x", also = self.tracker |= Self::transform_f())]
    #[set(prop = "rotation.y", ty = f32, name = "set_rotation_y", also = self.tracker |= Self::transform_f())]
    #[set(prop = "rotation.z", ty = f32, name = "set_rotation_z", also = self.tracker |= Self::transform_f())]
    #[set(into, prop = "scale", ty = Vector3, name = "set_scale", also = self.tracker |= Self::transform_f())]
    #[set(prop = "scale.x", ty = f32, name = "set_scale_x", also = self.tracker |= Self::transform_f())]
    #[set(prop = "scale.y", ty = f32, name = "set_scale_y", also = self.tracker |= Self::transform_f())]
    #[set(prop = "scale.z", ty = f32, name = "set_scale_z", also = self.tracker |= Self::transform_f())]
    transform: Transform,

    #[get(copied)]
    #[get(mut, also = self.tracker |= Self::visible_f())]
    #[set(also = self.tracker |= Self::visible_f())]
    visible: bool,

    gpu: MeshInstanceGpu,
}

impl Mesh {
    pub(crate) const INITIAL_INSTANCE_CAPACITY: usize = 128;

    #[inline]
    pub fn new(geometry: Geometry, material: Material, transform: Transform) -> Self {
        let mut mesh = Self {
            geometry,
            transform,
            material,
            visible: true,
            gpu: MeshInstanceGpu::default(),
            tracker: 0,
        };

        mesh.set_all_dirty();
        mesh
    }

    #[inline]
    pub(crate) fn sync_gpu(&mut self, assets: &AssetManager) -> bool {
        let mut changed = false;

        if self.is_dirty(Self::transform_f()) {
            self.gpu.position = self.transform.position;
            self.gpu.rotation = self.transform.rotation;
            self.gpu.scale = self.transform.scale;

            changed = true
        }

        if self.is_dirty(Self::material_f()) {
            self.gpu.color = self.material.color.into();

            let (x, y, w, h) = if let Some(kind) = self.material.texture {
                match kind {
                    TextureKind::Full(label) => assets.get_texture_coords(label),
                    TextureKind::Partial(label, x, y, w, h) => {
                        assets.get_subtexture_coords(label, x, y, w, h)
                    }
                }
            } else {
                assets.get_white_uv_coords()
            };

            self.gpu.uv_offset.x = x;
            self.gpu.uv_offset.y = y;
            self.gpu.uv_scale.x = w;
            self.gpu.uv_scale.y = h;

            changed = true
        }

        if changed {
            self.clear_all_dirty();
        }

        changed
    }

    #[inline]
    pub(crate) fn gpu(&self) -> MeshInstanceGpu {
        self.gpu
    }

    #[inline]
    pub fn set_position_2d<P: Into<Vector2>>(&mut self, pos: P) {
        let pos = pos.into();

        self.transform.position.x = pos.x;
        self.transform.position.y = pos.y;
        self.tracker |= Self::transform_f();
    }

    #[inline]
    pub fn set_scale_2d<S: Into<Vector2>>(&mut self, scale: S) {
        let scale = scale.into();

        self.transform.scale.x = scale.x;
        self.transform.scale.y = scale.y;
        self.tracker |= Self::transform_f();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Get)]
pub struct MeshHandle {
    #[get]
    pub(crate) layer: Layer,
    pub(crate) handle: Handle<Mesh>,
}

impl Deref for MeshHandle {
    type Target = Handle<Mesh>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl DerefMut for MeshHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

impl MeshHandle {
    pub fn dummy() -> Self {
        Self {
            layer: Layer::World,
            handle: Handle::dummy(),
        }
    }
}
