mod camera;
mod color;
mod draw;
mod imgui;
mod immediate;
mod layer;
mod mesh;
mod vertex;

use std::ops::Index;
use std::ops::IndexMut;

use gpu::Gpu;
use logging::warn;
use sdl3::gpu::ColorTargetInfo;
use sdl3::gpu::DepthStencilTargetInfo;
use sdl3::gpu::LoadOp;
use sdl3::gpu::StoreOp;
use utils::FastHashMap;

use crate::assets::AssetReader;
use crate::render::camera::CameraData;

pub use crate::render::camera::Camera;
pub use crate::render::camera::Projection;
pub use crate::render::color::Color;
pub use crate::render::draw::Draw;
pub use crate::render::imgui::ImguiBatch;
pub use crate::render::imgui::ImguiPacket;
pub use crate::render::imgui::ImguiTextureUpdate;
pub use crate::render::imgui::ImguiVertex;
pub use crate::render::layer::Layer;
pub use crate::render::layer::RenderLayer;
pub use crate::render::layer::RenderLayers;
use crate::render::mesh::geometry::GeometryBuffers;
pub use crate::render::mesh::Mesh;
pub use crate::render::mesh::MeshDraw;
pub use crate::render::mesh::MeshStorage;
pub use crate::render::mesh::geometry::Geometry;
pub use crate::render::mesh::material::Material;
pub use crate::render::mesh::material::MaterialUniforms;
pub use crate::render::mesh::transform::Transform;
pub use crate::render::vertex::LayoutDesc;
pub use crate::render::vertex::MeshVertex;
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
/// asked to create (font atlas and friends), and a linear sampler since the
/// shared engine sampler is nearest.
struct ImguiResources {
    vertices: gpu::Buffer<ImguiVertex>,
    indices: gpu::Buffer<u16>,
    textures: FastHashMap<u64, gpu::Texture>,
    sampler: sdl3::gpu::Sampler,
}

impl ImguiResources {
    fn new(gpu: &Gpu) -> Self {
        let sampler = gpu
            .device()
            .create_sampler(
                sdl3::gpu::SamplerCreateInfo::new()
                    .with_min_filter(sdl3::gpu::Filter::Linear)
                    .with_mag_filter(sdl3::gpu::Filter::Linear)
                    .with_mipmap_mode(sdl3::gpu::SamplerMipmapMode::Nearest)
                    .with_address_mode_u(sdl3::gpu::SamplerAddressMode::ClampToEdge)
                    .with_address_mode_v(sdl3::gpu::SamplerAddressMode::ClampToEdge)
                    .with_address_mode_w(sdl3::gpu::SamplerAddressMode::ClampToEdge),
            )
            .expect("Failed to create imgui sampler");

        Self {
            vertices: gpu::Buffer::new(gpu, "Imgui vertex buffer", gpu::BufferUsages::VERTEX, 1024),
            indices: gpu::Buffer::new(gpu, "Imgui index buffer", gpu::BufferUsages::INDEX, 1024),
            textures: FastHashMap::default(),
            sampler,
        }
    }
}

/// Maps imgui coordinates (pixels, y-down) to NDC. Matches the `Transform`
/// uniform block in imgui.vert (std140: two vec2s).
#[repr(C)]
struct ImguiTransform {
    scale: [f32; 2],
    translate: [f32; 2],
}

const GEOMETRY_TTL_FRAMES: u64 = 300;

#[repr(C)]
struct ModelData {
    model: math::Matrix4<f32>,
    normal: math::Matrix4<f32>,
}

pub struct Renderer {
    format: gpu::TextureFormat,
    pipelines: gpu::PipelineCache,
    assets: AssetReader,
    buffers: [LayerBuffers; 3],
    geometries: FastHashMap<u64, GeometryBuffers>,
    depth: gpu::DepthTexture,
    frame: u64,
    pages: Vec<PageTexture>,
    imgui: ImguiResources,
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
            geometries: FastHashMap::default(),
            depth: gpu::DepthTexture::new(gpu, math::Size::new(1, 1)),
            frame: 0,
            pages: Vec::new(),
            imgui: ImguiResources::new(gpu),
        }
    }

    fn immediate_desc(format: gpu::TextureFormat) -> gpu::PipelineDesc {
        gpu::PipelineDesc {
            shader: gpu::ShaderRef::Builtin(0),
            vertex_layout: Vertex::desc(),
            blend: gpu::BlendState::ALPHA_BLENDING,
            topology: gpu::PrimitiveTopology::TriangleList,
            cull: None,
            depth: gpu::DepthState::Disabled,
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
            depth: gpu::DepthState::Disabled,
            format,
        }
    }

    fn mesh_desc(&self, material: &Material) -> gpu::PipelineDesc {
        gpu::PipelineDesc {
            shader: material.shader,
            vertex_layout: MeshVertex::desc(),
            blend: material.blend,
            topology: gpu::PrimitiveTopology::TriangleList,
            cull: material.cull,
            depth: gpu::DepthState::ReadWrite,
            format: self.format,
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
            self.imgui
                .vertices
                .write_all(gpu, copy_pass, &packet.vertices);
            self.imgui
                .indices
                .write_all(gpu, copy_pass, &packet.indices);
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

        for batch in &packet.batches {
            let x0 = batch.clip[0].max(0.0) as i32;
            let y0 = batch.clip[1].max(0.0) as i32;
            let x1 = batch.clip[2].min(viewport.width as f32).ceil() as i32;
            let y1 = batch.clip[3].min(viewport.height as f32).ceil() as i32;

            if x1 <= x0 || y1 <= y0 {
                continue;
            }

            let Some(texture) = self.imgui.textures.get(&batch.texture) else {
                warn!("Imgui draw references unknown texture {}", batch.texture);
                continue;
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
                    .with_sampler(&self.imgui.sampler)],
            );

            pass.draw_indexed_primitives(
                batch.index_count,
                1,
                batch.first_index,
                batch.vertex_offset,
                0,
            );
        }

        // Restore the full-window scissor for whoever draws next
        pass.set_scissor(sdl3::rect::Rect::new(0, 0, viewport.width, viewport.height));
    }

    fn sync_meshes(&mut self, gpu: &Gpu, copy_pass: &gpu::CopyPass, layers: &RenderLayers) {
        self.frame += 1;
        let frame = self.frame;

        for l in Layer::ALL {
            for draw in &layers[l].meshes {
                let g = &draw.geometry;

                let entry = self.geometries.entry(g.id()).or_insert_with(|| {
                    GeometryBuffers {
                        vertex: gpu::Buffer::new(
                            gpu,
                            format!("geometry {} vertices", g.id()),
                            gpu::BufferUsages::VERTEX,
                            g.vertices().len(),
                        ),
                        index: gpu::Buffer::new(
                            gpu,
                            format!("geometry {} indices", g.id()),
                            gpu::BufferUsages::INDEX,
                            g.indices().len(),
                        ),
                        version: g.version().wrapping_sub(1),
                        last_seen: frame,
                    }
                });

                entry.last_seen = frame;

                if entry.version != g.version() {
                    entry.vertex.write_all(gpu, copy_pass, g.vertices());
                    entry.index.write_all(gpu, copy_pass, g.indices());
                    entry.version = g.version();
                }
            }
        }

        self.geometries
            .retain(|_, b| frame - b.last_seen < GEOMETRY_TTL_FRAMES);
    }

    fn draw_meshes(
        &mut self,
        gpu: &Gpu,
        cmd: &gpu::CommandBuffer,
        pass: &gpu::RenderPass,
        layer: &RenderLayer,
        camera: &CameraData,
        assets: &crate::assets::Assets,
    ) {
        if layer.meshes.is_empty() {
            return;
        }

        let mut order: Vec<usize> = (0..layer.meshes.len()).collect();

        order.sort_unstable_by(|&a, &b| {
            let (ma, mb) = (&layer.meshes[a].material, &layer.meshes[b].material);

            ma.sort_key()
                .cmp(&mb.sort_key())
                .then_with(|| ma.uniforms.cmp(&mb.uniforms))
        });

        cmd.push_vertex_uniform_data(0, camera);

        let mut current: Option<&Material> = None;

        for &i in &order {
            let draw = &layer.meshes[i];

            if current != Some(&draw.material) {
                let desc = self.mesh_desc(&draw.material);
                pass.bind_graphics_pipeline(self.pipelines.get_or_create(gpu, &desc));

                let (page, rect) = match draw.material.texture {
                    Some(handle) => {
                        let img = assets.get_image(handle);

                        (img.page as usize, [
                            img.uv_min.x,
                            img.uv_min.y,
                            img.uv_max.x - img.uv_min.x,
                            img.uv_max.y - img.uv_min.y,
                        ])
                    }

                    None => {
                        let white = assets.white_pixel();

                        (white.page as usize, [
                            (white.uv_min.x + white.uv_max.x) * 0.5,
                            (white.uv_min.y + white.uv_max.y) * 0.5,
                            0.0,
                            0.0,
                        ])
                    }
                };

                pass.bind_fragment_samplers(0, &[self.pages[page].texture.binding(gpu)]);
                gpu::push_fragment_uniform_bytes(cmd, 0, &draw.material.uniforms);
                cmd.push_fragment_uniform_data(1, &rect);

                current = Some(&draw.material);
            }

            let Some(buffers) = self.geometries.get(&draw.geometry.id()) else {
                continue;
            };

            cmd.push_vertex_uniform_data(1, &ModelData {
                model: draw.model,
                normal: draw.normal,
            });

            pass.bind_vertex_buffers(0, &[buffers.vertex.binding()]);
            pass.bind_index_buffer(&buffers.index.binding(), gpu::IndexElementSize::_32BIT);
            pass.draw_indexed_primitives(draw.geometry.indices().len() as u32, 1, 0, 0, 0);
        }
    }

    /// Re-uploads atlas pages whose CPU pixels changed since the last frame
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
        self.sync_meshes(gpu, &copy_pass, &frame.layers);

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

        let swapchain_size = math::Size::new(swapchain.width(), swapchain.height());

        if self.depth.size != swapchain_size {
            self.depth = gpu::DepthTexture::new(gpu, swapchain_size);
        }

        let depth_target = DepthStencilTargetInfo::new()
            .with_texture(self.depth.inner_mut())
            .with_clear_depth(1.0)
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::DONT_CARE)
            .with_stencil_load_op(LoadOp::DONT_CARE)
            .with_stencil_store_op(StoreOp::DONT_CARE)
            .with_cycle(true);

        let pass = gpu
            .device()
            .begin_render_pass(&cmd, &targets, Some(&depth_target))
            .expect("Failed to begin render pass");

        for l in Layer::ALL {
            let layer = &frame.layers[l];
            let camera = frame.packet[l].camera;

            self.draw_meshes(gpu, &cmd, &pass, layer, &camera, &assets);

            if layer.indices.is_empty() {
                continue;
            }

            let buffers = &self.buffers[l as usize];

            pass.bind_graphics_pipeline(self.pipelines.get(&Self::immediate_desc(self.format)));
            cmd.push_vertex_uniform_data(0, &camera);
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

        self.draw_imgui(&cmd, &pass, &frame.layers.imgui, frame.packet.viewport);

        gpu.device().end_render_pass(pass);

        if let Err(e) = cmd.submit() {
            warn!("Failed to submit frame: {e}");
        }
    }
}
