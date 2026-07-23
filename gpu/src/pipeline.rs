use logging::debug;
use sdl3::gpu::BlendFactor;
use sdl3::gpu::BlendOp;
use sdl3::gpu::ColorTargetBlendState;
use sdl3::gpu::ColorTargetDescription;
use sdl3::gpu::CullMode;
use sdl3::gpu::FillMode;
use sdl3::gpu::FrontFace;
use sdl3::gpu::GraphicsPipelineTargetInfo;
use sdl3::gpu::RasterizerState;
use sdl3::gpu::VertexBufferDescription;
use sdl3::gpu::VertexInputRate;
use sdl3::gpu::VertexInputState;
use utils::FastHashMap;

use crate::Gpu;
use crate::PrimitiveTopology;
use crate::TextureFormat;
use crate::VertexLayout;
use crate::shaders::ShaderRef;

pub use sdl3::gpu::GraphicsPipeline as RenderPipeline;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendState {
    None,
    Alpha,
}

impl BlendState {
    pub const ALPHA_BLENDING: Self = Self::Alpha;

    fn apply(&self, target: ColorTargetDescription) -> ColorTargetDescription {
        match self {
            Self::None => target,
            Self::Alpha => target.with_blend_state(
                ColorTargetBlendState::new()
                    .with_enable_blend(true)
                    .with_color_blend_op(BlendOp::Add)
                    .with_src_color_blendfactor(BlendFactor::SrcAlpha)
                    .with_dst_color_blendfactor(BlendFactor::OneMinusSrcAlpha)
                    .with_alpha_blend_op(BlendOp::Add)
                    .with_src_alpha_blendfactor(BlendFactor::One)
                    .with_dst_alpha_blendfactor(BlendFactor::OneMinusSrcAlpha),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cull {
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepthState {
    Disabled,
    ReadWrite,
}

fn build_pipeline(gpu: &Gpu, desc: &PipelineDesc) -> RenderPipeline {
    let shader = gpu.shaders.get(&desc.shader);

    let buffer_descriptions = [VertexBufferDescription::new()
        .with_slot(0)
        .with_pitch(desc.vertex_layout.pitch)
        .with_input_rate(VertexInputRate::Vertex)
        .with_instance_step_rate(0)];

    let attributes: Vec<_> = desc
        .vertex_layout
        .attributes
        .iter()
        .map(|a| {
            sdl3::gpu::VertexAttribute::new()
                .with_buffer_slot(0)
                .with_location(a.location)
                .with_format(a.format)
                .with_offset(a.offset)
        })
        .collect();

    let color_targets =
        [desc
            .blend
            .apply(ColorTargetDescription::new().with_format(desc.format))];

    gpu.device()
        .create_graphics_pipeline()
        .with_vertex_shader(&shader.vertex)
        .with_fragment_shader(&shader.fragment)
        .with_primitive_type(desc.topology)
        .with_vertex_input_state(
            VertexInputState::new()
                .with_vertex_buffer_descriptions(&buffer_descriptions)
                .with_vertex_attributes(&attributes),
        )
        .with_rasterizer_state(
            RasterizerState::new()
                .with_fill_mode(FillMode::Fill)
                .with_front_face(FrontFace::CounterClockwise)
                .with_cull_mode(match desc.cull {
                    Some(Cull::Front) => CullMode::Front,
                    Some(Cull::Back) => CullMode::Back,
                    None => CullMode::None,
                }),
        )
        .with_depth_stencil_state(match desc.depth {
            DepthState::Disabled => sdl3::gpu::DepthStencilState::new(),
            DepthState::ReadWrite => sdl3::gpu::DepthStencilState::new()
                .with_enable_depth_test(true)
                .with_enable_depth_write(true)
                .with_compare_op(sdl3::gpu::CompareOp::LessOrEqual),
        })
        .with_target_info(
            GraphicsPipelineTargetInfo::new()
                .with_color_target_descriptions(&color_targets)
                .with_has_depth_stencil_target(true)
                .with_depth_stencil_format(crate::texture::DepthTexture::FORMAT),
        )
        .build()
        .expect("Failed to create render pipeline")
}

#[derive(Debug, Clone)]
pub struct PipelineDesc {
    pub shader: ShaderRef,
    pub vertex_layout: VertexLayout,
    pub blend: BlendState,
    pub topology: PrimitiveTopology,
    pub cull: Option<Cull>,
    pub depth: DepthState,
    pub format: TextureFormat,
}

/// The sdl3 enums don't implement `Hash`, so they are stored as their raw
/// discriminants here.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    pub shader: ShaderRef,
    pub vertex_pitch: u32,
    pub vertex_attributes: Vec<(u32, u32, u32)>,
    pub blend: BlendState,
    pub topology: u32,
    pub cull: Option<Cull>,
    pub depth: DepthState,
    pub format: u32,
}

impl PipelineKey {
    pub fn new(desc: &PipelineDesc) -> Self {
        Self {
            shader: desc.shader,
            vertex_pitch: desc.vertex_layout.pitch,
            vertex_attributes: desc
                .vertex_layout
                .attributes
                .iter()
                .map(|a| (a.location, a.format as u32, a.offset))
                .collect(),
            blend: desc.blend,
            topology: desc.topology as u32,
            cull: desc.cull,
            depth: desc.depth,
            format: desc.format as u32,
        }
    }
}

#[derive(Default)]
pub struct PipelineCache {
    pip: FastHashMap<PipelineKey, RenderPipeline>,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, gpu: &Gpu, desc: &PipelineDesc) {
        let key = PipelineKey::new(desc);
        let pip = build_pipeline(gpu, desc);

        debug!("Created new render pipeline {:?}", desc);

        self.pip.insert(key, pip);
    }

    pub fn get(&self, desc: &PipelineDesc) -> &RenderPipeline {
        let key = PipelineKey::new(desc);
        self.pip.get(&key).expect("Failed to get render pipeline")
    }

    pub fn get_or_create(&mut self, gpu: &Gpu, desc: &PipelineDesc) -> &RenderPipeline {
        let key = PipelineKey::new(desc);

        self.pip.entry(key).or_insert_with(|| {
            debug!("Created new render pipeline {:?}", desc);
            build_pipeline(gpu, desc)
        })
    }
}
