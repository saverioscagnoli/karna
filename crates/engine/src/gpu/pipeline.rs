use std::ptr::NonNull;

use sdl3::SDL_CreateGPUGraphicsPipeline;
use sdl3::SDL_CreateGPUShader;
use sdl3::SDL_GPU_COLORCOMPONENT_A;
use sdl3::SDL_GPU_COLORCOMPONENT_B;
use sdl3::SDL_GPU_COLORCOMPONENT_G;
use sdl3::SDL_GPU_COLORCOMPONENT_R;
use sdl3::SDL_GPU_SHADERFORMAT_SPIRV;
use sdl3::SDL_GPUBlendFactor;
use sdl3::SDL_GPUBlendOp;
use sdl3::SDL_GPUColorTargetBlendState;
use sdl3::SDL_GPUColorTargetDescription;
use sdl3::SDL_GPUCullMode;
use sdl3::SDL_GPUFillMode;
use sdl3::SDL_GPUFrontFace;
use sdl3::SDL_GPUGraphicsPipeline;
use sdl3::SDL_GPUGraphicsPipelineCreateInfo;
use sdl3::SDL_GPUGraphicsPipelineTargetInfo;
use sdl3::SDL_GPUPrimitiveType;
use sdl3::SDL_GPURasterizerState;
use sdl3::SDL_GPUShader;
use sdl3::SDL_GPUShaderCreateInfo;
use sdl3::SDL_GPUShaderStage;
use sdl3::SDL_GPUTextureFormat;
use sdl3::SDL_GPUVertexAttribute;
use sdl3::SDL_GPUVertexBufferDescription;
use sdl3::SDL_GPUVertexInputRate;
use sdl3::SDL_GPUVertexInputState;
use sdl3::SDL_ReleaseGPUGraphicsPipeline;
use sdl3::SDL_ReleaseGPUShader;

use crate::err::sdl_last_error;
use crate::gpu::Device;
use crate::gpu::VertexLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

impl ShaderStage {
    fn raw(self) -> SDL_GPUShaderStage {
        match self {
            Self::Vertex => SDL_GPUShaderStage::SDL_GPU_SHADERSTAGE_VERTEX,
            Self::Fragment => SDL_GPUShaderStage::SDL_GPU_SHADERSTAGE_FRAGMENT,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ShaderDesc {
    pub stage: ShaderStage,
    pub samplers: u32,
    pub storage_textures: u32,
    pub storage_buffers: u32,
    pub uniform_buffers: u32,
}

impl ShaderDesc {
    pub const fn new(stage: ShaderStage) -> Self {
        Self {
            stage,
            samplers: 0,
            storage_textures: 0,
            storage_buffers: 0,
            uniform_buffers: 0,
        }
    }

    pub const fn with_samplers(mut self, count: u32) -> Self {
        self.samplers = count;
        self
    }

    pub const fn with_uniform_buffers(mut self, count: u32) -> Self {
        self.uniform_buffers = count;
        self
    }

    pub const fn with_storage_textures(mut self, count: u32) -> Self {
        self.storage_textures = count;
        self
    }

    pub const fn with_storage_buffers(mut self, count: u32) -> Self {
        self.storage_buffers = count;
        self
    }
}

pub struct Shader {
    device: Device,
    raw: NonNull<SDL_GPUShader>,
    desc: ShaderDesc,
}

impl Shader {
    pub fn new(device: Device, code: &[u8], desc: ShaderDesc) -> Self {
        assert!(!code.is_empty(), "shader has no code");

        let raw = unsafe {
            let info = SDL_GPUShaderCreateInfo {
                code_size: code.len(),
                code: code.as_ptr(),
                entrypoint: c"main".as_ptr(),
                format: SDL_GPU_SHADERFORMAT_SPIRV,
                stage: desc.stage.raw(),
                num_samplers: desc.samplers,
                num_storage_textures: desc.storage_textures,
                num_storage_buffers: desc.storage_buffers,
                num_uniform_buffers: desc.uniform_buffers,
                props: 0,
            };

            let Some(shader) = NonNull::new(SDL_CreateGPUShader(device.raw(), &info)) else {
                panic!(
                    "Failed to create {:?} shader: {}",
                    desc.stage,
                    sdl_last_error()
                );
            };

            shader
        };

        Self { device, raw, desc }
    }

    pub fn raw(&self) -> *mut SDL_GPUShader {
        self.raw.as_ptr()
    }

    pub fn stage(&self) -> ShaderStage {
        self.desc.stage
    }

    pub fn desc(&self) -> &ShaderDesc {
        &self.desc
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { SDL_ReleaseGPUShader(self.device.raw(), self.raw.as_ptr()) }
    }
}

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blend {
    Opaque,
    #[default]
    Alpha,
    Additive,
}

impl Blend {
    fn state(self) -> SDL_GPUColorTargetBlendState {
        let mask = (SDL_GPU_COLORCOMPONENT_R
            | SDL_GPU_COLORCOMPONENT_G
            | SDL_GPU_COLORCOMPONENT_B
            | SDL_GPU_COLORCOMPONENT_A) as u8;

        let base = SDL_GPUColorTargetBlendState {
            color_write_mask: mask,
            color_blend_op: SDL_GPUBlendOp::SDL_GPU_BLENDOP_ADD,
            alpha_blend_op: SDL_GPUBlendOp::SDL_GPU_BLENDOP_ADD,
            ..Default::default()
        };

        match self {
            Self::Opaque => SDL_GPUColorTargetBlendState {
                enable_blend: false,
                ..base
            },

            Self::Alpha => SDL_GPUColorTargetBlendState {
                enable_blend: true,
                src_color_blendfactor: SDL_GPUBlendFactor::SDL_GPU_BLENDFACTOR_SRC_ALPHA,
                dst_color_blendfactor: SDL_GPUBlendFactor::SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
                src_alpha_blendfactor: SDL_GPUBlendFactor::SDL_GPU_BLENDFACTOR_ONE,
                dst_alpha_blendfactor: SDL_GPUBlendFactor::SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
                ..base
            },

            Self::Additive => SDL_GPUColorTargetBlendState {
                enable_blend: true,
                src_color_blendfactor: SDL_GPUBlendFactor::SDL_GPU_BLENDFACTOR_SRC_ALPHA,
                dst_color_blendfactor: SDL_GPUBlendFactor::SDL_GPU_BLENDFACTOR_ONE,
                src_alpha_blendfactor: SDL_GPUBlendFactor::SDL_GPU_BLENDFACTOR_ONE,
                dst_alpha_blendfactor: SDL_GPUBlendFactor::SDL_GPU_BLENDFACTOR_ONE,
                ..base
            },
        }
    }
}

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cull {
    #[default]
    None,
    Front,
    Back,
}

impl Cull {
    fn raw(self) -> SDL_GPUCullMode {
        match self {
            Self::None => SDL_GPUCullMode::SDL_GPU_CULLMODE_NONE,
            Self::Front => SDL_GPUCullMode::SDL_GPU_CULLMODE_FRONT,
            Self::Back => SDL_GPUCullMode::SDL_GPU_CULLMODE_BACK,
        }
    }
}

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    #[default]
    TriangleList,
    TriangleStrip,
    LineList,
    LineStrip,
    PointList,
}

impl Primitive {
    fn raw(self) -> SDL_GPUPrimitiveType {
        match self {
            Self::TriangleList => SDL_GPUPrimitiveType::SDL_GPU_PRIMITIVETYPE_TRIANGLELIST,
            Self::TriangleStrip => SDL_GPUPrimitiveType::SDL_GPU_PRIMITIVETYPE_TRIANGLESTRIP,
            Self::LineList => SDL_GPUPrimitiveType::SDL_GPU_PRIMITIVETYPE_LINELIST,
            Self::LineStrip => SDL_GPUPrimitiveType::SDL_GPU_PRIMITIVETYPE_LINESTRIP,
            Self::PointList => SDL_GPUPrimitiveType::SDL_GPU_PRIMITIVETYPE_POINTLIST,
        }
    }
}

pub struct GraphicsPipelineDesc {
    pub layout: VertexLayout,
    pub target_format: SDL_GPUTextureFormat,
    pub primitive: Primitive,
    pub blend: Blend,
    pub cull: Cull,
}

impl GraphicsPipelineDesc {
    pub fn new(layout: VertexLayout, target_format: SDL_GPUTextureFormat) -> Self {
        Self {
            layout,
            target_format,
            primitive: Primitive::default(),
            blend: Blend::default(),
            cull: Cull::default(),
        }
    }

    pub fn with_primitive(mut self, primitive: Primitive) -> Self {
        self.primitive = primitive;
        self
    }

    pub fn with_blend(mut self, blend: Blend) -> Self {
        self.blend = blend;
        self
    }

    pub fn with_cull(mut self, cull: Cull) -> Self {
        self.cull = cull;
        self
    }
}

pub struct GraphicsPipeline {
    label: String,
    device: Device,
    raw: NonNull<SDL_GPUGraphicsPipeline>,
    target_format: SDL_GPUTextureFormat,
}

impl GraphicsPipeline {
    pub fn new<L>(
        device: Device,
        label: L,
        vertex: &Shader,
        fragment: &Shader,
        desc: GraphicsPipelineDesc,
    ) -> Self
    where
        L: AsRef<str>,
    {
        let label = label.as_ref();

        assert!(
            vertex.stage() == ShaderStage::Vertex,
            "pipeline '{}' was given a non-vertex shader in the vertex slot",
            label
        );

        assert!(
            fragment.stage() == ShaderStage::Fragment,
            "pipeline '{}' was given a non-fragment shader in the fragment slot",
            label
        );

        let attributes = desc
            .layout
            .attributes
            .iter()
            .map(|a| SDL_GPUVertexAttribute {
                location: a.location,
                buffer_slot: 0,
                format: a.format,
                offset: a.offset,
            })
            .collect::<Vec<_>>();

        let buffers = [SDL_GPUVertexBufferDescription {
            slot: 0,
            pitch: desc.layout.pitch,
            input_rate: SDL_GPUVertexInputRate::SDL_GPU_VERTEXINPUTRATE_VERTEX,
            instance_step_rate: 0,
        }];

        let targets = [SDL_GPUColorTargetDescription {
            format: desc.target_format,
            blend_state: desc.blend.state(),
        }];

        let raw = unsafe {
            let info = SDL_GPUGraphicsPipelineCreateInfo {
                vertex_shader: vertex.raw(),
                fragment_shader: fragment.raw(),
                vertex_input_state: SDL_GPUVertexInputState {
                    vertex_buffer_descriptions: buffers.as_ptr(),
                    num_vertex_buffers: buffers.len() as u32,
                    vertex_attributes: attributes.as_ptr(),
                    num_vertex_attributes: attributes.len() as u32,
                },
                primitive_type: desc.primitive.raw(),
                rasterizer_state: SDL_GPURasterizerState {
                    fill_mode: SDL_GPUFillMode::SDL_GPU_FILLMODE_FILL,
                    cull_mode: desc.cull.raw(),
                    front_face: SDL_GPUFrontFace::SDL_GPU_FRONTFACE_COUNTER_CLOCKWISE,
                    enable_depth_clip: true,
                    ..Default::default()
                },
                target_info: SDL_GPUGraphicsPipelineTargetInfo {
                    color_target_descriptions: targets.as_ptr(),
                    num_color_targets: targets.len() as u32,
                    has_depth_stencil_target: false,
                    ..Default::default()
                },
                props: 0,
                ..Default::default()
            };

            let Some(pipeline) = NonNull::new(SDL_CreateGPUGraphicsPipeline(device.raw(), &info))
            else {
                panic!(
                    "Failed to create graphics pipeline '{}': {}",
                    label,
                    sdl_last_error()
                );
            };

            pipeline
        };

        Self {
            label: label.to_string(),
            device,
            raw,
            target_format: desc.target_format,
        }
    }

    pub fn raw(&self) -> *mut SDL_GPUGraphicsPipeline {
        self.raw.as_ptr()
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn target_format(&self) -> SDL_GPUTextureFormat {
        self.target_format
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe { SDL_ReleaseGPUGraphicsPipeline(self.device.raw(), self.raw.as_ptr()) }
    }
}
