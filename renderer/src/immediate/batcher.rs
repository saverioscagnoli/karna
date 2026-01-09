use globals::{consts, profiling};
use gpu::core::{GpuBuffer, GpuBufferBuilder};

#[derive(Debug)]
pub struct Batcher<V> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
    vertex_buffer: GpuBuffer<V>,
    index_buffer: GpuBuffer<u32>,
    pipeline: wgpu::RenderPipeline,
}

impl<V> Batcher<V> {
    pub fn new(pipeline: wgpu::RenderPipeline) -> Self {
        let vertex_buffer = GpuBufferBuilder::new()
            .label("Immediate Vertex Buffer")
            .capacity(consts::IMMEDIATE_VERTEX_BASE_CAPACITY)
            .vertex()
            .copy_dst()
            .build();

        let index_buffer = GpuBufferBuilder::new()
            .label("Immediate Vertex Buffer")
            .capacity(consts::IMMEDIATE_INDEX_BASE_CAPACITY)
            .index()
            .copy_dst()
            .build();

        Self {
            vertices: Vec::with_capacity(consts::IMMEDIATE_VERTEX_BASE_CAPACITY),
            indices: Vec::with_capacity(consts::IMMEDIATE_INDEX_BASE_CAPACITY),
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
    pub fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.vertices.is_empty() {
            return;
        }

        let vertex_count = self.vertices.len() as u32;
        let index_count = self.indices.len() as u32;

        if vertex_count > self.vertex_buffer.capacity() as u32 {
            let new_capacity = self.vertex_buffer.capacity() * 2;
            self.vertex_buffer.resize(new_capacity);
        }

        if index_count > self.index_buffer.capacity() as u32 {
            let new_capacity = self.index_buffer.capacity() * 2;
            self.index_buffer.resize(new_capacity);
        }

        self.vertex_buffer.write(0, &self.vertices);
        self.index_buffer.write(0, &self.indices);

        render_pass.set_pipeline(&self.pipeline);
        profiling::record_pipeline_switches(1);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice_all());
        render_pass.set_index_buffer(self.index_buffer.slice_all(), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..index_count, 0, 0..1);
        profiling::record_draw_call(vertex_count, index_count);

        self.clear();
    }
}
