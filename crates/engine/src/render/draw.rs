use crate::Color;
use crate::assets::AssetServer;
use crate::assets::Image;
use crate::config;
use crate::render::Camera;
use crate::render::layer::Layer;
use crate::render::layer::LayerData;
use crate::render::layer::LayerMap;
use crate::render::vertex::Vertex;
use math as m;
use utils::Handle;

pub struct Draw<'w> {
    color: Color,
    layer: Layer,
    cameras: &'w LayerMap<Camera>,
    data: &'w mut LayerMap<LayerData>,
    viewport: m::Size<u32>,
    assets: &'w AssetServer,
}

impl<'w> Draw<'w> {
    pub(crate) fn new(
        cameras: &'w LayerMap<Camera>,
        data: &'w mut LayerMap<LayerData>,
        viewport: m::Size<u32>,
        assets: &'w AssetServer,
    ) -> Self {
        let config = config();

        Self {
            color: config.render.draw_color,
            layer: config.render.layer,
            cameras,
            data,
            viewport,
            assets,
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn color_mut(&mut self) -> &mut Color {
        &mut self.color
    }

    pub fn set_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.color = color.into();
    }

    pub fn layer(&self) -> Layer {
        self.layer
    }

    pub fn layer_mut(&mut self) -> &mut Layer {
        &mut self.layer
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.layer = layer;
    }

    pub fn camera(&self) -> &Camera {
        &self.cameras[self.layer]
    }

    pub fn viewport(&self) -> m::Size<u32> {
        self.viewport
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let uv = self.assets.white_uv();
        let page = self.assets.white_page();
        let color: m::Vector4<f32> = self.color.into();

        self.data[self.layer].immediate_mut().push_quad(
            page,
            [
                Vertex {
                    position: m::Vector3::new(x, y, 0.0),
                    color,
                    uv,
                },
                Vertex {
                    position: m::Vector3::new(x + w, y, 0.0),
                    color,
                    uv,
                },
                Vertex {
                    position: m::Vector3::new(x + w, y + h, 0.0),
                    color,
                    uv,
                },
                Vertex {
                    position: m::Vector3::new(x, y + h, 0.0),
                    color,
                    uv,
                },
            ],
        );
    }

    pub fn image(&mut self, handle: Handle<Image>, x: f32, y: f32) {
        let image = self.assets.get_image(handle);
        let size = image.size.cast::<f32>();

        self.image_sized(handle, x, y, size.w(), size.h());
    }

    pub fn image_sized(&mut self, handle: Handle<Image>, x: f32, y: f32, w: f32, h: f32) {
        let image = *self.assets.get_image(handle);
        let min = image.uv_min;
        let max = image.uv_max;
        let color: m::Vector4<f32> = self.color.into();

        self.data[self.layer].immediate_mut().push_quad(
            image.page,
            [
                Vertex {
                    position: m::Vector3::new(x, y, 0.0),
                    color,
                    uv: min,
                },
                Vertex {
                    position: m::Vector3::new(x + w, y, 0.0),
                    color,
                    uv: m::Vector2::new(max.x, min.y),
                },
                Vertex {
                    position: m::Vector3::new(x + w, y + h, 0.0),
                    color,
                    uv: max,
                },
                Vertex {
                    position: m::Vector3::new(x, y + h, 0.0),
                    color,
                    uv: m::Vector2::new(min.x, max.y),
                },
            ],
        );
    }
}
