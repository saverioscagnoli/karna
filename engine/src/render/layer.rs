use std::ops::{Index, IndexMut};

use gpu::Gpu;

use crate::render::{
    camera::{Camera, Projection},
    vertex::Vertex,
};

#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub enum Layer {
    #[default]
    World,
    Ui,
    Debug,
}

impl Layer {
    pub const ALL: [Layer; 3] = [Layer::World, Layer::Ui, Layer::Debug];
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct LayerCPU {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct LayerGPU {
    pub(crate) vertex: gpu::Buffer<Vertex>,
    pub(crate) index: gpu::Buffer<u32>,
}

impl LayerGPU {
    pub fn new(gpu: &Gpu) -> Self {
        let vertex = gpu::Buffer::new(
            gpu,
            "immediate vertex buffer",
            gpu::BufferUsages::VERTEX,
            1024,
        );

        let index = gpu::Buffer::new(
            gpu,
            "immediate index buffer",
            gpu::BufferUsages::INDEX,
            1024,
        );

        Self { vertex, index }
    }
}

#[derive(Debug)]
pub struct LayerCameras {
    world: Camera,
    ui: Camera,
    debug: Camera,
}

impl Index<Layer> for LayerCameras {
    type Output = Camera;

    fn index(&self, l: Layer) -> &Self::Output {
        match l {
            Layer::World => &self.world,
            Layer::Ui => &self.ui,
            Layer::Debug => &self.debug,
        }
    }
}

impl IndexMut<Layer> for LayerCameras {
    fn index_mut(&mut self, l: Layer) -> &mut Self::Output {
        match l {
            Layer::World => &mut self.world,
            Layer::Ui => &mut self.ui,
            Layer::Debug => &mut self.debug,
        }
    }
}

impl LayerCameras {
    pub fn new(view: math::Size<u32>) -> Self {
        Self {
            world: Camera::new(Projection::standard_2d(view)),
            ui: Camera::new(Projection::standard_2d(view)),
            debug: Camera::new(Projection::standard_2d(view)),
        }
    }
}
