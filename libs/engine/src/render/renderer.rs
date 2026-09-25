use core::mem::offset_of;

use nostd::alloc::vec::Vec;
use sdl3::SdlError;
use sdl3::gpu::BufferUsage;
use sdl3::gpu::Device;
use sdl3::gpu::GpuBuffer;
use sdl3::gpu::GraphicsPipeline;
use sdl3::gpu::IndexSize;
use sdl3::gpu::LoadOp;
use sdl3::gpu::PipelineDesc;
use sdl3::gpu::Sampler;
use sdl3::gpu::SamplerDesc;
use sdl3::gpu::Texture;
use sdl3::gpu::VertexAttribute;
use sdl3::gpu::VertexBuffer;
use sdl3::render::Color;
use sdl3::shadercross::CompileOptions;
use sdl3::shadercross::ShaderCross;
use sdl3::shadercross::ShaderSource;
use sdl3::window::Window;

use crate::assets::AssetServer;
use crate::render::Camera;
use crate::render::Draw;
use crate::render::ImmediateVertex;
use crate::render::Layer;
use crate::render::LayerData;
use crate::render::LayerMap;
use crate::render::Projection;

const IMMEDIATE_VERT: &[u8] = include_bytes!("../../../../shaders/immediate.vert.spv");
const IMMEDIATE_FRAG: &[u8] = include_bytes!("../../../../shaders/immediate.frag.spv");

struct Batch {
    layer: Layer,
    first_index: u32,
    indices: u32,
    vertex_offset: i32,
}

pub struct Renderer {
    device: Device,
    cameras: LayerMap<Camera>,
    data: LayerMap<LayerData>,
    pipeline: GraphicsPipeline,
    sampler: Sampler,
    vertex_buffer: GpuBuffer<ImmediateVertex>,
    index_buffer: GpuBuffer<u32>,
    vertices: Vec<ImmediateVertex>,
    indices: Vec<u32>,
    batches: Vec<Batch>,
}

impl Renderer {
    pub fn new(device: Device, shadercross: &ShaderCross, window: &Window) -> Self {
        let default_camera = Camera::new(Projection::topleft_ortho(window.pixel_size()));
        let cameras = LayerMap::new(default_camera, default_camera, default_camera);
        let data = LayerMap::new(
            LayerData::default(),
            LayerData::default(),
            LayerData::default(),
        );

        let pipeline = Self::immediate_pipeline(&device, shadercross, window)
            .expect("Failed to create immediate pipeline");
        let sampler = Sampler::new(device.share(), SamplerDesc::nearest());

        let vertex_buffer = GpuBuffer::new(
            device.share(),
            "immediate vertices",
            4096,
            BufferUsage::VERTEX,
        );
        let index_buffer = GpuBuffer::new(
            device.share(),
            "immediate indices",
            6144,
            BufferUsage::INDEX,
        );

        Self {
            device,
            cameras,
            data,
            pipeline,
            sampler,
            vertex_buffer,
            index_buffer,
            vertices: Vec::new(),
            indices: Vec::new(),
            batches: Vec::new(),
        }
    }

    fn immediate_pipeline(
        device: &Device,
        shadercross: &ShaderCross,
        window: &Window,
    ) -> Result<GraphicsPipeline, SdlError> {
        let vertex = shadercross.create_shader(
            device.share(),
            ShaderSource::Spirv(IMMEDIATE_VERT),
            &CompileOptions::vertex().with_name("immediate.vert"),
        )?;
        let fragment = shadercross.create_shader(
            device.share(),
            ShaderSource::Spirv(IMMEDIATE_FRAG),
            &CompileOptions::fragment().with_name("immediate.frag"),
        )?;

        let buffers = [VertexBuffer::of::<ImmediateVertex>(0)];
        let attributes = [
            VertexAttribute::float3(0, offset_of!(ImmediateVertex, position) as u32),
            VertexAttribute::float4(1, offset_of!(ImmediateVertex, color) as u32),
            VertexAttribute::float2(2, offset_of!(ImmediateVertex, uv) as u32),
            VertexAttribute::float(3, offset_of!(ImmediateVertex, page) as u32),
        ];
        let targets = [device.swapchain_format(window)];

        GraphicsPipeline::new(
            device.share(),
            PipelineDesc::new(&vertex, &fragment)
                .with_vertex_layout(&buffers, &attributes)
                .with_targets(&targets),
        )
    }

    pub fn camera(&self, layer: Layer) -> Option<&Camera> {
        self.cameras.get(layer)
    }

    pub fn set_camera(&mut self, layer: Layer, camera: Camera) {
        if self.cameras.contains(layer) {
            self.cameras[layer] = camera;
        } else {
            self.cameras.insert(layer, camera);
        }
    }

    pub fn draw_handle<'a>(&'a mut self, assets: &'a AssetServer) -> Draw<'a> {
        for layer in self.data.values_mut() {
            layer.vertices.clear();
            layer.indices.clear();
        }

        Draw::new(&mut self.data, assets)
    }

    fn collect(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.batches.clear();

        let order = [Layer::WORLD]
            .into_iter()
            .chain(self.data.order().iter().copied())
            .chain([Layer::UI, Layer::DEBUG]);

        for layer in order {
            let data = &self.data[layer];

            if data.indices.is_empty() {
                continue;
            }

            self.batches.push(Batch {
                layer,
                first_index: self.indices.len() as u32,
                indices: data.indices.len() as u32,
                vertex_offset: self.vertices.len() as i32,
            });

            self.vertices.extend_from_slice(&data.vertices);
            self.indices.extend_from_slice(&data.indices);
        }
    }

    pub fn flush(
        &mut self,
        window: &Window,
        atlas: &Texture,
        clear_color: Color,
    ) -> Result<(), SdlError> {
        let Some(mut frame) = self.device.begin_frame(window)? else {
            return Ok(());
        };

        for camera in self.cameras.values_mut() {
            camera.update(frame.size());
        }

        self.collect();

        if !self.indices.is_empty() {
            self.device
                .upload(&mut self.vertex_buffer, &self.vertices)?;
            self.device.upload(&mut self.index_buffer, &self.indices)?;
        }

        {
            let mut pass = frame.render_pass(LoadOp::Clear(clear_color))?;

            if !self.batches.is_empty() {
                pass.bind_pipeline(&self.pipeline);
                pass.bind_vertex_buffer(0, &self.vertex_buffer, 0);
                pass.bind_index_buffer(&self.index_buffer, IndexSize::U32, 0);
                pass.bind_fragment_sampler(0, atlas, &self.sampler);

                let fallback = self.cameras[Layer::WORLD];

                for batch in &self.batches {
                    let camera = self.cameras.get(batch.layer).unwrap_or(&fallback);

                    pass.push_vertex_uniform(0, &camera.mvp());
                    pass.draw_indexed_instanced(
                        batch.indices,
                        1,
                        batch.first_index,
                        batch.vertex_offset,
                        0,
                    );
                }
            }
        }

        frame.submit()
    }
}
