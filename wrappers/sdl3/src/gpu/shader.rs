use core::fmt;
use core::ops::BitOr;
use core::ptr::NonNull;

use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;
use sdl3_sys::SDL_CreateGPUShader;
use sdl3_sys::SDL_GPU_SHADERFORMAT_DXBC;
use sdl3_sys::SDL_GPU_SHADERFORMAT_DXIL;
use sdl3_sys::SDL_GPU_SHADERFORMAT_METALLIB;
use sdl3_sys::SDL_GPU_SHADERFORMAT_MSL;
use sdl3_sys::SDL_GPU_SHADERFORMAT_PRIVATE;
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderFormat(SDL_GPUShaderFormat);

impl ShaderFormat {
    pub const NONE: Self = Self(0);
    pub const PRIVATE: Self = Self(SDL_GPU_SHADERFORMAT_PRIVATE);
    pub const SPIRV: Self = Self(SDL_GPU_SHADERFORMAT_SPIRV);
    pub const DXBC: Self = Self(SDL_GPU_SHADERFORMAT_DXBC);
    pub const DXIL: Self = Self(SDL_GPU_SHADERFORMAT_DXIL);
    pub const MSL: Self = Self(SDL_GPU_SHADERFORMAT_MSL);
    pub const METALLIB: Self = Self(SDL_GPU_SHADERFORMAT_METALLIB);

    const NAMED: [(Self, &'static str); 6] = [
        (Self::PRIVATE, "PRIVATE"),
        (Self::SPIRV, "SPIRV"),
        (Self::DXBC, "DXBC"),
        (Self::DXIL, "DXIL"),
        (Self::MSL, "MSL"),
        (Self::METALLIB, "METALLIB"),
    ];

    pub const fn from_bits(bits: SDL_GPUShaderFormat) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> SDL_GPUShaderFormat {
        self.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn is_single(self) -> bool {
        self.0.is_power_of_two()
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub fn iter(self) -> impl Iterator<Item = Self> {
        Self::NAMED
            .into_iter()
            .map(|(format, _)| format)
            .filter(move |format| self.contains(*format))
    }
}

impl BitOr for ShaderFormat {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl fmt::Debug for ShaderFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("NONE");
        }

        let mut first = true;

        for (format, name) in Self::NAMED {
            if self.contains(format) {
                if !first {
                    f.write_str(" | ")?;
                }

                f.write_str(name)?;
                first = false;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderResources {
    pub samplers: u32,
    pub storage_textures: u32,
    pub storage_buffers: u32,
    pub uniform_buffers: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledShader {
    pub format: ShaderFormat,
    pub stage: ShaderStage,
    pub entrypoint: String,
    pub code: Vec<u8>,
    pub resources: ShaderResources,
}

impl CompiledShader {
    pub fn desc(&self) -> ShaderDesc<'_> {
        ShaderDesc::new(&self.code, self.stage)
            .with_format(self.format)
            .with_entrypoint(&self.entrypoint)
            .with_resources(self.resources)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ShaderDesc<'a> {
    pub code: &'a [u8],
    pub entrypoint: &'a str,
    pub format: ShaderFormat,
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
            format: ShaderFormat::SPIRV,
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

    pub fn with_format(mut self, format: ShaderFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_resources(mut self, resources: ShaderResources) -> Self {
        self.samplers = resources.samplers;
        self.storage_textures = resources.storage_textures;
        self.storage_buffers = resources.storage_buffers;
        self.uniform_buffers = resources.uniform_buffers;
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
            format: desc.format.bits(),
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

    #[cfg(feature = "shadercross")]
    pub(crate) fn from_raw(
        device: Device,
        raw: NonNull<SDL_GPUShader>,
        stage: ShaderStage,
    ) -> Self {
        Self { device, raw, stage }
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
