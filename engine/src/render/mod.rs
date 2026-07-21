mod camera;
mod color;
mod draw;
mod immediate;
mod layer;
pub mod layouts;
mod transform;
mod vertex;

use std::ops::Index;
use std::ops::IndexMut;

use gpu::GpuState;
use logging::debug;
use logging::warn;

use crate::assets::AssetReader;
use crate::render::camera::CameraData;
use crate::render::layer::RenderLayer;
use crate::render::layer::RenderLayers;

pub use crate::render::camera::Camera;
pub use crate::render::camera::Projection;
pub use crate::render::color::Color;
pub use crate::render::draw::Draw;
pub use crate::render::layer::Layer;
pub use crate::render::layouts::LayoutDesc;
pub use crate::render::layouts::Layouts;
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

#[derive(Default)]
struct RenderData {
    layers: RenderLayers,
}

pub struct Renderer {
    surface: gpu::WindowSurface,
    pipelines: gpu::PipelineCache,
    assets: AssetReader,
    data: RenderData,
}

impl Renderer {
    pub(crate) fn new(surface: gpu::WindowSurface, assets: AssetReader) -> Self {
        let pipelines = Self::init_pipelines(surface.format());
        debug!("Builtin render pipelines initialized.");

        Self {
            surface,
            pipelines,
            assets,
            data: RenderData::default(),
        }
    }

    fn init_pipelines(format: gpu::TextureFormat) -> gpu::PipelineCache {
        let mut cache = gpu::PipelineCache::new();

        let desc = gpu::PipelineDesc {
            shader: gpu::ShaderRef::Builtin(0),
            vertex_layout: Vertex::desc(),
            blend: gpu::BlendState::ALPHA_BLENDING,
            topology: gpu::PrimitiveTopology::TriangleList,
            cull: None,
            format,
        };

        cache.create(&desc, &layouts::immediate());
        cache
    }

    pub(crate) fn resize(&mut self, gpu: &gpu::GpuState, size: math::Size<u32>) {
        self.surface.resize(gpu, size);
    }

    pub(crate) fn layer_mut(&mut self, l: Layer) -> &mut RenderLayer {
        &mut self.data.layers[l]
    }

    pub(crate) fn present(&mut self, packet: &FramePacket) {
        let gpu = GpuState::get();
        let assets = self.assets.snapshot();

        let output = match self.surface.acquire() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.resize(gpu, packet.viewport);
                warn!("Received an outdated texture, skipping this frame");
                return;
            }

            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                warn!("Failed to acquire a texture, skipping this frame");
                return;
            }

            wgpu::CurrentSurfaceTexture::Lost => {
                panic!("Device lost");
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("clear"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(gpu::Color {
                            r: packet.clear_color.x as f64,
                            g: packet.clear_color.y as f64,
                            b: packet.clear_color.z as f64,
                            a: packet.clear_color.w as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });

            for l in Layer::ALL {
                let layer = &mut self.data.layers[l];
                let layer_packet = &packet[l];

                if layer.indices.is_empty() {
                    layer.clear();
                    continue;
                }

                layer.camera_buffer.write_all(&[layer_packet.camera]);

                layer.vertex_buffer.write_all(&layer.vertices);
                layer.index_buffer.write_all(&layer.indices);

                let immediate_key = gpu::PipelineDesc {
                    shader: gpu::ShaderRef::Builtin(0),
                    vertex_layout: Vertex::desc(),
                    blend: gpu::BlendState::ALPHA_BLENDING,
                    topology: gpu::PrimitiveTopology::TriangleList,
                    cull: None,
                    format: view.texture().format(),
                };

                let pip = self.pipelines.get(&immediate_key);

                pass.set_pipeline(pip);
                pass.set_bind_group(0, &layer.camera_bind_group, &[]);
                pass.set_vertex_buffer(0, layer.vertex_buffer.slice_all());
                pass.set_index_buffer(layer.index_buffer.slice_all(), gpu::IndexFormat::Uint32);

                for batch in &layer.batches {
                    let bind_group = match batch.page {
                        Some(p) => assets.atlas_page_bind_group(p),
                        None => assets.white_pixel_bind_group(),
                    };

                    pass.set_bind_group(1, bind_group, &[]);
                    pass.draw_indexed(batch.indices.clone(), 0, 0..1);
                }
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        gpu.queue.present(output);

        self.data.layers.clear();
    }
}
