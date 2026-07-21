use std::ops::Index;
use std::ops::IndexMut;

use crate::render::Vertex;
use crate::render::immediate::Batch;

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

/// CPU-side geometry for one layer. Window threads fill this while drawing;
/// the GPU buffers live in the main-thread renderer, which uploads this data
/// when the frame is presented.
#[derive(Default)]
#[derive(Debug, Clone)]
pub struct RenderLayer {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub batches: Vec<Batch>,
}

impl RenderLayer {
    pub fn new() -> Self {
        Self::default()
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

#[derive(Default)]
#[derive(Debug, Clone)]
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
