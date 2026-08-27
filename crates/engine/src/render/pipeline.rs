use sdl3::SDL_GPUTextureFormat;
use utils::FastHashMap;

use crate::gpu::Blend;
use crate::gpu::Cull;
use crate::gpu::Device;
use crate::gpu::Filter;
use crate::gpu::GraphicsPipeline;
use crate::gpu::GraphicsPipelineDesc;
use crate::gpu::LayoutDescriptor;
use crate::gpu::Primitive;
use crate::gpu::Sampler;
use crate::gpu::Shader;
use crate::gpu::ShaderDesc;
use crate::gpu::ShaderStage;
use crate::render::vertex::Vertex;

#[repr(C, align(4))]
struct Spirv<T: ?Sized> {
    _align: [u32; 0],
    code: T,
}

static IMMEDIATE_VERT: &Spirv<[u8]> = &Spirv {
    _align: [],
    code: *include_bytes!(concat!(env!("OUT_DIR"), "/immediate.vert.spv")),
};

static IMMEDIATE_FRAG: &Spirv<[u8]> = &Spirv {
    _align: [],
    code: *include_bytes!(concat!(env!("OUT_DIR"), "/immediate.frag.spv")),
};

pub fn immediate(device: Device, target_format: SDL_GPUTextureFormat) -> GraphicsPipeline {
    let vertex = Shader::new(
        device.clone(),
        &IMMEDIATE_VERT.code,
        ShaderDesc::new(ShaderStage::Vertex).with_uniform_buffers(1),
    );

    let fragment = Shader::new(
        device.clone(),
        &IMMEDIATE_FRAG.code,
        ShaderDesc::new(ShaderStage::Fragment).with_samplers(1),
    );

    GraphicsPipeline::new(
        device,
        "immediate",
        &vertex,
        &fragment,
        GraphicsPipelineDesc::new(Vertex::desc(), target_format)
            .with_primitive(Primitive::TriangleList)
            .with_blend(Blend::Alpha)
            .with_cull(Cull::None),
    )
}

#[derive(Default)]
pub struct PipelineCache {
    immediate: FastHashMap<SDL_GPUTextureFormat, GraphicsPipeline>,
}

impl PipelineCache {
    pub fn immediate(
        &mut self,
        device: &Device,
        target_format: SDL_GPUTextureFormat,
    ) -> &GraphicsPipeline {
        self.immediate
            .entry(target_format)
            .or_insert_with(|| immediate(device.clone(), target_format))
    }

    pub fn len(&self) -> usize {
        self.immediate.len()
    }

    pub fn is_empty(&self) -> bool {
        self.immediate.is_empty()
    }
}

#[derive(Default)]
pub struct SamplerCache {
    samplers: FastHashMap<Filter, Sampler>,
}

impl SamplerCache {
    pub fn get(&mut self, device: &Device, filter: Filter) -> &Sampler {
        self.samplers
            .entry(filter)
            .or_insert_with(|| Sampler::new(device.clone(), filter.sampler_desc()))
    }
}
