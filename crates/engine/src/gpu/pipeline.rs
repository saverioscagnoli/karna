use std::mem;
use std::ptr::NonNull;

use sdl3::SDL_BeginGPUCopyPass;
use sdl3::SDL_CreateGPUGraphicsPipeline;
use sdl3::SDL_CreateGPUSampler;
use sdl3::SDL_CreateGPUShader;
use sdl3::SDL_CreateGPUTexture;
use sdl3::SDL_EndGPUCopyPass;
use sdl3::SDL_GPUBuffer;
use sdl3::SDL_GPUColorTargetBlendState;
use sdl3::SDL_GPUColorTargetDescription;
use sdl3::SDL_GPUDevice;
use sdl3::SDL_GPUFilter;
use sdl3::SDL_GPUGraphicsPipeline;
use sdl3::SDL_GPUGraphicsPipelineCreateInfo;
use sdl3::SDL_GPUGraphicsPipelineTargetInfo;
use sdl3::SDL_GPUPrimitiveType;
use sdl3::SDL_GPURasterizerState;
use sdl3::SDL_GPUSampleCount;
use sdl3::SDL_GPUSampler;
use sdl3::SDL_GPUSamplerAddressMode;
use sdl3::SDL_GPUSamplerCreateInfo;
use sdl3::SDL_GPUSamplerMipmapMode;
use sdl3::SDL_GPUShader;
use sdl3::SDL_GPUShaderCreateInfo;
use sdl3::SDL_GPUShaderStage;
use sdl3::SDL_GPUTexture;
use sdl3::SDL_GPUTextureCreateInfo;
use sdl3::SDL_GPUTextureFormat;
use sdl3::SDL_GPUTextureRegion;
use sdl3::SDL_GPUTextureTransferInfo;
use sdl3::SDL_GPUTextureType;
use sdl3::SDL_GPUVertexAttribute;
use sdl3::SDL_GPUVertexBufferDescription;
use sdl3::SDL_GPUVertexInputRate;
use sdl3::SDL_GPUVertexInputState;
use sdl3::SDL_GPU_SHADERFORMAT_SPIRV;
use sdl3::SDL_GPU_TEXTUREUSAGE_SAMPLER;
use sdl3::SDL_ReleaseGPUShader;
use sdl3::SDL_SubmitGPUCommandBuffer;
use sdl3::SDL_UploadToGPUTexture;
use sdl3::SDL_AcquireGPUCommandBuffer;

use crate::gpu::buffer::GpuTransferBuffer;
use crate::gpu::vertex::LayoutDescriptor;
use crate::render::vertex::Vertex;

const VERTEX_SPIRV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/immediate.vert.spv"));
const FRAGMENT_SPIRV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/immediate.frag.spv"));

pub(crate) struct DrawCall {
    pub mvp: math::Matrix4<f32>,
    pub vertex_buffer: *mut SDL_GPUBuffer,
    pub index_buffer: *mut SDL_GPUBuffer,
    pub num_indices: u32,
}

pub(crate) struct Pipeline {
    raw: NonNull<SDL_GPUGraphicsPipeline>,
    device: *mut SDL_GPUDevice,
    white_texture: NonNull<SDL_GPUTexture>,
    white_sampler: NonNull<SDL_GPUSampler>,
}

fn create_shader(
    device: *mut SDL_GPUDevice,
    code: &[u8],
    stage: SDL_GPUShaderStage,
    num_samplers: u32,
    num_uniform_buffers: u32,
) -> *mut SDL_GPUShader {
    unsafe {
        let mut info: SDL_GPUShaderCreateInfo = mem::zeroed();

        info.code_size = code.len();
        info.code = code.as_ptr();
        info.entrypoint = c"main".as_ptr();
        info.format = SDL_GPU_SHADERFORMAT_SPIRV;
        info.stage = stage;
        info.num_samplers = num_samplers;
        info.num_uniform_buffers = num_uniform_buffers;

        let shader = SDL_CreateGPUShader(device, &info);
        assert!(!shader.is_null(), "failed to create GPU shader");
        shader
    }
}

fn create_white_texture(device: *mut SDL_GPUDevice) -> (NonNull<SDL_GPUTexture>, NonNull<SDL_GPUSampler>) {
    unsafe {
        let mut tex_info: SDL_GPUTextureCreateInfo = mem::zeroed();
        tex_info.type_ = SDL_GPUTextureType::SDL_GPU_TEXTURETYPE_2D;
        tex_info.format = SDL_GPUTextureFormat::SDL_GPU_TEXTUREFORMAT_R8G8B8A8_UNORM;
        tex_info.usage = SDL_GPU_TEXTUREUSAGE_SAMPLER;
        tex_info.width = 1;
        tex_info.height = 1;
        tex_info.layer_count_or_depth = 1;
        tex_info.num_levels = 1;
        tex_info.sample_count = SDL_GPUSampleCount::SDL_GPU_SAMPLECOUNT_1;

        let texture = NonNull::new(SDL_CreateGPUTexture(device, &tex_info))
            .expect("failed to create white texture");

        let mut sampler_info: SDL_GPUSamplerCreateInfo = mem::zeroed();
        sampler_info.min_filter = SDL_GPUFilter::SDL_GPU_FILTER_NEAREST;
        sampler_info.mag_filter = SDL_GPUFilter::SDL_GPU_FILTER_NEAREST;
        sampler_info.mipmap_mode = SDL_GPUSamplerMipmapMode::SDL_GPU_SAMPLERMIPMAPMODE_NEAREST;
        sampler_info.address_mode_u = SDL_GPUSamplerAddressMode::SDL_GPU_SAMPLERADDRESSMODE_CLAMP_TO_EDGE;
        sampler_info.address_mode_v = SDL_GPUSamplerAddressMode::SDL_GPU_SAMPLERADDRESSMODE_CLAMP_TO_EDGE;
        sampler_info.address_mode_w = SDL_GPUSamplerAddressMode::SDL_GPU_SAMPLERADDRESSMODE_CLAMP_TO_EDGE;

        let sampler = NonNull::new(SDL_CreateGPUSampler(device, &sampler_info))
            .expect("failed to create white sampler");

        let mut staging = GpuTransferBuffer::new(device, 4);
        {
            let mut mapped = staging.map(true).expect("failed to map staging buffer");
            mapped.write(&[255u8, 255, 255, 255]).expect("failed to write texel");
        }

        let cmd = SDL_AcquireGPUCommandBuffer(device);
        assert!(!cmd.is_null(), "failed to acquire command buffer");

        let pass = SDL_BeginGPUCopyPass(cmd);

        let source = SDL_GPUTextureTransferInfo {
            transfer_buffer: staging.raw(),
            offset: 0,
            pixels_per_row: 1,
            rows_per_layer: 1,
        };

        let destination = SDL_GPUTextureRegion {
            texture: texture.as_ptr(),
            mip_level: 0,
            layer: 0,
            x: 0,
            y: 0,
            z: 0,
            w: 1,
            h: 1,
            d: 1,
        };

        SDL_UploadToGPUTexture(pass, &source, &destination, false);
        SDL_EndGPUCopyPass(pass);
        SDL_SubmitGPUCommandBuffer(cmd);

        (texture, sampler)
    }
}

impl Pipeline {
    pub fn new(device: *mut SDL_GPUDevice, color_format: SDL_GPUTextureFormat) -> Self {
        unsafe {
            let vertex_shader = create_shader(
                device,
                VERTEX_SPIRV,
                SDL_GPUShaderStage::SDL_GPU_SHADERSTAGE_VERTEX,
                0,
                1,
            );
            let fragment_shader = create_shader(
                device,
                FRAGMENT_SPIRV,
                SDL_GPUShaderStage::SDL_GPU_SHADERSTAGE_FRAGMENT,
                1,
                0,
            );

            let layout = Vertex::desc();

            let buffer_descriptions = [SDL_GPUVertexBufferDescription {
                slot: 0,
                pitch: layout.pitch,
                input_rate: SDL_GPUVertexInputRate::SDL_GPU_VERTEXINPUTRATE_VERTEX,
                instance_step_rate: 0,
            }];

            let attributes = layout
                .attributes
                .iter()
                .map(|a| SDL_GPUVertexAttribute {
                    location: a.location,
                    buffer_slot: 0,
                    format: a.format,
                    offset: a.offset,
                })
                .collect::<Vec<_>>();

            let color_targets = [SDL_GPUColorTargetDescription {
                format: color_format,
                blend_state: SDL_GPUColorTargetBlendState {
                    src_color_blendfactor: sdl3::SDL_GPUBlendFactor::SDL_GPU_BLENDFACTOR_SRC_ALPHA,
                    dst_color_blendfactor:
                        sdl3::SDL_GPUBlendFactor::SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
                    color_blend_op: sdl3::SDL_GPUBlendOp::SDL_GPU_BLENDOP_ADD,
                    src_alpha_blendfactor: sdl3::SDL_GPUBlendFactor::SDL_GPU_BLENDFACTOR_ONE,
                    dst_alpha_blendfactor:
                        sdl3::SDL_GPUBlendFactor::SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
                    alpha_blend_op: sdl3::SDL_GPUBlendOp::SDL_GPU_BLENDOP_ADD,
                    color_write_mask: 0,
                    enable_blend: true,
                    enable_color_write_mask: false,
                    padding1: 0,
                    padding2: 0,
                },
            }];

            let mut info: SDL_GPUGraphicsPipelineCreateInfo = mem::zeroed();

            info.vertex_shader = vertex_shader;
            info.fragment_shader = fragment_shader;
            info.vertex_input_state = SDL_GPUVertexInputState {
                vertex_buffer_descriptions: buffer_descriptions.as_ptr(),
                num_vertex_buffers: buffer_descriptions.len() as u32,
                vertex_attributes: attributes.as_ptr(),
                num_vertex_attributes: attributes.len() as u32,
            };
            info.primitive_type = SDL_GPUPrimitiveType::SDL_GPU_PRIMITIVETYPE_TRIANGLELIST;
            info.rasterizer_state = SDL_GPURasterizerState {
                fill_mode: sdl3::SDL_GPUFillMode::SDL_GPU_FILLMODE_FILL,
                cull_mode: sdl3::SDL_GPUCullMode::SDL_GPU_CULLMODE_NONE,
                front_face: sdl3::SDL_GPUFrontFace::SDL_GPU_FRONTFACE_COUNTER_CLOCKWISE,
                depth_bias_constant_factor: 0.0,
                depth_bias_clamp: 0.0,
                depth_bias_slope_factor: 0.0,
                enable_depth_bias: false,
                enable_depth_clip: true,
                padding1: 0,
                padding2: 0,
            };
            info.multisample_state.sample_count = SDL_GPUSampleCount::SDL_GPU_SAMPLECOUNT_1;
            info.target_info = SDL_GPUGraphicsPipelineTargetInfo {
                color_target_descriptions: color_targets.as_ptr(),
                num_color_targets: color_targets.len() as u32,
                depth_stencil_format: sdl3::SDL_GPUTextureFormat::SDL_GPU_TEXTUREFORMAT_INVALID,
                has_depth_stencil_target: false,
                padding1: 0,
                padding2: 0,
                padding3: 0,
            };

            let raw = NonNull::new(SDL_CreateGPUGraphicsPipeline(device, &info))
                .expect("failed to create GPU graphics pipeline");

            SDL_ReleaseGPUShader(device, vertex_shader);
            SDL_ReleaseGPUShader(device, fragment_shader);

            let (white_texture, white_sampler) = create_white_texture(device);

            Self {
                raw,
                device,
                white_texture,
                white_sampler,
            }
        }
    }

    pub fn raw(&self) -> *mut SDL_GPUGraphicsPipeline {
        self.raw.as_ptr()
    }

    pub fn white_texture(&self) -> *mut SDL_GPUTexture {
        self.white_texture.as_ptr()
    }

    pub fn white_sampler(&self) -> *mut SDL_GPUSampler {
        self.white_sampler.as_ptr()
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            sdl3::SDL_ReleaseGPUGraphicsPipeline(self.device, self.raw.as_ptr());
            sdl3::SDL_ReleaseGPUTexture(self.device, self.white_texture.as_ptr());
            sdl3::SDL_ReleaseGPUSampler(self.device, self.white_sampler.as_ptr());
        }
    }
}
