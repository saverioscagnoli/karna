mod renderer;

use crate::{Color, text::renderer::GlyphInstance};
use assets::AssetManager;
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use macros::{Get, Set, With};
use math::Vector2;
use std::cell::Cell;
use utils::map::Label;

// Re-exports
pub use renderer::TextRenderer;

#[derive(Get, Set, With)]
pub struct Text {
    #[get(visibility = "pub(crate)")]
    font_label: Label,

    #[get(ty = &str)]
    #[get(mut, also = self.mark_layout())]
    #[set(into, also = self.mark_layout())]
    #[with(into)]
    content: String,

    #[get]
    #[get(mut, also = self.mark_visuals())]
    #[get(mut, prop = "x", ty = &mut f32, also = self.mark_visuals())]
    #[get(mut, prop = "y", ty = &mut f32, also = self.mark_visuals())]
    #[set(into, also = self.mark_visuals())]
    #[set(prop = "x", ty = f32, also = self.mark_visuals())]
    #[set(prop = "y", ty = f32, also = self.mark_visuals())]
    position: Vector2,

    #[get]
    #[get(mut, also = self.mark_visuals())]
    #[get(mut, prop = "x", ty = &mut f32, also = self.mark_visuals())]
    #[get(mut, prop = "y", ty = &mut f32, also = self.mark_visuals())]
    #[set(into, also = self.mark_visuals())]
    #[set(prop = "x", ty = f32, also = self.mark_visuals())]
    #[set(prop = "y", ty = f32, also = self.mark_visuals())]
    scale: Vector2,

    #[get(copied)]
    #[get(mut, also = self.mark_visuals())]
    #[set(also = self.mark_visuals())]
    rotation: f32,

    #[get]
    #[get(mut, also = self.mark_visuals())]
    #[get(mut, prop = "r", ty = &mut f32, also = self.mark_visuals())]
    #[get(mut, prop = "g", ty = &mut f32, also = self.mark_visuals())]
    #[get(mut, prop = "b", ty = &mut f32, also = self.mark_visuals())]
    #[get(mut, prop = "a", ty = &mut f32, also = self.mark_visuals())]
    #[set(into, also = self.mark_visuals())]
    #[set(prop = "r", ty = f32, also = self.mark_visuals())]
    #[set(prop = "g", ty = f32, also = self.mark_visuals())]
    #[set(prop = "b", ty = f32, also = self.mark_visuals())]
    #[set(prop = "a", ty = f32, also = self.mark_visuals())]
    color: Color,

    dirty_layout: Cell<bool>,
    dirty_visuals: Cell<bool>,

    layout: Layout,
    glyph_instances: Vec<GlyphInstance>,
}

impl Text {
    pub fn new(font_label: Label) -> Self {
        Self {
            font_label,
            content: String::new(),
            position: Vector2::new(0.0, 0.0),
            scale: Vector2::ones(),
            rotation: 0.0,
            color: Color::White,
            dirty_layout: Cell::new(true),
            dirty_visuals: Cell::new(true),
            layout: Layout::new(CoordinateSystem::PositiveYDown),
            glyph_instances: Vec::new(),
        }
    }

    #[inline]
    pub(crate) fn glyph_instances(&self) -> &Vec<GlyphInstance> {
        &self.glyph_instances
    }

    #[inline]
    pub(crate) fn rebuild(&mut self, assets: &AssetManager) {
        if !self.dirty_layout.get() && !self.dirty_visuals.get() {
            return;
        }

        // Only runs if content changes
        // Separate from visuals, cause it's heavier to compute
        if self.dirty_layout.get() {
            self.layout.clear();

            let font = assets.get_font(&self.font_label);

            self.layout.append(
                &[font.inner()],
                &TextStyle::new(&self.content, font.size() as f32, 0),
            );

            self.dirty_layout.set(false);

            // If layout changed, visuals must be regenerated
            self.dirty_visuals.set(true);
        }

        // Runs if layout or something like the color and/or scale changed
        if self.dirty_visuals.get() {
            self.glyph_instances.clear();

            let color_array: [f32; 4] = self.color.into();
            let cos_rot = self.rotation.cos();
            let sin_rot = self.rotation.sin();

            for glyph in self.layout.glyphs() {
                if glyph.width == 0 || glyph.height == 0 {
                    continue;
                }

                let texture_label =
                    Label::new(&format!("{}_{}", self.font_label.raw(), glyph.parent));
                let (uv_x, uv_y, uv_w, uv_h) = assets.get_texture_coords(texture_label);

                let local_x = glyph.x * self.scale.x;
                let local_y = glyph.y * self.scale.y;

                let rotated_x = local_x * cos_rot - local_y * sin_rot;
                let rotated_y = local_x * sin_rot + local_y * cos_rot;

                let instance = GlyphInstance {
                    position: [self.position.x + rotated_x, self.position.y + rotated_y],
                    size: [
                        glyph.width as f32 * self.scale.x,
                        glyph.height as f32 * self.scale.y,
                    ],
                    uv_min: [uv_x, uv_y],
                    uv_max: [uv_x + uv_w, uv_y + uv_h],
                    color: color_array,
                    rotation: self.rotation,
                };

                self.glyph_instances.push(instance);
            }

            self.dirty_visuals.set(false);
        }
    }

    #[inline]
    fn mark_layout(&self) {
        self.dirty_layout.set(true);
    }

    #[inline]
    fn mark_visuals(&self) {
        self.dirty_visuals.set(true);
    }
}
