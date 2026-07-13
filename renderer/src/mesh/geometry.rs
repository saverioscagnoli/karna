use gpu::Vertex;

#[derive(Debug, Clone)]
pub struct Geometry {
    vertex_buffer: gpu::Buffer<Vertex>,
    index_buffer: gpu::Buffer<u32>,
    index_count: u32,
}

impl Geometry {
    pub fn new(vertices: &[Vertex], indices: &[u32]) -> Self {
        let vertex_buffer = gpu::Buffer::new_filled(
            "geometry vertex buffer",
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC,
            vertices,
        );

        let index_buffer = gpu::Buffer::new_filled(
            "geometry index buffer",
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_SRC,
            indices,
        );

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    pub fn cube() -> Self {
        Self::cube_sized(1.0)
    }

    pub fn cube_sized(size: f32) -> Self {
        let h = size * 0.5;
        let white = math::Vector4::new(1.0, 1.0, 1.0, 1.0);

        // 4 verts per face, ordered so uv (0,0)-(1,1) maps consistently
        let mut positions = [
            // +X face
            [h, -h, -h],
            [h, h, -h],
            [h, h, h],
            [h, -h, h],
            // -X face
            [-h, -h, h],
            [-h, h, h],
            [-h, h, -h],
            [-h, -h, -h],
            // +Y face
            [-h, h, -h],
            [-h, h, h],
            [h, h, h],
            [h, h, -h],
            // -Y face
            [-h, -h, h],
            [-h, -h, -h],
            [h, -h, -h],
            [h, -h, h],
            // +Z face
            [-h, -h, h],
            [h, -h, h],
            [h, h, h],
            [-h, h, h],
            // -Z face
            [h, -h, -h],
            [-h, -h, -h],
            [-h, h, -h],
            [h, h, -h],
        ];

        let uvs = [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];

        let vertices: Vec<Vertex> = positions
            .iter_mut()
            .enumerate()
            .map(|(i, p)| {
                let uv = uvs[i % 4];
                Vertex::new(
                    math::Vector3::new(p[0], p[1], p[2]),
                    white,
                    math::Vector2::new(uv[0], uv[1]),
                )
            })
            .collect();

        // two triangles per face, 6 faces
        let mut indices = Vec::with_capacity(36);
        for face in 0..6u32 {
            let base = face * 4;
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        Self::new(&vertices, &indices)
    }
}
