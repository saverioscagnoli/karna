mod camera;
mod color;
mod immediate;
mod layouts;
mod mesh;
mod retained;
mod vertex;

use std::array;
use std::ops::Index;
use std::ops::IndexMut;
use std::sync::Arc;

use assets::AssetsReader;
use gpu::GpuState;
use logging::warn;
use utils::SlotMap;

pub use crate::camera::Camera;
use crate::camera::CameraData;
pub use crate::camera::Projection;
pub use crate::color::Color;
use crate::color::srgb_to_linear;
use crate::immediate::ImmediateRenderer;
use crate::immediate::imgui::ImguiPacket;
use crate::immediate::imgui::ImguiRenderer;
pub use crate::mesh::*;
use crate::retained::RetainedRenderer;
pub use crate::vertex::Vertex;

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

    pub fn len(&self) -> usize {
        self.indices.len()
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
    pub points: Shape<Vertex>,
    pub lines: Shape<Vertex>,
    pub triangles: Shape<Vertex>,
}

impl LayerPacket {
    pub fn clear(&mut self) {
        self.points.clear();
        self.lines.clear();
        self.triangles.clear();
    }
}

#[derive(Default)]
#[derive(Debug)]
pub struct FramePacket {
    pub viewport: math::Size<u32>,
    pub clear_color: math::Vector4<f32>,
    pub queried_geometries: Vec<Geometry>,
    pub queried_materials: Vec<Material>,
    pub world: LayerPacket,
    pub ui: LayerPacket,
    pub debug: LayerPacket,
    pub imgui: ImguiPacket,
}

impl FramePacket {
    pub fn layer_mut(&mut self, layer: Layer) -> &mut LayerPacket {
        match layer {
            Layer::World => &mut self.world,
            Layer::Ui => &mut self.ui,
            Layer::Debug => &mut self.debug,
        }
    }

    pub fn clear(&mut self) {
        self.world.clear();
        self.ui.clear();
        self.debug.clear();
    }
}

impl Layer {
    const ALL: [Layer; 3] = [Layer::World, Layer::Ui, Layer::Debug];
}

pub struct LayerGpu {
    pub camera_buffer: gpu::Buffer<CameraData>,
    pub camera_bg: wgpu::BindGroup,
    pub immediate: ImmediateRenderer,
    pub retained: RetainedRenderer,
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
            retained: RetainedRenderer::new(),
        }
    }
}

pub struct WindowRenderData {
    pub layers: [LayerGpu; 3],
    // Imgui rendering  happens on a separate layer
    pub imgui: ImguiRenderer,
}

impl Index<Layer> for WindowRenderData {
    type Output = LayerGpu;

    fn index(&self, l: Layer) -> &Self::Output {
        match l {
            Layer::World => &self.layers[0],
            Layer::Ui => &self.layers[1],
            Layer::Debug => &self.layers[2],
        }
    }
}

impl IndexMut<Layer> for WindowRenderData {
    fn index_mut(&mut self, l: Layer) -> &mut Self::Output {
        match l {
            Layer::World => &mut self.layers[0],
            Layer::Ui => &mut self.layers[1],
            Layer::Debug => &mut self.layers[2],
        }
    }
}

impl WindowRenderData {
    fn new(gpu: &GpuState, layouts: &Layouts) -> Self {
        Self {
            layers: array::from_fn(|_| LayerGpu::new(gpu, &layouts.camera)),
            imgui: ImguiRenderer::new(gpu, &layouts.camera),
        }
    }
}

pub struct Layouts {
    pub texture_atlas: wgpu::BindGroupLayout,
    pub camera: wgpu::BindGroupLayout,
    pub material: wgpu::BindGroupLayout,
}

impl Layouts {
    pub fn new(a: &wgpu::BindGroupLayout) -> Self {
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

        let material = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("material bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        Self {
            texture_atlas: a.clone(),
            camera: camera_layout,
            material,
        }
    }
}

impl Layouts {
    const fn as_array(&self) -> [&wgpu::BindGroupLayout; 2] {
        [&self.camera, &self.texture_atlas]
    }

    const fn as_mesh_array(&self) -> [&wgpu::BindGroupLayout; 3] {
        [&self.camera, &self.texture_atlas, &self.material]
    }
}

pub struct Renderer {
    pipelines: Arc<gpu::PipelineCache>,
    pub layouts: Arc<Layouts>,
    pub data: WindowRenderData,
    pub assets: AssetsReader,

    pub geometries: SlotMap<Geometry>,
    pub materials: SlotMap<Material>,

    depth: Option<gpu::DepthTexture>,
}

impl Renderer {
    pub fn new(
        pipelines: Arc<gpu::PipelineCache>,
        layouts: Arc<Layouts>,
        assets: AssetsReader,
    ) -> Self {
        Self {
            pipelines,
            data: WindowRenderData::new(GpuState::get(), &layouts),
            layouts,
            assets,
            geometries: SlotMap::new(),
            materials: SlotMap::new(),
            depth: None,
        }
    }

    pub fn present(&mut self, surface: &mut gpu::WindowSurface, packet: &mut FramePacket) {
        let gpu = GpuState::get();

        let output = match surface.acquire() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
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

        // Render through the sRGB view of the swapchain: shaders output
        // linear color and the hardware encodes on write. Pipelines must
        // target this same format.
        let view_format = surface.view_format();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(view_format),
            ..Default::default()
        });

        // Depth buffer must match the swapchain size; recreate on resize.
        let fb_size = math::Size::new(output.texture.width(), output.texture.height());
        if self.depth.as_ref().map(|d| d.size) != Some(fb_size) {
            self.depth = Some(gpu::DepthTexture::new(fb_size));
        }
        let depth_view = &self.depth.as_ref().unwrap().view;

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command encoder"),
            });

        for (i, &layer) in Layer::ALL.iter().enumerate() {
            let load_op = if i == 0 {
                // The user-facing clear color is sRGB; the sRGB attachment
                // expects linear values (it encodes on write).
                wgpu::LoadOp::Clear(wgpu::Color {
                    r: srgb_to_linear(packet.clear_color.x) as f64,
                    g: srgb_to_linear(packet.clear_color.y) as f64,
                    b: srgb_to_linear(packet.clear_color.z) as f64,
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
                    // Cleared per layer: each layer depth-tests only against
                    // itself, so Ui/Debug always draw over World.
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });

                if !packet.imgui.cmds.is_empty() {
                    let assets = self.assets.read();

                    let fb = math::Size::new(output.texture.width(), output.texture.height());

                    self.data.imgui.present(
                        &packet.imgui,
                        fb,
                        &mut pass,
                        &self.pipelines,
                        &self.layouts,
                        view_format,
                        assets,
                    );
                }

                let assets = self.assets.read();

                pass.set_bind_group(0, &layer_gpu.camera_bg, &[]);
                pass.set_bind_group(1, assets.atlas_bg(), &[]);

                layer_gpu.retained.present(
                    &self.geometries,
                    &self.materials,
                    &mut pass,
                    &self.pipelines,
                    &self.layouts,
                    view_format,
                );

                layer_gpu.immediate.present(
                    &layer_packet.points,
                    &layer_packet.lines,
                    &layer_packet.triangles,
                    &mut pass,
                    &self.pipelines,
                    &self.layouts,
                    view_format,
                    assets,
                );
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        gpu.queue.present(output);
    }
}
