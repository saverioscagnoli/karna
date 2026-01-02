use crate::{Vertex, immediate::ImmediateRenderer};
use globals::profiling;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use math::Vector4;

pub struct LineBatcher {
    list_vertices: Vec<Vertex>,
    list_indices: Vec<u32>,

    strip_vertices: Vec<Vertex>,
    strip_indices: Vec<u32>,

    all_vertices: Vec<Vertex>,
    all_indices: Vec<u32>,

    vertex_buffer: GpuBuffer<Vertex>,
    index_buffer: GpuBuffer<u32>,
    list_pipeline: wgpu::RenderPipeline,
    strip_pipeline: wgpu::RenderPipeline,
}

impl LineBatcher {
    pub fn new(list_pipeline: wgpu::RenderPipeline, strip_pipeline: wgpu::RenderPipeline) -> Self {
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
            list_vertices: Vec::with_capacity(ImmediateRenderer::BASE_VERTEX_CAPACITY),
            list_indices: Vec::with_capacity(ImmediateRenderer::BASE_INDEX_CAPACITY),
            strip_vertices: Vec::with_capacity(ImmediateRenderer::BASE_VERTEX_CAPACITY),
            strip_indices: Vec::with_capacity(ImmediateRenderer::BASE_INDEX_CAPACITY),
            all_vertices: Vec::with_capacity(ImmediateRenderer::BASE_VERTEX_CAPACITY),
            all_indices: Vec::with_capacity(ImmediateRenderer::BASE_INDEX_CAPACITY),
            vertex_buffer,
            index_buffer,
            list_pipeline,
            strip_pipeline,
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.list_vertices.clear();
        self.list_indices.clear();
        self.strip_vertices.clear();
        self.strip_indices.clear();
        self.all_vertices.clear();
        self.all_indices.clear();
    }

    #[inline]
    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, z: f32, color: Vector4) {
        let color: [f32; 4] = color.into();
        let base = self.list_vertices.len() as u32;

        self.list_vertices.extend_from_slice(&[
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

        self.list_indices.extend_from_slice(&[base, base + 1]);
    }

    #[inline]
    pub fn stroke_rect(&mut self, x: f32, y: f32, z: f32, w: f32, h: f32, color: Vector4) {
        let color: [f32; 4] = color.into();
        let base = self.strip_vertices.len() as u32;

        self.strip_vertices.extend_from_slice(&[
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
            Vertex {
                position: [x, y, z], // Close the loop
                color,
                uv_coords: [0.0, 0.0],
            },
        ]);

        // For LineStrip, indices are just sequential
        self.strip_indices
            .extend_from_slice(&[base, base + 1, base + 2, base + 3, base + 4]);
    }

    #[inline]
    pub fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        let list_vertices_n = self.list_vertices.len() as u32;
        let list_indices_n = self.list_indices.len() as u32;
        let strip_vertices_n = self.strip_vertices.len() as u32;
        let strip_indices_n = self.strip_indices.len() as u32;

        if list_vertices_n == 0 && strip_vertices_n == 0 {
            return;
        }

        let total_vertices = list_vertices_n + strip_vertices_n;
        let total_indices = list_indices_n + strip_indices_n;

        if total_vertices > self.vertex_buffer.capacity() as u32 {
            let new_capacity = (total_vertices as usize).next_power_of_two();
            self.vertex_buffer.resize(new_capacity);
        }

        if total_indices > self.index_buffer.capacity() as u32 {
            let new_capacity = (total_indices as usize).next_power_of_two();
            self.index_buffer.resize(new_capacity);
        }

        self.all_vertices.extend_from_slice(&self.list_vertices);
        self.all_indices.extend_from_slice(&self.list_indices);

        let vertex_offset = list_vertices_n;

        self.all_vertices.extend_from_slice(&self.strip_vertices);
        self.all_indices
            .extend(self.strip_indices.iter().map(|&idx| idx + vertex_offset));

        self.vertex_buffer.write(0, &self.all_vertices);
        self.index_buffer.write(0, &self.all_indices);

        if list_indices_n > 0 {
            render_pass.set_pipeline(&self.list_pipeline);
            profiling::record_pipeline_switches(1);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice_all());
            render_pass.set_index_buffer(self.index_buffer.slice_all(), wgpu::IndexFormat::Uint32);

            render_pass.draw_indexed(0..list_indices_n, 0, 0..1);
            profiling::record_draw_call(list_vertices_n, list_indices_n);
        }

        if strip_indices_n > 0 {
            render_pass.set_pipeline(&self.strip_pipeline);
            profiling::record_pipeline_switches(1);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice_all());
            render_pass.set_index_buffer(self.index_buffer.slice_all(), wgpu::IndexFormat::Uint32);

            render_pass.draw_indexed(list_indices_n..(list_indices_n + strip_indices_n), 0, 0..1);
            profiling::record_draw_call(strip_vertices_n, strip_indices_n);
        }

        self.clear();
    }
}
