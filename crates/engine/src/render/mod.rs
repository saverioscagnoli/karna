use std::rc::Rc;

use crate::gpu::Gpu;
use crate::render::stage::Stage;

pub mod camera;
pub mod color;
pub mod draw;
pub mod geometry;
pub mod layer;
pub mod stage;
pub mod vertex;

pub struct Renderer {
    pub gpu: Rc<Gpu>,
}

impl Renderer {
    pub fn new(gpu: Rc<Gpu>) -> Self {
        Self { gpu: gpu.clone() }
    }

    pub fn create_stage(&self, viewport: math::Size<u32>) -> Stage {
        Stage::new(self.gpu.clone(), viewport)
    }
}
