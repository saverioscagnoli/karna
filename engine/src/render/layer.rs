use std::ops::Index;
use std::ops::IndexMut;

use crate::render::camera::CameraData;
use crate::render::layouts::camera;
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

        Self {
            camera_buffer,
            camera_bind_group,
        }
    }
}

pub struct RenderLayers {
    world: RenderLayer,
    ui: RenderLayer,
    debug: RenderLayer,
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
