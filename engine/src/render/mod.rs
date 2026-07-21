mod camera;
mod color;
mod draw;
mod immediate;
mod layer;
mod transform;
mod vertex;

use std::ops::Index;
use std::ops::IndexMut;

use gpu::Gpu;
use logging::warn;
use sdl3::gpu::ColorTargetInfo;
use sdl3::gpu::LoadOp;
use sdl3::gpu::StoreOp;

use crate::assets::AssetReader;
use crate::render::camera::CameraData;

pub use crate::render::camera::Camera;
pub use crate::render::camera::Projection;
pub use crate::render::color::Color;
pub use crate::render::draw::Draw;
pub use crate::render::layer::Layer;
pub use crate::render::layer::RenderLayer;
pub use crate::render::layer::RenderLayers;
pub use crate::render::vertex::LayoutDesc;
pub use crate::render::vertex::Vertex;

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct LayerPacket {
    pub camera: CameraData,
}

#[derive(Debug, Clone)]
pub struct FramePacket {
    pub viewport: math::Size<u32>,
    pub clear_color: math::Vector4<f32>,
    pub world: LayerPacket,
    pub ui: LayerPacket,
    pub debug: LayerPacket,

    /// SDL timestamp (ns) of the oldest input event this frame is the first to reflect.
    pub input_timestamp: Option<u64>,
}

impl Default for FramePacket {
    fn default() -> Self {
        Self {
            viewport: math::Size::new(1280, 720),
            clear_color: Color::Black.into(),
            world: LayerPacket::default(),
            ui: LayerPacket::default(),
            debug: LayerPacket::default(),
            input_timestamp: None,
        }
    }
}

impl Index<Layer> for FramePacket {
    type Output = LayerPacket;

    fn index(&self, l: Layer) -> &Self::Output {
        match l {
            Layer::World => &self.world,
            Layer::Ui => &self.ui,
            Layer::Debug => &self.debug,
        }
    }
}

impl IndexMut<Layer> for FramePacket {
    fn index_mut(&mut self, l: Layer) -> &mut Self::Output {
        match l {
            Layer::World => &mut self.world,
            Layer::Ui => &mut self.ui,
            Layer::Debug => &mut self.debug,
        }
    }
}

impl FramePacket {
    pub fn clear(&mut self) {
        self.input_timestamp = None;
    }
}

/// A finished frame handed from a window's logic thread to the main thread:
/// per-layer camera state plus the CPU geometry to upload and draw.
#[derive(Debug, Clone)]
pub struct FrameSubmission {
    pub packet: FramePacket,
    pub layers: RenderLayers,
}

/// GPU buffers backing one layer's immediate-mode geometry.
struct LayerBuffers {
    vertex: gpu::Buffer<Vertex>,
    index: gpu::Buffer<u32>,
}

impl LayerBuffers {
    fn new(gpu: &Gpu) -> Self {
        Self {
            vertex: gpu::Buffer::new(
                gpu,
                "Immediate mode vertex buffer",
                gpu::BufferUsages::VERTEX,
                1024,
            ),
            index: gpu::Buffer::new(
                gpu,
                "Immediate mode index buffer",
                gpu::BufferUsages::INDEX,
                1024,
            ),
        }
    }
}

/// An atlas page texture and the CPU-side version it was last synced with.
struct PageTexture {
    texture: gpu::Texture,
    version: u64,
}

pub struct Renderer {
    format: gpu::TextureFormat,
    pipelines: gpu::PipelineCache,
    assets: AssetReader,
    buffers: [LayerBuffers; 3],
    pages: Vec<PageTexture>,
}

impl Renderer {
    pub(crate) fn new(gpu: &Gpu, format: gpu::TextureFormat, assets: AssetReader) -> Self {
        let mut pipelines = gpu::PipelineCache::new();
        pipelines.create(gpu, &Self::immediate_desc(format));

        Self {
            format,
            pipelines,
            assets,
            buffers: [
                LayerBuffers::new(gpu),
                LayerBuffers::new(gpu),
                LayerBuffers::new(gpu),
            ],
            pages: Vec::new(),
        }
    }

    fn immediate_desc(format: gpu::TextureFormat) -> gpu::PipelineDesc {
        gpu::PipelineDesc {
            shader: gpu::ShaderRef::Builtin(0),
            vertex_layout: Vertex::desc(),
            blend: gpu::BlendState::ALPHA_BLENDING,
            topology: gpu::PrimitiveTopology::TriangleList,
            cull: None,
            format,
        }
    }

    /// Re-uploads atlas pages whose CPU pixels changed since the last frame.
    fn sync_atlas(&mut self, gpu: &Gpu, copy_pass: &gpu::CopyPass, assets: &crate::assets::Assets) {
        for (i, page) in assets.atlas_pages().enumerate() {
            if self.pages.len() <= i {
                let size = math::Size::new(page.size(), page.size());
                let texture = gpu::Texture::new_blank(gpu, format!("atlas page {i}"), size);

                self.pages.push(PageTexture {
                    texture,
                    version: page.version().wrapping_sub(1),
                });
            }

            let entry = &mut self.pages[i];

            if entry.version != page.version() {
                entry.texture.write(
                    gpu,
                    copy_pass,
                    page.bytes(),
                    math::Vector2::new(0, 0),
                    entry.texture.size,
                );
                entry.version = page.version();
            }
        }
    }

    pub(crate) fn present(
        &mut self,
        gpu: &Gpu,
        window: &sdl3::video::Window,
        frame: &FrameSubmission,
    ) {
        let assets = self.assets.snapshot();
        let mut cmd = gpu.acquire_command_buffer();

        // Upload phase: atlas pages and this frame's geometry.
        let copy_pass = gpu.begin_copy_pass(&cmd);

        self.sync_atlas(gpu, &copy_pass, &assets);

        for l in Layer::ALL {
            let layer = &frame.layers[l];

            if layer.indices.is_empty() {
                continue;
            }

            let buffers = &mut self.buffers[l as usize];
            buffers.vertex.write_all(gpu, &copy_pass, &layer.vertices);
            buffers.index.write_all(gpu, &copy_pass, &layer.indices);
        }

        gpu.end_copy_pass(copy_pass);

        let swapchain = match cmd.wait_and_acquire_swapchain_texture(window) {
            Ok(t) => t,
            Err(e) => {
                warn!("Failed to acquire swapchain texture ({e}), skipping this frame");
                cmd.cancel();
                return;
            }
        };

        let c = frame.packet.clear_color;
        let clear = sdl3::pixels::Color::RGBA(
            (c.x.clamp(0.0, 1.0) * 255.0) as u8,
            (c.y.clamp(0.0, 1.0) * 255.0) as u8,
            (c.z.clamp(0.0, 1.0) * 255.0) as u8,
            (c.w.clamp(0.0, 1.0) * 255.0) as u8,
        );

        let targets = [ColorTargetInfo::default()
            .with_texture(&swapchain)
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::STORE)
            .with_clear_color(clear)];

        let pass = gpu
            .device()
            .begin_render_pass(&cmd, &targets, None)
            .expect("Failed to begin render pass");

        let pip = self.pipelines.get(&Self::immediate_desc(self.format));

        for l in Layer::ALL {
            let layer = &frame.layers[l];

            if layer.indices.is_empty() {
                continue;
            }

            let buffers = &self.buffers[l as usize];

            pass.bind_graphics_pipeline(pip);
            cmd.push_vertex_uniform_data(0, &frame.packet[l].camera);
            pass.bind_vertex_buffers(0, &[buffers.vertex.binding()]);
            pass.bind_index_buffer(&buffers.index.binding(), gpu::IndexElementSize::_32BIT);

            for batch in &layer.batches {
                // Page 0 always exists and holds the white pixel.
                let page = &self.pages[batch.page.unwrap_or(0)];

                pass.bind_fragment_samplers(0, &[page.texture.binding(gpu)]);
                pass.draw_indexed_primitives(
                    batch.indices.end - batch.indices.start,
                    1,
                    batch.indices.start,
                    0,
                    0,
                );
            }
        }

        gpu.device().end_render_pass(pass);

        if let Err(e) = cmd.submit() {
            warn!("Failed to submit frame: {e}");
        }
    }
}
