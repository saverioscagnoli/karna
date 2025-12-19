use crate::{camera::Camera, mesh::Descriptor, shader::Shader};
use assets::AssetManager;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct QuadVertex {
    position: [f32; 2],
}

impl Descriptor for QuadVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Gpu data
pub struct GlyphInstance {
    /// Position in screen space
    pub position: [f32; 2],

    /// Size of this glyph in pixels
    pub size: [f32; 2],

    /// Minimum UV coordinate in the atlas
    pub uv_min: [f32; 2],

    /// Maximum UV coordinate in the atlas
    pub uv_max: [f32; 2],

    /// Color tint for this glyph
    pub color: [f32; 4],

    /// Rotation in radians
    pub rotation: f32,
}

impl Descriptor for GlyphInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GlyphInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Position at location 1
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Size at location 2
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // UV Min at location 3
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // UV Max at location 4
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color at location 5
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Rotation at location 6
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

pub struct TextRenderer {
    pipeline: wgpu::RenderPipeline,

    // Shared quad geometry for all glyphs
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,

    // Instance buffer containing per-glyph data
    glyph_instance_buffer: wgpu::Buffer,
    glyph_instances: Vec<GlyphInstance>,
    instance_capacity: usize,
}

impl TextRenderer {
    const INITIAL_INSTANCE_CAPACITY: usize = 1024;

    pub fn new(
        surface_format: wgpu::TextureFormat,
        camera: &Camera,
        assets: &AssetManager,
    ) -> Self {
        let gpu = gpu::get();

        let shader =
            Shader::from_wgsl_file(include_str!("../../shaders/text.wgsl"), Some("text_shader"));

        let pipeline = shader
            .pipeline_builder()
            .label("text pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[
                    camera.view_projection_bind_group_layout(),
                    assets.bind_group_layout(),
                ],
                &[QuadVertex::desc(), GlyphInstance::desc()],
            );

        // Unit rect
        let vertices = [
            QuadVertex {
                position: [0.0, 0.0],
            }, // Bottom-left
            QuadVertex {
                position: [1.0, 0.0],
            }, // Bottom-right
            QuadVertex {
                position: [1.0, 1.0],
            }, // Top-right
            QuadVertex {
                position: [0.0, 1.0],
            }, // Top-left
        ];

        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = gpu
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("text quad vertex buffer"),
                contents: utils::as_u8_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = gpu
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("text quad index buffer"),
                contents: utils::as_u8_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let glyph_instance_buffer = gpu.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("text glyph instance buffer"),
            size: (std::mem::size_of::<GlyphInstance>() * Self::INITIAL_INSTANCE_CAPACITY) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            glyph_instance_buffer,
            glyph_instances: Vec::with_capacity(Self::INITIAL_INSTANCE_CAPACITY),
            instance_capacity: Self::INITIAL_INSTANCE_CAPACITY,
        }
    }

    /// Add a glyph to be rendered this frame
    #[inline]
    pub fn add_glyph(&mut self, glyph: GlyphInstance) {
        self.glyph_instances.push(glyph);
    }

    /// Clear all glyphs (called at the beginning of each frame)
    #[inline]
    pub fn clear(&mut self) {
        self.glyph_instances.clear();
    }

    fn update_instance_buffer(&mut self) {
        if self.glyph_instances.is_empty() {
            return;
        }

        let gpu = gpu::get();

        // Resize buffer if needed
        if self.glyph_instances.len() > self.instance_capacity {
            self.instance_capacity = self.glyph_instances.len().next_power_of_two();

            self.glyph_instance_buffer = gpu.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("text glyph instance buffer"),
                size: (std::mem::size_of::<GlyphInstance>() * self.instance_capacity) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        gpu.queue().write_buffer(
            &self.glyph_instance_buffer,
            0,
            utils::as_u8_slice(&self.glyph_instances),
        );
    }

    pub fn render<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera: &'a Camera,
        assets: &'a AssetManager,
    ) {
        if self.glyph_instances.is_empty() {
            return;
        }

        self.update_instance_buffer();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, camera.view_projection_bind_group(), &[]);
        render_pass.set_bind_group(1, assets.bind_group(), &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.glyph_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..self.index_count, 0, 0..self.glyph_instances.len() as u32);
    }
}
