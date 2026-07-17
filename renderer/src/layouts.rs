use std::mem;
use std::sync::OnceLock;

use gpu::GpuState;

use crate::MeshInstanceData;
use crate::Vertex;
use crate::vertex::ShapeVertex;

pub trait LayoutDesc: Sized {
    const STEP_MODE: wgpu::VertexStepMode;
    const ATTRIBUTES: &'static [wgpu::VertexAttribute];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: Self::STEP_MODE,
            attributes: Self::ATTRIBUTES,
        }
    }
}

impl LayoutDesc for Vertex {
    const STEP_MODE: wgpu::VertexStepMode = wgpu::VertexStepMode::Vertex;
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x4,
        2 => Float32x2
    ];
}

impl LayoutDesc for MeshInstanceData {
    const STEP_MODE: wgpu::VertexStepMode = wgpu::VertexStepMode::Instance;
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x4
    ];
}

impl LayoutDesc for ShapeVertex {
    const STEP_MODE: wgpu::VertexStepMode = wgpu::VertexStepMode::Vertex;
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x4,
        2 => Float32x2,
        3 => Float32x2,
        4 => Float32x4,
        5 => Float32x4,
        6 => Uint32,
    ];
}

#[derive(Debug)]
pub struct Layouts {
    pub texture_atlas: wgpu::BindGroupLayout,
    pub camera: wgpu::BindGroupLayout,
    pub material: wgpu::BindGroupLayout,
}

impl Layouts {
    pub fn as_array(&self) -> [&wgpu::BindGroupLayout; 3] {
        [&self.texture_atlas, &self.camera, &self.material]
    }
}

static LAYOUTS: OnceLock<Layouts> = OnceLock::new();

pub fn init() {
    let gpu = GpuState::get();

    let texture_atlas = gpu
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture atlas bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

    let camera = gpu
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let material = gpu
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    LAYOUTS
        .set(Layouts {
            texture_atlas,
            camera,
            material,
        })
        .expect("Layouts already initialized")
}

fn all() -> &'static Layouts {
    &LAYOUTS.get().expect("Layouts not initialized")
}

pub fn texture_atlas() -> &'static wgpu::BindGroupLayout {
    &LAYOUTS
        .get()
        .expect("Layouts not initialized")
        .texture_atlas
}

pub fn camera() -> &'static wgpu::BindGroupLayout {
    &LAYOUTS.get().expect("Layouts not inizialized").camera
}

pub fn material() -> &'static wgpu::BindGroupLayout {
    &LAYOUTS.get().expect("Layouts not inizialized").material
}

pub fn immediate() -> [&'static wgpu::BindGroupLayout; 2] {
    let l = all();
    [&l.camera, &l.texture_atlas]
}

pub fn retained() -> [&'static wgpu::BindGroupLayout; 3] {
    let l = all();
    [&l.camera, &l.texture_atlas, &l.material]
}
