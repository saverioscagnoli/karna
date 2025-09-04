use crate::{Color, Descriptor, Renderer};
use math::Vec2;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    pub position: Vec2,
    pub color: Color,
}

impl Descriptor for Pixel {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Pixel>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
impl Pixel {
    pub fn new<P: Into<Vec2>>(pos: P) -> Self {
        Self {
            position: pos.into(),
            color: Color::default(),
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn render(&self, renderer: &mut Renderer) {
        renderer.pixel_renderer.push_data(*self);
    }
}
