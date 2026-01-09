mod batch;
mod geometry;
mod material;
mod transform;

use crate::{color::Color, traits::LayoutDescriptor};
use assets::AssetServerGuard;
use macros::{Get, Set, track_dirty};
use math::{Vector2, Vector3, Vector4};
use std::mem;

pub use batch::*;
pub use geometry::*;
pub use material::*;
pub use transform::*;

#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct MeshGpu {
    position: Vector3,
    scale: Vector3,
    rotation: Vector3,
    color: Vector4,
    uv_offset: Vector2,
    uv_scale: Vector2,
}

impl LayoutDescriptor for MeshGpu {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vector3>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<Vector3>() * 2) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<Vector3>() * 3) as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<Vector3>() * 3 + std::mem::size_of::<Vector4>())
                        as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<Vector3>() * 3
                        + mem::size_of::<Vector4>()
                        + mem::size_of::<Vector2>())
                        as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[track_dirty]
#[derive(Debug, Clone)]
#[derive(Get, Set)]
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
    transform: Transform3d,

    pub(crate) gpu: MeshGpu,
}

impl Mesh {
    pub fn new(geometry: Geometry, material: Material, transform: Transform3d) -> Self {
        let mut mesh = Self {
            geometry,
            material,
            transform,
            gpu: MeshGpu::default(),
            tracker: 0,
        };

        mesh.set_all_dirty();
        mesh
    }

    #[inline]
    pub(crate) fn prepare(&mut self, assets: &AssetServerGuard<'_>) -> bool {
        let mut changed = false;

        if self.is_dirty(Self::transform_f()) {
            self.gpu.position = self.transform.position;
            self.gpu.rotation = self.transform.rotation;
            self.gpu.scale = self.transform.scale;

            changed = true;
        }

        if self.is_dirty(Self::material_f()) {
            self.gpu.color = self.material.color.into();

            let (uvx, uvy, uvw, uvh, _, _) = match self.material.texture {
                TextureKind::Full(handle) => assets.get_texture_uv(handle),
                TextureKind::None => assets.get_white_uv_coords(),
            };

            self.gpu.uv_offset.x = uvx;
            self.gpu.uv_offset.y = uvy;
            self.gpu.uv_scale.x = uvw;
            self.gpu.uv_scale.y = uvh;

            changed = true;
        }

        if changed {
            self.clear_all_dirty();
        }

        changed
    }
}
