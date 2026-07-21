use std::ops::Index;
use std::ops::IndexMut;

use crate::render::Camera;
use crate::render::Color;
use crate::render::FramePacket;
use crate::render::Layer;

pub struct Cameras {
    pub world: Camera,
    pub ui: Camera,
    pub debug: Camera,
}

impl Cameras {
    pub fn update(&mut self, viewport: math::Size<u32>) {
        self.world.update(viewport);
        self.ui.update(viewport);
        self.debug.update(viewport);
    }

    pub fn write_to(&mut self, packet: &mut FramePacket, viewport: math::Size<u32>) {
        self.world.update(viewport);
        self.ui.update(viewport);
        self.debug.update(viewport);

        packet[Layer::World].camera = self.world.data();
        packet[Layer::Ui].camera = self.ui.data();
        packet[Layer::Debug].camera = self.debug.data();
    }
}

impl Index<Layer> for Cameras {
    type Output = Camera;

    fn index(&self, l: Layer) -> &Self::Output {
        match l {
            Layer::World => &self.world,
            Layer::Ui => &self.ui,
            Layer::Debug => &self.debug,
        }
    }
}

impl IndexMut<Layer> for Cameras {
    fn index_mut(&mut self, l: Layer) -> &mut Self::Output {
        match l {
            Layer::World => &mut self.world,
            Layer::Ui => &mut self.ui,
            Layer::Debug => &mut self.debug,
        }
    }
}

pub struct SceneView<'a> {
    pub(crate) cameras: &'a mut Cameras,
    pub(crate) active_layer: Layer,
    pub(crate) packet: &'a mut FramePacket,
}

impl<'a> SceneView<'a> {
    pub fn layer(&self) -> Layer {
        self.active_layer
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.active_layer = layer;
    }

    pub fn clear_color(&self) -> Color {
        self.packet.clear_color.into()
    }

    pub fn clear_color_mut(&mut self) -> &mut math::Vector4<f32> {
        &mut self.packet.clear_color
    }

    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<math::Vector4<f32>>,
    {
        self.packet.clear_color = color.into();
    }

    pub fn camera(&self) -> &Camera {
        &self.cameras[self.active_layer]
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.cameras[self.active_layer]
    }
}
