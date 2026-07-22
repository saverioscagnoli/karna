mod camera;
mod canvas;
mod color;
mod draw;
mod imgui;
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
use utils::FastHashMap;

use crate::assets::AssetReader;
use crate::render::camera::CameraData;

pub use crate::render::camera::Camera;
pub use crate::render::camera::Projection;
pub use crate::render::canvas::Canvas;
pub use crate::render::canvas::CanvasPacket;
pub use crate::render::canvas::SamplerKind;
pub use crate::render::color::Color;
pub use crate::render::draw::Draw;
pub use crate::render::imgui::ImguiBatch;
pub use crate::render::imgui::ImguiPacket;
pub use crate::render::imgui::ImguiTextureUpdate;
pub use crate::render::imgui::ImguiVertex;
pub use crate::render::immediate::Batch;
pub use crate::render::immediate::BatchTexture;
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

/// GPU state backing imgui rendering: geometry buffers, the textures imgui
/// asked to create (font atlas and friends), and one sampler per
/// `SamplerKind`, selected through the high bits of the texture id.
struct ImguiResources {
    vertices: gpu::Buffer<ImguiVertex>,
    indices: gpu::Buffer<u16>,
    textures: FastHashMap<u64, gpu::Texture>,
    samplers: [sdl3::gpu::Sampler; SamplerKind::COUNT],
}

impl ImguiResources {
    fn new(gpu: &Gpu) -> Self {
        use sdl3::gpu::Filter;
        use sdl3::gpu::SamplerAddressMode as Wrap;

        // Indexed by `SamplerKind`; imgui's own textures (id high bits zero)
        // get LinearClamp.
        let samplers = [
            (Filter::Linear, Wrap::ClampToEdge),
            (Filter::Nearest, Wrap::ClampToEdge),
            (Filter::Nearest, Wrap::Repeat),
            (Filter::Linear, Wrap::MirroredRepeat),
        ]
        .map(|(filter, wrap)| {
            gpu.device()
                .create_sampler(
                    sdl3::gpu::SamplerCreateInfo::new()
                        .with_min_filter(filter)
                        .with_mag_filter(filter)
                        .with_mipmap_mode(sdl3::gpu::SamplerMipmapMode::Nearest)
                        .with_address_mode_u(wrap)
                        .with_address_mode_v(wrap)
                        .with_address_mode_w(wrap),
                )
                .expect("Failed to create imgui sampler")
        });

        Self {
            vertices: gpu::Buffer::new(gpu, "Imgui vertex buffer", gpu::BufferUsages::VERTEX, 1024),
            indices: gpu::Buffer::new(gpu, "Imgui index buffer", gpu::BufferUsages::INDEX, 1024),
            textures: FastHashMap::default(),
            samplers,
        }
    }
}

/// GPU state backing one offscreen canvas: its render target texture and the
/// buffers holding the geometry drawn into it this frame.
struct CanvasResources {
    texture: gpu::Texture,
    vertex: gpu::Buffer<Vertex>,
    index: gpu::Buffer<u32>,
}

fn to_sdl_color(c: math::Vector4<f32>) -> sdl3::pixels::Color {
    sdl3::pixels::Color::RGBA(
        (c.x.clamp(0.0, 1.0) * 255.0) as u8,
        (c.y.clamp(0.0, 1.0) * 255.0) as u8,
        (c.z.clamp(0.0, 1.0) * 255.0) as u8,
        (c.w.clamp(0.0, 1.0) * 255.0) as u8,
    )
}

/// Maps imgui coordinates (pixels, y-down) to NDC. Matches the `Transform`
/// uniform block in imgui.vert (std140: two vec2s).
#[repr(C)]
struct ImguiTransform {
    scale: [f32; 2],
    translate: [f32; 2],
}

pub struct Renderer {
    format: gpu::TextureFormat,
    pipelines: gpu::PipelineCache,
    assets: AssetReader,
    buffers: [LayerBuffers; 3],
    pages: Vec<PageTexture>,
    imgui: ImguiResources,
    canvases: FastHashMap<u64, CanvasResources>,
}

impl Renderer {
    pub(crate) fn new(gpu: &Gpu, format: gpu::TextureFormat, assets: AssetReader) -> Self {
        let mut pipelines = gpu::PipelineCache::new();
        pipelines.create(gpu, &Self::immediate_desc(format));
        pipelines.create(gpu, &Self::imgui_desc(format));

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
            imgui: ImguiResources::new(gpu),
            canvases: FastHashMap::default(),
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

    fn imgui_desc(format: gpu::TextureFormat) -> gpu::PipelineDesc {
        gpu::PipelineDesc {
            shader: gpu::ShaderRef::Builtin(1),
            vertex_layout: ImguiVertex::desc(),
            blend: gpu::BlendState::ALPHA_BLENDING,
            topology: gpu::PrimitiveTopology::TriangleList,
            cull: None,
            format,
        }
    }

    /// Runs the texture ops imgui requested this frame and uploads its
    /// geometry. Must happen inside the frame's copy pass, before drawing.
    fn sync_imgui(&mut self, gpu: &Gpu, copy_pass: &gpu::CopyPass, packet: &ImguiPacket) {
        for update in &packet.textures {
            match update {
                ImguiTextureUpdate::Create { id, size, pixels } => {
                    let texture =
                        gpu::Texture::new_blank(gpu, format!("imgui texture {id}"), *size);

                    texture.write(gpu, copy_pass, pixels, math::Vector2::new(0, 0), *size);
                    self.imgui.textures.insert(*id, texture);
                }

                ImguiTextureUpdate::Write {
                    id,
                    origin,
                    size,
                    pixels,
                } => match self.imgui.textures.get(id) {
                    Some(texture) => texture.write(gpu, copy_pass, pixels, *origin, *size),
                    None => warn!("Imgui update for unknown texture {}", id),
                },

                ImguiTextureUpdate::Destroy { id } => {
                    self.imgui.textures.remove(id);
                }
            }
        }

        if !packet.indices.is_empty() {
            self.imgui.vertices.write_all(gpu, copy_pass, &packet.vertices);
            self.imgui.indices.write_all(gpu, copy_pass, &packet.indices);
        }
    }

    /// Creates/resizes canvas render targets and uploads their geometry.
    fn sync_canvases(&mut self, gpu: &Gpu, copy_pass: &gpu::CopyPass, canvases: &[CanvasPacket]) {
        for packet in canvases {
            // Nothing to upload for a canvas no one drew into this frame, and
            // its texture must be left as-is for `draw_canvases` to keep.
            if !packet.touched {
                continue;
            }

            let entry = self.canvases.entry(packet.id).or_insert_with(|| {
                CanvasResources {
                    texture: gpu::Texture::new_target(
                        gpu,
                        format!("canvas {}", packet.id),
                        packet.size,
                        self.format,
                    ),
                    vertex: gpu::Buffer::new(
                        gpu,
                        "Canvas vertex buffer",
                        gpu::BufferUsages::VERTEX,
                        1024,
                    ),
                    index: gpu::Buffer::new(
                        gpu,
                        "Canvas index buffer",
                        gpu::BufferUsages::INDEX,
                        1024,
                    ),
                }
            });

            if entry.texture.size != packet.size {
                entry.texture = gpu::Texture::new_target(
                    gpu,
                    format!("canvas {}", packet.id),
                    packet.size,
                    self.format,
                );
            }

            entry.vertex.write_all(gpu, copy_pass, &packet.layer.vertices);
            entry.index.write_all(gpu, copy_pass, &packet.layer.indices);
        }
    }

    /// Resolves what a batch samples to a texture, or `None` if it references
    /// a canvas that has never been rendered.
    fn batch_texture(&self, texture: BatchTexture) -> Option<&gpu::Texture> {
        match texture {
            BatchTexture::Page(i) => self.pages.get(i).map(|p| &p.texture),
            BatchTexture::Canvas(id) => self.canvases.get(&id).map(|c| &c.texture),
        }
    }

    /// Renders each canvas in its own pass, before the main pass so their
    /// textures can be sampled by it (e.g. from imgui).
    fn draw_canvases(&self, gpu: &Gpu, cmd: &gpu::CommandBuffer, canvases: &[CanvasPacket]) {
        for packet in canvases {
            // Untouched canvases keep whatever they were last rendered with,
            // so a canvas can be painted once and reused for many frames.
            if !packet.touched {
                continue;
            }

            let resources = &self.canvases[&packet.id];

            let targets = [ColorTargetInfo::default()
                .with_texture(resources.texture.raw())
                .with_load_op(LoadOp::CLEAR)
                .with_store_op(StoreOp::STORE)
                .with_clear_color(to_sdl_color(packet.clear_color))];

            let pass = gpu
                .device()
                .begin_render_pass(cmd, &targets, None)
                .expect("Failed to begin canvas render pass");

            if !packet.layer.indices.is_empty() {
                pass.bind_graphics_pipeline(self.pipelines.get(&Self::immediate_desc(self.format)));
                cmd.push_vertex_uniform_data(0, &packet.camera);
                pass.bind_vertex_buffers(0, &[resources.vertex.binding()]);
                pass.bind_index_buffer(&resources.index.binding(), gpu::IndexElementSize::_32BIT);

                for batch in &packet.layer.batches {
                    let Some(texture) = self.batch_texture(batch.texture) else {
                        warn!("Canvas draw references unknown {:?}", batch.texture);
                        continue;
                    };

                    pass.bind_fragment_samplers(0, &[texture.binding(gpu)]);
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
        }
    }

    /// Draws the imgui frame on top of everything, scissoring each command.
    fn draw_imgui(
        &self,
        cmd: &gpu::CommandBuffer,
        pass: &gpu::RenderPass,
        packet: &ImguiPacket,
        viewport: math::Size<u32>,
    ) {
        let [w, h] = packet.display_size;

        if packet.batches.is_empty() || w <= 0.0 || h <= 0.0 {
            return;
        }

        let scale = [2.0 / w, -2.0 / h];
        let translate = [
            -1.0 - packet.display_pos[0] * scale[0],
            1.0 - packet.display_pos[1] * scale[1],
        ];

        pass.bind_graphics_pipeline(self.pipelines.get(&Self::imgui_desc(self.format)));
        cmd.push_vertex_uniform_data(0, &ImguiTransform { scale, translate });
        pass.bind_vertex_buffers(0, &[self.imgui.vertices.binding()]);
        pass.bind_index_buffer(&self.imgui.indices.binding(), gpu::IndexElementSize::_16BIT);

        use crate::render::canvas::SAMPLER_SHIFT;
        use crate::render::canvas::TEXTURE_ID_MASK;

        for batch in &packet.batches {
            let x0 = batch.clip[0].max(0.0) as i32;
            let y0 = batch.clip[1].max(0.0) as i32;
            let x1 = batch.clip[2].min(viewport.width as f32).ceil() as i32;
            let y1 = batch.clip[3].min(viewport.height as f32).ceil() as i32;

            if x1 <= x0 || y1 <= y0 {
                continue;
            }

            // The high bits of the id pick the sampler; the id proper is
            // either an imgui-managed texture or a canvas.
            let id = batch.texture & TEXTURE_ID_MASK;
            let sampler_index = (batch.texture >> SAMPLER_SHIFT) as usize;
            let sampler = &self.imgui.samplers[sampler_index.min(self.imgui.samplers.len() - 1)];

            let texture = match self.imgui.textures.get(&id) {
                Some(texture) => texture,
                None => match self.canvases.get(&id) {
                    Some(canvas) => &canvas.texture,
                    None => {
                        warn!("Imgui draw references unknown texture {}", id);
                        continue;
                    }
                },
            };

            pass.set_scissor(sdl3::rect::Rect::new(
                x0,
                y0,
                (x1 - x0) as u32,
                (y1 - y0) as u32,
            ));

            pass.bind_fragment_samplers(
                0,
                &[gpu::TextureSamplerBinding::new()
                    .with_texture(texture.raw())
                    .with_sampler(sampler)],
            );

            pass.draw_indexed_primitives(
                batch.index_count,
                1,
                batch.first_index,
                batch.vertex_offset,
                0,
            );
        }

        // Restore the full-window scissor for whoever draws next.
        pass.set_scissor(sdl3::rect::Rect::new(
            0,
            0,
            viewport.width,
            viewport.height,
        ));
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

        self.sync_imgui(gpu, &copy_pass, &frame.layers.imgui);
        self.sync_canvases(gpu, &copy_pass, &frame.layers.canvases);

        gpu.end_copy_pass(copy_pass);

        self.draw_canvases(gpu, &cmd, &frame.layers.canvases);

        let swapchain = match cmd.wait_and_acquire_swapchain_texture(window) {
            Ok(t) => t,
            Err(e) => {
                warn!("Failed to acquire swapchain texture ({e}), skipping this frame");
                cmd.cancel();
                return;
            }
        };

        let targets = [ColorTargetInfo::default()
            .with_texture(&swapchain)
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::STORE)
            .with_clear_color(to_sdl_color(frame.packet.clear_color))];

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
                let Some(texture) = self.batch_texture(batch.texture) else {
                    warn!("Draw references unknown {:?}", batch.texture);
                    continue;
                };

                pass.bind_fragment_samplers(0, &[texture.binding(gpu)]);
                pass.draw_indexed_primitives(
                    batch.indices.end - batch.indices.start,
                    1,
                    batch.indices.start,
                    0,
                    0,
                );
            }
        }

        self.draw_imgui(&cmd, &pass, &frame.layers.imgui, frame.packet.viewport);

        gpu.device().end_render_pass(pass);

        if let Err(e) = cmd.submit() {
            warn!("Failed to submit frame: {e}");
        }
    }
}
