#[derive(Debug)]
pub struct Batcher<V> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,

    vertex_buffer: gpu::Buffer<V>,
    index_buffer: gpu::Buffer<u32>,
}

impl<V> Batcher<V> {
    pub fn new() -> Self {
        let vertex_buffer = gpu::Buffer::new_with_capacity(
            "immediate vertex buffer",
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            10000,
        );

        let index_buffer = gpu::Buffer::new_with_capacity(
            "immediate index buffer",
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            10000,
        );

        Self {
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(1024),
            vertex_buffer,
            index_buffer,
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    #[inline]
    pub fn present<'a>(
        &'a mut self,
        rp: &mut wgpu::RenderPass<'a>,
        pipeline: &wgpu::RenderPipeline,
    ) {
        if self.vertices.is_empty() {
            return;
        }

        let vertex_count = self.vertices.len();
        let index_count = self.indices.len();

        if vertex_count > self.vertex_buffer.capacity() {
            let new_capacity = self.vertex_buffer.capacity() * 2;
            self.vertex_buffer.resize(new_capacity);
        }

        if index_count > self.index_buffer.capacity() {
            let new_capacity = self.index_buffer.capacity() * 2;
            self.index_buffer.resize(new_capacity);
        }

        self.vertex_buffer.write(0, &self.vertices);
        self.index_buffer.write(0, &self.indices);

        rp.set_pipeline(pipeline);

        rp.set_vertex_buffer(0, self.vertex_buffer.slice_all());
        rp.set_index_buffer(self.index_buffer.slice_all(), wgpu::IndexFormat::Uint32);

        rp.draw_indexed(0..index_count as u32, 0, 0..1);

        self.clear();
    }
}
