use std::sync::Arc;

use assets::AssetServerView;
use assets::Font;
use assets::Image;
use assets::ReadOnly;
use glyph_brush_layout::GlyphPositioner;
use imgui::ActiveImgui;
use math::Size;
use math::Vector2;
use math::Vector4;
use utils::Handle;

use crate::Color;
use crate::Layer;
use crate::LayerId;
use crate::Renderer;

#[derive(Clone, Copy, Default)]
pub struct SrcRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Clone, Copy, Default)]
pub struct Flip {
    pub x: bool,
    pub y: bool,
}

impl Flip {
    pub const NONE: Flip = Flip { x: false, y: false };
    pub const X: Flip = Flip { x: true, y: false };
    pub const Y: Flip = Flip { x: false, y: true };
    pub const BOTH: Flip = Flip { x: true, y: true };
}

pub struct Draw<'r> {
    renderer: &'r mut Renderer,
    assets: AssetServerView<'r, ReadOnly>,
    imgui: ActiveImgui<'r>,
}

impl<'r> Draw<'r> {
    #[doc(hidden)]
    pub fn _new(
        r: &'r mut Renderer,
        assets: AssetServerView<'r, ReadOnly>,
        imgui: ActiveImgui<'r>,
    ) -> Self {
        Self {
            renderer: r,
            assets,
            imgui,
        }
    }

    pub fn push_state(&mut self) {
        self.renderer.active_layer_mut().immediate.push_state();
    }

    pub fn pop_state(&mut self) {
        self.renderer.active_layer_mut().immediate.pop_state();
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.renderer.active_layer = LayerId(layer as usize)
    }

    pub fn color(&self) -> Color {
        self.renderer.active_layer().immediate.draw_color().into()
    }

    pub fn set_color<C: Into<Vector4<f32>>>(&mut self, color: C) {
        self.renderer
            .active_layer_mut()
            .immediate
            .set_draw_color(color.into());
    }

    pub fn clear_color(&self) -> Color {
        self.renderer.clear_color.into()
    }

    pub fn set_clear_color<C: Into<Vector4<f32>>>(&mut self, color: C) {
        self.renderer.clear_color = color.into()
    }

    pub fn depth(&self) -> f32 {
        self.renderer.active_layer().immediate.depth()
    }

    pub fn set_depth(&mut self, d: f32) {
        self.renderer.active_layer_mut().immediate.set_depth(d);
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.renderer.active_layer_mut().immediate.translate(x, y);
    }

    pub fn translate_v<T: Into<Vector2<f32>>>(&mut self, translation: T) {
        let t: Vector2<f32> = translation.into();

        self.translate(t.x, t.y);
    }

    pub fn rotate(&mut self, angle_radians: f32) {
        self.renderer
            .active_layer_mut()
            .immediate
            .rotate(angle_radians);
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        self.renderer.active_layer_mut().immediate.scale(x, y);
    }

    #[inline]
    pub fn scale_v<S: Into<Vector2<f32>>>(&mut self, scale: S) {
        let s: Vector2<f32> = scale.into();

        self.scale(s.x, s.y)
    }

    pub fn point(&mut self, x: f32, y: f32) {
        let layer = self.renderer.active_layer_mut();
        layer.immediate.push_point(x, y, &self.assets);
    }

    pub fn point_v<P: Into<Vector2<f32>>>(&mut self, pos: P) {
        let pos: Vector2<f32> = pos.into();
        self.point(pos.x, pos.y);
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let layer = self.renderer.active_layer_mut();
        layer.immediate.push_line(x1, y1, x2, y2, &self.assets);
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
            .push_untextured_quad(x, y, w, h, &self.assets);
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
            image.uv,
        );
    }

    pub fn image_v<P: Into<math::Vector2<f32>>>(&mut self, image_h: Handle<Image>, pos: P) {
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
                info.uv,
            );
        }
    }

    pub fn image_ex<P, S>(
        &mut self,
        image_h: Handle<Image>,
        dst_pos: P,
        dst_size: S,
        src: SrcRect,
        flip: Flip,
    ) where
        P: Into<Vector2<f32>>,
        S: Into<Size<f32>>,
    {
        let dst_pos: Vector2<f32> = dst_pos.into();
        let dst_size: Size<f32> = dst_size.into();

        let layer = self.renderer.active_layer_mut();
        let image = self.assets.get_image(image_h);

        // Full UV rect (u0, v0, u1, v1) this image occupies in its texture/atlas.
        let uv = image.uv;
        let uv_w = uv.z - uv.x;
        let uv_h = uv.w - uv.y;

        let img_w = image.size.width as f32;
        let img_h = image.size.height as f32;

        // Map the pixel-space src rect into the image's UV rect.
        let u_left = uv.x + (src.x as f32 / img_w) * uv_w;
        let v_top = uv.y + (src.y as f32 / img_h) * uv_h;
        let u_right = uv.x + ((src.x as f32 + src.w as f32) / img_w) * uv_w;
        let v_bottom = uv.y + ((src.y as f32 + src.h as f32) / img_h) * uv_h;

        let (u0, u1) = if flip.x {
            (u_right, u_left)
        } else {
            (u_left, u_right)
        };
        let (v0, v1) = if flip.y {
            (v_bottom, v_top)
        } else {
            (v_top, v_bottom)
        };

        let src_uv = Vector4::new(u0, v0, u1, v1);

        layer.immediate.push_textured_quad(
            dst_pos.x,
            dst_pos.y,
            dst_size.width,
            dst_size.height,
            src_uv,
        );
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
        layer.immediate.push_circle(r, x, y);
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

    pub fn console(&mut self) {
        self.push_state();

        let entries: Vec<(logging::Level, Arc<str>)> = {
            let logs = self.renderer.logs.read();
            logs.iter()
                .map(|(level, message)| (*level, message.clone()))
                .collect()
        };

        let mut offset_y = 10.0;

        for (level, message) in &entries {
            let color = match level {
                logging::Level::Trace => Color::Purple,
                logging::Level::Debug => Color::Blue,
                logging::Level::Info => Color::Green,
                logging::Level::Warn => Color::Yellow,
                logging::Level::Error => Color::Red,
            };

            self.set_color(color);
            self.debug_text(message, 10.0, offset_y);

            if message.len() > 150 {
                offset_y += 70.0;
            } else {
                offset_y += 20.0;
            }
        }

        self.pop_state();
    }
}
