use core::f32::consts::TAU;

use math::SdlFloat;
use math::Vector2;
use math::Vector4;
use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use nostd::collections::HashMap;
use sdl3::render::Color;
use traccia::warn;

use crate::assets::AssetServer;
use crate::assets::Image;
use crate::assets::SpriteFrame;
use crate::render::Canvas;
use crate::render::CanvasData;
use crate::render::ImmediateVertex;
use crate::render::Layer;
use crate::render::LayerData;
use crate::render::LayerMap;
use crate::render::Sprite;
use crate::render::TextureSource;
use crate::text::Font;
use crate::text::Text;
use crate::text::TextLayout;
use crate::text::TextStyle;

#[derive(Debug, Clone, Copy)]
pub struct Affine2 {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    tx: f32,
    ty: f32,
}

impl Affine2 {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    pub fn apply(&self, p: math::Vector2<f32>) -> math::Vector2<f32> {
        math::vec2!(
            self.a * p.x + self.c * p.y + self.tx,
            self.b * p.x + self.d * p.y + self.ty
        )
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.tx += self.a * x + self.c * y;
        self.ty += self.b * x + self.d * y;
    }

    pub fn rotate(&mut self, rad: f32) {
        let (s, c) = (rad.sdl_sin(), rad.sdl_cos());
        let (a, b, cc, d) = (self.a, self.b, self.c, self.d);

        self.a = a * c + cc * s;
        self.b = b * c + d * s;
        self.c = cc * c - a * s;
        self.d = d * c - b * s;
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.a *= sx;
        self.b *= sx;
        self.c *= sy;
        self.d *= sy;
    }

    #[inline]
    pub fn is_translation(&self) -> bool {
        self.a == 1.0 && self.b == 0.0 && self.c == 0.0 && self.d == 1.0
    }
}

#[inline]
fn vertex(
    t: &Affine2,
    p: Vector2<f32>,
    color: Vector4<f32>,
    uv: Vector2<f32>,
    page: u32,
) -> ImmediateVertex {
    let p = t.apply(p);
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

#[derive(Debug, Clone, Copy)]
struct DrawState {
    transform: Affine2,
    layer: Layer,
    target: Option<Handle<Canvas>>,
    color: Color,
    thickness: f32,
    text_style: TextStyle,
}

pub struct Draw<'a> {
    data: &'a mut LayerMap<LayerData>,
    canvases: &'a mut HashMap<Handle<Canvas>, CanvasData>,
    assets: &'a AssetServer,
    white: Image,
    state: DrawState,
    stack: Vec<DrawState>,
}

impl<'a> Draw<'a> {
    pub(crate) fn new(
        data: &'a mut LayerMap<LayerData>,
        canvases: &'a mut HashMap<Handle<Canvas>, CanvasData>,
        assets: &'a AssetServer,
    ) -> Self {
        let images = assets.images();
        let white = images
            .resolve(images.white_texel)
            .expect("white texel must be baked before drawing");

        Self {
            data,
            canvases,
            assets,
            white,
            state: DrawState {
                transform: Affine2::IDENTITY,
                layer: Layer::WORLD,
                target: None,
                color: Color::WHITE,
                thickness: 1.0,
                text_style: TextStyle::default(),
            },
            stack: Vec::new(),
        }
    }

    #[inline]
    pub fn push(&mut self) {
        self.stack.push(self.state);
    }

    #[inline]
    pub fn pop(&mut self) {
        match self.stack.pop() {
            Some(s) => self.state = s,
            None => warn!("Draw::pop with empty stack"),
        }
    }

    #[inline]
    pub fn translate(&mut self, x: f32, y: f32) {
        self.state.transform.translate(x, y);
    }

    #[inline]
    pub fn rotate(&mut self, rad: f32) {
        self.state.transform.rotate(rad);
    }

    #[inline]
    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.state.transform.scale(sx, sy);
    }

    #[inline]
    pub fn reset_transform(&mut self) {
        self.state.transform = Affine2::IDENTITY;
    }

    #[inline]
    pub fn layer(&self) -> Layer {
        self.state.layer
    }

    #[inline]
    pub fn set_layer(&mut self, layer: Layer) {
        if !self.data.contains(layer) {
            self.data.insert(layer, LayerData::default());
        }

        self.state.layer = layer;
    }

    #[inline]
    pub fn color(&self) -> Color {
        self.state.color
    }

    #[inline]
    pub fn set_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.state.color = color.into();
    }

    #[inline]
    pub fn thickness(&self) -> f32 {
        self.state.thickness
    }

    #[inline]
    pub fn set_thickness(&mut self, t: f32) {
        self.state.thickness = t;
    }

    #[inline]
    pub fn text_style(&self) -> TextStyle {
        self.state.text_style
    }

    #[inline]
    pub fn set_text_style(&mut self, style: TextStyle) {
        self.state.text_style = style;
    }

    #[inline]
    pub fn set_text_style_font(&mut self, font: Handle<Font>) {
        self.state.text_style.font = Some(font);
    }

    #[inline]
    pub fn set_text_style_size(&mut self, size: f32) {
        self.state.text_style.size = Some(size);
    }

    #[inline]
    pub fn set_text_style_line_heigth(&mut self, h: f32) {
        self.state.text_style.line_height = h;
    }

    #[inline]
    pub fn set_text_style_bold(&mut self, b: bool) {
        self.state.text_style.bold = b;
    }

    #[inline]
    pub fn set_text_style_italic(&mut self, i: bool) {
        self.state.text_style.italic = i;
    }

    pub fn scoped<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.push();
        let r = f(self);
        self.pop();
        r
    }

    #[inline]
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.line_v([x1, y1], [x2, y2]);
    }

    pub fn line_v<P, Q>(&mut self, p1: P, p2: Q)
    where
        P: Into<math::Vector2<f32>>,
        Q: Into<math::Vector2<f32>>,
    {
        let p1 = p1.into();
        let p2 = p2.into();
        let n = (p2 - p1).normalize().perp() * (self.state.thickness * 0.5);

        self.solid_quad([p1 + n, p2 + n, p2 - n, p1 - n]);
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.solid_quad(corners(x, y, w, h));
    }

    pub fn rect_v<P, S>(&mut self, pos: P, size: S)
    where
        P: Into<math::Vector2<f32>>,
        S: Into<math::Size<f32>>,
    {
        let pos = pos.into();
        let size = size.into();

        self.rect(pos.x, pos.y, size.w(), size.h());
    }

    pub fn rect_outline(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let t = self.state.thickness.sdl_min(w * 0.5).sdl_min(h * 0.5);

        self.rect(x, y, w, t);
        self.rect(x, y + h - t, w, t);
        self.rect(x, y + t, t, h - 2.0 * t);
        self.rect(x + w - t, y + t, t, h - 2.0 * t);
    }

    pub fn rect_outline_v<P, S>(&mut self, pos: P, size: S)
    where
        P: Into<math::Vector2<f32>>,
        S: Into<math::Size<f32>>,
    {
        let pos = pos.into();
        let size = size.into();

        self.rect_outline(pos.x, pos.y, size.w(), size.h());
    }

    pub fn triangle(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.triangle_v([x1, y1], [x2, y2], [x3, y3]);
    }

    pub fn triangle_v<P, Q, S>(&mut self, p1: P, p2: Q, p3: S)
    where
        P: Into<math::Vector2<f32>>,
        Q: Into<math::Vector2<f32>>,
        S: Into<math::Vector2<f32>>,
    {
        let p1 = p1.into();
        let p2 = p2.into();
        let p3 = p3.into();

        self.polygon([p1, p2, p3]);
    }

    pub fn polygon<V, I>(&mut self, points: I)
    where
        V: Into<math::Vector2<f32>>,
        I: IntoIterator<Item = V>,
    {
        let points = points.into_iter().map(|p| p.into()).collect::<Vec<_>>();

        if points.len() < 3 {
            return;
        }

        let (uv, page) = self.solid();
        let color = self.color_vec();
        let t = self.state.transform;
        let d = self.target();
        let start = d.indices.len();
        let base = d.vertices.len() as u32;

        d.vertices
            .extend(points.iter().map(|p| vertex(&t, *p, color, uv, page)));

        for i in 1..points.len() as u32 - 1 {
            d.indices.extend_from_slice(&[base, base + i, base + i + 1]);
        }

        d.record(TextureSource::Atlas, start);
    }

    pub fn circle(&mut self, x: f32, y: f32, radius: f32) {
        let n = segments(radius);
        let (uv, page) = self.solid();
        let color = self.color_vec();
        let t = self.state.transform;
        let d = self.target();
        let start = d.indices.len();
        let center = d.vertices.len() as u32;

        d.vertices
            .push(vertex(&t, math::vec2!(x, y), color, uv, page));

        for i in 0..n {
            let p = Vector2::from_angle(i as f32 * TAU / n as f32) * radius;
            d.vertices
                .push(vertex(&t, math::vec2!(x + p.x, y + p.y), color, uv, page));
        }

        for i in 0..n {
            let next = (i + 1) % n;
            d.indices
                .extend_from_slice(&[center, center + 1 + i, center + 1 + next]);
        }

        d.record(TextureSource::Atlas, start);
    }

    pub fn circle_v<P>(&mut self, pos: P, radius: f32)
    where
        P: Into<math::Vector2<f32>>,
    {
        let pos = pos.into();

        self.circle(pos.x, pos.y, radius);
    }

    pub fn circle_outline(&mut self, x: f32, y: f32, radius: f32) {
        let n = segments(radius);
        let inner = (radius - self.state.thickness).sdl_max(0.0);
        let (uv, page) = self.solid();
        let color = self.color_vec();
        let t = self.state.transform;
        let d = self.target();
        let start = d.indices.len();
        let base = d.vertices.len() as u32;

        // Interleaved ring: outer, inner, outer, inner, ...
        for i in 0..n {
            let dir = Vector2::from_angle(i as f32 * TAU / n as f32);
            let o = dir * radius;
            let r = dir * inner;
            d.vertices
                .push(vertex(&t, math::vec2!(x + o.x, y + o.y), color, uv, page));
            d.vertices
                .push(vertex(&t, math::vec2!(x + r.x, y + r.y), color, uv, page));
        }

        for i in 0..n {
            let o0 = base + 2 * i;
            let i0 = o0 + 1;
            let o1 = base + 2 * ((i + 1) % n);
            let i1 = o1 + 1;
            d.indices.extend_from_slice(&[o0, o1, i1, o0, i1, i0]);
        }

        d.record(TextureSource::Atlas, start);
    }

    pub fn circle_outline_v<P>(&mut self, pos: P, radius: f32)
    where
        P: Into<math::Vector2<f32>>,
    {
        let pos = pos.into();

        self.circle_outline(pos.x, pos.y, radius);
    }

    pub fn image(&mut self, image: Handle<Image>, x: f32, y: f32) {
        let Some(img) = self.lookup(image) else {
            return;
        };

        self.image_uv(
            &img,
            img.uv_min(),
            img.uv_max(),
            x,
            y,
            img.width() as f32,
            img.height() as f32,
        );
    }

    pub fn image_v<P>(&mut self, image: Handle<Image>, pos: P)
    where
        P: Into<math::Vector2<f32>>,
    {
        let pos = pos.into();

        self.image(image, pos.x, pos.y);
    }

    pub fn image_sized(&mut self, image: Handle<Image>, x: f32, y: f32, w: f32, h: f32) {
        let Some(img) = self.lookup(image) else {
            return;
        };

        self.image_uv(&img, img.uv_min(), img.uv_max(), x, y, w, h);
    }

    pub fn image_sized_v<P, S>(&mut self, image: Handle<Image>, pos: P, size: S)
    where
        P: Into<math::Vector2<f32>>,
        S: Into<math::Size<f32>>,
    {
        let pos = pos.into();
        let size = size.into();

        self.image_sized(image, pos.x, pos.y, size.w(), size.h());
    }

    #[allow(clippy::too_many_arguments)]
    pub fn image_region(
        &mut self,
        image: Handle<Image>,
        sx: f32,
        sy: f32,
        sw: f32,
        sh: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let Some(img) = self.lookup(image) else {
            return;
        };

        self.region(&img, [sx, sy, sw, sh], x, y, w, h);
    }

    pub fn image_region_v<P, Q, R, S>(
        &mut self,
        image: Handle<Image>,
        src_pos: P,
        src_size: Q,
        pos: R,
        size: S,
    ) where
        P: Into<math::Vector2<f32>>,
        Q: Into<math::Size<f32>>,
        R: Into<math::Vector2<f32>>,
        S: Into<math::Size<f32>>,
    {
        let (src_pos, src_size) = (src_pos.into(), src_size.into());
        let (pos, size) = (pos.into(), size.into());

        self.image_region(
            image,
            src_pos.x,
            src_pos.y,
            src_size.w(),
            src_size.h(),
            pos.x,
            pos.y,
            size.w(),
            size.h(),
        );
    }

    pub fn sprite(&mut self, sprite: &Sprite, x: f32, y: f32) {
        let Some((img, frame, _)) = self.sprite_frame(sprite) else {
            return;
        };

        self.sprite_region(&img, frame, x, y, 1.0, 1.0);
    }

    pub fn sprite_v<P>(&mut self, sprite: &Sprite, pos: P)
    where
        P: Into<math::Vector2<f32>>,
    {
        let pos = pos.into();

        self.sprite(sprite, pos.x, pos.y);
    }

    pub fn sprite_sized(&mut self, sprite: &Sprite, x: f32, y: f32, w: f32, h: f32) {
        let Some((img, frame, [fw, fh])) = self.sprite_frame(sprite) else {
            return;
        };

        if fw == 0 || fh == 0 {
            return;
        }

        self.sprite_region(&img, frame, x, y, w / fw as f32, h / fh as f32);
    }

    pub fn sprite_sized_v<P, S>(&mut self, sprite: &Sprite, pos: P, size: S)
    where
        P: Into<math::Vector2<f32>>,
        S: Into<math::Size<f32>>,
    {
        let pos = pos.into();
        let size = size.into();

        self.sprite_sized(sprite, pos.x, pos.y, size.w(), size.h());
    }

    pub fn text(&mut self, text: &Text, x: f32, y: f32) {
        self.glyphs(text.layout(self.assets), x, y);
    }

    pub fn text_v<P>(&mut self, text: &Text, pos: P)
    where
        P: Into<math::Vector2<f32>>,
    {
        let pos = pos.into();

        self.text(text, pos.x, pos.y);
    }

    pub fn print<S>(&mut self, text: S, x: f32, y: f32)
    where
        S: AsRef<str>,
    {
        let layout = self
            .assets
            .layout_str(text.as_ref(), &self.state.text_style);
        self.glyphs(&layout, x, y);
    }

    pub fn print_v<S, P>(&mut self, text: S, pos: P)
    where
        S: AsRef<str>,
        P: Into<math::Vector2<f32>>,
    {
        let pos = pos.into();

        self.print(text, pos.x, pos.y);
    }

    pub fn on_canvas<F>(&mut self, canvas: Handle<Canvas>, clear: Color, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.push();
        self.state.target = Some(canvas);
        self.state.transform = Affine2::IDENTITY;
        self.canvases.entry(canvas).or_default().clear = Some(clear);
        f(self);
        self.pop();
    }

    pub fn canvas(&mut self, canvas: Handle<Canvas>, x: f32, y: f32) {
        let Some(c) = self.assets.try_canvas(canvas) else {
            return;
        };

        let size = c.size().cast::<f32>();

        self.canvas_sized(canvas, x, y, size.w(), size.h());
    }

    pub fn canvas_v<P>(&mut self, canvas: Handle<Canvas>, pos: P)
    where
        P: Into<math::Vector2<f32>>,
    {
        let pos = pos.into();

        self.canvas(canvas, pos.x, pos.y);
    }

    pub fn canvas_sized(&mut self, canvas: Handle<Canvas>, x: f32, y: f32, w: f32, h: f32) {
        if self.state.target == Some(canvas) {
            warn!("Cannot draw a canvas onto itself");
            return;
        }

        if self.assets.try_canvas(canvas).is_none() {
            return;
        }

        let uvs = [
            math::vec2!(0.0, 0.0),
            math::vec2!(1.0, 0.0),
            math::vec2!(1.0, 1.0),
            math::vec2!(0.0, 1.0),
        ];

        self.quad(
            corners(x, y, w, h),
            uvs,
            0,
            self.color_vec(),
            TextureSource::Canvas(canvas),
        );
    }

    pub fn canvas_sized_v<P, S>(&mut self, canvas: Handle<Canvas>, pos: P, size: S)
    where
        P: Into<math::Vector2<f32>>,
        S: Into<math::Vector2<f32>>,
    {
        let (pos, size) = (pos.into(), size.into());

        self.canvas_sized(canvas, pos.x, pos.y, size.x, size.y);
    }

    // Internals

    fn glyphs(&mut self, layout: &TextLayout, x: f32, y: f32) {
        let (x, y) = (x.sdl_round(), y.sdl_round());

        for glyph in &layout.glyphs {
            let Some(image) = glyph.image else {
                continue;
            };

            let color = if glyph.colored {
                Color::rgba(1.0, 1.0, 1.0, self.state.color.a())
            } else {
                glyph.color.unwrap_or(self.state.color)
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
                TextureSource::Atlas,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn image_uv(
        &mut self,
        img: &Image,
        min: Vector2<f32>,
        max: Vector2<f32>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let uvs = [
            min,
            math::vec2!(max.x, min.y),
            max,
            math::vec2!(min.x, max.y),
        ];

        self.quad(
            corners(x, y, w, h),
            uvs,
            img.page(),
            self.color_vec(),
            TextureSource::Atlas,
        );
    }

    fn sprite_frame(&self, sprite: &Sprite) -> Option<(Image, SpriteFrame, [u32; 2])> {
        let sheet = self.assets.try_spritesheet(sprite.spritesheet())?;
        let anim = sprite.animation(sheet)?;
        let index = sprite.frame(sheet)?;
        let img = self.lookup(anim.image(sheet))?;
        let frame = anim.frame(sheet, index, img.width(), img.height())?;
        let size = anim.size(sheet, img.width(), img.height());

        Some((img, frame, size))
    }

    fn sprite_region(&mut self, img: &Image, frame: SpriteFrame, x: f32, y: f32, sx: f32, sy: f32) {
        let [rx, ry, rw, rh] = frame.rect.map(|v| v as f32);
        let [ox, oy] = frame.offset.map(|v| v as f32);

        self.region(
            img,
            [rx, ry, rw, rh],
            x + ox * sx,
            y + oy * sy,
            rw * sx,
            rh * sy,
        );
    }

    fn region(&mut self, img: &Image, src: [f32; 4], x: f32, y: f32, w: f32, h: f32) {
        let [sx, sy, sw, sh] = src;
        let (min, max) = (img.uv_min(), img.uv_max());
        let (iw, ih) = (img.width() as f32, img.height() as f32);
        let uv = |px: f32, py: f32| {
            math::vec2!(
                min.x + (max.x - min.x) * (px / iw).clamp(0.0, 1.0),
                min.y + (max.y - min.y) * (py / ih).clamp(0.0, 1.0)
            )
        };

        self.image_uv(img, uv(sx, sy), uv(sx + sw, sy + sh), x, y, w, h);
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
        color_vec(self.state.color)
    }

    #[inline]
    fn solid_quad(&mut self, p: [Vector2<f32>; 4]) {
        let (uv, page) = self.solid();
        self.quad(p, [uv; 4], page, self.color_vec(), TextureSource::Atlas);
    }

    #[inline]
    fn quad(
        &mut self,
        p: [Vector2<f32>; 4],
        uv: [Vector2<f32>; 4],
        page: u32,
        color: Vector4<f32>,
        texture: TextureSource,
    ) {
        let t = self.state.transform;
        let d = self.target();
        let start = d.indices.len();
        let b = d.vertices.len() as u32;

        d.vertices
            .extend((0..4).map(|i| vertex(&t, p[i], color, uv[i], page)));
        d.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
        d.record(texture, start);
    }

    #[inline]
    fn target(&mut self) -> &mut LayerData {
        match self.state.target {
            Some(canvas) => &mut self.canvases.entry(canvas).or_default().data,
            None => &mut self.data[self.state.layer],
        }
    }
}
