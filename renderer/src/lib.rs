mod batcher;
mod color;
mod util;

use crate::batcher::Batcher;
use math::{Size, Vec2, Vec3, Vec4};
use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};
use wgpu::Backends;
use winit::window::Window;

// Re-exports
pub use color::*;

pub struct GpuState {
    surface: wgpu::Surface<'static>,
    device: Rc<wgpu::Device>,
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

        let device = Rc::new(device);

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

pub struct FrameInfo {
    pub draw_calls: u32,
}

pub struct Renderer {
    state: GpuState,

    point_batcher: Batcher,
    line_batcher: Batcher,
    line_strip_batcher: Batcher,
    triangle_batcher: Batcher,
    triangle_strip_batcher: Batcher,
    draw_calls: u32,

    projection_buffer: wgpu::Buffer,
    projection_bind_group: wgpu::BindGroup,

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

        let point_batcher = Batcher::new(
            "Point Batcher",
            state.device.clone(),
            &shader,
            &state.bind_group_layout,
            state.surface_format,
            wgpu::PrimitiveTopology::PointList,
            Self::VERTEX_CAPACITY,
            Self::INDEX_CAPACITY,
        );

        let line_batcher = Batcher::new(
            "Line Batcher",
            state.device.clone(),
            &shader,
            &state.bind_group_layout,
            state.surface_format,
            wgpu::PrimitiveTopology::LineList,
            Self::VERTEX_CAPACITY,
            Self::INDEX_CAPACITY,
        );

        let line_strip_batcher = Batcher::new(
            "Line Strip Batcher",
            state.device.clone(),
            &shader,
            &state.bind_group_layout,
            state.surface_format,
            wgpu::PrimitiveTopology::LineStrip,
            Self::VERTEX_CAPACITY,
            Self::INDEX_CAPACITY,
        );

        let triangle_batcher = Batcher::new(
            "Triangle Batcher",
            state.device.clone(),
            &shader,
            &state.bind_group_layout,
            state.surface_format,
            wgpu::PrimitiveTopology::TriangleList,
            Self::VERTEX_CAPACITY,
            Self::INDEX_CAPACITY,
        );

        let triangle_strip_batcher = Batcher::new(
            "Triangle Strip Batcher",
            state.device.clone(),
            &shader,
            &state.bind_group_layout,
            state.surface_format,
            wgpu::PrimitiveTopology::TriangleStrip,
            Self::VERTEX_CAPACITY,
            Self::INDEX_CAPACITY,
        );

        Self {
            state,

            point_batcher,
            line_batcher,
            line_strip_batcher,
            triangle_batcher,
            triangle_strip_batcher,
            draw_calls: 0,

            projection_buffer,
            projection_bind_group,
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

    pub fn draw_pixel<P: Into<Vec2>>(&mut self, pos: P) {
        let pos: Vec2 = pos.into();
        let vertex_index = self.point_batcher.vertex_count();

        self.point_batcher.vertices.push(Vertex {
            pos: pos.resize_zeros(),
            color: self.draw_color.into(),
        });

        self.point_batcher.indices.push(vertex_index);
    }

    pub fn fill_triangle<P: Into<Vec2>>(&mut self, p1: P, p2: P, p3: P) {
        let p1: Vec2 = p1.into();
        let p2: Vec2 = p2.into();
        let p3: Vec2 = p3.into();
        let color: Vec4 = self.draw_color.into();

        let start_index = self.triangle_batcher.vertex_count();

        self.triangle_batcher.vertices.extend_from_slice(&[
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

        self.triangle_batcher.indices.extend_from_slice(&[
            start_index,
            start_index + 1,
            start_index + 2,
        ]);
    }

    pub fn fill_rect<P: Into<Vec2>, S: Into<Size<f32>>>(&mut self, pos: P, size: S) {
        let pos: Vec2 = pos.into();
        let size: Size<f32> = size.into();
        let color: Vec4 = self.draw_color.into();

        let start_index = self.triangle_batcher.vertex_count();

        // Define the four corners of the rectangle
        let top_left = pos;
        let top_right = Vec2::new(pos.x + size.width, pos.y);
        let bottom_left = Vec2::new(pos.x, pos.y + size.height);
        let bottom_right = Vec2::new(pos.x + size.width, pos.y + size.height);

        // Add vertices for the rectangle
        self.triangle_batcher.vertices.extend_from_slice(&[
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

        self.triangle_batcher.indices.extend_from_slice(&[
            start_index,
            start_index + 1,
            start_index + 2, // First triangle
            start_index + 1,
            start_index + 3,
            start_index + 2, // Second triangle
        ]);
    }

    pub fn info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    pub fn frame_info(&self) -> FrameInfo {
        FrameInfo {
            draw_calls: self.draw_calls,
        }
    }

    fn clear(&mut self) {
        self.point_batcher.clear();
        self.line_batcher.clear();
        self.line_strip_batcher.clear();
        self.triangle_batcher.clear();
        self.triangle_strip_batcher.clear();
    }

    pub fn _resize(&mut self, size: Size<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
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

    fn update_batchers(&self) {
        self.point_batcher.update(&self.queue);
        self.line_batcher.update(&self.queue);
        self.line_strip_batcher.update(&self.queue);
        self.triangle_batcher.update(&self.queue);
        self.triangle_strip_batcher.update(&self.queue);
    }

    fn check_resize_buffers(&mut self) {
        self.point_batcher.check_resize_buffers();
    }

    pub fn _present(&mut self) {
        self.update_projection(self.config.width as f32, self.config.height as f32);
        self.check_resize_buffers();
        self.update_batchers();
        self.draw_calls = 0;

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

            // 1 point batcher
            // 2 line batcher
            // 3 line strip batcher
            // 4 triangle batcher
            // 5 triangle strip batcher
            self.draw_calls += self.point_batcher.flush(&mut render_pass);
            self.draw_calls += self.line_batcher.flush(&mut render_pass);
            self.draw_calls += self.line_strip_batcher.flush(&mut render_pass);
            self.draw_calls += self.triangle_batcher.flush(&mut render_pass);
            self.draw_calls += self.triangle_strip_batcher.flush(&mut render_pass);
        }

        self.queue.submit([encoder.finish()]);
        frame.present();

        self.clear();
    }
}
