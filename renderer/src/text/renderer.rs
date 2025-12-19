use crate::{camera::Camera, mesh::Descriptor, shader::Shader};
use assets::AssetManager;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct TextVertex {
    position: [f32; 2],
}

impl Descriptor for TextVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
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
    vertex_buffer: gpu::core::Buffer<TextVertex>,
    index_buffer: gpu::core::Buffer<u32>,
    index_count: u32,

    // Instance buffer containing per-glyph data
    glyph_instance_buffer: gpu::core::Buffer<GlyphInstance>,
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
        let shader = Shader::from_wgsl_file(
            include_str!("../../../shaders/text.wgsl"),
            Some("text_shader"),
        );

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
                &[TextVertex::desc(), GlyphInstance::desc()],
            );

        // Unit rect
        let vertices = [
            TextVertex {
                position: [0.0, 0.0],
            }, // Bottom-left
            TextVertex {
                position: [1.0, 0.0],
            }, // Bottom-right
            TextVertex {
                position: [1.0, 1.0],
            }, // Top-right
            TextVertex {
                position: [0.0, 1.0],
            }, // Top-left
        ];

        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = gpu::core::Buffer::new(
            "text quad vertex buffer",
            wgpu::BufferUsages::VERTEX,
            &vertices,
        );

        let index_buffer = gpu::core::Buffer::new(
            "text quad index buffer",
            wgpu::BufferUsages::INDEX,
            &indices,
        );

        let glyph_instance_buffer = gpu::core::Buffer::new_with_capacity(
            "glyph instance buffer",
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            Self::INITIAL_INSTANCE_CAPACITY,
        );

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

    #[inline]
    fn update_instance_buffer(&mut self) {
        if self.glyph_instances.is_empty() {
            return;
        }

        if self.glyph_instances.len() > self.instance_capacity {
            self.glyph_instance_buffer.resize(self.instance_capacity);
        }

        self.glyph_instance_buffer.write(&self.glyph_instances);
    }

    #[inline]
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
