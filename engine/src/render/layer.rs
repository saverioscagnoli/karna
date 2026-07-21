use std::ops::Index;
use std::ops::IndexMut;

use crate::render::Vertex;
use crate::render::camera::CameraData;
use crate::render::immediate::Batch;
use crate::render::layouts::{self};

/// Nominative of each render layer,
/// Used for ease of access
#[repr(usize)]
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub enum Layer {
    #[default]
    World = 0,
    Ui = 1,
    Debug = 2,
}

impl Layer {
    pub const ALL: [Layer; 3] = [Layer::World, Layer::Ui, Layer::Debug];
}

pub struct RenderLayer {
    pub camera_buffer: gpu::Buffer<CameraData>,
    pub camera_bind_group: gpu::BindGroup,

    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub batches: Vec<Batch>,
    pub vertex_buffer: gpu::Buffer<Vertex>,
    pub index_buffer: gpu::Buffer<u32>,
}

impl RenderLayer {
    pub fn new() -> Self {
        let camera_buffer = gpu::Buffer::builder("Camera buffer")
            .uniform()
            .writable()
            .capacity(1)
            .build();

        let camera_bind_group = gpu::BindGroupBuilder::new("Camera bind group", layouts::camera())
            .buffer(&camera_buffer)
            .build();

        let vertex_buffer = gpu::Buffer::builder("Immediate mode vertex buffer")
            .vertex()
            .writable()
            .capacity(1024)
            .build();

        let index_buffer = gpu::Buffer::builder("Immediate mode index buffer")
            .index()
            .writable()
            .capacity(1024)
            .build();

        Self {
            camera_buffer,
            camera_bind_group,
            vertices: Vec::new(),
            indices: Vec::new(),
            batches: Vec::new(),
            vertex_buffer,
            index_buffer,
        }
    }

    fn batch_for(&mut self, page: Option<usize>) -> &mut Batch {
        let need_new = match self.batches.last() {
            Some(b) => b.page != page,
            None => true,
        };

        if need_new {
            let start = self.indices.len() as u32;

            self.batches.push(Batch {
                indices: start..start,
                page,
            });
        }

        self.batches.last_mut().unwrap()
    }

    pub fn push_quad(&mut self, corners: [Vertex; 4], page: Option<usize>) {
        let base = self.vertices.len() as u32;

        self.batch_for(page);

        self.vertices.extend_from_slice(&corners);
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        self.batches.last_mut().unwrap().indices.end += 6;
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.batches.clear();
    }
}

pub struct RenderLayers {
    world: RenderLayer,
    ui: RenderLayer,
    debug: RenderLayer,
}

impl RenderLayers {
    pub fn clear(&mut self) {
        self.world.clear();
        self.ui.clear();
        self.debug.clear();
    }
}

impl Default for RenderLayers {
    fn default() -> Self {
        Self {
            world: RenderLayer::new(),
            ui: RenderLayer::new(),
            debug: RenderLayer::new(),
        }
    }
}

impl Index<Layer> for RenderLayers {
    type Output = RenderLayer;

    fn index(&self, l: Layer) -> &Self::Output {
        match l {
            Layer::World => &self.world,
            Layer::Ui => &self.ui,
            Layer::Debug => &self.debug,
        }
    }
}

impl IndexMut<Layer> for RenderLayers {
    fn index_mut(&mut self, l: Layer) -> &mut Self::Output {
        match l {
            Layer::World => &mut self.world,
            Layer::Ui => &mut self.ui,
            Layer::Debug => &mut self.debug,
        }
    }
}
