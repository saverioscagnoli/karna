use std::borrow::Cow;

/// A wrapper around a WGPU shader module for easy shader management
#[derive(Debug)]
pub struct Shader {
    module: wgpu::ShaderModule,
}

impl Shader {
    pub fn from_wgsl(source: &str, label: Option<&str>) -> Self {
        let module = gpu::device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
        });

        Self { module }
    }

    pub fn from_wgsl_file(source: &'static str, label: Option<&str>) -> Self {
        Self::from_wgsl(source, label)
    }

    pub fn pipeline_builder(&self) -> PipelineBuilder<'_> {
        PipelineBuilder::new(&self.module)
    }
}

pub struct PipelineBuilder<'a> {
    module: &'a wgpu::ShaderModule,
    vertex_entry: &'static str,
    fragment_entry: &'static str,
    label: Option<&'static str>,
    cull_mode: Option<wgpu::Face>,
    topology: wgpu::PrimitiveTopology,
    blend_state: Option<wgpu::BlendState>,
    polygon_mode: wgpu::PolygonMode,
}

impl<'a> PipelineBuilder<'a> {
    fn new(module: &'a wgpu::ShaderModule) -> Self {
        Self {
            module,
            vertex_entry: "vs_main",
            fragment_entry: "fs_main",
            label: None,
            cull_mode: None,
            topology: wgpu::PrimitiveTopology::TriangleList,
            blend_state: Some(wgpu::BlendState::ALPHA_BLENDING),
            polygon_mode: wgpu::PolygonMode::Fill,
        }
    }

    pub fn vertex_entry(mut self, entry: &'static str) -> Self {
        self.vertex_entry = entry;
        self
    }

    pub fn fragment_entry(mut self, entry: &'static str) -> Self {
        self.fragment_entry = entry;
        self
    }

    pub fn label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn cull_mode(mut self, mode: wgpu::Face) -> Self {
        self.cull_mode = Some(mode);
        self
    }

    pub fn topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }

    pub fn blend_state(mut self, blend_state: Option<wgpu::BlendState>) -> Self {
        self.blend_state = blend_state;
        self
    }

    pub fn polygon_mode(mut self, mode: wgpu::PolygonMode) -> Self {
        self.polygon_mode = mode;
        self
    }

    pub fn build(
        self,
        format: wgpu::TextureFormat,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        vertex_buffers: &[wgpu::VertexBufferLayout],
    ) -> wgpu::RenderPipeline {
        let device = gpu::device();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: self.label,
            bind_group_layouts,
            immediate_size: 0,
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: self.label,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: self.module,
                entry_point: Some(self.vertex_entry),
                buffers: vertex_buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: self.module,
                entry_point: Some(self.fragment_entry),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: self.blend_state,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: self.topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: self.cull_mode,
                polygon_mode: self.polygon_mode,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }
}
