mod atlas;
mod batcher;
mod color;
mod util;

use crate::{atlas::TextureAtlas, batcher::Batcher};
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use image::GenericImageView;
use math::{Size, Vec2, Vec3, Vec4};
use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};
use traccia::error;
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
    texture_bind_group_layout: wgpu::BindGroupLayout,
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

        // Add texture bind group layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Texture Bind Group Layout"),
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
            texture_bind_group_layout,
            surface_format,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    pos: Vec3,
    color: Vec4,
    tex_coords: Vec2,
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
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
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

    atlas: TextureAtlas,
    atlas_bind_group: wgpu::BindGroup,

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

        let atlas = TextureAtlas::new(&state.device, &state.queue, 800);

        let atlas_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas Bind Group"),
            layout: &state.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas.sampler),
                },
            ],
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
            &state.texture_bind_group_layout,
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
            &state.texture_bind_group_layout,
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
            &state.texture_bind_group_layout,
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
            &state.texture_bind_group_layout,
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
            &state.texture_bind_group_layout,
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
            atlas,
            atlas_bind_group,
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
        @group(1) @binding(0) var t_diffuse: texture_2d<f32>;
        @group(1) @binding(1) var s_diffuse: sampler;

        struct VertexInput {
            @location(0) pos: vec3<f32>,
            @location(1) color: vec4<f32>,
            @location(2) tex_coords: vec2<f32>,
        }

        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) color: vec4<f32>,
            @location(1) tex_coords: vec2<f32>,
        }

        @vertex
        fn vs_main(model: VertexInput) -> VertexOutput {
            var out: VertexOutput;
            out.color = model.color;
            out.tex_coords = model.tex_coords;
            let pos = vec4<f32>(model.pos, 1.0);
            out.clip_position = projection.matrix * pos;
            return out;
        }

         @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
        
        // For solid colors (when tex_coords are zero), bypass texture sampling
        if (in.tex_coords.x == 0.0 && in.tex_coords.y == 0.0) {
            return in.color;
        }
        
        return tex_color * in.color;
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
            tex_coords: Vec2::zero(),
        });

        self.point_batcher.indices.push(vertex_index);
    }

    pub fn draw_line<P1: Into<Vec2>, P2: Into<Vec2>>(&mut self, p1: P1, p2: P2) {
        let p1: Vec2 = p1.into();
        let p2: Vec2 = p2.into();
        let start_index = self.line_batcher.vertex_count();
        let color: Vec4 = self.draw_color.into();

        self.line_batcher.vertices.extend_from_slice(&[
            Vertex {
                pos: p1.resize_zeros(),
                color,
                tex_coords: Vec2::zero(),
            },
            Vertex {
                pos: p2.resize_zeros(),
                color,
                tex_coords: Vec2::zero(),
            },
        ]);

        self.line_batcher
            .indices
            .extend_from_slice(&[start_index, start_index + 1]);
    }

    pub fn draw_line_strip(&mut self, points: &[Vec2]) {
        if points.len() < 2 {
            return;
        }

        let start_index = self.line_strip_batcher.vertex_count();
        let color: Vec4 = self.draw_color.into();

        for &point in points {
            self.line_strip_batcher.vertices.push(Vertex {
                pos: point.resize_zeros(),
                color,
                tex_coords: Vec2::zero(),
            });
        }

        for i in 0..points.len() - 1 {
            self.line_strip_batcher.indices.push(start_index + i as u32);
            self.line_strip_batcher
                .indices
                .push(start_index + i as u32 + 1);
        }
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
                tex_coords: Vec2::zero(),
            },
            Vertex {
                pos: p2.resize_zeros(),
                color,
                tex_coords: Vec2::zero(),
            },
            Vertex {
                pos: p3.resize_zeros(),
                color,
                tex_coords: Vec2::zero(),
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
                tex_coords: Vec2::zero(),
            },
            Vertex {
                pos: top_right.resize_zeros(),
                color,
                tex_coords: Vec2::zero(),
            },
            Vertex {
                pos: bottom_left.resize_zeros(),
                color,
                tex_coords: Vec2::zero(),
            },
            Vertex {
                pos: bottom_right.resize_zeros(),
                color,
                tex_coords: Vec2::zero(),
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

    pub fn load_font_from_bytes<L: AsRef<str>>(&mut self, name: L, bytes: &[u8]) {
        self.atlas.load_font(name.as_ref(), bytes);
    }

    pub fn load_image_from_bytes<L: AsRef<str>>(&mut self, name: L, bytes: &[u8]) {
        let img = image::load_from_memory(bytes)
            .ok()
            .expect("Failed to load image from bytes");
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let device = self.device.clone();
        let queue = &self.queue.clone();

        let was_resized = self.atlas.add_texture(
            &device,
            queue,
            name.as_ref(),
            &rgba,
            Size::new(dimensions.0, dimensions.1),
            None,
        );

        // If the atlas was expanded, update the bind group
        if was_resized {
            self.update_atlas_bind_group();
        }
    }

    pub fn draw_text<L: AsRef<str>, T: AsRef<str>>(&mut self, font: L, text: T, pos: Vec2) {
        let font_name = font.as_ref();
        let text_str = text.as_ref();
        let pos: Vec2 = pos.into();

        // Collect the characters first
        let chars: Vec<char> = text_str.chars().collect();

        // Do all the font work in a separate scope to end immutable borrows
        {
            let font_clone = {
                let fonts = &self.atlas.fonts;
                fonts.get(font_name).unwrap().clone()
            };

            let mut layout = Layout::new(CoordinateSystem::PositiveYUp);
            layout.append(&[font_clone], &TextStyle::new(text_str, 16.0, 0));
        } // font_clone and layout are dropped here, ending any immutable borrows

        // Now we can safely make the mutable borrow
        self.atlas.handle_text(
            &self.device.clone(),
            &self.queue.clone(),
            font_name,
            text_str,
            16.0,
        );

        // Now collect regions after mutable borrow is done
        // Clone the region data to break borrow dependencies
        let char_regions: Vec<(char, Option<_>)> = chars
            .iter()
            .map(|&c| (c, self.atlas.get_char_region(c).cloned()))
            .collect();

        // Process the collected regions
        for (c, region_opt) in &char_regions {
            if c.is_whitespace() {
                continue; // Skip whitespace characters
            }
            if let Some(region) = region_opt {
                let start_index = self.triangle_batcher.vertex_count();
                let top_left = pos + Vec2::new(region.uv_min.x, region.uv_min.y);
                let top_right = pos + Vec2::new(region.uv_max.x, region.uv_min.y);
                let bottom_left = pos + Vec2::new(region.uv_min.x, region.uv_max.y);
                let bottom_right = pos + Vec2::new(region.uv_max.x, region.uv_max.y);

                self.triangle_batcher.vertices.extend_from_slice(&[
                    Vertex {
                        pos: top_left.resize_zeros(),
                        color: Color::White.into(),
                        tex_coords: Vec2::new(region.uv_min.x, region.uv_min.y),
                    },
                    Vertex {
                        pos: top_right.resize_zeros(),
                        color: Color::White.into(),
                        tex_coords: Vec2::new(region.uv_max.x, region.uv_min.y),
                    },
                    Vertex {
                        pos: bottom_left.resize_zeros(),
                        color: Color::White.into(),
                        tex_coords: Vec2::new(region.uv_min.x, region.uv_max.y),
                    },
                    Vertex {
                        pos: bottom_right.resize_zeros(),
                        color: Color::White.into(),
                        tex_coords: Vec2::new(region.uv_max.x, region.uv_max.y),
                    },
                ]);

                self.triangle_batcher.indices.extend_from_slice(&[
                    start_index,
                    start_index + 1,
                    start_index + 2,
                    start_index + 1,
                    start_index + 3,
                    start_index + 2,
                ]);
            } else {
                error!("Character '{}' not found in atlas", c);
            }
        }
    }

    pub fn draw_image<P: Into<Vec2>>(&mut self, label: &str, pos: P) {
        let pos: Vec2 = pos.into();
        let color: Vec4 = Color::White.into();

        if let Some(region) = self.atlas.get_region(label) {
            let start_index = self.triangle_batcher.vertex_count();

            let top_left = pos;
            let top_right = Vec2::new(pos.x + region.size.width as f32, pos.y);
            let bottom_left = Vec2::new(pos.x, pos.y + region.size.height as f32);
            let bottom_right = Vec2::new(
                pos.x + region.size.width as f32,
                pos.y + region.size.height as f32,
            );

            self.triangle_batcher.vertices.extend_from_slice(&[
                Vertex {
                    pos: top_left.resize_zeros(),
                    color,
                    tex_coords: Vec2::new(region.uv_min.x, region.uv_min.y),
                },
                Vertex {
                    pos: top_right.resize_zeros(),
                    color,
                    tex_coords: Vec2::new(region.uv_max.x, region.uv_min.y),
                },
                Vertex {
                    pos: bottom_left.resize_zeros(),
                    color,
                    tex_coords: Vec2::new(region.uv_min.x, region.uv_max.y),
                },
                Vertex {
                    pos: bottom_right.resize_zeros(),
                    color,
                    tex_coords: Vec2::new(region.uv_max.x, region.uv_max.y),
                },
            ]);

            self.triangle_batcher.indices.extend_from_slice(&[
                start_index,
                start_index + 1,
                start_index + 2,
                start_index + 1,
                start_index + 3,
                start_index + 2,
            ]);
        }
    }

    pub fn info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    pub fn frame_info(&self) -> FrameInfo {
        FrameInfo {
            draw_calls: self.draw_calls,
        }
    }

    pub fn update_atlas_bind_group(&mut self) {
        self.atlas_bind_group = self
            .atlas
            .create_bind_group(&self.device, &self.texture_bind_group_layout);
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
        self.line_batcher.check_resize_buffers();
        self.line_strip_batcher.check_resize_buffers();
        self.triangle_batcher.check_resize_buffers();
        self.triangle_strip_batcher.check_resize_buffers();
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
            render_pass.set_bind_group(1, &self.atlas_bind_group, &[]);

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
