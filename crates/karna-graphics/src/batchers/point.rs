use karna_opengl::{bind_vertex_buffer, buffers::VertexBuffer, draw_arrays, DrawMode, Vertex};

pub struct PointBatcher {
    vbo: VertexBuffer,
    vertices: Vec<Vertex>,
}

impl PointBatcher {
    pub fn new() -> Self {
        Self {
            vbo: VertexBuffer::new(),
            vertices: Vec::new(),
        }
    }

    pub fn draw_pixel(&mut self, x: f32, y: f32, z: f32, color: [f32; 4]) {
        self.vertices.push(Vertex {
            position: [x, y, z],
            color,
            tex_coords: [0.0, 0.0],
        });
    }

    pub fn flush(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        bind_vertex_buffer(0, &self.vbo, std::mem::size_of::<Vertex>());

        self.vbo.bind();
        self.vbo.buffer_data(&self.vertices);

        draw_arrays(DrawMode::Points, 0, self.vertices.len());

        self.vertices.clear();
    }
}
