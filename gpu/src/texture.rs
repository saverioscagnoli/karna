use logging::debug;
use sdl3::gpu::TextureRegion;
use sdl3::gpu::TextureTransferInfo;
use sdl3::gpu::TransferBufferUsage;

use crate::Filter;
use crate::Gpu;
use crate::TextureFormat;

pub use sdl3::gpu::TextureSamplerBinding;
pub use sdl3::gpu::TextureType;
pub use sdl3::gpu::TextureUsage as TextureUsages;

/// Bytes per pixel for [`TextureFormat::R8g8b8a8Unorm`].
const RGBA_STRIDE: u32 = 4;

/// Staged writes start on this boundary. Not every backend needs it, but
/// D3D12/Metal copy alignment rules are stricter than Vulkan's and a few
/// wasted bytes per write is cheaper than finding out the hard way.
const COPY_ALIGN: usize = 256;

const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

/// A sampled 2D GPU texture.
///
/// Single-region writes go through [`Texture::write_rgba`], which submits its
/// own command buffer. When filling many regions at once — atlas pages, glyph
/// caches — use [`TextureBatch`] instead: one transfer buffer, one copy pass,
/// one submit for the whole set.
///
/// Transfer buffers are never retained. Textures are written rarely, and
/// keeping a full-size staging copy alive per texture doubles their cost.
pub struct Texture {
    label: String,
    inner: sdl3::gpu::Texture<'static>,
    width: u32,
    height: u32,
    format: TextureFormat,
    filter: Filter,
}

impl Texture {
    /// Create an uninitialized RGBA8 texture usable as a fragment sampler.
    ///
    /// The contents are undefined until written. Callers that sample regions
    /// they never wrote (or sample across region boundaries with linear
    /// filtering) are responsible for making that safe — see the padding in
    /// the atlas for how.
    pub fn new<L: AsRef<str>>(gpu: &Gpu, label: L, width: u32, height: u32) -> Self {
        let label: &str = label.as_ref();
        let width = width.max(1);
        let height = height.max(1);

        let inner = gpu
            .device
            .create_texture(
                sdl3::gpu::TextureCreateInfo::new()
                    .with_type(TextureType::_2D)
                    .with_format(TextureFormat::R8g8b8a8Unorm)
                    .with_usage(TextureUsages::SAMPLER)
                    .with_width(width)
                    .with_height(height)
                    .with_layer_count_or_depth(1)
                    .with_num_levels(1),
            )
            .expect("Failed to create texture");

        debug!("Created texture '{}' ({}x{})", label, width, height);

        Self {
            label: label.to_owned(),
            inner,
            width,
            height,
            format: TextureFormat::R8g8b8a8Unorm,
            filter: Filter::default(),
        }
    }

    /// Create a depth target sized to a render pass.
    ///
    /// Never sampled, so it carries no useful filter, and never written from
    /// the CPU — the driver clears and fills it during the pass. Recreate it
    /// whenever the target it pairs with resizes; a depth attachment smaller
    /// than its color attachment is a validation error.
    pub fn depth<L: AsRef<str>>(gpu: &Gpu, label: L, width: u32, height: u32) -> Self {
        let label: &str = label.as_ref();
        let width = width.max(1);
        let height = height.max(1);
        let format = TextureFormat::D24Unorm;

        let inner = gpu
            .device
            .create_texture(
                sdl3::gpu::TextureCreateInfo::new()
                    .with_type(TextureType::_2D)
                    .with_format(format)
                    .with_usage(TextureUsages::DEPTH_STENCIL_TARGET)
                    .with_width(width)
                    .with_height(height)
                    .with_layer_count_or_depth(1)
                    .with_num_levels(1),
            )
            .expect("Failed to create depth texture");

        debug!("Created depth texture '{}' ({}x{})", label, width, height);

        Self {
            label: label.to_owned(),
            inner,
            width,
            height,
            format,
            filter: Filter::default(),
        }
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    /// Pick the filter this texture is sampled with. The sampler behind it is
    /// only created once something actually binds the texture.
    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }

    pub fn filter(&self) -> Filter {
        self.filter
    }

    /// Create a texture and fill it with `pixels`, which must be
    /// `width * height * 4` bytes of RGBA8.
    pub fn from_rgba<L: AsRef<str>>(
        gpu: &Gpu,
        label: L,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Self {
        let texture = Self::new(gpu, label, width, height);
        texture.write_rgba(gpu, 0, 0, width, height, pixels);
        texture
    }

    /// Upload RGBA8 `pixels` into the `w`x`h` region at (`x`, `y`).
    ///
    /// Submits immediately. Prefer [`TextureBatch`] when writing more than one
    /// region in a frame.
    pub fn write_rgba(&self, gpu: &Gpu, x: u32, y: u32, w: u32, h: u32, pixels: &[u8]) {
        let mut batch = TextureBatch::new();

        if batch.stage(self, x, y, w, h, pixels) {
            batch.submit(gpu);
        }
    }

    /// Validate a write against the texture's bounds and the caller's slice
    /// length. A short slice would otherwise read out of bounds inside the
    /// driver, so this is checked rather than trusted.
    fn accepts(&self, x: u32, y: u32, w: u32, h: u32, len: usize) -> bool {
        let expected = (w * h * RGBA_STRIDE) as usize;

        if len != expected {
            logging::warn!(
                "Texture '{}': expected {} bytes for a {}x{} write, got {}",
                self.label,
                expected,
                w,
                h,
                len
            );

            return false;
        }

        if x + w > self.width || y + h > self.height {
            logging::warn!(
                "Texture '{}': {}x{} write at ({}, {}) does not fit {}x{}",
                self.label,
                w,
                h,
                x,
                y,
                self.width,
                self.height
            );

            return false;
        }

        true
    }

    pub fn raw(&self) -> &sdl3::gpu::Texture<'static> {
        &self.inner
    }

    /// Mutable access to the underlying texture.
    ///
    /// Only needed because `DepthStencilTargetInfo::with_texture` takes `&mut`
    /// — nothing about attaching a depth target actually mutates it.
    pub fn raw_mut(&mut self) -> &mut sdl3::gpu::Texture<'static> {
        &mut self.inner
    }

    /// Pair this texture with the sampler for its filter, creating that sampler
    /// if this is the first texture to ask for it.
    pub fn binding<'a>(&'a self, gpu: &'a Gpu) -> TextureSamplerBinding<'a> {
        TextureSamplerBinding::new()
            .with_texture(&self.inner)
            .with_sampler(gpu.sampler(self.filter))
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

struct StagedWrite<'a> {
    texture: &'a Texture,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    offset: usize,
}

/// Collects pixel writes across any number of textures and uploads them in a
/// single copy pass.
///
/// Staging is CPU-side and cheap; nothing touches the GPU until [`submit`].
///
/// [`submit`]: TextureBatch::submit
pub struct TextureBatch<'a> {
    staging: Vec<u8>,
    writes: Vec<StagedWrite<'a>>,
}

impl<'a> TextureBatch<'a> {
    pub fn new() -> Self {
        Self {
            staging: Vec::new(),
            writes: Vec::new(),
        }
    }

    pub fn with_capacity(bytes: usize) -> Self {
        Self {
            staging: Vec::with_capacity(bytes),
            writes: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.writes.is_empty()
    }

    /// Total bytes staged so far, including alignment padding.
    pub fn staged_bytes(&self) -> usize {
        self.staging.len()
    }

    /// Queue a write of RGBA8 `pixels` into the `w`x`h` region of `texture` at
    /// (`x`, `y`). Returns false if the write was rejected, in which case
    /// nothing was staged.
    pub fn stage(
        &mut self,
        texture: &'a Texture,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        pixels: &[u8],
    ) -> bool {
        if !texture.accepts(x, y, w, h, pixels.len()) {
            return false;
        }

        let offset = align_up(self.staging.len(), COPY_ALIGN);

        self.staging.resize(offset, 0);
        self.staging.extend_from_slice(pixels);

        self.writes.push(StagedWrite {
            texture,
            x,
            y,
            w,
            h,
            offset,
        });

        true
    }

    /// Upload everything staged. A no-op if nothing was staged.
    pub fn submit(self, gpu: &Gpu) {
        if self.writes.is_empty() {
            return;
        }

        let transfer = gpu
            .device
            .create_transfer_buffer()
            .with_usage(TransferBufferUsage::UPLOAD)
            .with_size(self.staging.len() as u32)
            .build()
            .expect("Failed to create texture transfer buffer");

        let mut map = transfer.map::<u8>(&gpu.device, true);
        map.mem_mut()[..self.staging.len()].copy_from_slice(&self.staging);
        map.unmap();

        let Ok(mut cmd) = gpu.device.acquire_command_buffer() else {
            logging::warn!("Failed to acquire command buffer for texture upload");
            return;
        };

        let Ok(copy_pass) = gpu.device.begin_copy_pass(&cmd) else {
            logging::warn!("Failed to begin copy pass for texture upload");
            cmd.cancel();
            return;
        };

        for write in &self.writes {
            copy_pass.upload_to_gpu_texture(
                TextureTransferInfo::new()
                    .with_transfer_buffer(&transfer)
                    .with_offset(write.offset as u32)
                    .with_pixels_per_row(write.w)
                    .with_rows_per_layer(write.h),
                TextureRegion::new()
                    .with_texture(write.texture.raw())
                    .with_mip_level(0)
                    .with_layer(0)
                    .with_x(write.x)
                    .with_y(write.y)
                    .with_z(0)
                    .with_width(write.w)
                    .with_height(write.h)
                    .with_depth(1),
                false,
            );
        }

        gpu.device.end_copy_pass(copy_pass);

        debug!(
            "Uploaded {} texture region(s), {} bytes",
            self.writes.len(),
            self.staging.len()
        );

        if let Err(e) = cmd.submit() {
            logging::warn!("Failed to submit texture upload: {}", e);
        }
    }
}

impl Default for TextureBatch<'_> {
    fn default() -> Self {
        Self::new()
    }
}
