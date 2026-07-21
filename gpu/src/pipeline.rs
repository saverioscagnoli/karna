use logging::debug;
use utils::FastHashMap;

use crate::BlendState;
use crate::Cull;
use crate::GpuState;
use crate::PrimitiveTopology;
use crate::RenderPipeline;
use crate::VertexBufferLayout;
use crate::shaders::ShaderRef;

fn build_pipeline(
    gpu: &GpuState,
    desc: &PipelineDesc,
    bgl: &[&wgpu::BindGroupLayout],
    surface_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = gpu.shaders.get(&desc.shader);
    let buffers = vec![Some(desc.vertex_layout.clone())];

    // pipeline layout — empty for now, add bind group layouts later (camera uniform, textures)
    let pipeline_layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &bgl.iter().map(|b| Some(*b)).collect::<Vec<_>>(),
            ..Default::default()
        });

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(desc.blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: desc.topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: desc.cull,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            cache: None,
            multiview_mask: None,
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineDesc {
    pub shader: ShaderRef,
    pub vertex_layout: VertexBufferLayout,
    pub blend: BlendState,
    pub topology: PrimitiveTopology,
    pub cull: Option<Cull>,
    pub format: wgpu::TextureFormat,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    pub shader: ShaderRef,
    pub vertex_stride: u64,
    pub vertex_step_mode: wgpu::VertexStepMode,
    pub vertex_attributes: Vec<wgpu::VertexAttribute>,
    pub blend: BlendState,
    pub topology: PrimitiveTopology,
    pub format: wgpu::TextureFormat,
}

impl PipelineKey {
    pub fn new(desc: &PipelineDesc) -> Self {
        Self {
            shader: desc.shader,
            vertex_stride: desc.vertex_layout.array_stride,
            vertex_step_mode: desc.vertex_layout.step_mode,
            vertex_attributes: desc.vertex_layout.attributes.to_vec(),
            blend: desc.blend,
            topology: desc.topology,
            format: desc.format,
        }
    }
}

#[derive(Default)]
pub struct PipelineCache {
    pip: FastHashMap<PipelineKey, wgpu::RenderPipeline>,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, desc: &PipelineDesc, layouts: &[&wgpu::BindGroupLayout]) {
        let gpu = GpuState::get();
        let key = PipelineKey::new(&desc);
        let pip = build_pipeline(gpu, desc, layouts, desc.format);

        debug!("Created new render pipeline {:?}", desc);

        self.pip.insert(key, pip);
    }

    pub fn get(&self, desc: &PipelineDesc) -> &RenderPipeline {
        let key = PipelineKey::new(&desc);
        self.pip.get(&key).expect("Failed to get render pipeline")
    }
}
