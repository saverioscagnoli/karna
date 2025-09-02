use crate::{Color, Descriptor, Renderer};
use math::{Size, Vec2};
use std::ops::{Deref, DerefMut};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub position: Vec2,
    pub size: Size<f32>,
    pub color: Color,
}

impl Deref for Rect {
    type Target = Size<f32>;

    fn deref(&self) -> &Self::Target {
        &self.size
    }
}

impl DerefMut for Rect {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.size
    }
}

impl Descriptor for Rect {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Rect>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vec2>() * 2) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Rect {
    pub fn new<P: Into<Vec2>, S: Into<Size<f32>>>(pos: P, size: S) -> Self {
        Self {
            position: pos.into(),
            size: size.into(),
            color: Color::default(),
        }
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
        renderer.quad_renderer.push_data(*self);
    }
}
