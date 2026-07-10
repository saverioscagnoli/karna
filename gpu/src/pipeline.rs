use std::sync::Arc;

use parking_lot::RwLock;
use utils::FastHashMap;

use crate::GpuState;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PipelineDesc {
    pub shader: &'static str,
    pub vertex_layout: wgpu::VertexBufferLayout<'static>,
    pub blend: wgpu::BlendState,
    pub topology: wgpu::PrimitiveTopology,
}

fn build_pipeline(
    gpu: &GpuState,
    desc: &PipelineDesc,
    bgl: &[&wgpu::BindGroupLayout],
    surface_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = gpu.shaders.get(&desc.shader);

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
                buffers: &[Some(desc.vertex_layout.clone())],
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
                cull_mode: None, // no culling for 2d
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // add later if you need depth testing
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
pub struct PipelineKey {
    shader: &'static str,
    vertex_stride: u64,
    vertex_step_mode: wgpu::VertexStepMode,
    vertex_attributes: Vec<wgpu::VertexAttribute>,
    blend: wgpu::BlendState,
    topology: wgpu::PrimitiveTopology,
    format: wgpu::TextureFormat,
}

impl PipelineKey {
    pub fn new(desc: &PipelineDesc, format: wgpu::TextureFormat) -> Self {
        Self {
            shader: desc.shader,
            vertex_stride: desc.vertex_layout.array_stride,
            vertex_step_mode: desc.vertex_layout.step_mode,
            vertex_attributes: desc.vertex_layout.attributes.to_vec(),
            blend: desc.blend,
            topology: desc.topology,
            format,
        }
    }
}

#[derive(Debug)]
pub struct PipelineCache {
    pipelines: RwLock<FastHashMap<PipelineKey, Arc<wgpu::RenderPipeline>>>,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self {
            pipelines: RwLock::new(FastHashMap::default()),
        }
    }

    pub fn get_or_create(
        &self,
        desc: PipelineDesc,
        format: wgpu::TextureFormat,
        layouts: &[&wgpu::BindGroupLayout],
    ) -> Arc<wgpu::RenderPipeline> {
        let key = PipelineKey::new(&desc, format);

        if let Some(p) = self.pipelines.read().get(&key) {
            return p.clone();
        }

        let gpu = GpuState::get();
        let pipeline = Arc::new(build_pipeline(gpu, &desc, layouts, format));

        let mut map = self.pipelines.write();
        match map.get(&key) {
            Some(p) => p.clone(),
            None => {
                map.insert(key, pipeline.clone());
                pipeline
            }
        }
    }
}
