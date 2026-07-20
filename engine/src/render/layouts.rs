use std::mem;
use std::sync::OnceLock;

use gpu::LayoutBuilder;

pub trait LayoutDesc: Sized {
    const STEP_MODE: gpu::VertexStepMode;
    const ATTRIBUTES: &'static [gpu::VertexAttribute];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: Self::STEP_MODE,
            attributes: Self::ATTRIBUTES,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Layouts {
    atlas: gpu::BindGroupLayout,
    camera: gpu::BindGroupLayout,
}

static LAYOUTS: OnceLock<Layouts> = OnceLock::new();

pub fn init() {
    let atlas = LayoutBuilder::new("Texture Atlas")
        .texture()
        .sampler()
        .build();

    let camera = LayoutBuilder::new("Camera")
        .visibility(gpu::ShaderStages::VERTEX)
        .uniform()
        .build();

    LAYOUTS
        .set(Layouts { atlas, camera })
        .expect("Layout singleton already set")
}

pub fn atlas() -> &'static gpu::BindGroupLayout {
    &LAYOUTS.get().unwrap().atlas
}

pub fn camera() -> &'static gpu::BindGroupLayout {
    &LAYOUTS.get().unwrap().camera
}
