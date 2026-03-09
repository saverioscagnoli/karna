use gpu::core::GpuBuffer;
use math::Matrix4;
use math::Size;

const MAX_VERTICES: usize = 16384;
const MAX_INDICES: usize = MAX_VERTICES * 6 / 4; // worst case: quads

/// Per-vertex data for immediate-mode rendering.
///
/// Layout matches the WGSL shader:
///   location(0) position: vec2<f32>
///   location(1) color:    vec4<f32>
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImmediateVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl ImmediateVertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            // position
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            // color
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: std::mem::size_of::<[f32; 2]>() as u64,
                shader_location: 1,
            },
        ],
    };
}

/// Handles all GPU resources and batch logic for immediate-mode 2D rendering.
///
/// Vertices and indices are accumulated during the frame via `push_quad` (and
/// friends), then flushed in a single draw call through `flush`.
pub struct ImmediateRenderer {
    pipeline: wgpu::RenderPipeline,

    projection_buffer: GpuBuffer<Matrix4>,
    projection_bind_group: wgpu::BindGroup,

    vertex_buffer: GpuBuffer<ImmediateVertex>,
    index_buffer: GpuBuffer<u32>,

    vertices: Vec<ImmediateVertex>,
    indices: Vec<u32>,

    /// The last view size we computed a projection for, so we only re-upload
    /// the uniform when the window is resized.
    last_view: Size<u32>,
}

impl ImmediateRenderer {
    /// Creates a new `ImmediateRenderer`.
    ///
    /// `surface_format` must match the swap-chain texture format so the
    /// pipeline's color target is compatible.
    pub fn new(surface_format: wgpu::TextureFormat) -> Self {
        let device = gpu::device();

        // --- shader ---
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Immediate Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/immediate.wgsl").into()),
        });

        // --- projection uniform ---
        let projection_buffer = GpuBuffer::<Matrix4>::builder()
            .label("Immediate Projection Buffer")
            .uniform()
            .copy_dst()
            .capacity(1)
            .build();

        let projection_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Immediate Projection BGL"),
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

        let projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Immediate Projection BG"),
            layout: &projection_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &projection_buffer.inner(),
                    offset: 0,
                    size: None,
                }),
            }],
        });

        // --- pipeline ---
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Immediate Pipeline Layout"),
            bind_group_layouts: &[&projection_bgl],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Immediate Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ImmediateVertex::LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // 2D – no culling
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // --- vertex / index buffers (pre-allocated) ---
        let vertex_buffer = GpuBuffer::<ImmediateVertex>::builder()
            .label("Immediate VB")
            .vertex()
            .copy_dst()
            .capacity(MAX_VERTICES)
            .build();

        let index_buffer = GpuBuffer::<u32>::builder()
            .label("Immediate IB")
            .index()
            .copy_dst()
            .capacity(MAX_INDICES)
            .build();

        Self {
            pipeline,
            projection_buffer,
            projection_bind_group,
            vertex_buffer,
            index_buffer,
            vertices: Vec::with_capacity(MAX_VERTICES),
            indices: Vec::with_capacity(MAX_INDICES),
            last_view: Size::new(0, 0),
        }
    }

    // ------------------------------------------------------------------
    // Geometry accumulation
    // ------------------------------------------------------------------

    /// Pushes a colored axis-aligned quad (two triangles, four vertices).
    ///
    /// Coordinates are in **screen-space pixels** with `(0, 0)` at the
    /// top-left corner.
    pub fn push_quad(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let base = self.vertices.len() as u32;

        // top-left, top-right, bottom-right, bottom-left
        self.vertices.push(ImmediateVertex {
            position: [x, y],
            color,
        });
        self.vertices.push(ImmediateVertex {
            position: [x + w, y],
            color,
        });
        self.vertices.push(ImmediateVertex {
            position: [x + w, y + h],
            color,
        });
        self.vertices.push(ImmediateVertex {
            position: [x, y + h],
            color,
        });

        // Two triangles: 0-1-2 and 0-2-3
        self.indices.push(base);
        self.indices.push(base + 1);
        self.indices.push(base + 2);
        self.indices.push(base);
        self.indices.push(base + 2);
        self.indices.push(base + 3);
    }

    /// Pushes a colored triangle (three vertices, three indices).
    pub fn push_triangle(&mut self, p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], color: [f32; 4]) {
        let base = self.vertices.len() as u32;

        self.vertices.push(ImmediateVertex {
            position: p0,
            color,
        });
        self.vertices.push(ImmediateVertex {
            position: p1,
            color,
        });
        self.vertices.push(ImmediateVertex {
            position: p2,
            color,
        });

        self.indices.push(base);
        self.indices.push(base + 1);
        self.indices.push(base + 2);
    }

    /// Returns `true` when there is nothing to draw.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Discards all accumulated geometry without drawing.
    #[inline]
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    // ------------------------------------------------------------------
    // Rendering
    // ------------------------------------------------------------------

    /// Updates the orthographic projection uniform if the viewport changed.
    fn update_projection(&mut self, view: Size<u32>) {
        if self.last_view == view {
            return;
        }

        let proj = Matrix4::orthographic_2d(view.width as f32, view.height as f32);
        self.projection_buffer.write(0, &[proj]);
        self.last_view = view;
    }

    /// Uploads the accumulated geometry, records draw commands into the given
    /// render pass, then clears the CPU-side buffers for the next frame.
    ///
    /// Call this once per frame **after** all `push_*` calls and inside an
    /// active render pass.
    pub fn flush<'pass>(
        &'pass mut self,
        view: Size<u32>,
        render_pass: &mut wgpu::RenderPass<'pass>,
    ) {
        if self.is_empty() {
            return;
        }

        self.update_projection(view);

        // Upload vertices
        self.vertex_buffer.write(0, &self.vertices);

        // Upload indices
        self.index_buffer.write(0, &self.indices);

        let index_count = self.indices.len() as u32;

        // Draw
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.projection_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.inner().slice(..));
        render_pass.set_index_buffer(
            self.index_buffer.inner().slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..index_count, 0, 0..1);

        // Reset for next frame
        self.vertices.clear();
        self.indices.clear();
    }
}
