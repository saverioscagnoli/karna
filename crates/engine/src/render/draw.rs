use utils::Handle;

use crate::Camera;
use crate::assets::AssetServer;
use crate::assets::image::Image;
use crate::config::config;
use crate::render::color::Color;
use crate::render::geometry::BatchTexture;
use crate::render::geometry::ImmediateGeometry;
use crate::render::layer::Layer;
use crate::render::layer::LayerKind;
use crate::render::layer::LayerMap;
use crate::render::vertex::Vertex;

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
    pub(crate) assets: &'a AssetServer,
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

    pub fn layer(&self) -> Layer {
        self.state.layer
    }

    /// Redirects subsequent draw calls to `layer`.
    ///
    /// Only layers present in the stage's [`LayerMap`] are valid; unknown
    /// layers are ignored rather than panicking on the next draw call.
    pub fn set_layer(&mut self, layer: Layer) {
        if !self.data.contains(layer) {
            logging::error!("Ignoring draw on unregistered layer: {:?}", layer);
            return;
        }

        self.state.layer = layer;
    }

    pub fn camera(&self) -> &Camera {
        &self.cameras[self.state.layer]
    }

    pub fn mvp(&self) -> math::Matrix4<f32> {
        self.camera().mvp()
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let color = self.state.color.into();
        let target = self.target();
        let base = target.vertices.len() as u32;

        target.vertices.extend_from_slice(&[
            Vertex {
                position: math::Vector3::new(x, y, 0.0),
                color,
                uv: math::Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: math::Vector3::new(x + w, y, 0.0),
                color,
                uv: math::Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: math::Vector3::new(x + w, y + h, 0.0),
                color,
                uv: math::Vector2::new(1.0, 1.0),
            },
            Vertex {
                position: math::Vector3::new(x, y + h, 0.0),
                color,
                uv: math::Vector2::new(0.0, 1.0),
            },
        ]);

        target
            .indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        target.push_batch(BatchTexture::White, 6);
    }

    pub fn rect_v<P, S>(&mut self, pos: P, size: S)
    where
        P: Into<math::Vector2<f32>>,
        S: Into<math::Size<f32>>,
    {
        let pos: math::Vector2<f32> = pos.into();
        let size: math::Size<f32> = size.into();

        self.rect(pos.x, pos.y, size.width, size.height);
    }

    pub fn image(&mut self, handle: Handle<Image>, x: f32, y: f32) {
        let image = self.assets.get_image(handle);
        let color = self.state.color.into();

        let w = image.size.width as f32;
        let h = image.size.height as f32;

        let target = self.target();
        let base = target.vertices.len() as u32;

        target.vertices.extend_from_slice(&[
            Vertex {
                position: math::Vector3::new(x, y, 0.0),
                color,
                uv: image.uv_min,
            },
            Vertex {
                position: math::Vector3::new(x + w, y, 0.0),
                color,
                uv: math::Vector2::new(image.uv_max.x, image.uv_min.y),
            },
            Vertex {
                position: math::Vector3::new(x + w, y + h, 0.0),
                color,
                uv: image.uv_max,
            },
            Vertex {
                position: math::Vector3::new(x, y + h, 0.0),
                color,
                uv: math::Vector2::new(image.uv_min.x, image.uv_max.y),
            },
        ]);

        target
            .indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        target.push_batch(BatchTexture::Atlas(image.page), 6);
    }
}
