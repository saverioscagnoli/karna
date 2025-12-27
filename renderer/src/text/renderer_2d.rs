//! Text rendering module.
//! This text renderer is responsible for rendering
//! bidimensional text
//!
//! For 3d text, you should use a mesh (that is not implemented rn :3)

use crate::{Camera, Descriptor, shader::Shader};
use assets::AssetManager;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use math::{Vector2, Vector3, Vector4};
use traccia::info;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct TextVertex {
    pub position: Vector3,
}

impl Descriptor for TextVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlyphInstance {
    pub position: Vector3,
    pub size: Vector2,
    pub uv_offset: Vector2,
    pub uv_scale: Vector2,
    pub color: Vector4,
    pub rotation: Vector3,
}

impl Descriptor for GlyphInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Position (Vector3) at location 1
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Size (Vector2) at location 2
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vector3>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // UV Offset (Vector2) at location 3
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() + std::mem::size_of::<Vector2>())
                        as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // UV Scale (Vector2) at location 4
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() + std::mem::size_of::<Vector2>() * 2)
                        as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color (Vector4) at location 5
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() + std::mem::size_of::<Vector2>() * 3)
                        as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Rotation (Vector3) at location 6
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>()
                        + std::mem::size_of::<Vector2>() * 3
                        + std::mem::size_of::<Vector4>())
                        as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct TextRenderer2d {
    pipeline: wgpu::RenderPipeline,

    vertex_buffer: GpuBuffer<TextVertex>,
    index_buffer: GpuBuffer<u32>,
    instance_buffer: GpuBuffer<GlyphInstance>,
    instances: Vec<GlyphInstance>,
    instance_capacity: usize,
}

impl TextRenderer2d {
    const INITIAL_INSTANCE_CAPACITY: usize = 512; // Increased from 128

    pub fn new(
        layer_name: &str,
        surface_format: wgpu::TextureFormat,
        camera: &Camera,
        assets: &AssetManager,
    ) -> Self {
        let shader = Shader::from_wgsl_file(
            include_str!("../../../shaders/text.wgsl"),
            Some("text2d shader"),
        );

        let pipeline = shader
            .pipeline_builder()
            .label("text2d pipeline")
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

        // Insert a unit quad in the buffers
        let vertices = [
            TextVertex {
                position: Vector3::new(0.0, 0.0, 0.0),
            }, // Bottom-left
            TextVertex {
                position: Vector3::new(1.0, 0.0, 0.0),
            }, // Bottom-right
            TextVertex {
                position: Vector3::new(1.0, 1.0, 0.0),
            }, // Top-right
            TextVertex {
                position: Vector3::new(0.0, 1.0, 0.0),
            }, // Top-left
        ];

        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = GpuBufferBuilder::new()
            .label("text2d quad vertex buffer")
            .usage(wgpu::BufferUsages::VERTEX)
            .data(vertices.to_vec())
            .build();

        let index_buffer = GpuBufferBuilder::new()
            .label("text2d quad index buffer")
            .usage(wgpu::BufferUsages::INDEX)
            .data(indices.to_vec())
            .build();

        let instance_buffer = GpuBufferBuilder::new()
            .label("text2d quad instance buffer")
            .usage(wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST)
            .capacity(Self::INITIAL_INSTANCE_CAPACITY)
            .build();

        info!("Initialized text renderer for layer '{}'", layer_name);

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            instances: Vec::new(),
            instance_capacity: Self::INITIAL_INSTANCE_CAPACITY,
        }
    }

    #[inline]
    pub fn add_glyph(&mut self, glyph: GlyphInstance) {
        self.instances.push(glyph);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    #[inline]
    fn update_instance_buffer(&mut self) {
        if self.instances.is_empty() {
            return;
        }

        if self.instances.len() > self.instance_capacity {
            self.instance_capacity = (self.instances.len() * 2).max(self.instance_capacity * 2);

            info!(
                "Resizing text instance buffer: {} glyphs needed, new capacity: {}",
                self.instances.len(),
                self.instance_capacity
            );

            self.instance_buffer.resize(self.instance_capacity);
        }

        self.instance_buffer.write(0, &self.instances);
    }

    #[inline]
    pub fn present<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera: &'a Camera,
        assets: &AssetManager,
    ) {
        if self.instances.is_empty() {
            return;
        }

        self.update_instance_buffer();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, camera.view_projection_bind_group(), &[]);
        render_pass.set_bind_group(1, assets.bind_group(), &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice_all());
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice_all());
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(
            0..self.index_buffer.len() as u32,
            0,
            0..self.instances.len() as u32,
        );
    }
}
