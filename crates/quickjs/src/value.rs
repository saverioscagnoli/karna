//! Plain value types that JavaScript owns outright: `Vec2`, `Size`, `Color`.
//!
//! These carry no engine borrow, so unlike everything in [`crate::context`]
//! they are ordinary `rquickjs` classes with a constructor, and scripts may
//! keep them across frames.
//!
//! JavaScript has no operator overloading, so `math::Vector`'s operators show
//! up as methods. Every one of them returns a new value; the only mutation is
//! through the `x` / `y` setters and [`JsVec2::set`].

use engine::Color;
use math as m;
use rquickjs::Ctx;
use rquickjs::Exception;
use rquickjs::JsLifetime;
use rquickjs::Result;
use rquickjs::Value;
use rquickjs::class::Trace;
use rquickjs::function::Opt;

#[derive(Clone, Copy, Trace, JsLifetime)]
#[rquickjs::class(rename = "Vec2")]
pub struct JsVec2 {
    #[qjs(get, set, enumerable)]
    pub x: f32,
    #[qjs(get, set, enumerable)]
    pub y: f32,
}

impl From<m::Vector2<f32>> for JsVec2 {
    fn from(v: m::Vector2<f32>) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<JsVec2> for m::Vector2<f32> {
    fn from(v: JsVec2) -> Self {
        Self::new(v.x, v.y)
    }
}

// `clone`, `eq` and `toString` are the names JavaScript expects, not
// attempts at the Rust traits clippy has in mind.
#[allow(clippy::should_implement_trait, clippy::inherent_to_string)]
#[rquickjs::methods]
impl JsVec2 {
    #[qjs(constructor)]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[qjs(static)]
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    #[qjs(static)]
    pub fn one() -> Self {
        Self::new(1.0, 1.0)
    }

    #[qjs(static)]
    pub fn splat(v: f32) -> Self {
        Self::new(v, v)
    }

    /// The unit vector pointing `angle` radians from `+x`.
    #[qjs(static, rename = "fromAngle")]
    pub fn from_angle(angle: f32) -> Self {
        m::Vector2::from_angle(angle).into()
    }

    pub fn set(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn clone(&self) -> Self {
        *self
    }

    pub fn add(&self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    pub fn sub(&self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    /// Componentwise product. See [`scale`](Self::scale) for the scalar one.
    pub fn mul(&self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y)
    }

    pub fn div(&self, other: Self) -> Self {
        Self::new(self.x / other.x, self.y / other.y)
    }

    pub fn scale(&self, factor: f32) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }

    pub fn neg(&self) -> Self {
        Self::new(-self.x, -self.y)
    }

    pub fn eq(&self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn length(&self) -> f32 {
        m::Vector2::from(*self).length()
    }

    #[qjs(rename = "lengthSq")]
    pub fn length_sq(&self) -> f32 {
        m::Vector2::from(*self).length_sq()
    }

    pub fn normalize(&self) -> Self {
        m::Vector2::from(*self).normalize().into()
    }

    pub fn perp(&self) -> Self {
        m::Vector2::from(*self).perp().into()
    }

    pub fn angle(&self) -> f32 {
        m::Vector2::from(*self).angle()
    }

    pub fn rotate(&self, angle: f32) -> Self {
        m::Vector2::from(*self).rotate(angle).into()
    }

    pub fn dot(&self, other: Self) -> f32 {
        m::Vector2::from(*self).dot(&other.into())
    }

    pub fn distance(&self, other: Self) -> f32 {
        m::Vector2::from(*self).distance(&other.into())
    }

    pub fn lerp(&self, other: Self, t: f32) -> Self {
        m::Vector2::from(*self).lerp(&other.into(), t).into()
    }

    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        format!("Vec2({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Copy, Trace, JsLifetime)]
#[rquickjs::class(rename = "Size")]
pub struct JsSize {
    #[qjs(get, set, enumerable)]
    pub width: f32,
    #[qjs(get, set, enumerable)]
    pub height: f32,
}

impl From<m::Size<f32>> for JsSize {
    fn from(s: m::Size<f32>) -> Self {
        Self {
            width: s.width,
            height: s.height,
        }
    }
}

impl From<m::Size<u32>> for JsSize {
    fn from(s: m::Size<u32>) -> Self {
        s.cast::<f32>().into()
    }
}

impl From<JsSize> for m::Size<f32> {
    fn from(s: JsSize) -> Self {
        Self::new(s.width, s.height)
    }
}

// `clone`, `eq` and `toString` are the names JavaScript expects, not
// attempts at the Rust traits clippy has in mind.
#[allow(clippy::should_implement_trait, clippy::inherent_to_string)]
#[rquickjs::methods]
impl JsSize {
    #[qjs(constructor)]
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    #[qjs(static)]
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    #[qjs(static)]
    pub fn square(side: f32) -> Self {
        Self::new(side, side)
    }

    pub fn clone(&self) -> Self {
        *self
    }

    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    #[qjs(rename = "aspectRatio")]
    pub fn aspect_ratio(&self) -> f32 {
        m::Size::from(*self).aspect_ratio()
    }

    pub fn scale(&self, factor: f32) -> Self {
        Self::new(self.width * factor, self.height * factor)
    }

    pub fn eq(&self, other: Self) -> bool {
        self.width == other.width && self.height == other.height
    }

    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        format!("Size({}, {})", self.width, self.height)
    }
}

#[derive(Clone, Copy, Trace, JsLifetime)]
#[rquickjs::class(rename = "Color")]
pub struct JsColor {
    #[qjs(skip_trace)]
    pub inner: Color,
}

impl From<Color> for JsColor {
    fn from(inner: Color) -> Self {
        Self { inner }
    }
}

// `clone`, `eq` and `toString` are the names JavaScript expects, not
// attempts at the Rust traits clippy has in mind.
#[allow(clippy::should_implement_trait, clippy::inherent_to_string)]
#[rquickjs::methods]
impl JsColor {
    #[qjs(constructor)]
    pub fn new(r: f32, g: f32, b: f32, a: Opt<f32>) -> Self {
        Color::rgba(r, g, b, a.0.unwrap_or(1.0)).into()
    }

    #[qjs(static)]
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color::rgb(r, g, b).into()
    }

    #[qjs(static)]
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color::rgba(r, g, b, a).into()
    }

    /// `Color.hex("#89b4fa")`, `Color.hex("#89b4faff")` or `Color.hex(0x89b4fa)`.
    #[qjs(static)]
    pub fn hex<'js>(ctx: Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Ok(color_from(&ctx, value)?.into())
    }

    #[qjs(get)]
    pub fn r(&self) -> f32 {
        self.inner.r
    }

    #[qjs(set, rename = "r")]
    pub fn set_r(&mut self, v: f32) {
        self.inner.r = v;
    }

    #[qjs(get)]
    pub fn g(&self) -> f32 {
        self.inner.g
    }

    #[qjs(set, rename = "g")]
    pub fn set_g(&mut self, v: f32) {
        self.inner.g = v;
    }

    #[qjs(get)]
    pub fn b(&self) -> f32 {
        self.inner.b
    }

    #[qjs(set, rename = "b")]
    pub fn set_b(&mut self, v: f32) {
        self.inner.b = v;
    }

    #[qjs(get)]
    pub fn a(&self) -> f32 {
        self.inner.a
    }

    #[qjs(set, rename = "a")]
    pub fn set_a(&mut self, v: f32) {
        self.inner.a = v;
    }

    pub fn clone(&self) -> Self {
        *self
    }

    #[qjs(rename = "withAlpha")]
    pub fn with_alpha(&self, a: f32) -> Self {
        Color::rgba(self.inner.r, self.inner.g, self.inner.b, a).into()
    }

    pub fn eq(&self, other: Self) -> bool {
        self.inner.array() == other.inner.array()
    }

    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        let (r, g, b, a) = self.inner.tuple();
        format!("Color({r}, {g}, {b}, {a})")
    }
}

/// Coerces whatever a script passed where a color was expected.
///
/// Accepts a `Color`, a `"#rrggbb"` / `"#rrggbbaa"` string, or a `0xrrggbb`
/// number, so `draw.color = "#89b4fa"` reads as naturally as the constructor.
pub fn color_from(ctx: &Ctx<'_>, value: Value<'_>) -> Result<Color> {
    if let Some(c) = value.as_object().and_then(|o| o.as_class::<JsColor>()) {
        return Ok(c.try_borrow()?.inner);
    }

    if let Some(s) = value.as_string() {
        let s = s.to_string()?;

        return Color::try_hex(&s)
            .ok_or_else(|| Exception::throw_type(ctx, &format!("not a hex color: {s}")));
    }

    if let Some(n) = value.as_int() {
        return Ok(Color::hex(n as u32));
    }

    Err(Exception::throw_type(
        ctx,
        "expected a Color, a \"#rrggbb\" string or a 0xrrggbb number",
    ))
}
