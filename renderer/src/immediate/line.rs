use crate::{Vertex, immediate::ImmediateRenderer};
use globals::profiling;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use math::Vector4;

pub struct LineBatcher {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer: GpuBuffer<Vertex>,
    index_buffer: GpuBuffer<u32>,
    pipeline: wgpu::RenderPipeline,
}

impl LineBatcher {
    pub fn new(pipeline: wgpu::RenderPipeline) -> Self {
        let vertex_buffer = GpuBufferBuilder::new()
            .label("Immediate Renderer line vertex buffer")
            .capacity(ImmediateRenderer::BASE_VERTEX_CAPACITY)
            .vertex()
            .copy_dst()
            .build();

        let index_buffer = GpuBufferBuilder::new()
            .label("Immediate Renderer line index buffer")
            .capacity(ImmediateRenderer::BASE_INDEX_CAPACITY)
            .index()
            .copy_dst()
            .build();

        Self {
            vertices: Vec::with_capacity(ImmediateRenderer::BASE_VERTEX_CAPACITY),
            indices: Vec::with_capacity(ImmediateRenderer::BASE_INDEX_CAPACITY),
            vertex_buffer,
            index_buffer,
            pipeline,
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    #[inline]
    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, z: f32, color: Vector4) {
        let color: [f32; 4] = color.into();
        let base = self.vertices.len() as u32;

        self.vertices.extend_from_slice(&[
            Vertex {
                position: [x1, y1, z],
                color,
                uv_coords: [0.0, 0.0],
            },
            Vertex {
                position: [x2, y2, z],
                color,
                uv_coords: [0.0, 0.0],
            },
        ]);

        self.indices.extend_from_slice(&[base, base + 1]);
    }

    #[inline]
    pub fn stroke_rect(&mut self, x: f32, y: f32, z: f32, w: f32, h: f32, color: Vector4) {
        let color: [f32; 4] = color.into();
        let base = self.vertices.len() as u32;

        self.vertices.extend_from_slice(&[
            Vertex {
                position: [x, y, z],
                color,
                uv_coords: [0.0, 0.0],
            },
            Vertex {
                position: [x + w, y, z],
                color,
                uv_coords: [0.0, 0.0],
            },
            Vertex {
                position: [x + w, y + h, z],
                color,
                uv_coords: [0.0, 0.0],
            },
            Vertex {
                position: [x, y + h, z],
                color,
                uv_coords: [0.0, 0.0],
            },
        ]);

        self.indices.extend_from_slice(&[
            base,
            base + 1, // top
            base + 1,
            base + 2, // right
            base + 2,
            base + 3, // bottom
            base + 3,
            base, // left
        ]);
    }

    #[inline]
    pub fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.vertices.is_empty() {
            return;
        }

        let vertices_n = self.vertices.len() as u32;
        let indices_n = self.indices.len() as u32;

        if vertices_n > self.vertex_buffer.capacity() as u32 {
            let new_capacity = self.vertex_buffer.capacity() * 2;
            self.vertex_buffer.resize(new_capacity);
        }

        if indices_n > self.index_buffer.capacity() as u32 {
            let new_capacity = self.index_buffer.capacity() * 2;
            self.index_buffer.resize(new_capacity);
        }

        self.vertex_buffer.write(0, &self.vertices);
        self.index_buffer.write(0, &self.indices);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice_all());
        render_pass.set_index_buffer(self.index_buffer.slice_all(), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..indices_n, 0, 0..1);
        profiling::record_draw_call(vertices_n, indices_n);

        self.clear();
    }
}
