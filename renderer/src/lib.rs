mod camera;
mod color;
mod immediate;

use std::array;
use std::hash::Hash;

use gpu::GpuState;
use gpu::Vertex;
use logging::warn;
use utils::FastHashMap;

use crate::camera::CameraData;

#[derive(Debug, Clone, Copy)]
pub enum DrawCommand {
    ImmediatePoint { x: f32, y: f32 },
    ImmediateLine { x1: f32, y1: f32, x2: f32, y2: f32 },
    ImmediateRect { x: f32, y: f32, w: f32, h: f32 },
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    World = 0,
    Ui = 1,
    Debug = 2,
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct LayerPacket {
    pub camera: CameraData,
    pub commands: Vec<DrawCommand>,
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct FramePacket {
    pub viewport: math::Size<u32>,
    pub world: LayerPacket,
    pub ui: LayerPacket,
    pub debug: LayerPacket,
}

impl FramePacket {
    pub fn expose<'a>(&'a mut self, layer: Layer) -> &'a mut Vec<DrawCommand> {
        match layer {
            Layer::World => &mut self.world.commands,
            Layer::Ui => &mut self.ui.commands,
            Layer::Debug => &mut self.debug.commands,
        }
    }

    pub fn clear(&mut self) {
        self.world.commands.clear();
        self.ui.commands.clear();
        self.debug.commands.clear();
    }
}

impl Layer {
    const ALL: [Layer; 3] = [Layer::World, Layer::Ui, Layer::Debug];
}

pub struct LayerGpu {
    camera_buffer: gpu::Buffer<CameraData>,
    camera_bg: wgpu::BindGroup,
    vertex_buffer: gpu::Buffer<Vertex>,
    index_buffer: gpu::Buffer<u32>,
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

        let vertex_buffer = gpu::Buffer::new_with_capacity(
            "layer vertex buffer",
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            1024,
        );

        let index_buffer = gpu::Buffer::new_with_capacity(
            "layer index buffer",
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            1024,
        );

        Self {
            camera_buffer,
            camera_bg,
            vertex_buffer,
            index_buffer,
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
    fn as_array(&self) -> [&wgpu::BindGroupLayout; 1] {
        [&self.camera]
    }
}

pub struct Renderer<W: Hash + Eq> {
    pipelines: gpu::PipelineCache,
    layouts: Layouts,
    windows: FastHashMap<W, WindowRenderData>,
}

impl<W: Hash + Eq> Renderer<W> {
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

        let layouts = Layouts {
            camera: camera_layout,
        };

        Self {
            pipelines: gpu::PipelineCache::new(),
            layouts,
            windows: FastHashMap::default(),
        }
    }

    pub fn present(&mut self, w: W, surface: &mut gpu::WindowSurface, packet: FramePacket) {
        let gpu = GpuState::get();
        let data = self
            .windows
            .entry(w)
            .or_insert_with(|| WindowRenderData::new(gpu, &self.layouts));

        let output = match surface.acquire(gpu) {
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

        for &layer in &Layer::ALL {
            let layer_gpu = &mut data.layers[layer as usize];
            let layer_packet = match layer {
                Layer::World => &packet.world,
                Layer::Ui => &packet.ui,
                Layer::Debug => &packet.debug,
            };

            layer_gpu.camera_buffer.write(0, &[layer_packet.camera]);

            {
                let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });

                let pipeline = self.pipelines.get_or_create(
                    gpu::PipelineDesc {
                        shader: "immediate-2d",
                        vertex_layout: Vertex::desc(),
                        blend: wgpu::BlendState::ALPHA_BLENDING,
                        topology: wgpu::PrimitiveTopology::TriangleList,
                    },
                    output.texture.format(),
                    &self.layouts.as_array(),
                );

                rp.set_bind_group(0, &layer_gpu.camera_bg, &[]);
                rp.set_pipeline(pipeline);
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        gpu.queue.present(output);
    }
}
