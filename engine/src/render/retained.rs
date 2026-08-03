use utils::{Handle, SlotMap};

use crate::{
    Color,
    render::{
        camera::{Camera, CameraPacket, Projection},
        layer::Layer,
        packet::FramePacket,
    },
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
        let view = match layer {
            Layer::World => self.views[0],
            Layer::Ui => self.views[1],
            Layer::Debug => self.views[1],
        };

        match (view.camera, layer) {
            (Some(handle), _) => Some(handle),
            (None, Layer::Debug) => self.views[0].camera,
            (None, _) => None,
        }
    }

    pub(crate) fn view_enabled(&self, layer: Layer) -> bool {
        let view = match layer {
            Layer::World => self.views[0],
            Layer::Ui => self.views[1],
            Layer::Debug => self.views[1],
        };

        view.enabled
    }

    pub(crate) fn view_needs_depth(&self, layer: Layer) -> bool {
        self.resolve(layer)
            .and_then(|h| self.cameras.get(h))
            .map(|c| c.is_perspective())
            .unwrap_or(false)
    }

    pub(crate) fn update_cameras(&mut self, viewport: math::Size<u32>) {
        for camera in self.cameras.values_mut() {
            camera.update(viewport);
        }
    }

    pub(crate) fn camera_packet(&self, layer: Layer, viewport: math::Size<u32>) -> CameraPacket {
        match self.resolve(layer).and_then(|h| self.cameras.get(h)) {
            Some(camera) => camera.packet(),
            None => CameraPacket {
                view_projection: Projection::standard_2d(viewport).matrix(),
            },
        }
    }
}

pub struct SceneRef<'a> {
    world: &'a mut RenderWorld,
}

impl<'a> SceneRef<'a> {
    pub fn clear_color(&self) -> Color {
        self.world.clear_color
    }

    pub fn clear_color_mut(&mut self) -> &mut Color {
        &mut self.world.clear_color
    }

    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.world.clear_color = color.into();
    }
}
