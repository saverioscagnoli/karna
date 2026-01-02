use crate::{Camera, Descriptor, retained::Text};
use assets::AssetManager;
use globals::profiling;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use utils::{Handle, SlotMap};

#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct TextVertex {
    pub position: [f32; 3],
}

impl TextVertex {
    fn unit_rect() -> (Vec<Self>, Vec<u32>) {
        let vertices = vec![
            TextVertex {
                position: [0.0, 0.0, 0.0],
            },
            TextVertex {
                position: [1.0, 0.0, 0.0],
            },
            TextVertex {
                position: [1.0, 1.0, 0.0],
            },
            TextVertex {
                position: [0.0, 1.0, 0.0],
            },
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        (vertices, indices)
    }
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
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct GlyphInstance {
    pub position: [f32; 3],
    pub size: [f32; 2],
    pub uv_offset: [f32; 2],
    pub uv_scale: [f32; 2],
    pub color: [f32; 4],
    pub rotation: [f32; 3],
}

impl Descriptor for GlyphInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Position (offset 0)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Size (offset 12)
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // UV Offset (offset 20)
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // UV Scale (offset 28) - WAS WRONG: calculated 36
                wgpu::VertexAttribute {
                    offset: 28,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color (offset 36) - WAS WRONG: calculated 48
                wgpu::VertexAttribute {
                    offset: 36,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Rotation (offset 52)
                wgpu::VertexAttribute {
                    offset: 52,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct RetainedTextRenderer {
    vertex_buffer: GpuBuffer<TextVertex>,
    index_buffer: GpuBuffer<u32>,
    instance_buffer: GpuBuffer<GlyphInstance>,
    instances: Vec<GlyphInstance>,
    texts: SlotMap<Text>,
}

impl RetainedTextRenderer {
    const INITIAL_INSTANCE_CAPACITY: usize = 1024;

    pub fn new() -> Self {
        let (vertices, indices) = TextVertex::unit_rect();

        let vertex_buffer = GpuBufferBuilder::new()
            .label("Retained text renderer unit quad buffer")
            .vertex()
            .data(vertices)
            .build();

        let index_buffer = GpuBufferBuilder::new()
            .label("Retained text renderer unit quad index buffer")
            .index()
            .data(indices)
            .build();

        let instance_buffer = GpuBufferBuilder::new()
            .label("Retained text renderer instance buffer")
            .vertex()
            .copy_dst()
            .capacity(Self::INITIAL_INSTANCE_CAPACITY)
            .build();

        Self {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            instances: Vec::with_capacity(Self::INITIAL_INSTANCE_CAPACITY),
            texts: SlotMap::new(),
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
    pub fn add_text(&mut self, text: Text) -> Handle<Text> {
        self.texts.insert(text)
    }

    #[inline]
    pub fn get_text(&self, handle: Handle<Text>) -> &Text {
        self.texts.get(handle).expect("Failed to get text instance")
    }

    #[inline]
    pub fn get_text_mut(&mut self, handle: Handle<Text>) -> &mut Text {
        self.texts
            .get_mut(handle)
            .expect("Failed to get text instance")
    }

    #[inline]
    pub fn remove_text(&mut self, handle: Handle<Text>) {
        self.texts.remove(handle);
    }

    #[inline]
    fn update_instance_buffer(&mut self) {
        if self.instances.is_empty() {
            return;
        }

        if self.instances.len() > self.instance_buffer.capacity() {
            let new_capacity = self.instances.len().next_power_of_two();
            self.instance_buffer.resize(new_capacity);
        }

        self.instance_buffer.write(0, &self.instances);
    }

    #[inline]
    pub fn prepare(&mut self, assets: &AssetManager) {
        self.instances.clear();

        for text in self.texts.values_mut() {
            text.rebuild(assets);
            self.instances.extend_from_slice(text.glyph_instances());
        }
    }

    #[inline]
    pub fn present<'a>(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        pipeline: &'a wgpu::RenderPipeline,
    ) {
        if self.instances.is_empty() {
            return;
        }

        self.update_instance_buffer();

        render_pass.set_pipeline(pipeline);
        profiling::record_pipeline_switches(1);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice_all());
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice_all());
        render_pass.set_index_buffer(self.index_buffer.slice_all(), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(
            0..self.index_buffer.len() as u32,
            0,
            0..self.instances.len() as u32,
        );

        let vertex_count = self.vertex_buffer.len() as u32;
        let index_count = self.index_buffer.len() as u32;
        let instance_count = self.instances.len() as u32;

        profiling::record_draw_call(vertex_count, index_count);
        profiling::record_triangles(index_count * instance_count);
    }
}
