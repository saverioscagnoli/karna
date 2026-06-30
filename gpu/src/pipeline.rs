use utils::FastHashMap;

use crate::GpuState;
use crate::shaders::ShaderStore;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PipelineDesc {
    pub shader: &'static str,
    pub vertex_layout: wgpu::VertexBufferLayout<'static>,
    pub blend: wgpu::BlendState,
    pub topology: wgpu::PrimitiveTopology,
}

fn build_pipeline(
    desc: &PipelineDesc,
    shaders: &ShaderStore,
    bgl: &[&wgpu::BindGroupLayout],
    surface_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let gpu = GpuState::get();
    let shader = shaders.get(&desc.shader);

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
                buffers: &[desc.vertex_layout.clone()],
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

#[derive(Debug)]
pub struct PipelineCache {
    pipelines: FastHashMap<PipelineDesc, wgpu::RenderPipeline>,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self {
            pipelines: FastHashMap::default(),
        }
    }

    pub fn create_pipeline(
        &mut self,
        desc: PipelineDesc,
        bgls: &[&wgpu::BindGroupLayout],
        surface_format: wgpu::TextureFormat,
    ) {
        let gpu = GpuState::get();
        let pipeline = build_pipeline(&desc, &gpu.shaders, bgls, surface_format);

        self.pipelines.insert(desc, pipeline);
    }

    pub fn get_pipeline(&self, desc: &PipelineDesc) -> &wgpu::RenderPipeline {
        self.pipelines.get(desc).expect("Failed to get pipeline")
    }
}
