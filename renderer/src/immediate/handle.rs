use assets::AssetServerGuard;
use assets::Font;
use assets::Image;
use glyph_brush_layout::GlyphPositioner;
use imgui::ActiveImgui;
use math::Size;
use math::Vector2;
use math::Vector4;
use utils::Handle;

use crate::Color;
use crate::Renderer;

pub struct Draw<'r> {
    color: Vector4<f32>,
    renderer: &'r mut Renderer,
    assets: AssetServerGuard<'r>,
    imgui: ActiveImgui<'r>,
}

impl<'r> Draw<'r> {
    #[doc(hidden)]
    pub fn _new(r: &'r mut Renderer, assets: AssetServerGuard<'r>, imgui: ActiveImgui<'r>) -> Self {
        Self {
            color: Color::White.into(),
            renderer: r,
            assets,
            imgui,
        }
    }

    pub fn color(&self) -> Color {
        self.color.into()
    }

    pub fn set_color<C: Into<Vector4<f32>>>(&mut self, color: C) {
        self.color = color.into();
    }

    pub fn clear_color(&self) -> Color {
        self.renderer.clear_color.into()
    }

    pub fn set_clear_color<C: Into<Vector4<f32>>>(&mut self, color: C) {
        self.renderer.clear_color = color.into()
    }

    pub fn point(&mut self, x: f32, y: f32) {
        let layer = self.renderer.active_layer_mut();
        layer.immediate.push_point(x, y, self.color, &self.assets);
    }

    pub fn point_v<P: Into<Vector2<f32>>>(&mut self, pos: P) {
        let pos: Vector2<f32> = pos.into();
        self.point(pos.x, pos.y);
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let layer = self.renderer.active_layer_mut();
        layer
            .immediate
            .push_line(x1, y1, x2, y2, self.color, &self.assets);
    }

    pub fn line_v<P: Into<Vector2<f32>>, Q: Into<Vector2<f32>>>(&mut self, pos1: P, pos2: Q) {
        let pos1: Vector2<f32> = pos1.into();
        let pos2: Vector2<f32> = pos2.into();

        self.line(pos1.x, pos1.y, pos2.x, pos2.y);
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let layer = self.renderer.active_layer_mut();
        layer
            .immediate
            .push_untextured_quad(x, y, w, h, self.color, &self.assets);
    }

    pub fn rect_v<P: Into<Vector2<f32>>, S: Into<Size<f32>>>(&mut self, pos: P, size: S) {
        let pos: Vector2<f32> = pos.into();
        let size: Size<f32> = size.into();
        self.rect(pos.x, pos.y, size.width, size.height);
    }

    pub fn image(&mut self, image: Handle<Image>, x: f32, y: f32) {
        let layer = self.renderer.active_layer_mut();
        let image = self.assets.get_image(image);

        layer.immediate.push_textured_quad(
            x,
            y,
            image.size.width as f32,
            image.size.height as f32,
            self.color,
            image.uv,
        );
    }

    pub fn image_v<P: Into<math::Vector2<f32>>>(&mut self, pos: P, image_h: Handle<Image>) {
        let pos: math::Vector2<f32> = pos.into();
        self.image(image_h, pos.x, pos.y);
    }

    pub fn text<T: AsRef<str>>(&mut self, font_h: Handle<Font>, text: T, x: f32, y: f32) {
        let text: &str = text.as_ref();
        let view = self.renderer.size();
        let font = self.assets.get_font(font_h);

        let geometry = glyph_brush_layout::SectionGeometry {
            screen_position: (x, y),
            bounds: (view.width as f32, f32::INFINITY),
        };

        let sections = &[glyph_brush_layout::SectionText {
            text,
            scale: glyph_brush_layout::ab_glyph::PxScale::from(font.size() as f32),
            font_id: glyph_brush_layout::FontId(0),
            ..Default::default()
        }];

        let fonts = [font._inner()];
        let glyphs = glyph_brush_layout::Layout::default_wrap()
            .calculate_glyphs(&fonts, &geometry, sections);

        let layer = self.renderer.active_layer_mut();

        for sg in &glyphs {
            // Recover the actual char from byte_index into the original text
            let ch = text[sg.byte_index..]
                .chars()
                .next()
                .expect("Invalid byte index");
            if ch.is_whitespace() {
                continue; // whitespace has no cached glyph/quad, position already accounts for its advance
            }

            let info = self.assets.get_glyph(font_h, ch, font.size());

            let draw_x = sg.glyph.position.x + info.bearing.x;
            let draw_y = sg.glyph.position.y + info.bearing.y;

            layer.immediate.push_textured_quad(
                draw_x,
                draw_y,
                info.size.width,
                info.size.height,
                self.color,
                info.uv,
            );
        }
    }

    pub fn text_v<T: AsRef<str>, P: Into<math::Vector2<f32>>>(
        &mut self,
        font_h: Handle<Font>,
        text: T,
        pos: P,
    ) {
        let pos: math::Vector2<f32> = pos.into();

        self.text(font_h, text, pos.x, pos.y);
    }

    pub fn debug_text<T: AsRef<str>>(&mut self, text: T, x: f32, y: f32) {
        self.text(self.assets.debug_font_handle(), text, x, y);
    }

    pub fn debug_text_v<T: AsRef<str>, P: Into<math::Vector2<f32>>>(&mut self, text: T, pos: P) {
        let pos: math::Vector2<f32> = pos.into();

        self.debug_text(text, pos.x, pos.y);
    }

    pub fn circle(&mut self, x: f32, y: f32, r: f32) {
        let layer = self.renderer.active_layer_mut();
        layer.immediate.push_cirlce(r, x, y, self.color);
    }

    pub fn circle_v<P: Into<Vector2<f32>>>(&mut self, pos: P, r: f32) {
        let pos: Vector2<f32> = pos.into();
        self.circle(pos.x, pos.y, r);
    }

    pub fn texture_atlas(&mut self, x: f32, y: f32) {
        self.image(self.assets.atlas_handle(), x, y);
    }

    pub fn texture_atlas_v<P: Into<math::Vector2<f32>>>(&mut self, pos: P) {
        let pos: math::Vector2<f32> = pos.into();
        self.texture_atlas(pos.x, pos.y);
    }

    pub fn imgui<F: FnOnce(&imgui::Ui)>(&mut self, f: F) {
        let ui = self.imgui.new_frame();
        self.renderer.imgui_renderer.frame_created = true;

        f(ui);
    }
}
