use std::sync::atomic::Ordering;

use crate::render::Camera;
use crate::render::Projection;
use crate::render::camera::CameraData;
use crate::render::color::Color;
use crate::render::imgui::NEXT_TEXTURE_ID;
use crate::render::layer::RenderLayer;

/// How a texture is sampled when shown in imgui (`Canvas::texture_id`).
/// Stored in the high bits of the imgui texture id, so the same texture can
/// appear multiple times with different samplers.
#[repr(u8)]
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplerKind {
    #[default]
    LinearClamp = 0,
    NearestClamp = 1,
    NearestRepeat = 2,
    LinearMirror = 3,
}

impl SamplerKind {
    pub const COUNT: usize = 4;
}

/// Bits 56.. of an imgui texture id select the sampler; the rest is the
/// texture id proper. Ids come from `NEXT_TEXTURE_ID`, so they never grow
/// into the sampler bits in practice.
pub(crate) const SAMPLER_SHIFT: u32 = 56;
pub(crate) const TEXTURE_ID_MASK: u64 = (1 << SAMPLER_SHIFT) - 1;

/// An offscreen render target. Draw into it with `Draw::canvas`, then show it
/// in imgui via `Canvas::texture_id`.
///
/// This is only a handle: the GPU texture is created lazily by the renderer
/// the first time a frame references it, and resized when the handle's size
/// changes.
#[derive(Debug, Clone)]
pub struct Canvas {
    id: u64,
    size: math::Size<u32>,
    pub clear_color: math::Vector4<f32>,
}

impl Canvas {
    pub fn new<S>(size: S) -> Self
    where
        S: Into<math::Size<u32>>,
    {
        Self {
            // Shares the imgui texture id space so imgui draw commands can
            // reference canvases directly.
            id: NEXT_TEXTURE_ID.fetch_add(1, Ordering::Relaxed),
            size: size.into(),
            clear_color: Color::Black.into(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn size(&self) -> math::Size<u32> {
        self.size
    }

    /// Imgui texture id showing this canvas with the given sampler:
    /// `ui.image(canvas.texture_id(SamplerKind::NearestClamp), size)`.
    pub fn texture_id(&self, sampler: SamplerKind) -> imgui::TextureId {
        imgui::TextureId::new(self.id | (sampler as u64) << SAMPLER_SHIFT)
    }
}

/// CPU snapshot of one canvas for a frame: its geometry plus everything the
/// renderer needs to (re)create and clear the backing texture.
#[derive(Debug, Clone)]
pub struct CanvasPacket {
    pub id: u64,
    pub size: math::Size<u32>,
    pub clear_color: math::Vector4<f32>,
    pub camera: CameraData,
    pub layer: RenderLayer,
    /// Whether `Draw::canvas` targeted this canvas during the frame.
    ///
    /// Packets outlive the frames that created them so their allocations can
    /// be reused, so this is what separates "drawn into, clear and redraw"
    /// from "not touched this frame, leave the texture alone". Without it a
    /// canvas painted once would be cleared to empty on the next frame.
    pub touched: bool,
}

impl CanvasPacket {
    pub(crate) fn new(canvas: &Canvas) -> Self {
        Self {
            id: canvas.id,
            size: canvas.size,
            clear_color: canvas.clear_color,
            camera: Camera::new(Projection::standard_2d(canvas.size)).data(),
            layer: RenderLayer::new(),
            touched: true,
        }
    }

    /// Refreshes everything but the geometry from the user's handle.
    pub(crate) fn sync(&mut self, canvas: &Canvas) {
        self.size = canvas.size;
        self.clear_color = canvas.clear_color;
        self.camera = Camera::new(Projection::standard_2d(canvas.size)).data();
        self.touched = true;
    }
}
