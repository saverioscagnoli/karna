pub mod camera;
pub mod mesh;
mod util;

pub use camera::Camera;
pub use wgpu::Color;
use wgpu::{naga::FastHashMap, util::DeviceExt};

use crate::{
    camera::CameraKind,
    mesh::{InstanceData, InstanceDataGpu, Mesh, MeshData, MeshId},
};
use common::error::RendererError;
use nalgebra::{Quaternion, Vector2, Vector3, Vector4};
use std::{borrow::Cow, sync::Arc};

fn shader_src() -> String {
    r#"
        @group(0) @binding(0) var<uniform> projection: mat4x4<f32>;

        struct VertexInput {
            @location(0) pos: vec3<f32>,
            @location(1) color: vec4<f32>,
        }

        struct InstanceInput {
            @location(2) instance_position: vec3<f32>,
            @location(3) instance_rotation: vec4<f32>, // quaternion (x, y, z, w)
            @location(4) instance_scale: vec3<f32>,
            @location(5) instance_color: vec4<f32>,
        }

        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) color: vec3<f32>,
        }

        // Rotate a vector by a quaternion
        fn quat_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
            let qvec = q.xyz;
            let uv = cross(qvec, v);
            let uuv = cross(qvec, uv);
            return v + ((uv * q.w) + uuv) * 2.0;
        }

        @vertex
        fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
            var out: VertexOutput;

            // Apply transformations in order: scale -> rotate -> translate
            let scaled_pos = model.pos * instance.instance_scale;
            let rotated_pos = quat_rotate(instance.instance_rotation, scaled_pos);
            let world_pos = rotated_pos + instance.instance_position;

            // Multiply vertex color by instance color
            out.color = model.color.xyz * instance.instance_color.xyz;

            let pos = vec4<f32>(world_pos, 1.0);
            out.clip_position = projection * pos;

            return out;
        }

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return vec4<f32>(in.color, 1.0);
        }
    "#
    .to_string()
}

#[derive(Debug)]
pub struct GpuState {
    pub(crate) surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    #[allow(unused)]
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) surface_format: wgpu::TextureFormat,
    config: wgpu::SurfaceConfiguration,
}

impl GpuState {
    async fn new(window: Arc<winit::window::Window>) -> Result<Self, RendererError> {
        let (width, height) = window.inner_size().into();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_limits: wgpu::Limits::defaults(),
                label: Some("device"),
                required_features: wgpu::Features::default(),
                ..Default::default()
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            adapter,
            surface_format,
            config,
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    position: Vector3<f32>,
    color: Vector4<f32>,
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vector3<f32>>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrawCall {
    vertex_start: u32,
    vertex_count: u32,
    index_start: u32,
    index_count: u32,
    topology: wgpu::PrimitiveTopology,
}

pub struct Renderer {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    draw_calls: Vec<DrawCall>,

    triangle_pipeline: wgpu::RenderPipeline,
    clear_color: wgpu::Color,

    camera: Camera,
    meshes: FastHashMap<MeshId, MeshData>,
    instances: FastHashMap<MeshId, Vec<InstanceDataGpu>>,
    instance_buffer: wgpu::Buffer,
    pub state: GpuState,
}

impl Renderer {
    const VERTEX_CAPACITY: usize = 1024;
    const INDEX_CAPACITY: usize = 1024;
    const INSTANCE_CAPACITY: usize = 4096;

    #[doc(hidden)]
    pub async fn new(window: Arc<winit::window::Window>) -> Result<Self, RendererError> {
        let state = GpuState::new(window.clone()).await?;

        let vertex_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex buffer"),
            size: (std::mem::size_of::<Vertex>() * Self::VERTEX_CAPACITY) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("index buffer"),
            size: (std::mem::size_of::<u32>() * Self::INDEX_CAPACITY) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_src())),
            });

        let screen_size = window.inner_size();
        let camera = Camera::new(
            &state.device,
            Vector2::new(screen_size.width, screen_size.height),
            CameraKind::Orthographic,
        );

        let triangle_pipeline = Self::create_render_pipeline(
            "triangle pipeline",
            &state.device,
            &shader,
            &camera.bind_group_layout(),
            state.surface_format,
            wgpu::PrimitiveTopology::TriangleList,
        );

        let instance_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buffer"),
            size: (std::mem::size_of::<InstanceData>() * Self::INSTANCE_CAPACITY) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            state,
            vertices: Vec::with_capacity(Self::VERTEX_CAPACITY),
            indices: Vec::with_capacity(Self::INDEX_CAPACITY),
            vertex_buffer,
            index_buffer,
            draw_calls: Vec::new(),
            triangle_pipeline,
            clear_color: wgpu::Color::BLACK,
            camera,
            instance_buffer,
            instances: FastHashMap::default(),
            meshes: FastHashMap::default(),
        })
    }

    pub fn register_mesh<M: Mesh + 'static>(&mut self) {
        let mesh_id = MeshId::of::<M>();

        if self.meshes.contains_key(&mesh_id) {
            return;
        }

        let vertices = M::vertices();
        let indices = M::indices();

        let vertex_buffer =
            self.state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("mesh vertex buffer {:?}", mesh_id)),
                    contents: util::as_u8_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            self.state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("mesh index buffer {:?}", mesh_id)),
                    contents: util::as_u8_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        self.meshes.insert(
            mesh_id,
            MeshData {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
            },
        );

        self.instances.insert(mesh_id, Vec::new());
    }

    pub fn draw_mesh<M: Mesh + 'static>(&mut self, instance: &InstanceData) {
        let mesh_id = MeshId::of::<M>();

        if !self.meshes.contains_key(&mesh_id) {
            self.register_mesh::<M>();
        }

        self.instances
            .get_mut(&mesh_id)
            .unwrap()
            .push(instance.to_gpu());
    }

    #[inline]
    #[doc(hidden)]
    pub fn resize(&mut self, size: Vector2<u32>) {
        self.camera.update_projection(size, &self.state.queue);

        self.state.config.width = size.x;
        self.state.config.height = size.y;
        self.state
            .surface
            .configure(&self.state.device, &self.state.config);
    }

    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.clear_color = color;
    }

    fn create_render_pipeline<S>(
        label: S,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        bind_group_layout: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
        topology: wgpu::PrimitiveTopology,
    ) -> wgpu::RenderPipeline
    where
        S: AsRef<str>,
    {
        let label = label.as_ref();
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let compilation_options = wgpu::PipelineCompilationOptions {
            constants: &[],
            zero_initialize_workgroup_memory: false,
        };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceDataGpu::desc()],
                compilation_options: compilation_options.clone(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options,
            }),
            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    fn update_buffers(&self) {
        if !self.vertices.is_empty() {
            self.state.queue.write_buffer(
                &self.vertex_buffer,
                0,
                &util::as_u8_slice(&self.vertices),
            );
        }

        if !self.indices.is_empty() {
            self.state
                .queue
                .write_buffer(&self.index_buffer, 0, &util::as_u8_slice(&self.indices));
        }
    }

    #[doc(hidden)]
    pub fn end_frame(&mut self) {
        self.update_buffers();

        let frame = match self.state.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };

        let output = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("command encoder"),
                });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                label: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
            render_pass.set_pipeline(&self.triangle_pipeline);

            for (mesh_id, instances) in &self.instances {
                if instances.is_empty() {
                    continue;
                }

                let mesh_data = self.meshes.get(mesh_id).unwrap();

                self.state.queue.write_buffer(
                    &self.instance_buffer,
                    0,
                    util::as_u8_slice(instances),
                );

                render_pass.set_vertex_buffer(0, mesh_data.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                render_pass.draw_indexed(0..mesh_data.index_count, 0, 0..instances.len() as u32);
            }
        }

        self.state.queue.submit([encoder.finish()]);
        frame.present();

        for instances in self.instances.values_mut() {
            instances.clear();
        }
    }
}
