//! A 'packet' is essentially just cpu data that comes
//! from the scene simulation, draw calls, buffer commands, etc.
//!
//! Gpu data lives in the renderer

use std::ops::{Index, IndexMut};

use crate::render::{
    camera::CameraPacket,
    color::Color,
    layer::{Layer, LayerData},
};

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct LayerPacket {
    pub(crate) camera: CameraPacket,
    pub(crate) data: LayerData,
}

#[derive(Debug, Clone)]
pub struct FramePacket {
    pub clear_color: Color,
    pub world: LayerPacket,
    pub ui: LayerPacket,
    pub debug: LayerPacket,
}

impl Default for FramePacket {
    fn default() -> Self {
        Self {
            clear_color: Color::Black,
            world: LayerPacket::default(),
            ui: LayerPacket::default(),
            debug: LayerPacket::default(),
        }
    }
}

impl Index<Layer> for FramePacket {
    type Output = LayerPacket;

    fn index(&self, l: Layer) -> &Self::Output {
        match l {
            Layer::World => &self.world,
            Layer::Ui => &self.ui,
            Layer::Debug => &self.debug,
        }
    }
}

impl IndexMut<Layer> for FramePacket {
    fn index_mut(&mut self, l: Layer) -> &mut Self::Output {
        match l {
            Layer::World => &mut self.world,
            Layer::Ui => &mut self.ui,
            Layer::Debug => &mut self.debug,
        }
    }
}

impl FramePacket {
    pub fn clear(&mut self) {
        for l in Layer::ALL {
            self[l].data.vertices.clear();
            self[l].data.indices.clear();
        }
    }
}
