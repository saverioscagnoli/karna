mod camera;
mod color;
mod immediate;

use std::array;
use std::sync::Arc;

use gpu::GpuState;
use gpu::Vertex;
use logging::warn;

pub use crate::camera::Camera;
use crate::camera::CameraData;
pub use crate::camera::Projection;
pub use crate::color::Color;
use crate::immediate::ImmediateRenderer;

#[repr(usize)]
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    #[default]
    World = 0,
    Ui = 1,
    Debug = 2,
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct Shape<V: Copy> {
    vertices: Vec<V>,
    indices: Vec<u32>,
}

impl<V: Copy> Shape<V> {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn push(&mut self, vertices: &[V], pattern: &[u32]) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(pattern.iter().map(|i| base + i));
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct LayerPacket {
    pub camera: CameraData,
    pub triangles: Shape<Vertex>,
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct FramePacket {
    pub viewport: math::Size<u32>,
    pub clear_color: math::Vector4<f32>,
    pub world: LayerPacket,
    pub ui: LayerPacket,
    pub debug: LayerPacket,
}

impl FramePacket {
    pub fn layer_mut(&mut self, layer: Layer) -> &mut LayerPacket {
        match layer {
            Layer::World => &mut self.world,
            Layer::Ui => &mut self.ui,
            Layer::Debug => &mut self.debug,
        }
    }
}

impl Layer {
    const ALL: [Layer; 3] = [Layer::World, Layer::Ui, Layer::Debug];
}

pub struct LayerGpu {
    camera_buffer: gpu::Buffer<CameraData>,
    camera_bg: wgpu::BindGroup,
    immediate: ImmediateRenderer,
}

impl LayerGpu {
    fn new(gpu: &GpuState, camera_layout: &wgpu::BindGroupLayout) -> Self {
        let camera_buffer = gpu::Buffer::new_with_capacity(
            "camera uniform buffer",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            1,
        );

        let camera_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("layer camera bind group"),
            layout: camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.wgpu().as_entire_binding(),
            }],
        });

        Self {
            camera_buffer,
            camera_bg,
            immediate: ImmediateRenderer::new(),
        }
    }
}

pub struct WindowRenderData {
    layers: [LayerGpu; 3],
}

impl WindowRenderData {
    fn new(gpu: &GpuState, layouts: &Layouts) -> Self {
        Self {
            layers: array::from_fn(|_| LayerGpu::new(gpu, &layouts.camera)),
        }
    }
}

pub struct Layouts {
    pub camera: wgpu::BindGroupLayout,
}

impl Layouts {
    pub fn new() -> Self {
        let gpu = GpuState::get();
        let camera_layout = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        Self {
            camera: camera_layout,
        }
    }
}

impl Layouts {
    const fn as_array(&self) -> [&wgpu::BindGroupLayout; 1] {
        [&self.camera]
    }
}

pub struct Renderer {
    pipelines: Arc<gpu::PipelineCache>,
    layouts: Arc<Layouts>,
    data: WindowRenderData,
}

impl Renderer {
    pub fn new(pipelines: Arc<gpu::PipelineCache>, layouts: Arc<Layouts>) -> Self {
        Self {
            pipelines,
            data: WindowRenderData::new(GpuState::get(), &layouts),
            layouts,
        }
    }

    pub fn present(&mut self, surface: &mut gpu::WindowSurface, packet: FramePacket) {
        let gpu = GpuState::get();

        let output = match surface.acquire() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                surface.resize(gpu, packet.viewport);
                t
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                surface.resize(gpu, packet.viewport);
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
                label: Some("command encoder"),
            });

        for (i, &layer) in Layer::ALL.iter().enumerate() {
            let load_op = if i == 0 {
                wgpu::LoadOp::Clear(wgpu::Color {
                    r: packet.clear_color.x as f64,
                    g: packet.clear_color.y as f64,
                    b: packet.clear_color.z as f64,
                    a: packet.clear_color.w as f64,
                })
            } else {
                wgpu::LoadOp::Load
            };

            let layer_gpu = &mut self.data.layers[layer as usize];
            let layer_packet = match layer {
                Layer::World => &packet.world,
                Layer::Ui => &packet.ui,
                Layer::Debug => &packet.debug,
            };

            layer_gpu.camera_buffer.write(0, &[layer_packet.camera]);

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: load_op,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });

                pass.set_bind_group(0, &layer_gpu.camera_bg, &[]);

                layer_gpu.immediate.present(
                    &layer_packet.triangles,
                    &mut pass,
                    &self.pipelines,
                    &self.layouts,
                    output.texture.format(),
                );
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        gpu.queue.present(output);
    }
}
