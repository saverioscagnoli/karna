use logging::debug;
use sdl3::gpu::BlendFactor;
use sdl3::gpu::BlendOp;
use sdl3::gpu::ColorTargetBlendState;
use sdl3::gpu::ColorTargetDescription;
use sdl3::gpu::CompareOp;
use sdl3::gpu::CullMode;
use sdl3::gpu::DepthStencilState;
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

/// Depth testing for a pipeline. `None` on [`PipelineDesc`] means the pass has
/// no depth attachment at all — 2d and ui passes.
///
/// A pipeline built with depth must be recorded into a render pass that
/// actually has a depth target, and the formats must agree. [`DEPTH_FORMAT`] is
/// the one both sides use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepthState {
    pub test: bool,
    pub write: bool,
    pub compare: CompareOp,
}

/// sdl3 derives `Eq` on `CompareOp` but not `Hash`, so the discriminant stands
/// in for it — the same substitution `PipelineKey` makes for `PrimitiveType`.
impl std::hash::Hash for DepthState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.test.hash(state);
        self.write.hash(state);
        (self.compare as u32).hash(state);
    }
}

impl Default for DepthState {
    fn default() -> Self {
        Self::OPAQUE
    }
}

impl DepthState {
    /// Test and write. What opaque geometry wants.
    pub const OPAQUE: Self = Self {
        test: true,
        write: true,
        compare: CompareOp::Less,
    };

    /// Test but don't write, so blended surfaces are occluded by opaque
    /// geometry without occluding each other.
    pub const TRANSPARENT: Self = Self {
        test: true,
        write: false,
        compare: CompareOp::Less,
    };

    fn apply(&self, state: DepthStencilState) -> DepthStencilState {
        state
            .with_enable_depth_test(self.test)
            .with_enable_depth_write(self.write)
            .with_compare_op(self.compare)
    }
}

/// Format shared by every depth texture and every depth-enabled pipeline.
///
/// D24 rather than D32F because it is the widely supported depth-only format;
/// SDL guarantees one of D24/D16 on all backends.
pub const DEPTH_FORMAT: TextureFormat = TextureFormat::D24Unorm;

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

    let color_targets = [desc
        .blend
        .apply(ColorTargetDescription::new().with_format(desc.format))];

    let mut target_info =
        GraphicsPipelineTargetInfo::new().with_color_target_descriptions(&color_targets);

    if desc.depth.is_some() {
        target_info = target_info
            .with_has_depth_stencil_target(true)
            .with_depth_stencil_format(DEPTH_FORMAT);
    }

    let mut builder = gpu
        .device
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
        .with_target_info(target_info);

    if let Some(depth) = desc.depth {
        builder = builder.with_depth_stencil_state(depth.apply(DepthStencilState::new()));
    }

    builder.build().expect("Failed to create render pipeline")
}

#[derive(Debug, Clone)]
pub struct PipelineDesc {
    pub shader: ShaderRef,
    pub vertex_layout: VertexLayout,
    pub blend: BlendState,
    pub topology: PrimitiveTopology,
    pub cull: Option<Cull>,
    pub format: TextureFormat,
    /// `None` for passes with no depth attachment.
    pub depth: Option<DepthState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    pub shader: ShaderRef,
    pub vertex_pitch: u32,
    pub vertex_attributes: Vec<(u32, u32, u32)>,
    pub blend: BlendState,
    pub topology: u32,
    pub cull: Option<Cull>,
    pub format: u32,
    pub depth: Option<DepthState>,
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
            format: desc.format as u32,
            depth: desc.depth,
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

        debug!("Created new render pipeline with shader {:?}", desc.shader);

        self.pip.insert(key, pip);
    }

    /// Build the pipeline if the cache doesn't have it yet
    pub fn ensure(&mut self, gpu: &Gpu, desc: &PipelineDesc) {
        let key = PipelineKey::new(desc);

        self.pip.entry(key).or_insert_with(|| {
            debug!("Created new render pipeline with shader {:?}", desc.shader);
            build_pipeline(gpu, desc)
        });
    }

    pub fn get(&self, desc: &PipelineDesc) -> &RenderPipeline {
        let key = PipelineKey::new(desc);
        self.pip.get(&key).expect("Failed to get render pipeline")
    }

    pub fn get_or_create(&mut self, gpu: &Gpu, desc: &PipelineDesc) -> &RenderPipeline {
        let key = PipelineKey::new(desc);

        self.pip.entry(key).or_insert_with(|| {
            debug!("Created new render pipeline with shader {:?}", desc.shader);
            build_pipeline(gpu, desc)
        })
    }
}
