use sokol::gfx as sg;

use crate::immediate::immediate_2d as shd;

pub struct Batcher<V> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
    vertex_buffer: sg::Buffer,
    index_buffer: sg::Buffer,
    capacity: usize,
}

impl<V: Copy> Batcher<V> {
    pub fn new() -> Self {
        let capacity = 1024;
        let vertex_buffer = sg::make_buffer(&sg::BufferDesc {
            size: capacity * std::mem::size_of::<V>(),
            usage: sg::BufferUsage {
                vertex_buffer: true,
                stream_update: true,
                ..Default::default()
            },
            label: c"immediate vertex buffer".as_ptr(),
            ..Default::default()
        });
        let index_buffer = sg::make_buffer(&sg::BufferDesc {
            size: capacity * std::mem::size_of::<u32>(),
            usage: sg::BufferUsage {
                index_buffer: true,
                stream_update: true,
                ..Default::default()
            },
            label: c"immediate index buffer".as_ptr(),
            ..Default::default()
        });

        Self {
            vertices: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(capacity),
            vertex_buffer,
            index_buffer,
            capacity,
        }
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Uploads and draws. `vp` is the view-projection to push as uniforms.
    pub fn present(&mut self, pipeline: sg::Pipeline, vp: &shd::VsParams) {
        if self.vertices.is_empty() {
            return;
        }

        // NOTE: sokol stream buffers can only be updated ONCE per frame.
        // If you have multiple batchers sharing draw order this is fine
        // since each owns its own buffer.
        sg::update_buffer(self.vertex_buffer, &sg::slice_as_range(&self.vertices));
        sg::update_buffer(self.index_buffer, &sg::slice_as_range(&self.indices));

        let bindings = sg::Bindings {
            vertex_buffers: {
                let mut vb = [sg::Buffer::default(); sg::MAX_VERTEXBUFFER_BINDSLOTS];
                vb[0] = self.vertex_buffer;
                vb
            },
            index_buffer: self.index_buffer,
            ..Default::default()
        };

        sg::apply_pipeline(pipeline);
        sg::apply_bindings(&bindings);
        sg::apply_uniforms(
            shd::UB_VS_PARAMS,
            &sg::slice_as_range(std::slice::from_ref(vp)),
        );
        sg::draw(0, self.indices.len(), 1);

        self.clear();
    }
}
