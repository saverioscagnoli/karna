use nostd::collections::Handle;
use sdl3::gpu::Blend;
use sdl3::gpu::CullMode;
use sdl3::render::Color;

use crate::assets::Image;
use crate::assets::ImageRegistry;

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shading {
    Unlit,
    #[default]
    Lit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialKey {
    pub transparent: bool,
    pub shading: Shading,
    pub blend: Blend,
    pub cull: CullMode,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MaterialUniform {
    pub color: math::Vector4<f32>,
    pub uv_offset: math::Vector2<f32>,
    pub uv_scale: math::Vector2<f32>,
    pub page: f32,
    pub lit: f32,
    _pad: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: Color,
    pub texture: Option<Handle<Image>>,
    pub shading: Shading,
    pub blend: Blend,
    pub cull: CullMode,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            texture: None,
            shading: Shading::Lit,
            blend: Blend::Replace,
            cull: CullMode::Back,
        }
    }
}

impl Material {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn unlit() -> Self {
        Self::default().with_shading(Shading::Unlit)
    }

    pub fn with_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_texture(mut self, texture: Handle<Image>) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn with_shading(mut self, shading: Shading) -> Self {
        self.shading = shading;
        self
    }

    pub fn with_blend(mut self, blend: Blend) -> Self {
        self.blend = blend;
        self
    }

    pub fn with_cull(mut self, cull: CullMode) -> Self {
        self.cull = cull;
        self
    }

    pub fn is_transparent(&self) -> bool {
        self.blend != Blend::Replace
    }

    pub fn key(&self) -> MaterialKey {
        MaterialKey {
            transparent: self.is_transparent(),
            shading: self.shading,
            blend: self.blend,
            cull: self.cull,
        }
    }

    pub fn uniform(&self, images: &ImageRegistry) -> MaterialUniform {
        let image = self
            .texture
            .and_then(|t| images.resolve(t))
            .or_else(|| images.resolve(images.white_texel))
            .expect("white texel must be baked before resolving materials");

        MaterialUniform {
            color: self.color.into(),
            uv_offset: image.uv_min(),
            uv_scale: image.uv_max() - image.uv_min(),
            page: image.page() as f32,
            lit: if self.shading == Shading::Lit {
                1.0
            } else {
                0.0
            },
            _pad: [0.0; 2],
        }
    }
}
