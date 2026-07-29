use crate::render::vertex::Vertex;

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Layer {
    #[default]
    World,
    Ui,
    Debug,
}

impl Layer {
    pub const ALL: [Layer; 3] = [Layer::World, Layer::Ui, Layer::Debug];
}

#[derive(Debug, Clone, Copy)]
pub struct Batch {
    pub page: usize,
    pub start: u32,
    pub count: u32,
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct LayerCpuData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub batches: Vec<Batch>,
}

impl LayerCpuData {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.batches.clear();
    }

    pub fn quad(
        &mut self,
        page: usize,
        corners: [math::Vector2<f32>; 4],
        uvs: [math::Vector2<f32>; 4],
        color: math::Vector4<f32>,
    ) {
        let base = self.vertices.len() as u32;

        for i in 0..4 {
            self.vertices.push(Vertex {
                position: math::Vector3::new(corners[i].x, corners[i].y, 0.0),
                color,
                uv: uvs[i],
            })
        }

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        match self.batches.last_mut() {
            Some(b) if b.page == page => b.count += 6,
            _ => self.batches.push(Batch {
                page,
                start: self.indices.len() as u32 - 6,
                count: 6,
            }),
        }
    }
}

pub struct LayerGpuData {
    pub vertex_buffer: gpu::Buffer<Vertex>,
    pub index_buffer: gpu::Buffer<u32>,
}
