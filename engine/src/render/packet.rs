use std::ops::Index;
use std::ops::IndexMut;

use crate::render::camera::CameraPacket;
use crate::render::layer::Layer;
use crate::render::layer::LayerCpuData;
use crate::render::mesh::DrawItem;

#[derive(Default, Clone)]
pub struct ViewPacket {
    pub camera: CameraPacket,
    pub depth: bool,
    pub enabled: bool,
    pub data: LayerCpuData,

    // Retained meshes, already sorted
    pub meshes: Vec<DrawItem>,
}

impl ViewPacket {
    fn clear(&mut self) {
        self.data.clear();
        self.meshes.clear();
    }

    fn is_empty(&self) -> bool {
        self.data.batches.is_empty() && self.meshes.is_empty()
    }
}

pub struct FramePacket {
    pub views: Vec<ViewPacket>,
}

impl Index<Layer> for FramePacket {
    type Output = ViewPacket;
    fn index(&self, l: Layer) -> &Self::Output {
        match l {
            Layer::World => &self.views[0],
            Layer::Ui => &self.views[1],
            Layer::Debug => &self.views[2],
        }
    }
}

impl IndexMut<Layer> for FramePacket {
    fn index_mut(&mut self, l: Layer) -> &mut Self::Output {
        match l {
            Layer::World => &mut self.views[0],
            Layer::Ui => &mut self.views[1],
            Layer::Debug => &mut self.views[2],
        }
    }
}

impl Default for FramePacket {
    fn default() -> Self {
        let mut views = Vec::with_capacity(3);
        views.resize_with(3, ViewPacket::default);

        Self { views }
    }
}

impl FramePacket {
    pub fn clear(&mut self) {
        for view in &mut self.views {
            view.clear();
        }
    }

    pub fn active(&self) -> impl Iterator<Item = (usize, &ViewPacket)> {
        self.views
            .iter()
            .enumerate()
            .filter(|(_, v)| v.enabled && !v.is_empty())
    }
}
