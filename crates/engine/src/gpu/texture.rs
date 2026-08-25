use std::ffi::CString;
use std::mem;
use std::ops::BitOr;
use std::ptr::NonNull;
use std::rc::Rc;

use sdl3::SDL_AcquireGPUCommandBuffer;
use sdl3::SDL_BeginGPUCopyPass;
use sdl3::SDL_CalculateGPUTextureFormatSize;
use sdl3::SDL_CreateGPUSampler;
use sdl3::SDL_CreateGPUTexture;
use sdl3::SDL_EndGPUCopyPass;
use sdl3::SDL_GPU_TEXTUREUSAGE_COLOR_TARGET;
use sdl3::SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_READ;
use sdl3::SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_WRITE;
use sdl3::SDL_GPU_TEXTUREUSAGE_DEPTH_STENCIL_TARGET;
use sdl3::SDL_GPU_TEXTUREUSAGE_GRAPHICS_STORAGE_READ;
use sdl3::SDL_GPU_TEXTUREUSAGE_SAMPLER;
use sdl3::SDL_GPUFilter;
use sdl3::SDL_GPUSampleCount;
use sdl3::SDL_GPUSampler;
use sdl3::SDL_GPUSamplerAddressMode;
use sdl3::SDL_GPUSamplerCreateInfo;
use sdl3::SDL_GPUSamplerMipmapMode;
use sdl3::SDL_GPUTexture;
use sdl3::SDL_GPUTextureCreateInfo;
use sdl3::SDL_GPUTextureFormat;
use sdl3::SDL_GPUTextureRegion;
use sdl3::SDL_GPUTextureSamplerBinding;
use sdl3::SDL_GPUTextureTransferInfo;
use sdl3::SDL_GPUTextureType;
use sdl3::SDL_GPUTextureUsageFlags;
use sdl3::SDL_ReleaseGPUSampler;
use sdl3::SDL_ReleaseGPUTexture;
use sdl3::SDL_SetGPUTextureName;
use sdl3::SDL_SubmitGPUCommandBuffer;
use sdl3::SDL_UploadToGPUTexture;

use crate::err::SDL_LastError;
use crate::gpu::Gpu;
use crate::gpu::buffer::BufferError;

/// D3D12 wants texture transfer offsets on a 512-byte boundary; the other
/// backends are looser, so this is the conservative common denominator.
const TEXTURE_TRANSFER_ALIGN: u32 = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Filter {
    Nearest,
    Linear,
}

impl Filter {
    pub fn sampler_desc(self) -> SamplerDesc {
        match self {
            Self::Nearest => SamplerDesc::nearest(),
            Self::Linear => SamplerDesc::linear(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureUsage(SDL_GPUTextureUsageFlags);

impl TextureUsage {
    pub const SAMPLER: Self = Self(SDL_GPU_TEXTUREUSAGE_SAMPLER);
    pub const COLOR_TARGET: Self = Self(SDL_GPU_TEXTUREUSAGE_COLOR_TARGET);
    pub const DEPTH_STENCIL_TARGET: Self = Self(SDL_GPU_TEXTUREUSAGE_DEPTH_STENCIL_TARGET);
    pub const GRAPHICS_STORAGE_READ: Self = Self(SDL_GPU_TEXTUREUSAGE_GRAPHICS_STORAGE_READ);
    pub const COMPUTE_STORAGE_READ: Self = Self(SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_READ);
    pub const COMPUTE_STORAGE_WRITE: Self = Self(SDL_GPU_TEXTUREUSAGE_COMPUTE_STORAGE_WRITE);

    pub const fn bits(self) -> SDL_GPUTextureUsageFlags {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for TextureUsage {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureDesc {
    pub kind: SDL_GPUTextureType,
    pub format: SDL_GPUTextureFormat,
    pub usage: TextureUsage,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
    pub levels: u32,
    pub sample_count: SDL_GPUSampleCount,
}

impl TextureDesc {
    pub fn rgba8(width: u32, height: u32) -> Self {
        Self {
            kind: SDL_GPUTextureType::SDL_GPU_TEXTURETYPE_2D,
            format: SDL_GPUTextureFormat::SDL_GPU_TEXTUREFORMAT_R8G8B8A8_UNORM,
            usage: TextureUsage::SAMPLER,
            width,
            height,
            layers: 1,
            levels: 1,
            sample_count: SDL_GPUSampleCount::SDL_GPU_SAMPLECOUNT_1,
        }
    }

    pub fn with_usage(mut self, usage: TextureUsage) -> Self {
        self.usage = usage;
        self
    }

    pub fn with_format(mut self, format: SDL_GPUTextureFormat) -> Self {
        self.format = format;
        self
    }
}

pub struct Texture {
    label: String,
    gpu: Rc<Gpu>,
    raw: NonNull<SDL_GPUTexture>,
    desc: TextureDesc,
}

impl Texture {
    pub fn new<L>(gpu: Rc<Gpu>, label: L, desc: TextureDesc) -> Self
    where
        L: AsRef<str>,
    {
        let label = label.as_ref();

        assert!(desc.width > 0 && desc.height > 0, "texture has zero extent");
        assert!(desc.levels >= 1, "texture needs at least one mip level");
        assert!(desc.layers >= 1, "texture needs at least one layer");

        let raw = unsafe {
            let mut info: SDL_GPUTextureCreateInfo = mem::zeroed();

            info.type_ = desc.kind;
            info.format = desc.format;
            info.usage = desc.usage.bits();
            info.width = desc.width;
            info.height = desc.height;
            info.layer_count_or_depth = desc.layers;
            info.num_levels = desc.levels;
            info.sample_count = desc.sample_count;

            let Some(tex) = NonNull::new(SDL_CreateGPUTexture(gpu.device, &info)) else {
                panic!("Failed to create texture '{}': {}", label, SDL_LastError());
            };

            if let Ok(c) = CString::new(label) {
                SDL_SetGPUTextureName(gpu.device, tex.as_ptr(), c.as_ptr());
            }

            tex
        };

        Self {
            label: label.to_string(),
            gpu,
            raw,
            desc,
        }
    }

    pub fn white(gpu: Rc<Gpu>) -> Self {
        let texture = Self::new(gpu.clone(), "white", TextureDesc::rgba8(1, 1));
        gpu.upload_texture(&texture, &[255u8, 255, 255, 255])
            .expect("failed to upload white texel");
        texture
    }

    pub fn raw(&self) -> *mut SDL_GPUTexture {
        self.raw.as_ptr()
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn desc(&self) -> &TextureDesc {
        &self.desc
    }

    pub fn width(&self) -> u32 {
        self.desc.width
    }

    pub fn height(&self) -> u32 {
        self.desc.height
    }

    pub fn format(&self) -> SDL_GPUTextureFormat {
        self.desc.format
    }

    pub fn usage(&self) -> TextureUsage {
        self.desc.usage
    }

    pub fn byte_size(&self) -> u32 {
        unsafe {
            SDL_CalculateGPUTextureFormatSize(
                self.desc.format,
                self.desc.width,
                self.desc.height,
                1,
            )
        }
    }

    pub fn binding(&self, sampler: &Sampler) -> SDL_GPUTextureSamplerBinding {
        SDL_GPUTextureSamplerBinding {
            texture: self.raw.as_ptr(),
            sampler: sampler.raw(),
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { SDL_ReleaseGPUTexture(self.gpu.device, self.raw.as_ptr()) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SamplerDesc {
    pub min_filter: SDL_GPUFilter,
    pub mag_filter: SDL_GPUFilter,
    pub mipmap_mode: SDL_GPUSamplerMipmapMode,
    pub address_u: SDL_GPUSamplerAddressMode,
    pub address_v: SDL_GPUSamplerAddressMode,
    pub address_w: SDL_GPUSamplerAddressMode,
}

impl Default for SamplerDesc {
    fn default() -> Self {
        Self {
            min_filter: SDL_GPUFilter::SDL_GPU_FILTER_NEAREST,
            mag_filter: SDL_GPUFilter::SDL_GPU_FILTER_NEAREST,
            mipmap_mode: SDL_GPUSamplerMipmapMode::SDL_GPU_SAMPLERMIPMAPMODE_NEAREST,
            address_u: SDL_GPUSamplerAddressMode::SDL_GPU_SAMPLERADDRESSMODE_CLAMP_TO_EDGE,
            address_v: SDL_GPUSamplerAddressMode::SDL_GPU_SAMPLERADDRESSMODE_CLAMP_TO_EDGE,
            address_w: SDL_GPUSamplerAddressMode::SDL_GPU_SAMPLERADDRESSMODE_CLAMP_TO_EDGE,
        }
    }
}

impl SamplerDesc {
    pub fn nearest() -> Self {
        Self::default()
    }

    pub fn linear() -> Self {
        Self {
            min_filter: SDL_GPUFilter::SDL_GPU_FILTER_LINEAR,
            mag_filter: SDL_GPUFilter::SDL_GPU_FILTER_LINEAR,
            mipmap_mode: SDL_GPUSamplerMipmapMode::SDL_GPU_SAMPLERMIPMAPMODE_LINEAR,
            ..Self::default()
        }
    }

    pub fn repeat(mut self) -> Self {
        let mode = SDL_GPUSamplerAddressMode::SDL_GPU_SAMPLERADDRESSMODE_REPEAT;
        self.address_u = mode;
        self.address_v = mode;
        self.address_w = mode;
        self
    }
}

pub struct Sampler {
    gpu: Rc<Gpu>,
    raw: NonNull<SDL_GPUSampler>,
}

impl Sampler {
    pub fn new(gpu: Rc<Gpu>, desc: SamplerDesc) -> Self {
        let raw = unsafe {
            let mut info: SDL_GPUSamplerCreateInfo = mem::zeroed();

            info.min_filter = desc.min_filter;
            info.mag_filter = desc.mag_filter;
            info.mipmap_mode = desc.mipmap_mode;
            info.address_mode_u = desc.address_u;
            info.address_mode_v = desc.address_v;
            info.address_mode_w = desc.address_w;

            let Some(sampler) = NonNull::new(SDL_CreateGPUSampler(gpu.device, &info)) else {
                panic!("Failed to create sampler: {}", SDL_LastError());
            };

            sampler
        };

        Self { gpu, raw }
    }

    pub fn raw(&self) -> *mut SDL_GPUSampler {
        self.raw.as_ptr()
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe { SDL_ReleaseGPUSampler(self.gpu.device, self.raw.as_ptr()) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureRegion {
    pub mip_level: u32,
    pub layer: u32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl TextureRegion {
    pub fn full(texture: &Texture) -> Self {
        Self {
            mip_level: 0,
            layer: 0,
            x: 0,
            y: 0,
            w: texture.width(),
            h: texture.height(),
        }
    }
}

fn sdl_err() -> BufferError {
    BufferError::SDL(format!("{}", SDL_LastError()))
}

impl Gpu {
    pub fn upload_texture(&self, dst: &Texture, pixels: &[u8]) -> Result<(), BufferError> {
        self.upload_texture_region(dst, TextureRegion::full(dst), pixels)
    }

    pub fn upload_texture_region(
        &self,
        dst: &Texture,
        region: TextureRegion,
        pixels: &[u8],
    ) -> Result<(), BufferError> {
        if pixels.is_empty() || region.w == 0 || region.h == 0 {
            return Ok(());
        }

        let expected =
            unsafe { SDL_CalculateGPUTextureFormatSize(dst.format(), region.w, region.h, 1) };

        assert!(
            pixels.len() as u32 >= expected,
            "texture '{}': got {} bytes, need {} for a {}x{} region",
            dst.label(),
            pixels.len(),
            expected,
            region.w,
            region.h
        );

        let bytes = u32::try_from(pixels.len()).map_err(|_| BufferError::TooLarge)?;
        let mut staging = self.staging.borrow_mut();

        staging.reserve(bytes)?;

        let src_offset = {
            let mut mapped = staging.map(true)?;
            mapped.write(pixels)?
        };

        unsafe {
            let cmd = SDL_AcquireGPUCommandBuffer(self.device);

            if cmd.is_null() {
                return Err(sdl_err());
            }

            let pass = SDL_BeginGPUCopyPass(cmd);

            let source = SDL_GPUTextureTransferInfo {
                transfer_buffer: staging.raw(),
                offset: src_offset,
                pixels_per_row: region.w,
                rows_per_layer: region.h,
            };

            let destination = SDL_GPUTextureRegion {
                texture: dst.raw(),
                mip_level: region.mip_level,
                layer: region.layer,
                x: region.x,
                y: region.y,
                z: 0,
                w: region.w,
                h: region.h,
                d: 1,
            };

            SDL_UploadToGPUTexture(pass, &source, &destination, false);
            SDL_EndGPUCopyPass(pass);

            if !SDL_SubmitGPUCommandBuffer(cmd) {
                return Err(sdl_err());
            }
        }

        Ok(())
    }
}

impl Gpu {
    /// Uploads several full textures through one mapping and one command
    /// buffer. Direct replacement for the old `TextureBatch`.
    pub fn upload_textures(&self, uploads: &[(&Texture, &[u8])]) -> Result<(), BufferError> {
        if uploads.is_empty() {
            return Ok(());
        }

        let total: usize = uploads
            .iter()
            .map(|(_, pixels)| pixels.len() + TEXTURE_TRANSFER_ALIGN as usize)
            .sum();
        let total = u32::try_from(total).map_err(|_| BufferError::TooLarge)?;

        let mut staging = self.staging.borrow_mut();
        staging.reserve(total)?;

        let mut offsets = Vec::with_capacity(uploads.len());

        {
            let mut mapped = staging.map(true)?;

            for (_, pixels) in uploads {
                offsets.push(mapped.write_aligned(pixels, TEXTURE_TRANSFER_ALIGN)?);
            }
        }

        unsafe {
            let cmd = SDL_AcquireGPUCommandBuffer(self.device);

            if cmd.is_null() {
                return Err(sdl_err());
            }

            let pass = SDL_BeginGPUCopyPass(cmd);

            for ((texture, _), &offset) in uploads.iter().zip(&offsets) {
                let source = SDL_GPUTextureTransferInfo {
                    transfer_buffer: staging.raw(),
                    offset,
                    pixels_per_row: texture.width(),
                    rows_per_layer: texture.height(),
                };

                let destination = SDL_GPUTextureRegion {
                    texture: texture.raw(),
                    mip_level: 0,
                    layer: 0,
                    x: 0,
                    y: 0,
                    z: 0,
                    w: texture.width(),
                    h: texture.height(),
                    d: 1,
                };

                SDL_UploadToGPUTexture(pass, &source, &destination, false);
            }

            SDL_EndGPUCopyPass(pass);

            if !SDL_SubmitGPUCommandBuffer(cmd) {
                return Err(sdl_err());
            }
        }

        Ok(())
    }
}
