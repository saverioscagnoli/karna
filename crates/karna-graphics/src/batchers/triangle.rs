use karna_opengl::{
    bind_vertex_buffer,
    buffers::{IndexBuffer, VertexBuffer},
    draw_elements, DataType, DrawMode, Vertex,
};

/// A batcher for every vertex group that
/// is drawn with multiple triangles, such as a rectangle.
pub struct TriangleBatcher {
    vbo: VertexBuffer,
    ebo: IndexBuffer,

    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl TriangleBatcher {
    pub fn new() -> Self {
        Self {
            vbo: VertexBuffer::new(),
            ebo: IndexBuffer::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, z: f32, w: f32, h: f32, color: [f32; 4]) {
        let index = self.vertices.len() as u32;

        self.vertices.push(Vertex {
            position: [x, y, z],
            color,
            tex_coords: [0.0, 0.0],
        });

        self.vertices.push(Vertex {
            position: [x + w, y, z],
            color,
            tex_coords: [1.0, 0.0],
        });

        self.vertices.push(Vertex {
            position: [x + w, y + h, z],
            color,
            tex_coords: [1.0, 1.0],
        });

        self.vertices.push(Vertex {
            position: [x, y + h, z],
            color,
            tex_coords: [0.0, 1.0],
        });

        self.indices.push(index);
        self.indices.push(index + 1);
        self.indices.push(index + 2);

        self.indices.push(index);
        self.indices.push(index + 2);
        self.indices.push(index + 3);
    }

    pub fn flush(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        bind_vertex_buffer(0, &self.vbo, std::mem::size_of::<Vertex>());

        self.vbo.bind();
        self.ebo.bind();

        self.vbo.buffer_data(&self.vertices);
        self.ebo.buffer_data(&self.indices);

        draw_elements(
            DrawMode::Triangles,
            self.indices.len(),
            DataType::UnsignedInt,
            std::ptr::null(),
        );

        self.vertices.clear();
    }
}
