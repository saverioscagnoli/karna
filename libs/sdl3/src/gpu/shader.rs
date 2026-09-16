use core::ptr::NonNull;

use alloc::ffi::CString;
use sdl3_sys::SDL_CreateGPUShader;
use sdl3_sys::SDL_GPU_SHADERFORMAT_SPIRV;
use sdl3_sys::SDL_GPU_SHADERSTAGE_FRAGMENT;
use sdl3_sys::SDL_GPU_SHADERSTAGE_VERTEX;
use sdl3_sys::SDL_GPUShader;
use sdl3_sys::SDL_GPUShaderCreateInfo;
use sdl3_sys::SDL_GPUShaderFormat;
use sdl3_sys::SDL_GPUShaderStage;
use sdl3_sys::SDL_ReleaseGPUShader;
use sdl3_sys::SdlError;
use sdl3_sys::get_error;

use crate::gpu::Device;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

impl ShaderStage {
    pub const fn raw(self) -> SDL_GPUShaderStage {
        match self {
            Self::Vertex => SDL_GPU_SHADERSTAGE_VERTEX,
            Self::Fragment => SDL_GPU_SHADERSTAGE_FRAGMENT,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ShaderDesc<'a> {
    pub code: &'a [u8],
    pub entrypoint: &'a str,
    pub format: SDL_GPUShaderFormat,
    pub stage: ShaderStage,
    pub samplers: u32,
    pub storage_textures: u32,
    pub storage_buffers: u32,
    pub uniform_buffers: u32,
}

impl<'a> ShaderDesc<'a> {
    pub fn new(code: &'a [u8], stage: ShaderStage) -> Self {
        Self {
            code,
            entrypoint: "main",
            format: SDL_GPU_SHADERFORMAT_SPIRV,
            stage,
            samplers: 0,
            storage_textures: 0,
            storage_buffers: 0,
            uniform_buffers: 0,
        }
    }

    pub fn vertex(code: &'a [u8]) -> Self {
        Self::new(code, ShaderStage::Vertex)
    }

    pub fn fragment(code: &'a [u8]) -> Self {
        Self::new(code, ShaderStage::Fragment)
    }

    pub fn with_entrypoint(mut self, entrypoint: &'a str) -> Self {
        self.entrypoint = entrypoint;
        self
    }

    pub fn with_format(mut self, format: SDL_GPUShaderFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_samplers(mut self, count: u32) -> Self {
        self.samplers = count;
        self
    }

    pub fn with_storage_textures(mut self, count: u32) -> Self {
        self.storage_textures = count;
        self
    }

    pub fn with_storage_buffers(mut self, count: u32) -> Self {
        self.storage_buffers = count;
        self
    }

    pub fn with_uniform_buffers(mut self, count: u32) -> Self {
        self.uniform_buffers = count;
        self
    }
}

pub struct Shader {
    device: Device,
    raw: NonNull<SDL_GPUShader>,
    stage: ShaderStage,
}

impl Shader {
    pub fn new(device: Device, desc: ShaderDesc<'_>) -> Result<Self, SdlError> {
        let entrypoint = CString::new(desc.entrypoint).map_err(|_| SdlError::new("InteriorNul"))?;

        let info = SDL_GPUShaderCreateInfo {
            code_size: desc.code.len(),
            code: desc.code.as_ptr(),
            entrypoint: entrypoint.as_ptr(),
            format: desc.format,
            stage: desc.stage.raw(),
            num_samplers: desc.samplers,
            num_storage_textures: desc.storage_textures,
            num_storage_buffers: desc.storage_buffers,
            num_uniform_buffers: desc.uniform_buffers,
            props: 0,
        };

        let ptr = unsafe { SDL_CreateGPUShader(device.as_ptr(), &info) };
        let raw = NonNull::new(ptr).ok_or_else(get_error)?;

        Ok(Self {
            device,
            raw,
            stage: desc.stage,
        })
    }

    pub fn vertex(device: Device, code: &[u8]) -> Result<Self, SdlError> {
        Self::new(device, ShaderDesc::vertex(code))
    }

    pub fn fragment(device: Device, code: &[u8]) -> Result<Self, SdlError> {
        Self::new(device, ShaderDesc::fragment(code))
    }

    pub fn raw(&self) -> *mut SDL_GPUShader {
        self.raw.as_ptr()
    }

    pub fn stage(&self) -> ShaderStage {
        self.stage
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { SDL_ReleaseGPUShader(self.device.as_ptr(), self.raw.as_ptr()) }
    }
}
