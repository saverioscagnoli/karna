use std::path::Path;

use sdl3::image::ImageIOStream;
use sdl3::image::LoadSurface;
use sdl3::iostream::IOStream;
use sdl3::pixels::PixelFormat;
use sdl3::surface::Surface;

#[derive(Debug, Clone, Copy)]
pub struct Image {
    pub page: usize,
    pub uv_min: math::Vector2<f32>,
    pub uv_max: math::Vector2<f32>,
    pub size: math::Size<u32>,
}

impl Image {
    pub fn uv_center(&self) -> math::Vector2<f32> {
        let u = (self.uv_min.x + self.uv_max.x) * 0.5;
        let v = (self.uv_min.y + self.uv_max.y) * 0.5;
        math::Vector2::new(u, v)
    }
}

pub struct DecodedImage {
    pub size: math::Size<u32>,
    pub pixels: Vec<u8>,
}

pub fn decode_image(path: &Path) -> Result<DecodedImage, String> {
    let surface = Surface::from_file(path).map_err(|e| e.to_string())?;

    let surface = if surface.pixel_format_enum() == PixelFormat::ABGR8888 {
        surface
    } else {
        surface
            .convert_format(PixelFormat::ABGR8888)
            .map_err(|e| e.to_string())?
    };

    let width = surface.width();
    let height = surface.height();
    let pitch = surface.pitch() as usize;
    let row_bytes = width as usize * 4;

    let mut pixels = Vec::with_capacity(row_bytes * height as usize);

    surface.with_lock(|src: &[u8]| {
        for y in 0..height as usize {
            let start = y * pitch;
            pixels.extend_from_slice(&src[start..start + row_bytes]);
        }
    });

    Ok(DecodedImage {
        size: math::Size::new(width, height),
        pixels,
    })
}

pub fn decode_image_bytes(bytes: &[u8]) -> Result<DecodedImage, String> {
    let io = IOStream::from_bytes(bytes).map_err(|e| e.to_string())?;
    let surface = io.load().map_err(|e| e.to_string())?;

    let surface = if surface.pixel_format_enum() == PixelFormat::ABGR8888 {
        surface
    } else {
        surface
            .convert_format(PixelFormat::ABGR8888)
            .map_err(|e| e.to_string())?
    };

    let width = surface.width();
    let height = surface.height();
    let pitch = surface.pitch() as usize;
    let row_bytes = width as usize * 4;

    let mut pixels = Vec::with_capacity(row_bytes * height as usize);

    surface.with_lock(|src: &[u8]| {
        for y in 0..height as usize {
            let start = y * pitch;
            pixels.extend_from_slice(&src[start..start + row_bytes]);
        }
    });

    Ok(DecodedImage {
        size: math::Size::new(width, height),
        pixels,
    })
}
