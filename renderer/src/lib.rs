mod atlas;
mod camera;
mod color;
mod instanced;
mod util;

use crate::{camera::Camera, instanced::InstancedRenderer};
use math::{Size, Vec2, Vec3, Vec4};
use std::{
    borrow::Cow,
    num::NonZero,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};
use traccia::error;
use wgpu::{Backends, Instance, util::DeviceExt};
use winit::window::Window;

// Re-exports
pub use color::*;

pub struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    adapter: wgpu::Adapter,
    queue: Rc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    bind_group_layout: wgpu::BindGroupLayout,
    surface_format: wgpu::TextureFormat,
}

impl GpuState {
    pub fn new(window: Arc<Window>) -> Self {
        let (width, height): (u32, u32) = window.inner_size().into();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::all(),
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
            present_mode: wgpu::PresentMode::AutoVsync,
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

pub trait Descriptor {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub(crate) struct Vertex {
    pos: Vec3,
    color: Vec4,
}

impl Descriptor for Vertex {
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
struct InstanceData {
    translation: Vec2,
    scale: Vec2,
    color: Vec4,
}

impl Descriptor for InstanceData {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vec2>() * 2) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct PointInstanceData {
    position: Vec2,
    color: Vec4,
}

impl Descriptor for PointInstanceData {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PointInstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Rect {
    pub pos: Vec2,
    pub width: f32,
    pub height: f32,
    pub color: Color,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            width,
            height,
            color: Color::White,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn render(&self, renderer: &mut Renderer) {
        renderer.rect_instances.push(InstanceData {
            translation: self.pos,
            scale: Vec2::new(self.width, self.height),
            color: self.color.into(),
        });
    }
}

pub struct Point {
    pub pos: Vec2,
    pub color: Color,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            color: Color::White,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn render(&self, renderer: &mut Renderer) {
        renderer.point_instances.push(PointInstanceData {
            position: self.pos,
            color: self.color.into(),
        });
    }
}

pub struct Renderer {
    state: GpuState,
    camera: Camera,

    triangle_renderer: InstancedRenderer<InstanceData>,

    // Point rendering
    point_render_pipeline: wgpu::RenderPipeline,
    point_instance_buffer: wgpu::Buffer,
    point_instances: Vec<PointInstanceData>,
    point_instance_capacity: usize,

    point_instance_staging: [wgpu::Buffer; 3],
    current_point_staging: usize,
    rect_instance_staging: wgpu::Buffer,

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
        let (width, height): (u32, u32) = window.inner_size().into();
        let state = GpuState::new(window);

        let camera = Camera::new(
            &state.device,
            state.queue.clone(),
            &state.bind_group_layout,
            (width, height).into(),
        );

        // Create rectangle pipeline
        let rect_render_pipeline = Self::create_rect_pipeline(&state);

        // Create point pipeline
        let point_render_pipeline = Self::create_point_pipeline(&state);

        // Create square vertices for rectangles
        let vertices = [
            Vertex {
                pos: Vec3::new(0.0, 0.0, 0.0), // Top-left
                color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            },
            Vertex {
                pos: Vec3::new(1.0, 0.0, 0.0), // Top-right
                color: Vec4::new(0.0, 1.0, 0.0, 1.0),
            },
            Vertex {
                pos: Vec3::new(1.0, 1.0, 0.0), // Bottom-right
                color: Vec4::new(0.0, 0.0, 1.0, 1.0),
            },
            Vertex {
                pos: Vec3::new(0.0, 1.0, 0.0), // Bottom-left
                color: Vec4::new(1.0, 1.0, 0.0, 1.0),
            },
        ];

        let indices: [u16; 6] = [
            0, 1, 2, // First triangle
            2, 3, 0, // Second triangle
        ];

        // Create rectangle vertex buffer
        let rect_vertex_buffer =
            state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Rect Vertex Buffer"),
                    contents: util::as_u8_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        // Create rectangle index buffer
        let rect_index_buffer =
            state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Rect Index Buffer"),
                    contents: util::as_u8_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        // Create rectangle instance buffer
        let rect_instance_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect Instance Buffer"),
            size: (std::mem::size_of::<InstanceData>() * Self::VERTEX_CAPACITY as usize) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create point instance buffer
        let point_instance_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Point Instance Buffer"),
            size: (std::mem::size_of::<PointInstanceData>() * Self::VERTEX_CAPACITY as usize)
                as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rect_instance_staging = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect Instance Staging Buffer"),
            size: (std::mem::size_of::<InstanceData>() * Self::VERTEX_CAPACITY as usize) as u64,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let point_instance_staging = [
            state.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Point Instance Staging Buffer 0"),
                size: (std::mem::size_of::<PointInstanceData>() * Self::VERTEX_CAPACITY as usize)
                    as u64,
                usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            state.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Point Instance Staging Buffer 1"),
                size: (std::mem::size_of::<PointInstanceData>() * Self::VERTEX_CAPACITY as usize)
                    as u64,
                usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            state.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Point Instance Staging Buffer 2"),
                size: (std::mem::size_of::<PointInstanceData>() * Self::VERTEX_CAPACITY as usize)
                    as u64,
                usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
        ];

        let current_point_staging = 0;

        Self {
            state,
            camera,
            clear_color: Color::Black,
            draw_color: Color::White,

            // Point rendering
            point_render_pipeline,
            point_instance_buffer,
            point_instances: Vec::new(),
            point_instance_capacity: 100_000,

            rect_instance_staging,
            point_instance_staging,
            current_point_staging,
        }
    }

    fn create_rect_pipeline(state: &GpuState) -> wgpu::RenderPipeline {
        let shader_src = Self::rect_shader_src();
        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Rect Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_src)),
            });

        let render_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Rect Render Pipeline Layout"),
                    bind_group_layouts: &[&state.bind_group_layout],
                    push_constant_ranges: &[],
                });

        state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Rect Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc(), InstanceData::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: state.surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
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

    fn create_point_pipeline(state: &GpuState) -> wgpu::RenderPipeline {
        let shader_src = Self::point_shader_src();
        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Point Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_src)),
            });

        let render_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Point Render Pipeline Layout"),
                    bind_group_layouts: &[&state.bind_group_layout],
                    push_constant_ranges: &[],
                });

        state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Point Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[PointInstanceData::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: state.surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::PointList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Front),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
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

    fn rect_shader_src() -> String {
        r#"
        @group(0) @binding(0) var<uniform> projection: mat4x4<f32>;

        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) color: vec4<f32>,
        }

        struct VertexInput {
            @location(0) pos: vec3<f32>,
            @location(1) color: vec4<f32>,
            @location(2) instance_translation: vec2<f32>,
            @location(3) instance_scale: vec2<f32>,
            @location(4) instance_color: vec4<f32>,
        }

        @vertex
        fn vs_main(model: VertexInput) -> VertexOutput {
            var out: VertexOutput;
            out.color = model.instance_color;
            let scaled_pos = model.pos.xy * model.instance_scale;
            let final_pos = scaled_pos + model.instance_translation;
            let pos = vec4<f32>(final_pos, model.pos.z, 1.0);
            out.clip_position = projection * pos;
            return out;
        }

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return in.color;
        }
        "#
        .to_string()
    }

    fn point_shader_src() -> String {
        r#"
        @group(0) @binding(0) var<uniform> projection: mat4x4<f32>;

        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) color: vec4<f32>,
        }

        struct VertexInput {
            @location(2) position: vec2<f32>,
            @location(3) color: vec4<f32>,
        }

        @vertex
        fn vs_main(input: VertexInput) -> VertexOutput {
            var out: VertexOutput;
            out.color = input.color;
            out.clip_position = projection * vec4<f32>(input.position, 0.0, 1.0);
            return out;
        }

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return in.color;
        }
        "#
        .to_string()
    }

    pub fn vsync(&self) -> bool {
        self.config.present_mode == wgpu::PresentMode::AutoVsync
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        if vsync {
            self.config.present_mode = wgpu::PresentMode::AutoVsync;
        } else {
            self.config.present_mode = wgpu::PresentMode::AutoNoVsync;
        }

        self.surface.configure(&self.device, &self.config);
    }

    pub fn set_clear_color<C: Into<Color>>(&mut self, color: C) {
        self.clear_color = color.into();
    }

    pub fn set_draw_color<C: Into<Color>>(&mut self, color: C) {
        self.draw_color = color.into();
    }

    pub fn info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    pub fn _clear(&mut self) {
        self.rect_instances.clear();
        self.point_instances.clear();
    }

    pub fn _resize(&mut self, size: Size<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);

        self.camera.update_projection(size);
    }

    fn resize_rect_instance_buffer_if_needed(&mut self) {
        if self.rect_instances.len() > self.rect_instance_capacity {
            let new_capacity = (self.rect_instance_capacity * 2).max(self.rect_instances.len());

            let new_instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Rect Instance Buffer"),
                size: (std::mem::size_of::<InstanceData>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            self.rect_instance_buffer = new_instance_buffer;
            self.rect_instance_capacity = new_capacity;

            #[cfg(debug_assertions)]
            println!("Resized rect instance buffer to capacity: {}", new_capacity);
        }
    }

    fn resize_point_instance_buffer_if_needed(&mut self) {
        if self.point_instances.len() > self.point_instance_capacity {
            let new_capacity = (self.point_instance_capacity * 2).max(self.point_instances.len());

            let new_instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Point Instance Buffer"),
                size: (std::mem::size_of::<PointInstanceData>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            self.point_instance_buffer = new_instance_buffer;
            self.point_instance_capacity = new_capacity;

            #[cfg(debug_assertions)]
            println!(
                "Resized point instance buffer to capacity: {}",
                new_capacity
            );

            let new_desc = wgpu::BufferDescriptor {
                label: Some("Point Instance Staging Buffer"),
                size: (std::mem::size_of::<PointInstanceData>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            };

            let new_staging_buffers = [
                self.device.create_buffer(&new_desc),
                self.device.create_buffer(&new_desc),
                self.device.create_buffer(&new_desc),
            ];

            self.point_instance_staging = new_staging_buffers;

            #[cfg(debug_assertions)]
            println!(
                "Resized point instance staging buffer to capacity: {}",
                new_capacity
            );
        }
    }

    pub fn _present(&mut self) {
        self.resize_rect_instance_buffer_if_needed();
        self.resize_point_instance_buffer_if_needed();

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to acquire next swap chain texture: {:?}", e);
                return;
            }
        };

        let output = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command encoder"),
            });

        // Prepare point instance buffer if needed before render pass
        if !self.point_instances.is_empty() {
            if let Some(mut buffer) = self.queue.write_buffer_with(
                &self.point_instance_buffer,
                0,
                NonZero::new(
                    (self.point_instances.len() * std::mem::size_of::<PointInstanceData>()) as u64,
                )
                .unwrap(),
            ) {
                let bytes = util::as_u8_slice(&self.point_instances);
                buffer[..bytes.len()].copy_from_slice(bytes);
            }
        }

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

            // Render rectangles
            if !self.rect_instances.is_empty() {
                self.queue.write_buffer(
                    &self.rect_instance_buffer,
                    0,
                    util::as_u8_slice(&self.rect_instances),
                );

                render_pass.set_pipeline(&self.rect_render_pipeline);
                render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
                render_pass.set_vertex_buffer(0, self.rect_vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.rect_instance_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.rect_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..6, 0, 0..self.rect_instances.len() as u32);
            }

            // Render points
            if !self.point_instances.is_empty() {
                render_pass.set_pipeline(&self.point_render_pipeline);
                render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
                render_pass.set_vertex_buffer(0, self.point_instance_buffer.slice(..));
                render_pass.draw(0..1, 0..self.point_instances.len() as u32);
            }
        }

        let command_buffer = encoder.finish();
        self.queue.submit([command_buffer]);
        frame.present();
    }
}
