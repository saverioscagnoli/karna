mod util;

use common::error::RendererError;
use nalgebra::{Matrix4, Vector2, Vector3, Vector4};
use std::{borrow::Cow, rc::Rc, sync::Arc};

fn shader_src() -> String {
    r#"
        struct ProjectionUniform {
            matrix: mat4x4<f32>,
        }

        @group(0) @binding(0) var<uniform> projection: ProjectionUniform;

        struct VertexInput {
            @location(0) pos: vec3<f32>,
            @location(1) color: vec4<f32>,
        }

        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) color: vec3<f32>,
        }

        @vertex
        fn vs_main(model: VertexInput) -> VertexOutput {
            var out: VertexOutput;
            out.color = model.color.xyz;
            let pos = vec4<f32>(model.pos, 1.0);
            out.clip_position = projection.matrix * pos;
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
    pub(crate) surface: Rc<wgpu::Surface<'static>>,
    pub(crate) device: Rc<wgpu::Device>,
    pub(crate) queue: Rc<wgpu::Queue>,
    pub(crate) adapter: Rc<wgpu::Adapter>,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) surface_format: wgpu::TextureFormat,
}

impl GpuState {
    async fn new(
        window: Arc<winit::window::Window>,
    ) -> Result<(Self, wgpu::SurfaceConfiguration), RendererError> {
        let (width, height) = window.inner_size().into();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = Rc::new(instance.create_surface(window)?);
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

        let adapter = Rc::new(adapter);
        let device = Rc::new(device);
        let queue = Rc::new(queue);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("bind group layout"),
        });

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

        Ok((
            Self {
                surface,
                device,
                queue,
                adapter,
                bind_group_layout,
                surface_format,
            },
            config,
        ))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
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

#[derive(Debug)]
pub struct Renderer {
    state: GpuState,

    vertices: Vec<Vertex>,
    indices: Vec<u32>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    draw_calls: Vec<DrawCall>,

    triangle_pipeline: wgpu::RenderPipeline,

    projection_buffer: wgpu::Buffer,
    projection_bind_group: wgpu::BindGroup,

    clear_color: wgpu::Color,
    draw_color: wgpu::Color,
}

impl Renderer {
    const VERTEX_CAPACITY: usize = 1024;
    const INDEX_CAPACITY: usize = 1024;

    #[doc(hidden)]
    pub async fn new(window: Arc<winit::window::Window>) -> Result<Self, RendererError> {
        let (state, wgpu_config) = GpuState::new(window.clone()).await?;

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

        let projection_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("projection buffer"),
            size: std::mem::size_of::<Matrix4<f32>>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let projection_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("projection bind group"),
            layout: &state.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &projection_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_src())),
            });

        let triangle_pipeline = Self::create_render_pipeline(
            "triangle pipeline",
            &state.device,
            &shader,
            &state.bind_group_layout,
            state.surface_format,
            wgpu::PrimitiveTopology::TriangleList,
        );

        Ok(Self {
            state,
            vertices: Vec::with_capacity(Self::VERTEX_CAPACITY),
            indices: Vec::with_capacity(Self::INDEX_CAPACITY),
            vertex_buffer,
            index_buffer,
            draw_calls: Vec::new(),
            triangle_pipeline,
            projection_buffer,
            projection_bind_group,
            clear_color: wgpu::Color::BLACK,
            draw_color: wgpu::Color::WHITE,
        })
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
                buffers: &[Vertex::desc()],
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

    fn add_draw_call(
        &mut self,
        topology: wgpu::PrimitiveTopology,
        vertex_count: u32,
        index_count: u32,
    ) {
        let vertex_start = (self.vertices.len() - vertex_count as usize) as u32;
        let index_start = (self.indices.len() - index_count as usize) as u32;

        self.draw_calls.push(DrawCall {
            vertex_start,
            vertex_count,
            index_start,
            index_count,
            topology,
        });
    }

    pub fn fill_rect(&mut self, pos: Vector2<f32>, size: Vector2<f32>) {
        let start_index = self.vertices.len() as u32;

        // Define the four corners of the rectangle
        let top_left = Vector3::new(pos.x, pos.y, 1.0);
        let top_right = Vector3::new(pos.x + size.x, pos.y, 1.0);
        let bottom_left = Vector3::new(pos.x, pos.y + size.y, 1.0);
        let bottom_right = Vector3::new(pos.x + size.x, pos.y + size.y, 1.0);

        let color = Vector4::new(
            self.draw_color.r as f32,
            self.draw_color.g as f32,
            self.draw_color.b as f32,
            self.draw_color.a as f32,
        );

        // Add vertices for the rectangle
        self.vertices.extend_from_slice(&[
            Vertex {
                position: top_left,
                color,
            },
            Vertex {
                position: top_right,
                color,
            },
            Vertex {
                position: bottom_left,
                color,
            },
            Vertex {
                position: bottom_right,
                color,
            },
        ]);

        // Add indices for two triangles (top-left, top-right, bottom-left) and (top-right, bottom-right, bottom-left)
        self.indices.extend_from_slice(&[
            start_index,
            start_index + 1,
            start_index + 2, // First triangle
            start_index + 1,
            start_index + 3,
            start_index + 2, // Second triangle
        ]);

        self.add_draw_call(wgpu::PrimitiveTopology::TriangleList, 4, 6);
    }

    fn update_projection(&self, screen_width: f32, screen_height: f32) {
        let projection = Matrix4::new(
            2.0 / screen_width,
            0.0,
            0.0,
            -1.0,
            0.0,
            -2.0 / screen_height,
            0.0,
            1.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );

        self.state.queue.write_buffer(
            &self.projection_buffer,
            0,
            &util::as_u8_slice(&[projection]),
        );
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
        self.update_projection(1280.0, 720.0);
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

            render_pass.set_bind_group(0, &self.projection_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            let mut current_topology = None;

            for draw_call in &self.draw_calls {
                if current_topology != Some(draw_call.topology) {
                    match draw_call.topology {
                        wgpu::PrimitiveTopology::TriangleList => {
                            render_pass.set_pipeline(&self.triangle_pipeline);
                        }

                        _ => unimplemented!(),
                    }

                    current_topology = Some(draw_call.topology);
                }

                render_pass.draw_indexed(
                    draw_call.index_start..draw_call.index_start + draw_call.index_count,
                    0,
                    0..1,
                );
            }
        }

        self.state.queue.submit([encoder.finish()]);
        frame.present();

        // Clear for next frame
        self.vertices.clear();
        self.indices.clear();
        self.draw_calls.clear();
    }
}
