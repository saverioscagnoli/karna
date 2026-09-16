use core::mem;
use core::ptr::NonNull;

use alloc::vec::Vec;
use sdl3_sys::SDL_CreateGPUGraphicsPipeline;
use sdl3_sys::SDL_GPU_BLENDFACTOR_ONE;
use sdl3_sys::SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA;
use sdl3_sys::SDL_GPU_BLENDFACTOR_SRC_ALPHA;
use sdl3_sys::SDL_GPU_BLENDFACTOR_ZERO;
use sdl3_sys::SDL_GPU_BLENDOP_ADD;
use sdl3_sys::SDL_GPU_CULLMODE_NONE;
use sdl3_sys::SDL_GPU_FILLMODE_FILL;
use sdl3_sys::SDL_GPU_FRONTFACE_COUNTER_CLOCKWISE;
use sdl3_sys::SDL_GPU_PRIMITIVETYPE_TRIANGLELIST;
use sdl3_sys::SDL_GPU_SAMPLECOUNT_1;
use sdl3_sys::SDL_GPU_VERTEXELEMENTFORMAT_FLOAT;
use sdl3_sys::SDL_GPU_VERTEXELEMENTFORMAT_FLOAT2;
use sdl3_sys::SDL_GPU_VERTEXELEMENTFORMAT_FLOAT3;
use sdl3_sys::SDL_GPU_VERTEXELEMENTFORMAT_FLOAT4;
use sdl3_sys::SDL_GPU_VERTEXELEMENTFORMAT_UBYTE4_NORM;
use sdl3_sys::SDL_GPU_VERTEXINPUTRATE_INSTANCE;
use sdl3_sys::SDL_GPU_VERTEXINPUTRATE_VERTEX;
use sdl3_sys::SDL_GPUBlendFactor;
use sdl3_sys::SDL_GPUBlendOp;
use sdl3_sys::SDL_GPUColorTargetBlendState;
use sdl3_sys::SDL_GPUColorTargetDescription;
use sdl3_sys::SDL_GPUCullMode;
use sdl3_sys::SDL_GPUFillMode;
use sdl3_sys::SDL_GPUFrontFace;
use sdl3_sys::SDL_GPUGraphicsPipeline;
use sdl3_sys::SDL_GPUGraphicsPipelineCreateInfo;
use sdl3_sys::SDL_GPUPrimitiveType;
use sdl3_sys::SDL_GPUSampleCount;
use sdl3_sys::SDL_GPUTextureFormat;
use sdl3_sys::SDL_GPUVertexAttribute;
use sdl3_sys::SDL_GPUVertexBufferDescription;
use sdl3_sys::SDL_GPUVertexElementFormat;
use sdl3_sys::SDL_GPUVertexInputRate;
use sdl3_sys::SDL_ReleaseGPUGraphicsPipeline;
use sdl3_sys::SdlError;
use sdl3_sys::get_error;

use crate::gpu::Device;
use crate::gpu::Shader;

#[derive(Debug, Clone, Copy)]
pub struct VertexBuffer {
    pub slot: u32,
    pub pitch: u32,
    pub input_rate: SDL_GPUVertexInputRate,
}

impl VertexBuffer {
    pub const fn per_vertex(slot: u32, pitch: u32) -> Self {
        Self {
            slot,
            pitch,
            input_rate: SDL_GPU_VERTEXINPUTRATE_VERTEX,
        }
    }

    pub const fn per_instance(slot: u32, pitch: u32) -> Self {
        Self {
            slot,
            pitch,
            input_rate: SDL_GPU_VERTEXINPUTRATE_INSTANCE,
        }
    }

    pub const fn of<T>(slot: u32) -> Self {
        Self::per_vertex(slot, mem::size_of::<T>() as u32)
    }

    pub const fn of_instance<T>(slot: u32) -> Self {
        Self::per_instance(slot, mem::size_of::<T>() as u32)
    }

    const fn raw(self) -> SDL_GPUVertexBufferDescription {
        SDL_GPUVertexBufferDescription {
            slot: self.slot,
            pitch: self.pitch,
            input_rate: self.input_rate,
            instance_step_rate: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VertexAttribute {
    pub location: u32,
    pub slot: u32,
    pub format: SDL_GPUVertexElementFormat,
    pub offset: u32,
}

impl VertexAttribute {
    pub const fn new(location: u32, format: SDL_GPUVertexElementFormat, offset: u32) -> Self {
        Self {
            location,
            slot: 0,
            format,
            offset,
        }
    }

    pub const fn float(location: u32, offset: u32) -> Self {
        Self::new(location, SDL_GPU_VERTEXELEMENTFORMAT_FLOAT, offset)
    }

    pub const fn float2(location: u32, offset: u32) -> Self {
        Self::new(location, SDL_GPU_VERTEXELEMENTFORMAT_FLOAT2, offset)
    }

    pub const fn float3(location: u32, offset: u32) -> Self {
        Self::new(location, SDL_GPU_VERTEXELEMENTFORMAT_FLOAT3, offset)
    }

    pub const fn float4(location: u32, offset: u32) -> Self {
        Self::new(location, SDL_GPU_VERTEXELEMENTFORMAT_FLOAT4, offset)
    }

    pub const fn ubyte4_norm(location: u32, offset: u32) -> Self {
        Self::new(location, SDL_GPU_VERTEXELEMENTFORMAT_UBYTE4_NORM, offset)
    }

    pub const fn at_slot(mut self, slot: u32) -> Self {
        self.slot = slot;
        self
    }

    const fn raw(self) -> SDL_GPUVertexAttribute {
        SDL_GPUVertexAttribute {
            location: self.location,
            buffer_slot: self.slot,
            format: self.format,
            offset: self.offset,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Blend {
    /// Writes the source straight over the target.
    Replace,
    /// `src * src.a + dst * (1 - src.a)`, for colors that have not been
    /// multiplied through by their alpha.
    #[default]
    Alpha,
    /// `src + dst * (1 - src.a)`, for colors that have.
    Premultiplied,
    /// `src + dst`, for light.
    Additive,
}

impl Blend {
    pub fn state(self) -> SDL_GPUColorTargetBlendState {
        let (src, dst, enable) = match self {
            Self::Replace => (SDL_GPU_BLENDFACTOR_ONE, SDL_GPU_BLENDFACTOR_ZERO, false),
            Self::Alpha => (
                SDL_GPU_BLENDFACTOR_SRC_ALPHA,
                SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
                true,
            ),
            Self::Premultiplied => (
                SDL_GPU_BLENDFACTOR_ONE,
                SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
                true,
            ),
            Self::Additive => (SDL_GPU_BLENDFACTOR_ONE, SDL_GPU_BLENDFACTOR_ONE, true),
        };

        Self::blend_state(src, dst, enable)
    }

    fn blend_state(
        src: SDL_GPUBlendFactor,
        dst: SDL_GPUBlendFactor,
        enable: bool,
    ) -> SDL_GPUColorTargetBlendState {
        SDL_GPUColorTargetBlendState {
            src_color_blendfactor: src,
            dst_color_blendfactor: dst,
            color_blend_op: SDL_GPU_BLENDOP_ADD as SDL_GPUBlendOp,
            // Alpha accumulates rather than being blended by itself, so a
            // target rendered into and then composited stays correct.
            src_alpha_blendfactor: SDL_GPU_BLENDFACTOR_ONE,
            dst_alpha_blendfactor: SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
            alpha_blend_op: SDL_GPU_BLENDOP_ADD as SDL_GPUBlendOp,
            enable_blend: enable,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub struct PipelineDesc<'a> {
    pub vertex: &'a Shader,
    pub fragment: &'a Shader,
    pub buffers: &'a [VertexBuffer],
    pub attributes: &'a [VertexAttribute],
    pub targets: &'a [SDL_GPUTextureFormat],
    pub depth_stencil: Option<SDL_GPUTextureFormat>,
    pub primitive: SDL_GPUPrimitiveType,
    pub blend: Blend,
    pub fill: SDL_GPUFillMode,
    pub cull: SDL_GPUCullMode,
    pub front_face: SDL_GPUFrontFace,
    pub sample_count: SDL_GPUSampleCount,
}

impl<'a> PipelineDesc<'a> {
    pub fn new(vertex: &'a Shader, fragment: &'a Shader) -> Self {
        Self {
            vertex,
            fragment,
            buffers: &[],
            attributes: &[],
            targets: &[],
            depth_stencil: None,
            primitive: SDL_GPU_PRIMITIVETYPE_TRIANGLELIST,
            blend: Blend::Alpha,
            fill: SDL_GPU_FILLMODE_FILL,
            cull: SDL_GPU_CULLMODE_NONE,
            front_face: SDL_GPU_FRONTFACE_COUNTER_CLOCKWISE,
            sample_count: SDL_GPU_SAMPLECOUNT_1,
        }
    }

    pub fn with_vertex_layout(
        mut self,
        buffers: &'a [VertexBuffer],
        attributes: &'a [VertexAttribute],
    ) -> Self {
        self.buffers = buffers;
        self.attributes = attributes;
        self
    }

    pub fn with_targets(mut self, targets: &'a [SDL_GPUTextureFormat]) -> Self {
        self.targets = targets;
        self
    }

    pub fn with_depth_stencil(mut self, format: SDL_GPUTextureFormat) -> Self {
        self.depth_stencil = Some(format);
        self
    }

    pub fn with_primitive(mut self, primitive: SDL_GPUPrimitiveType) -> Self {
        self.primitive = primitive;
        self
    }

    pub fn with_blend(mut self, blend: Blend) -> Self {
        self.blend = blend;
        self
    }

    pub fn with_fill(mut self, fill: SDL_GPUFillMode) -> Self {
        self.fill = fill;
        self
    }

    pub fn with_cull(mut self, cull: SDL_GPUCullMode, front_face: SDL_GPUFrontFace) -> Self {
        self.cull = cull;
        self.front_face = front_face;
        self
    }

    pub fn with_sample_count(mut self, sample_count: SDL_GPUSampleCount) -> Self {
        self.sample_count = sample_count;
        self
    }
}

pub struct GraphicsPipeline {
    device: Device,
    raw: NonNull<SDL_GPUGraphicsPipeline>,
}

impl GraphicsPipeline {
    pub fn new(device: Device, desc: PipelineDesc<'_>) -> Result<Self, SdlError> {
        if desc.targets.is_empty() && desc.depth_stencil.is_none() {
            return Err(SdlError::new("pipeline has no render targets"));
        }

        let buffers: Vec<_> = desc.buffers.iter().map(|b| b.raw()).collect();
        let attributes: Vec<_> = desc.attributes.iter().map(|a| a.raw()).collect();
        let targets: Vec<_> = desc
            .targets
            .iter()
            .map(|&format| SDL_GPUColorTargetDescription {
                format,
                blend_state: desc.blend.state(),
            })
            .collect();

        let mut info = SDL_GPUGraphicsPipelineCreateInfo::default();

        info.vertex_shader = desc.vertex.raw();
        info.fragment_shader = desc.fragment.raw();
        info.primitive_type = desc.primitive;

        info.vertex_input_state.vertex_buffer_descriptions = buffers.as_ptr();
        info.vertex_input_state.num_vertex_buffers = buffers.len() as u32;
        info.vertex_input_state.vertex_attributes = attributes.as_ptr();
        info.vertex_input_state.num_vertex_attributes = attributes.len() as u32;

        info.rasterizer_state.fill_mode = desc.fill;
        info.rasterizer_state.cull_mode = desc.cull;
        info.rasterizer_state.front_face = desc.front_face;

        info.multisample_state.sample_count = desc.sample_count;

        info.target_info.color_target_descriptions = targets.as_ptr();
        info.target_info.num_color_targets = targets.len() as u32;

        if let Some(format) = desc.depth_stencil {
            info.target_info.has_depth_stencil_target = true;
            info.target_info.depth_stencil_format = format;
            info.depth_stencil_state.enable_depth_test = true;
            info.depth_stencil_state.enable_depth_write = true;
        }

        let ptr = unsafe { SDL_CreateGPUGraphicsPipeline(device.as_ptr(), &info) };
        let raw = NonNull::new(ptr).ok_or_else(get_error)?;

        Ok(Self { device, raw })
    }

    pub fn raw(&self) -> *mut SDL_GPUGraphicsPipeline {
        self.raw.as_ptr()
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe { SDL_ReleaseGPUGraphicsPipeline(self.device.as_ptr(), self.raw.as_ptr()) }
    }
}
