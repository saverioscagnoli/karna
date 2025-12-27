use crate::Vertex;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use math::{Size, Vector2, Vector4};

pub struct ImmediateRenderer {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer: GpuBuffer<Vertex>,
    index_buffer: GpuBuffer<u32>,

    vertex_capacity: usize,
    index_capacity: usize,
}

impl ImmediateRenderer {
    const BASE_VERTEX_CAPACITY: usize = 1024;
    const BASE_INDEX_CAPACITY: usize = 1024;

    pub(crate) fn new() -> Self {
        let vertex_buffer = GpuBufferBuilder::new()
            .label("Immediate Renderer vertex buffer")
            .capacity(Self::BASE_VERTEX_CAPACITY)
            .vertex()
            .copy_dst()
            .build();

        let index_buffer = GpuBufferBuilder::new()
            .label("Immediate Renderer index buffer")
            .capacity(Self::BASE_INDEX_CAPACITY)
            .index()
            .copy_dst()
            .build();

        Self {
            vertices: Vec::with_capacity(Self::BASE_VERTEX_CAPACITY),

            indices: Vec::with_capacity(Self::BASE_INDEX_CAPACITY),
            vertex_buffer,
            index_buffer,
            vertex_capacity: Self::BASE_VERTEX_CAPACITY,
            index_capacity: Self::BASE_INDEX_CAPACITY,
        }
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    #[inline]
    pub fn fill_rect(&mut self, pos: Vector2, size: Size<f32>, color: Vector4) {
        let top_left = pos;
        let top_right = pos + Vector2::new(size.width, 0.0);
        let bottom_left = pos + Vector2::new(0.0, size.height);
        let bottom_right = pos + Vector2::from(size);

        let top_left = Vertex {
            position: top_left.extend(0.0),
            color,
            uv_coords: Vector2::new(0.0, 0.0),
        };
        let top_right = Vertex {
            position: top_right.extend(0.0),
            color,
            uv_coords: Vector2::new(1.0, 0.0),
        };
        let bottom_left = Vertex {
            position: bottom_left.extend(0.0),
            color,
            uv_coords: Vector2::new(0.0, 1.0),
        };
        let bottom_right = Vertex {
            position: bottom_right.extend(0.0),
            color,
            uv_coords: Vector2::new(1.0, 1.0),
        };

        let base = self.vertices.len() as u32;

        self.vertices.push(top_left);
        self.vertices.push(top_right);
        self.vertices.push(bottom_left);
        self.vertices.push(bottom_right);

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 1, base + 3]);
    }

    #[inline]
    pub(crate) fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.vertices.is_empty() || self.indices.is_empty() {
            return;
        }

        if self.vertices.len() > self.vertex_capacity {
            self.vertex_capacity = (self.vertices.len() * 2).max(Self::BASE_VERTEX_CAPACITY);
            self.vertex_buffer.resize(self.vertex_capacity);
        }

        if self.indices.len() > self.index_capacity {
            self.index_capacity = (self.indices.len() * 2).max(Self::BASE_INDEX_CAPACITY);
            self.index_buffer.resize(self.index_capacity);
        }

        self.vertex_buffer.write(0, &self.vertices);
        self.index_buffer.write(0, &self.indices);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);

        self.clear();
    }
}
