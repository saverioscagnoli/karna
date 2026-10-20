use sdl3::gpu::Device;
use sdl3::gpu::Texture;
use sdl3::gpu::TextureDesc;
use sdl3::gpu::TextureFormat;
use sdl3::gpu::TextureUsage;
use sdl3::render::Color;

use crate::render::LayerData;

pub struct Canvas {
    pub(crate) color: Texture,
    pub(crate) depth: Texture,
    size: math::Size<u32>,
}

impl Canvas {
    pub(crate) fn new(device: &Device, w: u32, h: u32) -> Self {
        let color = Texture::new(
            device.share(),
            "canvas",
            Self::desc(w, h).with_usage(TextureUsage::SAMPLER | TextureUsage::COLOR_TARGET),
        );

        let depth = Texture::new(
            device.share(),
            "canvas depth",
            TextureDesc::depth(w, h, device.depth_format()),
        );

        Self {
            color,
            depth,
            size: math::size!(w, h),
        }
    }

    pub(crate) fn format() -> TextureFormat {
        Self::desc(1, 1).format
    }

    fn desc(w: u32, h: u32) -> TextureDesc {
        TextureDesc::rgba8_array(w, h, 1)
    }

    pub fn size(&self) -> math::Size<u32> {
        self.size
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }
}

#[derive(Default)]
pub struct CanvasData {
    pub clear: Option<Color>,
    pub data: LayerData,
}
