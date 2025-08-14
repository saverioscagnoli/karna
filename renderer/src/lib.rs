mod color;
mod util;

pub use color::*;

use math::{Size, Vec2, Vec3, Vec4};
use std::{
    borrow::Cow,
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use traccia::info;
use winit::window::Window;

pub struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    adapter: wgpu::Adapter,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    bind_group_layout: wgpu::BindGroupLayout,
    surface_format: wgpu::TextureFormat,
}

impl GpuState {
    pub fn new(window: Arc<Window>) -> Self {
        let (width, height) = window.inner_size().into();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = match instance.create_surface(window) {
            Ok(s) => s,
            Err(_) => todo!("handle surface error"),
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("Failed to create adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_limits: wgpu::Limits::default(),
            label: Some("device"),
            required_features: wgpu::Features::empty(),
            ..Default::default()
        }))
        .expect("Failed to request a device");

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
            label: Some("Bind Group Layout"),
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
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::default(),
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            adapter,
            queue,
            config,
            bind_group_layout,
            surface_format,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    pos: Vec3,
    color: Vec4,
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
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct ProjectionUniform([[f32; 4]; 4]);

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct DrawCall {
    vertex_start: u32,
    vertex_count: u32,
    index_start: u32,
    index_count: u32,
    topology: wgpu::PrimitiveTopology,
}

pub struct Renderer {
    state: GpuState,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    draw_calls: Vec<DrawCall>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    triangle_pipeline: wgpu::RenderPipeline,

    projection_buffer: wgpu::Buffer,
    projection_bind_group: wgpu::BindGroup,

    circle_cache: HashMap<(u32, u32), (Vec<Vertex>, Vec<u32>)>,

    clear_color: Color,
    draw_color: Color,
}

impl Deref for Renderer {
    type Target = GpuState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for Renderer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl Renderer {
    const VERTEX_CAPACITY: u64 = 100_000;
    const INDEX_CAPACITY: u64 = Self::VERTEX_CAPACITY * 10;

    pub fn _new(window: Arc<Window>) -> Self {
        let state = GpuState::new(window);
        let vertex_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (std::mem::size_of::<Vertex>()) as u64 * Self::VERTEX_CAPACITY,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: (std::mem::size_of::<u32>()) as u64 * Self::INDEX_CAPACITY,

            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let projection_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Projection Buffer"),
            size: std::mem::size_of::<ProjectionUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let projection_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Projection Bind Group"),
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

        let shader_src = Self::shader_src();
        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_src)),
            });

        let triangle_pipeline = Self::create_pipeline(
            "Triangle Pipeline",
            &state.device,
            &shader,
            &state.bind_group_layout,
            state.surface_format,
            wgpu::PrimitiveTopology::TriangleList,
        );

        Self {
            state,
            vertices: Vec::with_capacity(100_000),
            indices: Vec::with_capacity(100_000 * 10),
            draw_calls: Vec::new(),
            vertex_buffer,
            index_buffer,
            triangle_pipeline,
            projection_buffer,
            projection_bind_group,
            circle_cache: HashMap::new(),
            clear_color: Color::Black,
            draw_color: Color::White,
        }
    }

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

    fn create_pipeline<S: AsRef<str>>(
        label: S,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        bind_group_layout: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
        topology: wgpu::PrimitiveTopology,
    ) -> wgpu::RenderPipeline {
        let label = label.as_ref();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let compilation_options = wgpu::PipelineCompilationOptions {
            constants: &[],
            zero_initialize_workgroup_memory: true,
        };

        info!("Creating render pipeline: {}", label);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: Some(&pipeline_layout),
            label: Some(label),
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

    pub fn set_clear_color<C: Into<Color>>(&mut self, color: C) {
        self.clear_color = color.into();
    }

    pub fn set_draw_color<C: Into<Color>>(&mut self, color: C) {
        self.draw_color = color.into();
    }

    pub fn fill_triangle<P: Into<Vec2>>(&mut self, p1: P, p2: P, p3: P) {
        let p1: Vec2 = p1.into();
        let p2: Vec2 = p2.into();
        let p3: Vec2 = p3.into();
        let color: Vec4 = self.draw_color.into();

        let start_index = self.vertices.len() as u32;

        self.vertices.extend_from_slice(&[
            Vertex {
                pos: p1.resize_zeros(),
                color,
            },
            Vertex {
                pos: p2.resize_zeros(),
                color,
            },
            Vertex {
                pos: p3.resize_zeros(),
                color,
            },
        ]);

        self.indices
            .extend_from_slice(&[start_index, start_index + 1, start_index + 2]);

        self.add_draw_call(wgpu::PrimitiveTopology::TriangleList, 3, 3);
    }

    pub fn fill_rect<P: Into<Vec2>, S: Into<Size<f32>>>(&mut self, pos: P, size: S) {
        let pos: Vec2 = pos.into();
        let size: Size<f32> = size.into();
        let color: Vec4 = self.draw_color.into();

        let start_index = self.vertices.len() as u32;

        // Define the four corners of the rectangle
        let top_left = pos;
        let top_right = Vec2::new(pos.x + size.width, pos.y);
        let bottom_left = Vec2::new(pos.x, pos.y + size.height);
        let bottom_right = Vec2::new(pos.x + size.width, pos.y + size.height);

        // Add vertices for the rectangle
        self.vertices.extend_from_slice(&[
            Vertex {
                pos: top_left.resize_zeros(),
                color,
            },
            Vertex {
                pos: top_right.resize_zeros(),
                color,
            },
            Vertex {
                pos: bottom_left.resize_zeros(),
                color,
            },
            Vertex {
                pos: bottom_right.resize_zeros(),
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

    pub fn info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    pub fn _clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.draw_calls.clear();
    }

    pub fn _resize(&mut self, size: Size<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
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

    fn update_projection(&self, screen_width: f32, screen_height: f32) {
        let projection = [
            [2.0 / screen_width, 0.0, 0.0, 0.0],
            [0.0, -2.0 / screen_height, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        let uniform = ProjectionUniform(projection);
        self.queue
            .write_buffer(&self.projection_buffer, 0, util::as_u8_slice(&[uniform]));
    }

    fn update_buffers(&self) {
        if !self.vertices.is_empty() {
            self.queue
                .write_buffer(&self.vertex_buffer, 0, util::as_u8_slice(&self.vertices));
        }
        if !self.indices.is_empty() {
            self.queue
                .write_buffer(&self.index_buffer, 0, util::as_u8_slice(&self.indices));
        }
    }

    pub fn render(&mut self) {
        self.update_projection(self.config.width as f32, self.config.height as f32);
        self.update_buffers();

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_e) => return,
        };

        let output = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
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
                // Switch pipeline only when topology changes
                if current_topology != Some(draw_call.topology) {
                    match draw_call.topology {
                        wgpu::PrimitiveTopology::TriangleList => {
                            render_pass.set_pipeline(&self.triangle_pipeline);
                        }
                        _ => {}
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

        self.queue.submit([encoder.finish()]);
        frame.present();
    }
}
