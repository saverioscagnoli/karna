use gpu::Gpu;
use logging::warn;
use sdl3::gpu::ColorTargetInfo;
use sdl3::gpu::LoadOp;
use sdl3::gpu::StoreOp;
use utils::FastHashMap;
use utils::Handle;
use utils::WindowId;

use crate::Image;
use crate::assets::Assets;
use crate::assets::atlas::TextureAtlas;
use crate::assets::font::Font;
use crate::render::Pipelines;
use crate::render::color::Color;
use crate::render::layer::Layer;
use crate::render::layer::LayerGpuData;
use crate::render::packet::FramePacket;
use crate::window::platform::PlatformWindow;

/// Spaces a `\t` advances by.
const TAB_WIDTH: u32 = 4;

struct ImmediateTarget {
    layers: [LayerGpuData; 3],
}

impl ImmediateTarget {
    fn new(gpu: &Gpu, id: WindowId) -> Self {
        let layers = Layer::ALL.map(|l| LayerGpuData {
            vertex_buffer: gpu::Buffer::new(
                gpu,
                format!("{id:?}-{l:?}-vertices"),
                gpu::BufferUsages::VERTEX,
                4096,
            ),
            index_buffer: gpu::Buffer::new(
                gpu,
                format!("{id:?}-{l:?}-indices"),
                gpu::BufferUsages::INDEX,
                6144,
            ),
        });

        Self { layers }
    }
}

pub struct ImmediateRenderer {
    targets: FastHashMap<WindowId, ImmediateTarget>,
}

impl ImmediateRenderer {
    pub fn new() -> Self {
        Self {
            targets: FastHashMap::default(),
        }
    }

    pub fn present(
        &mut self,
        gpu: &Gpu,
        pips: &mut gpu::PipelineCache,
        window: &PlatformWindow,
        assets: &Assets,
        packet: &FramePacket,
    ) {
        let Self { targets } = self;
        let desc = Pipelines::immediate_desc(gpu.swapchain_format(window.inner()));
        let pip = pips.get_or_create(gpu, &desc);

        let target = targets
            .entry(window.id())
            .or_insert_with(|| ImmediateTarget::new(gpu, window.id()));

        let Ok(mut cmd) = gpu.device.acquire_command_buffer() else {
            warn!("Failed to acquire command buffer for {:?}", window.id());
            return;
        };

        let Ok(swapchain) = cmd.wait_and_acquire_swapchain_texture(window.inner()) else {
            cmd.cancel();
            return;
        };

        let color_targets = [ColorTargetInfo::default()
            .with_texture(&swapchain)
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::STORE)
            .with_clear_color(packet.clear_color.into())];

        let Ok(copy_pass) = gpu.device.begin_copy_pass(&cmd) else {
            warn!("Failed to begin a copy pass for {}", window.id());
            cmd.cancel();
            return;
        };

        for (&layer, data) in Layer::ALL.iter().zip(&mut target.layers) {
            let cpu = &packet[layer].data;

            data.vertex_buffer.write_all(gpu, &copy_pass, &cpu.vertices);
            data.index_buffer.write_all(gpu, &copy_pass, &cpu.indices);
        }

        gpu.device.end_copy_pass(copy_pass);

        let Ok(rpass) = gpu.device.begin_render_pass(&cmd, &color_targets, None) else {
            warn!("Failed to begin a render pass for {}", window.id());
            cmd.cancel();
            return;
        };

        rpass.bind_graphics_pipeline(pip);

        for (&layer, data) in Layer::ALL.iter().zip(&target.layers) {
            let layer = &packet[layer];

            if layer.data.batches.is_empty() {
                continue;
            }

            cmd.push_vertex_uniform_data(0, &layer.camera);

            rpass.bind_vertex_buffers(0, &[data.vertex_buffer.binding()]);
            rpass.bind_index_buffer(&data.index_buffer.binding(), gpu::IndexElementSize::_32BIT);

            for batch in &layer.data.batches {
                let Some(texture) = assets.page_texture(batch.page) else {
                    continue;
                };

                rpass.bind_fragment_samplers(0, &[texture.binding(gpu)]);
                rpass.draw_indexed_primitives(batch.count, 1, batch.start, 0, 0);
            }
        }

        gpu.device.end_render_pass(rpass);

        if let Err(e) = cmd.submit() {
            warn!("Failed to submit the frame for {:?}: {}", window.id(), e);
        }
    }
}

fn corners(x: f32, y: f32, w: f32, h: f32) -> [math::Vector2<f32>; 4] {
    [
        math::Vector2::new(x, y),
        math::Vector2::new(x + w, y),
        math::Vector2::new(x + w, y + h),
        math::Vector2::new(x, y + h),
    ]
}

pub struct Draw<'a> {
    pub(crate) packet: &'a mut FramePacket,
    pub(crate) assets: &'a Assets,
    layer: Layer,
    color: Color,
}

impl<'a> Draw<'a> {
    pub(crate) fn new(packet: &'a mut FramePacket, assets: &'a Assets) -> Self {
        Self {
            packet,
            assets,
            layer: Layer::World,
            color: Color::White,
        }
    }

    pub fn clear_color(&self) -> Color {
        self.packet.clear_color
    }

    pub fn clear_color_mut(&mut self) -> &mut Color {
        &mut self.packet.clear_color
    }

    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.packet.clear_color = color.into();
    }

    pub fn layer(&self) -> Layer {
        self.layer
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.layer = layer;
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn color_mut(&mut self) -> &mut Color {
        &mut self.color
    }

    pub fn set_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.color = color.into();
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let white = self.assets.white_pixel();
        let uv = white.uv_center();

        self.packet[self.layer].data.quad(
            white.page,
            corners(x, y, w, h),
            [uv; 4],
            self.color.into(),
        );
    }

    pub fn image(&mut self, handle: Handle<Image>, x: f32, y: f32) {
        let image = self.assets.get_image(handle);
        let (min, max) = (image.uv_min, image.uv_max);

        let uvs = [
            math::Vector2::new(min.x, min.y),
            math::Vector2::new(max.x, min.y),
            math::Vector2::new(max.x, max.y),
            math::Vector2::new(min.x, max.y),
        ];

        self.packet[self.layer].data.quad(
            image.page,
            corners(x, y, image.size.width as f32, image.size.height as f32),
            uvs,
            self.color.into(),
        );
    }

    pub fn text(&mut self, font: Handle<Font>, text: &str, x: f32, y: f32) {
        let font = self.assets.get_font(font);
        let line_height = font.line_height().round();
        let tab = font.glyph(' ').map(|g| g.advance.round()).unwrap_or(0.0) * TAB_WIDTH as f32;

        let left = x.round();
        let mut pen_x = left;
        let mut pen_y = y.round();

        for ch in text.chars() {
            match ch {
                '\n' => {
                    pen_x = left;
                    pen_y += line_height;
                    continue;
                }

                '\t' => {
                    pen_x += tab;
                    continue;
                }

                _ if ch.is_control() => continue,

                _ => {}
            }

            let Some(glyph) = font.glyph(ch) else {
                continue;
            };

            if let Some(handle) = glyph.image {
                let image = self.assets.get_image(handle);
                let (min, max) = (image.uv_min, image.uv_max);

                let uvs = [
                    math::Vector2::new(min.x, min.y),
                    math::Vector2::new(max.x, min.y),
                    math::Vector2::new(max.x, max.y),
                    math::Vector2::new(min.x, max.y),
                ];

                self.packet[self.layer].data.quad(
                    image.page,
                    corners(
                        pen_x + glyph.bearing.x,
                        pen_y + glyph.bearing.y,
                        image.size.width as f32,
                        image.size.height as f32,
                    ),
                    uvs,
                    self.color.into(),
                );
            }

            pen_x += glyph.advance.round()
        }
    }

    pub fn debug_text<T>(&mut self, text: T, x: f32, y: f32)
    where
        T: AsRef<str>,
    {
        self.text(self.assets.debug_font(), text.as_ref(), x, y);
    }

    pub fn atlas_page(&mut self, page: usize, x: f32, y: f32) {
        let uvs = [
            math::Vector2::new(0.0, 0.0),
            math::Vector2::new(1.0, 0.0),
            math::Vector2::new(1.0, 1.0),
            math::Vector2::new(0.0, 1.0),
        ];

        self.packet[self.layer].data.quad(
            page,
            corners(
                x,
                y,
                TextureAtlas::PAGE_SIZE as f32,
                TextureAtlas::PAGE_SIZE as f32,
            ),
            uvs,
            self.color.into(),
        );
    }
}
