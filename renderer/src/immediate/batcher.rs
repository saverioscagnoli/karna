#[derive(Debug)]
pub struct Batcher<V> {
    vertex_buffer: gpu::Buffer<V>,
    index_buffer: gpu::Buffer<u32>,
    index_count: usize,
}

impl<V> Batcher<V> {
    pub fn new() -> Self {
        Self {
            vertex_buffer: gpu::Buffer::new_with_capacity(
                "batcher-vertices",
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                0,
            ),
            index_buffer: gpu::Buffer::new_with_capacity(
                "batcher-indices",
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                0,
            ),
            index_count: 0,
        }
    }

    /// Upload CPU-side geometry for this frame.
    pub fn upload(&mut self, vertices: &[V], indices: &[u32]) {
        self.vertex_buffer.write_all(vertices);
        self.index_buffer.write_all(indices);
        self.index_count = indices.len();
    }

    pub fn bind<'rp>(&'rp self, rp: &mut wgpu::RenderPass<'rp>, pipeline: &wgpu::RenderPipeline) {
        if self.index_count == 0 {
            return;
        }

        rp.set_pipeline(pipeline);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice_all());
        rp.set_index_buffer(self.index_buffer.slice_all(), wgpu::IndexFormat::Uint32);
    }

    pub fn present<'rp>(
        &'rp self,
        rp: &mut wgpu::RenderPass<'rp>,
        pipeline: &wgpu::RenderPipeline,
    ) {
        if self.index_count == 0 {
            return;
        }

        self.bind(rp, pipeline);
        rp.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
