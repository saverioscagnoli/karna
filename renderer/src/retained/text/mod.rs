mod batch;
mod renderer;

use crate::{Transform3d, color::Color, traits::LayoutDescriptor};
use assets::{AssetServer, AssetServerGuard, Font};
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use macros::{Get, Set, With, track_dirty};
use math::{Vector2, Vector3, Vector4};
use std::mem::{self, offset_of};
use utils::Handle;

pub use renderer::TextRenderer;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GlyphGpu {
    pub position: Vector3,
    pub rotation: Vector3,
    pub offset: Vector2,
    pub size: Vector2,
    pub scale: Vector2,
    pub uv_offset: Vector2,
    pub uv_scale: Vector2,
    pub color: Vector4,
}

impl LayoutDescriptor for GlyphGpu {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: offset_of!(Self, position) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3, // position
                },
                wgpu::VertexAttribute {
                    offset: offset_of!(Self, rotation) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3, // rotation
                },
                wgpu::VertexAttribute {
                    offset: offset_of!(Self, offset) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x2, // offset
                },
                wgpu::VertexAttribute {
                    offset: offset_of!(Self, size) as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x2, // size
                },
                wgpu::VertexAttribute {
                    offset: offset_of!(Self, scale) as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x2, // scale
                },
                wgpu::VertexAttribute {
                    offset: offset_of!(Self, uv_offset) as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x2, // uv_offset
                },
                wgpu::VertexAttribute {
                    offset: offset_of!(Self, uv_scale) as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x2, // uv_scale
                },
                wgpu::VertexAttribute {
                    offset: offset_of!(Self, color) as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x4, // color
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    pub local_position: Vector2, // Position relative to text origin
    pub size: Vector2,
    pub uv_offset: Vector2,
    pub uv_scale: Vector2,
}

#[track_dirty]
#[derive(Get, Set, With)]
pub struct Text {
    #[get]
    #[get(mut, also = self.tracker |= Self::content_f())]
    #[set(also = self.tracker |= Self::content_f())]
    #[with(into)]
    content: String,

    #[get(copied)]
    #[get(mut, also = self.tracker |= Self::content_f())]
    #[set(also = self.tracker |= Self::content_f())]
    font: Handle<Font>,

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

    #[get]
    #[get(copied, prop = "r", ty = f32)]
    #[get(copied, prop = "g", ty = f32)]
    #[get(copied, prop = "b", ty = f32)]
    #[get(copied, prop = "a", ty = f32)]
    #[get(mut, also = self.tracker |= Self::color_f())]
    #[get(mut, prop = "r", ty = &mut f32, also = self.tracker |= Self::color_f())]
    #[get(mut, prop = "g", ty = &mut f32, also = self.tracker |= Self::color_f())]
    #[get(mut, prop = "b", ty = &mut f32, also = self.tracker |= Self::color_f())]
    #[get(mut, prop = "a", ty = &mut f32, also = self.tracker |= Self::color_f())]
    #[set(into, also = self.tracker |= Self::color_f())]
    #[set(prop = "r", ty = f32, also = self.tracker |= Self::color_f())]
    #[set(prop = "g", ty = f32, also = self.tracker |= Self::color_f())]
    #[set(prop = "b", ty = f32, also = self.tracker |= Self::color_f())]
    #[set(prop = "a", ty = f32, also = self.tracker |= Self::color_f())]
    #[with(into)]
    color: Color,

    layout: Layout,
    glyphs: Vec<Glyph>,
    pub(crate) gpu_glyphs: Vec<GlyphGpu>,
}

impl Text {
    pub fn new(font: Handle<Font>) -> Self {
        let mut text = Self {
            content: String::new(),
            font,
            transform: Transform3d::default(),
            color: Color::White,
            glyphs: Vec::new(),
            gpu_glyphs: Vec::new(),
            tracker: 0,
            layout: Layout::new(CoordinateSystem::PositiveYDown),
        };

        text.set_all_dirty();
        text
    }

    #[inline]
    pub fn glyph_count(&self) -> usize {
        self.gpu_glyphs.len()
    }

    #[inline]
    pub(crate) fn prepare(&mut self, assets: &AssetServerGuard<'_>) -> bool {
        let content_changed = self.is_dirty(Self::content_f());
        let transform_changed = self.is_dirty(Self::transform_f());
        let color_changed = self.is_dirty(Self::color_f());

        if !content_changed && !transform_changed && !color_changed {
            return false;
        }

        if content_changed {
            self.layout_glyphs(assets);
        }

        if content_changed || transform_changed || color_changed {
            self.update_gpu_data();
        }

        self.clear_all_dirty();

        true
    }

    #[inline]
    fn layout_glyphs(&mut self, assets: &AssetServerGuard<'_>) {
        let font = assets.get_font(self.font);

        self.layout.append(
            &[font.inner()],
            &TextStyle::new(&self.content, font.size() as f32, 0),
        );

        self.glyphs.clear();

        for glyph in self.layout.glyphs() {
            if glyph.width == 0 || glyph.height == 0 {
                continue;
            }

            let (uv_x, uv_y, uv_w, uv_h, _, _) = assets.get_glyph_uv(self.font, glyph.parent);

            self.glyphs.push(Glyph {
                local_position: Vector2::new(glyph.x, glyph.y),
                size: Vector2::new(glyph.width as f32, glyph.height as f32),
                uv_offset: Vector2::new(uv_x, uv_y),
                uv_scale: Vector2::new(uv_w, uv_h),
            });
        }

        self.gpu_glyphs
            .resize(self.glyphs.len(), GlyphGpu::default());
    }

    fn update_gpu_data(&mut self) {
        let color: Vector4 = self.color.into();
        let pos = &self.transform.position;
        let rot = &self.transform.rotation;
        let scale = Vector2::new(self.transform.scale.x, self.transform.scale.y);

        for (i, glyph) in self.glyphs.iter().enumerate() {
            let scaled_offset = Vector2::new(
                glyph.local_position.x * scale.x,
                glyph.local_position.y * scale.y,
            );

            self.gpu_glyphs[i] = GlyphGpu {
                position: *pos, // Text pivot point
                rotation: *rot,
                offset: scaled_offset, // Glyph's offset from pivot
                size: glyph.size,
                scale,
                uv_offset: glyph.uv_offset,
                uv_scale: glyph.uv_scale,
                color,
            };
        }
    }
}
