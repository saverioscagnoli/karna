use core::f32::consts::TAU;

use math::SdlFloat;
use math::Vector2;
use math::Vector4;
use nostd::collections::Handle;
use sdl3::render::Color;

use crate::assets::AssetServer;
use crate::assets::Image;
use crate::render::ImmediateVertex;
use crate::render::Layer;
use crate::render::LayerData;
use crate::render::LayerMap;
use crate::text::Text;

pub struct Draw<'a> {
    data: &'a mut LayerMap<LayerData>,
    assets: &'a AssetServer,
    layer: Layer,
    color: Color,
    white: Image,
    thickness: f32,
}

impl<'a> Draw<'a> {
    pub(crate) fn new(data: &'a mut LayerMap<LayerData>, assets: &'a AssetServer) -> Self {
        let images = assets.images();
        let white = images
            .resolve(images.white_texel)
            .expect("white texel must be baked before drawing");

        Self {
            data,
            assets,
            layer: Layer::WORLD,
            color: Color::WHITE,
            white,
            thickness: 1.0,
        }
    }

    #[inline]
    pub fn layer(&self) -> Layer {
        self.layer
    }

    #[inline]
    pub fn with_layer(&mut self, layer: Layer) -> &mut Self {
        if !self.data.contains(layer) {
            self.data.insert(layer, LayerData::default());
        }

        self.layer = layer;
        self
    }

    #[inline]
    pub fn color(&self) -> Color {
        self.color
    }

    #[inline]
    pub fn set_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.color = color.into();
    }

    #[inline]
    pub fn with_color<C>(&mut self, color: C) -> &mut Self
    where
        C: Into<Color>,
    {
        self.color = color.into();
        self
    }

    #[inline]
    pub fn thickness(&self) -> f32 {
        self.thickness
    }

    #[inline]
    pub fn set_thickness(&mut self, t: f32) {
        self.thickness = t;
    }

    #[inline]
    pub fn with_thickness(&mut self, t: f32) -> &mut Self {
        self.thickness = t;
        self
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.solid_quad(corners(x, y, w, h));
    }

    pub fn rect_outline(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let t = self.thickness.sdl_min(w * 0.5).sdl_min(h * 0.5);

        self.rect(x, y, w, t);
        self.rect(x, y + h - t, w, t);
        self.rect(x, y + t, t, h - 2.0 * t);
        self.rect(x + w - t, y + t, t, h - 2.0 * t);
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let a = math::vec2!(x1, y1);
        let b = math::vec2!(x2, y2);
        let n = (b - a).normalize().perp() * (self.thickness * 0.5);

        self.solid_quad([a + n, b + n, b - n, a - n]);
    }

    pub fn triangle(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.polygon(&[
            math::vec2!(x1, y1),
            math::vec2!(x2, y2),
            math::vec2!(x3, y3),
        ]);
    }

    pub fn polygon(&mut self, points: &[Vector2<f32>]) {
        if points.len() < 3 {
            return;
        }

        let (uv, page) = self.solid();
        let color = self.color_vec();
        let d = &mut self.data[self.layer];
        let base = d.vertices.len() as u32;

        d.vertices
            .extend(points.iter().map(|p| vertex(*p, color, uv, page)));

        for i in 1..points.len() as u32 - 1 {
            d.indices.extend_from_slice(&[base, base + i, base + i + 1]);
        }
    }

    pub fn circle(&mut self, x: f32, y: f32, radius: f32) {
        let n = segments(radius);
        let (uv, page) = self.solid();
        let color = self.color_vec();
        let d = &mut self.data[self.layer];
        let center = d.vertices.len() as u32;

        d.vertices.push(vertex(math::vec2!(x, y), color, uv, page));

        for i in 0..n {
            let p = Vector2::from_angle(i as f32 * TAU / n as f32) * radius;
            d.vertices
                .push(vertex(math::vec2!(x + p.x, y + p.y), color, uv, page));
        }

        for i in 0..n {
            let next = (i + 1) % n;
            d.indices
                .extend_from_slice(&[center, center + 1 + i, center + 1 + next]);
        }
    }

    pub fn circle_outline(&mut self, x: f32, y: f32, radius: f32) {
        let n = segments(radius);
        let inner = (radius - self.thickness).sdl_max(0.0);
        let (uv, page) = self.solid();
        let color = self.color_vec();
        let d = &mut self.data[self.layer];
        let base = d.vertices.len() as u32;

        // Interleaved ring: outer, inner, outer, inner, ...
        for i in 0..n {
            let dir = Vector2::from_angle(i as f32 * TAU / n as f32);
            let o = dir * radius;
            let r = dir * inner;
            d.vertices
                .push(vertex(math::vec2!(x + o.x, y + o.y), color, uv, page));
            d.vertices
                .push(vertex(math::vec2!(x + r.x, y + r.y), color, uv, page));
        }

        for i in 0..n {
            let o0 = base + 2 * i;
            let i0 = o0 + 1;
            let o1 = base + 2 * ((i + 1) % n);
            let i1 = o1 + 1;
            d.indices.extend_from_slice(&[o0, o1, i1, o0, i1, i0]);
        }
    }

    pub fn image(&mut self, image: Handle<Image>, x: f32, y: f32) {
        let Some(img) = self.lookup(image) else {
            return;
        };

        self.image_region(&img, x, y, img.width() as f32, img.height() as f32);
    }

    pub fn image_sized(&mut self, image: Handle<Image>, x: f32, y: f32, w: f32, h: f32) {
        let Some(img) = self.lookup(image) else {
            return;
        };

        self.image_region(&img, x, y, w, h);
    }

    pub fn text(&mut self, text: &Text, x: f32, y: f32) {
        let (x, y) = (x.sdl_round(), y.sdl_round());
        let layout = text.layout(self.assets);

        for glyph in &layout.glyphs {
            let Some(image) = glyph.image else {
                continue;
            };

            let color = if glyph.colored {
                Color::rgba(1.0, 1.0, 1.0, self.color.a())
            } else {
                glyph.color.unwrap_or(self.color)
            };

            let size = glyph.size.cast::<f32>();
            let (min, max) = (image.uv_min(), image.uv_max());
            let uvs = [
                min,
                math::vec2!(max.x, min.y),
                max,
                math::vec2!(min.x, max.y),
            ];

            self.quad(
                corners(x + glyph.pos.x, y + glyph.pos.y, size.w(), size.h()),
                uvs,
                image.page(),
                color_vec(color),
            );
        }
    }

    // Internals

    fn image_region(&mut self, img: &Image, x: f32, y: f32, w: f32, h: f32) {
        let (min, max) = (img.uv_min(), img.uv_max());
        let uvs = [
            min,
            math::vec2!(max.x, min.y),
            max,
            math::vec2!(min.x, max.y),
        ];

        self.quad(corners(x, y, w, h), uvs, img.page(), self.color_vec());
    }

    #[inline]
    fn lookup(&self, image: Handle<Image>) -> Option<Image> {
        let images = self.assets.images();

        images
            .resolve(image)
            .or_else(|| images.resolve(images.placeholder))
    }

    #[inline]
    fn solid(&self) -> (Vector2<f32>, u32) {
        let uv = (self.white.uv_min() + self.white.uv_max()) * 0.5;
        (uv, self.white.page())
    }

    #[inline]
    fn color_vec(&self) -> Vector4<f32> {
        color_vec(self.color)
    }

    #[inline]
    fn solid_quad(&mut self, p: [Vector2<f32>; 4]) {
        let (uv, page) = self.solid();
        self.quad(p, [uv; 4], page, self.color_vec());
    }

    #[inline]
    fn quad(
        &mut self,
        p: [Vector2<f32>; 4],
        uv: [Vector2<f32>; 4],
        page: u32,
        color: Vector4<f32>,
    ) {
        let d = &mut self.data[self.layer];
        let b = d.vertices.len() as u32;

        d.vertices
            .extend((0..4).map(|i| vertex(p[i], color, uv[i], page)));
        d.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }
}

#[inline]
fn vertex(p: Vector2<f32>, color: Vector4<f32>, uv: Vector2<f32>, page: u32) -> ImmediateVertex {
    ImmediateVertex {
        position: math::vec3!(p.x, p.y, 0.0),
        color,
        uv,
        page: page as f32,
    }
}

#[inline]
fn color_vec(color: Color) -> Vector4<f32> {
    let [r, g, b, a] = color.array();
    math::vec4!(r, g, b, a)
}

#[inline]
fn corners(x: f32, y: f32, w: f32, h: f32) -> [Vector2<f32>; 4] {
    [
        math::vec2!(x, y),
        math::vec2!(x + w, y),
        math::vec2!(x + w, y + h),
        math::vec2!(x, y + h),
    ]
}

/// Segment count scaled by circumference, roughly one segment per 4px.
#[inline]
fn segments(radius: f32) -> u32 {
    ((TAU * radius.sdl_abs() / 4.0) as u32).clamp(12, 128)
}
