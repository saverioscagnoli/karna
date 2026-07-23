use logging::debug;
use sdl3::gpu::SampleCount;
use sdl3::gpu::TextureCreateInfo;
use sdl3::gpu::TextureFormat;
use sdl3::gpu::TextureRegion;
use sdl3::gpu::TextureSamplerBinding;
use sdl3::gpu::TextureTransferInfo;
use sdl3::gpu::TextureType;
use sdl3::gpu::TextureUsage;
use sdl3::gpu::TransferBufferUsage;

use crate::Gpu;

/// An RGBA texture that can be sampled in fragment shaders.
pub struct Texture {
    inner: sdl3::gpu::Texture<'static>,
    pub size: math::Size<u32>,
}

impl Texture {
    pub fn new_blank<L>(gpu: &Gpu, label: L, size: math::Size<u32>) -> Self
    where
        L: AsRef<str>,
    {
        let inner = gpu
            .device()
            .create_texture(
                TextureCreateInfo::new()
                    .with_type(TextureType::_2D)
                    .with_format(TextureFormat::R8g8b8a8UnormSrgb)
                    .with_width(size.width)
                    .with_height(size.height)
                    .with_layer_count_or_depth(1)
                    .with_num_levels(1)
                    .with_sample_count(SampleCount::NoMultiSampling)
                    .with_usage(TextureUsage::SAMPLER),
            )
            .expect("Failed to create texture");

        debug!("Creating new texture '{}' {:?}", label.as_ref(), size);

        Self { inner, size }
    }

    /// Records an upload of RGBA pixels into a region of the texture.
    pub fn write(
        &self,
        gpu: &Gpu,
        copy_pass: &sdl3::gpu::CopyPass,
        data: &[u8],
        origin: math::Vector2<u32>,
        size: math::Size<u32>,
    ) {
        let transfer = gpu
            .device()
            .create_transfer_buffer()
            .with_usage(TransferBufferUsage::UPLOAD)
            .with_size(data.len() as u32)
            .build()
            .expect("Failed to create texture transfer buffer");

        let mut map = transfer.map::<u8>(gpu.device(), false);
        map.mem_mut()[..data.len()].copy_from_slice(data);
        map.unmap();

        // Cycling swaps in a fresh backing texture when the current one is
        // still in flight, which discards every texel outside the written
        // region. Only safe when the write covers the whole texture.
        let cycle = origin.x == 0 && origin.y == 0 && size == self.size;

        copy_pass.upload_to_gpu_texture(
            TextureTransferInfo::new()
                .with_transfer_buffer(&transfer)
                .with_offset(0)
                .with_pixels_per_row(size.width)
                .with_rows_per_layer(size.height),
            TextureRegion::new()
                .with_texture(&self.inner)
                .with_x(origin.x)
                .with_y(origin.y)
                .with_width(size.width)
                .with_height(size.height)
                .with_depth(1),
            cycle,
        );
    }

    pub fn raw(&self) -> &sdl3::gpu::Texture<'static> {
        &self.inner
    }

    /// Binding of this texture with the shared engine sampler, for
    /// `RenderPass::bind_fragment_samplers`.
    pub fn binding<'a>(&'a self, gpu: &'a Gpu) -> TextureSamplerBinding<'a> {
        TextureSamplerBinding::new()
            .with_texture(&self.inner)
            .with_sampler(gpu.sampler())
    }
}

pub struct DepthTexture {
    inner: sdl3::gpu::Texture<'static>,
    pub size: math::Size<u32>,
}

impl DepthTexture {
    pub const FORMAT: TextureFormat = TextureFormat::D32Float;

    pub fn new(gpu: &Gpu, size: math::Size<u32>) -> Self {
        let size = math::Size::new(size.width.max(1), size.height.max(1));

        let inner = gpu
            .device()
            .create_texture(
                TextureCreateInfo::new()
                    .with_type(TextureType::_2D)
                    .with_format(Self::FORMAT)
                    .with_width(size.width)
                    .with_height(size.height)
                    .with_layer_count_or_depth(1)
                    .with_num_levels(1)
                    .with_sample_count(SampleCount::NoMultiSampling)
                    .with_usage(TextureUsage::DEPTH_STENCIL_TARGET),
            )
            .expect("Failed to create depth texture");

        debug!("Creating depth texture {:?}", size);

        Self { inner, size }
    }

    pub fn inner_mut(&mut self) -> &mut sdl3::gpu::Texture<'static> {
        &mut self.inner
    }
}
