use utils::{Handle, SlotMap};

use crate::{
    Color,
    render::{camera::Camera, layer::Layer, packet::FramePacket},
};

#[derive(Debug, Clone, Copy)]
pub struct ViewConfig {
    pub camera: Option<Handle<Camera>>,
    pub enabled: bool,
}

impl ViewConfig {
    fn new() -> Self {
        Self {
            camera: None,
            enabled: true,
        }
    }
}

pub struct RenderWorld {
    pub clear_color: Color,
    cameras: SlotMap<Camera>,
    views: [ViewConfig; 3],
}

impl Default for RenderWorld {
    fn default() -> Self {
        Self {
            clear_color: Color::Black,
            cameras: SlotMap::new(),
            views: [ViewConfig::new(); 3],
        }
    }
}

impl RenderWorld {
    fn resolve(&self, layer: Layer) -> Option<Handle<Camera>> {
        let view = self.views[layer];
    }
}

pub struct SceneRef<'a> {}
