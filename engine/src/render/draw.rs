use imgui::ActiveImgui;
use utils::Handle;

use crate::assets::AssetsView;
use crate::assets::Font;
use crate::assets::Image;
use crate::render::FramePacket;
use crate::render::Vertex;
use crate::render::canvas::Canvas;
use crate::render::canvas::CanvasPacket;
use crate::render::color::Color;
use crate::render::immediate::BatchTexture;
use crate::render::layer::Layer;
use crate::render::layer::RenderLayers;

pub struct Draw<'ctx> {
    pub(crate) r: &'ctx mut RenderLayers,
    pub(crate) active_layer: Layer,
    /// Index into `r.canvases` while drawing inside `Draw::canvas`.
    pub(crate) active_canvas: Option<usize>,
    pub(crate) color: math::Vector4<f32>,
    pub(crate) packet: &'ctx mut FramePacket,
    pub(crate) assets: AssetsView<'ctx>,
    pub(crate) imgui: ActiveImgui<'ctx>,
}

impl<'ctx> Draw<'ctx> {
    pub fn layer(&self) -> Layer {
        self.active_layer
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.active_layer = layer;
    }

    pub fn color(&self) -> Color {
        self.color.into()
    }

    pub fn color_mut(&mut self) -> &mut math::Vector4<f32> {
        &mut self.color
    }

    pub fn set_color<C>(&mut self, color: C)
    where
        C: Into<math::Vector4<f32>>,
    {
        self.color = color.into();
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
        self.packet.clear_color = color.into()
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.quad([
            math::Vector2::new(x, y),
            math::Vector2::new(x + w, y),
            math::Vector2::new(x + w, y + h),
            math::Vector2::new(x, y + h),
        ]);
    }

    /// An arbitrary convex quad in the current color, corners in order.
    pub fn quad(&mut self, corners: [math::Vector2<f32>; 4]) {
        let color = self.color;
        let white = self.assets.white_pixel();
        let uv = math::Vector2::new(
            (white.uv_min.x + white.uv_max.x) * 0.5,
            (white.uv_min.y + white.uv_max.y) * 0.5,
        );

        let v = |p: math::Vector2<f32>| Vertex {
            position: math::Vector3::new(p.x, p.y, 0.0),
            color,
            uv,
        };

        self.r
            .target(self.active_canvas, self.active_layer)
            .push_quad(corners.map(v), BatchTexture::WHITE);
    }

    /// Draws an offscreen canvas' contents as a textured quad, like `image`
    /// but for a render target.
    ///
    /// The canvas is rendered before the layers that sample it, so content
    /// drawn into it this frame shows up in the same frame. Drawing a canvas
    /// into itself samples the previous frame's contents.
    pub fn canvas_image(&mut self, canvas: &Canvas, x: f32, y: f32, w: f32, h: f32) {
        let v = |px, py, u, vv| Vertex {
            position: math::Vector3::new(px, py, 0.0),
            color: Color::White.into(),
            uv: math::Vector2::new(u, vv),
        };

        self.r.target(self.active_canvas, self.active_layer).push_quad(
            [
                v(x, y, 0.0, 0.0),
                v(x + w, y, 1.0, 0.0),
                v(x + w, y + h, 1.0, 1.0),
                v(x, y + h, 0.0, 1.0),
            ],
            BatchTexture::Canvas(canvas.id()),
        );
    }

    /// Redirects drawing to an offscreen canvas for the duration of the
    /// closure. The canvas is cleared to its own clear color each frame and
    /// rendered before the window's main pass, so it can be displayed in the
    /// same frame, e.g. in imgui through [`Canvas::texture_id`].
    pub fn canvas<F>(&mut self, canvas: &Canvas, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let index = match self.r.canvases.iter().position(|p| p.id == canvas.id()) {
            Some(i) => {
                self.r.canvases[i].sync(canvas);
                i
            }
            None => {
                self.r.canvases.push(CanvasPacket::new(canvas));
                self.r.canvases.len() - 1
            }
        };

        let prev = self.active_canvas.replace(index);
        f(self);
        self.active_canvas = prev;
    }

    pub fn text<T>(&mut self, font: Handle<Font>, text: T, x: f32, y: f32)
    where
        T: AsRef<str>,
    {
        let color = self.color;
        let font = self.assets.get_font(font);

        let mut pen = math::Vector2::new(x, y);

        for ch in text.as_ref().chars() {
            if ch == '\n' {
                pen.x = x;
                pen.y += font.line_height();
                continue;
            }

            let Some(glyph) = font.glyph(ch) else {
                continue;
            };

            if glyph.image != Handle::INVALID {
                let img = self.assets.get_image(glyph.image);
                let size = img.size.as_f32();
                let (u0, v0, u1, v1) = (img.uv_min.x, img.uv_min.y, img.uv_max.x, img.uv_max.y);

                let v = |px, py, u, vv| Vertex {
                    position: math::Vector3::new(px, py, 0.0),
                    color,
                    uv: math::Vector2::new(u, vv),
                };

                self.r.target(self.active_canvas, self.active_layer).push_quad(
                    [
                        v(pen.x, pen.y, u0, v0),
                        v(pen.x + size.width, pen.y, u1, v0),
                        v(pen.x + size.width, pen.y + size.height, u1, v1),
                        v(pen.x, pen.y + size.height, u0, v1),
                    ],
                    BatchTexture::Page(img.page as usize),
                );
            }

            pen.x += glyph.advance;
        }
    }

    pub fn debug_text<T>(&mut self, text: T, x: f32, y: f32)
    where
        T: AsRef<str>,
    {
        self.text(self.assets.debug_font(), text, x, y);
    }

    pub fn image(&mut self, image: Handle<Image>, x: f32, y: f32) {
        let img = self.assets.get_image(image);
        let size = img.size.as_f32();
        let (u0, v0, u1, v1) = (img.uv_min.x, img.uv_min.y, img.uv_max.x, img.uv_max.y);

        let v = |px, py, u, vv| Vertex {
            position: math::Vector3::new(px, py, 0.0),
            color: Color::White.into(),
            uv: math::Vector2::new(u, vv),
        };

        let layer = self.r.target(self.active_canvas, self.active_layer);
        layer.push_quad(
            [
                v(x, y, u0, v0),
                v(x + size.width, y, u1, v0),
                v(x + size.width, y + size.height, u1, v1),
                v(x, y + size.height, u0, v1),
            ],
            BatchTexture::Page(img.page as usize),
        );
    }

    /// Begins a new imgui frame.
    ///
    /// Call this once per frame.
    pub fn imgui<F>(&mut self, f: F)
    where
        F: FnOnce(&mut imgui::Ui),
    {
        let ui = self.imgui.frame();
        f(ui);

        let draw_data = self.imgui.render();
        self.r.imgui.capture(draw_data);
    }
}
