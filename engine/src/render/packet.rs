use std::ops::Index;
use std::ops::IndexMut;

use crate::render::camera::CameraPacket;
use crate::render::color::Color;
use crate::render::layer::Layer;
use crate::render::layer::LayerCpuData;

#[derive(Default)]
pub struct LayerPacket {
    pub camera: CameraPacket,
    pub data: LayerCpuData,
}

pub struct FramePacket {
    pub clear_color: Color,
    pub world: LayerPacket,
    pub ui: LayerPacket,
    pub debug: LayerPacket,
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

impl FramePacket {
    pub fn clear(&mut self) {
        for layer in Layer::ALL {
            self[layer].data.clear();
        }
    }
}
