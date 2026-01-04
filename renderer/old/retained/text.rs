use crate::{Color, Layer, Transform, retained::text_renderer::GlyphInstance};
use assets::AssetManager;
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use macros::{Get, Set};
use math::Vector3;
use std::ops::{Deref, DerefMut};
use utils::{Handle, Label};

#[derive(Get, Set)]
pub struct Text {
    #[get(visibility = "pub(crate)")]
    #[get(copied, name = "font")]
    #[set(name = "set_font", also = self.tracker |= Self::font_label_f())]
    font_label: Label,

    #[get(ty = &str)]
    #[get(mut, also = self.tracker |= Self::content_f())]
    #[set(into, also = self.tracker |= Self::content_f())]
    content: String,

    #[get]
    #[get(prop = "position", ty = &Vector3, name = "position")]
    #[get(copied, prop = "position.x", ty = f32, name = "position_x")]
    #[get(copied, prop = "position.y", ty = f32, name = "position_y")]
    #[get(copied, prop = "position.z", ty = f32, name = "position_z")]
    #[get(prop = "rotation", ty = &Vector3, name = "rotation")]
    #[get(copied, prop = "rotation.x", ty = f32, name = "rotation_x")]
    #[get(copied, prop = "rotation.y", ty = f32, name = "rotation_y")]
    #[get(copied, prop = "rotation.z", ty = f32, name = "rotation_z")]
    #[get(prop = "scale", ty = &Vector3, name = "scale")]
    #[get(copied, prop = "scale.x", ty = f32, name = "scale_x")]
    #[get(copied, prop = "scale.y", ty = f32, name = "scale_y")]
    #[get(copied, prop = "scale.z", ty = f32, name = "scale_z")]
    #[get(mut, also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "position", ty = &mut Vector3, name = "position_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "position.x", ty = &mut f32, name = "position_x_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "position.y", ty = &mut f32, name = "position_y_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "position.z", ty = &mut f32, name = "position_z_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "rotation", ty = &mut Vector3, name = "rotation_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "rotation.x", ty = &mut f32, name = "rotation_x_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "rotation.y", ty = &mut f32, name = "rotation_y_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "rotation.z", ty = &mut f32, name = "rotation_z_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "scale", ty = &mut Vector3, name = "scale_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "scale.x", ty = &mut f32, name = "scale_x_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "scale.y", ty = &mut f32, name = "scale_y_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "scale.z", ty = &mut f32, name = "scale_z_mut", also = self.tracker |= Self::transform_f())]
    #[set(also = self.tracker |= Self::transform_f())]
    #[set(into, prop = "position", ty = Vector3, name = "set_position", also = self.tracker |= Self::transform_f())]
    #[set(prop = "position.x", ty = f32, name = "set_position_x", also = self.tracker |= Self::transform_f())]
    #[set(prop = "position.y", ty = f32, name = "set_position_y", also = self.tracker |= Self::transform_f())]
    #[set(prop = "position.z", ty = f32, name = "set_position_z", also = self.tracker |= Self::transform_f())]
    #[set(into, prop = "rotation", ty = Vector3, name = "set_rotation", also = self.tracker |= Self::transform_f())]
    #[set(prop = "rotation.x", ty = f32, name = "set_rotation_x", also = self.tracker |= Self::transform_f())]
    #[set(prop = "rotation.y", ty = f32, name = "set_rotation_y", also = self.tracker |= Self::transform_f())]
    #[set(prop = "rotation.z", ty = f32, name = "set_rotation_z", also = self.tracker |= Self::transform_f())]
    #[set(into, prop = "scale", ty = Vector3, name = "set_scale", also = self.tracker |= Self::transform_f())]
    #[set(prop = "scale.x", ty = f32, name = "set_scale_x", also = self.tracker |= Self::transform_f())]
    #[set(prop = "scale.y", ty = f32, name = "set_scale_y", also = self.tracker |= Self::transform_f())]
    #[set(prop = "scale.z", ty = f32, name = "set_scale_z", also = self.tracker |= Self::transform_f())]
    transform: Transform,

    #[get]
    #[get(mut, also = self.tracker |= Self::color_f())]
    #[get(mut, prop = "r", ty = &mut f32, also = self.tracker |= Self::color_f())]
    #[get(mut, prop = "g", ty = &mut f32, also = self.tracker |= Self::color_f())]
    #[get(mut, prop = "b", ty = &mut f32, also = self.tracker |= Self::color_f())]
    #[get(mut, prop = "a", ty = &mut f32, also = self.tracker |= Self::color_f())]
    #[set(into, also = self.tracker |= Self::color_f())]
    #[set(prop = "r", ty = f32, also = self.tracker |= Self::color_f())]
    #[set(prop = "g", ty = f32, also = self.tracker |= Self::color_f())]
    #[set(prop = "b", ty = f32, also = self.tracker |= Self::color_f())]
    #[set(prop = "a", ty = f32, also = self.tracker |= Self::color_f())]
    color: Color,

    tracker: u8,

    layout: Layout,
    glyphs: Vec<GlyphInstance>,
}

impl Text {
    pub fn new<T: Into<String>>(font: Label, text: T) -> Self {
        let mut text = Self {
            font_label: font,
            content: text.into(),
            transform: Transform::default(),
            color: Color::White,
            layout: Layout::new(CoordinateSystem::PositiveYDown),
            glyphs: Vec::new(),
            tracker: 0,
        };

        text.mark_all();

        text
    }

    #[inline]
    pub(crate) fn glyph_instances(&self) -> &[GlyphInstance] {
        &self.glyphs
    }

    #[inline]
    pub(crate) fn rebuild(&mut self, assets: &AssetManager) {
        if !self.changed(Self::content_f())
            && !self.changed(Self::transform_f())
            && !self.changed(Self::color_f())
        {
            return;
        }

        if self.changed(Self::content_f()) {
            self.layout.clear();

            let font = assets.get_font(&self.font_label);

            self.layout.append(
                &[font.inner()],
                &TextStyle::new(&self.content, font.size() as f32, 0),
            );

            self.reset_one(Self::content_f());
            self.tracker |= Self::transform_f() | Self::color_f();
        }

        if self.changed(Self::transform_f()) || self.changed(Self::color_f()) {
            self.glyphs.clear();

            let cos = self.transform.rotation.z.cos();
            let sin = self.transform.rotation.z.sin();

            for glyph in self.layout.glyphs() {
                if glyph.width == 0 || glyph.height == 0 {
                    continue;
                }

                let texture_label =
                    Label::new(&format!("{}_{}", self.font_label.raw(), glyph.parent));

                let (x, y, w, h) = assets.get_texture_coords(texture_label);

                let local_x = glyph.x * self.transform.scale.x;
                let local_y = glyph.y * self.transform.scale.y;

                let rotated_x = local_x * cos - local_y * sin;
                let rotated_y = local_x * sin + local_y * cos;

                let glyph = GlyphInstance {
                    position: [
                        self.transform.position.x + rotated_x,
                        self.transform.position.y + rotated_y,
                        self.transform.position.z,
                    ],
                    size: [
                        glyph.width as f32 * self.transform.scale.x,
                        glyph.height as f32 * self.transform.scale.y,
                    ],
                    uv_offset: [x as f32, y as f32],
                    uv_scale: [w as f32, h as f32],
                    color: self.color.into(),
                    rotation: [0.0, 0.0, self.transform.rotation.z],
                };

                self.glyphs.push(glyph);
            }

            self.reset_one(Self::transform_f());
            self.reset_one(Self::color_f());
        }
    }
}

// Dirty tracking
impl Text {
    #[inline]
    const fn font_label_f() -> u8 {
        1 << 0
    }

    #[inline]
    const fn content_f() -> u8 {
        1 << 1
    }

    const fn transform_f() -> u8 {
        1 << 2
    }

    const fn color_f() -> u8 {
        1 << 3
    }

    #[inline]
    const fn changed(&self, flag: u8) -> bool {
        self.tracker & flag != 0
    }

    #[inline]
    const fn mark_all(&mut self) {
        self.tracker =
            Self::font_label_f() | Self::content_f() | Self::transform_f() | Self::color_f();
    }

    #[inline]
    const fn reset_one(&mut self, flag: u8) {
        self.tracker &= !flag;
    }

    #[inline]
    pub(crate) const fn is_dirty(&self) -> bool {
        self.tracker != 0
    }
}

#[derive(Clone, Copy)]
#[derive(Get)]
pub struct TextHandle {
    #[get]
    pub(crate) layer: Layer,
    pub(crate) handle: Handle<Text>,
}

impl Deref for TextHandle {
    type Target = Handle<Text>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl DerefMut for TextHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

impl TextHandle {
    pub fn dummy() -> Self {
        Self {
            layer: Layer::World,
            handle: Handle::dummy(),
        }
    }
}
