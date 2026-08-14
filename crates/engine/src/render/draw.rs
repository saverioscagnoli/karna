use crate::Camera;
use crate::config::config;
use crate::render::color::Color;
use crate::render::geometry::ImmediateGeometry;
use crate::render::layer::Layer;
use crate::render::layer::LayerKind;
use crate::render::layer::LayerMap;

pub struct DrawState {
    pub color: Color,
    pub layer: Layer,
}

impl DrawState {
    pub fn new() -> Self {
        Self {
            color: config().draw_color,
            layer: Layer::WORLD,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new()
    }
}

pub struct Draw<'a> {
    pub(crate) state: &'a mut DrawState,
    pub(crate) cameras: &'a LayerMap<Camera>,
    pub(crate) data: &'a mut LayerMap<LayerKind>,
    pub(crate) viewport: math::Size<u32>,
}

impl<'a> Draw<'a> {
    fn target(&mut self) -> &mut ImmediateGeometry {
        self.data[self.state.layer].immediate_mut()
    }

    pub fn viewport(&self) -> math::Size<u32> {
        self.viewport
    }

    pub fn color(&self) -> Color {
        self.state.color
    }

    pub fn color_mut(&mut self) -> &mut Color {
        &mut self.state.color
    }

    pub fn set_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.state.color = color.into();
    }

    pub fn camera(&self) -> &Camera {
        &self.cameras[self.state.layer]
    }

    pub fn mvp(&self) -> math::Matrix4<f32> {
        let c = self.camera();
        c.projection().matrix().matmul(&c.view_matrix())
    }
}
