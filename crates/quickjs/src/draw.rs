//! The `draw` object, and the laid-out text it can hand back.
//!
//! Like everything in [`crate::context`] this is a [`Slot`] wrapper, but with
//! one extra wrinkle: `Draw<'w>` carries a lifetime, so the slot stores it as
//! `Draw<'static>` and [`crate::scene`] is responsible for only ever lending it
//! inside the borrow it came from.

use std::sync::Arc;

use engine::Color;
use engine::Draw;
use engine::Font;
use engine::Text;
use engine::TextAlign;
use engine::TextSpan;
use engine::TextStyle;
use rquickjs::Ctx;
use rquickjs::Exception;
use rquickjs::JsLifetime;
use rquickjs::Object;
use rquickjs::Result;
use rquickjs::Value;
use rquickjs::class::Trace;
use rquickjs::function::Opt;
use utils::Handle;

use crate::enums::JsFont;
use crate::enums::JsImage;
use crate::enums::JsLayer;
use crate::slot::Slot;
use crate::value::JsColor;
use crate::value::JsSize;
use crate::value::color_from;

/// Text that has already been shaped and laid out.
///
/// Laying out is the expensive half of drawing text. Text that does not change
/// every frame is worth laying out once — in `load`, say — and drawing with
/// [`JsDraw::text`] thereafter.
#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class(rename = "Text")]
pub struct JsText {
    #[qjs(skip_trace)]
    pub inner: Arc<Text>,
}

// `clone`, `eq` and `toString` are the names JavaScript expects, not
// attempts at the Rust traits clippy has in mind.
#[allow(clippy::should_implement_trait, clippy::inherent_to_string)]
#[rquickjs::methods]
impl JsText {
    pub fn size(&self) -> JsSize {
        self.inner.size().into()
    }

    pub fn width(&self) -> f32 {
        self.inner.size().width
    }

    pub fn height(&self) -> f32 {
        self.inner.size().height
    }

    pub fn content(&self) -> String {
        self.inner.as_str().to_string()
    }

    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        format!("Text({:?})", self.inner.as_str())
    }
}

#[derive(Trace, JsLifetime)]
#[rquickjs::class(rename = "Draw")]
pub struct JsDraw {
    #[qjs(skip_trace)]
    pub slot: Slot<Draw<'static>>,
}

#[rquickjs::methods]
impl JsDraw {
    /// The color subsequent calls paint with.
    pub fn color(&self, ctx: Ctx<'_>) -> Result<JsColor> {
        Ok(self.slot.borrow(&ctx)?.color().into())
    }

    /// Accepts a `Color`, a `"#rrggbb"` string or a `0xrrggbb` number.
    #[qjs(rename = "setColor")]
    pub fn set_color<'js>(&self, ctx: Ctx<'js>, color: Value<'js>) -> Result<()> {
        let color = color_from(&ctx, color)?;

        self.slot.borrow_mut(&ctx)?.set_color(color);
        Ok(())
    }

    /// Which layer subsequent calls land in. Layers are drawn in the order
    /// `WORLD`, `UI`, `DEBUG`, each with its own camera.
    pub fn layer(&self, ctx: Ctx<'_>) -> Result<JsLayer> {
        Ok(self.slot.borrow(&ctx)?.layer().into())
    }

    #[qjs(rename = "setLayer")]
    pub fn set_layer(&self, ctx: Ctx<'_>, layer: JsLayer) -> Result<()> {
        self.slot.borrow_mut(&ctx)?.set_layer(layer.inner);
        Ok(())
    }

    /// The size of the drawable area, in pixels.
    pub fn viewport(&self, ctx: Ctx<'_>) -> Result<JsSize> {
        Ok(self.slot.borrow(&ctx)?.viewport().into())
    }

    pub fn rect(&self, ctx: Ctx<'_>, x: f32, y: f32, width: f32, height: f32) -> Result<()> {
        self.slot.borrow_mut(&ctx)?.rect(x, y, width, height);
        Ok(())
    }

    /// Draws `image` at its natural size, tinted by the current color.
    pub fn image(&self, ctx: Ctx<'_>, image: JsImage, x: f32, y: f32) -> Result<()> {
        self.slot.borrow_mut(&ctx)?.image(image.inner, x, y);
        Ok(())
    }

    #[qjs(rename = "imageSized")]
    pub fn image_sized(
        &self,
        ctx: Ctx<'_>,
        image: JsImage,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<()> {
        self.slot
            .borrow_mut(&ctx)?
            .image_sized(image.inner, x, y, width, height);

        Ok(())
    }

    #[qjs(rename = "imageSize")]
    pub fn image_size(&self, ctx: Ctx<'_>, image: JsImage) -> Result<JsSize> {
        Ok(self
            .slot
            .borrow(&ctx)?
            .assets()
            .get_image(image.inner)
            .size
            .into())
    }

    /// Lays `content` out without drawing it. See [`JsText`].
    ///
    /// `content` is either a string or an array of spans — see
    /// [`print`](Self::print).
    pub fn layout<'js>(
        &self,
        ctx: Ctx<'js>,
        content: Value<'js>,
        style: Opt<Object<'js>>,
    ) -> Result<JsText> {
        let content = Content::from_js(&ctx, content)?;
        let style = style_from(&ctx, style.0)?;
        let draw = self.slot.borrow_mut(&ctx)?;

        let inner = match &content {
            Content::Plain(text) => draw.layout(text, &style),
            Content::Rich(spans) => draw.layout_rich(&borrow_spans(spans), &style),
        };

        Ok(JsText { inner })
    }

    /// Lays out and draws `content`, returning the size it occupied.
    ///
    /// `content` is either a string, or an array of spans for mixed styling:
    ///
    /// ```js
    /// draw.print([
    ///     { text: "hp ", color: "#a6adc8" },
    ///     { text: "42", color: Color.RED, bold: true },
    /// ], { font, size: 24 }, 8, 8);
    /// ```
    pub fn print<'js>(
        &self,
        ctx: Ctx<'js>,
        content: Value<'js>,
        style: Opt<Object<'js>>,
        x: f32,
        y: f32,
    ) -> Result<JsSize> {
        let content = Content::from_js(&ctx, content)?;
        let style = style_from(&ctx, style.0)?;
        let draw = self.slot.borrow_mut(&ctx)?;

        let size = match &content {
            Content::Plain(text) => draw.print(text, &style, x, y),
            Content::Rich(spans) => draw.print_rich(&borrow_spans(spans), &style, x, y),
        };

        Ok(size.into())
    }

    /// Draws text already laid out by [`layout`](Self::layout).
    pub fn text(&self, ctx: Ctx<'_>, text: JsText, x: f32, y: f32) -> Result<()> {
        self.slot.borrow_mut(&ctx)?.text(&text.inner, x, y);
        Ok(())
    }
}

/// A span with its text owned, so [`TextSpan`] has something to borrow from.
struct OwnedSpan {
    text: String,
    color: Option<Color>,
    font: Option<Handle<Font>>,
    bold: bool,
    italic: bool,
}

enum Content {
    Plain(String),
    Rich(Vec<OwnedSpan>),
}

impl Content {
    fn from_js<'js>(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        if let Some(s) = value.as_string() {
            return Ok(Self::Plain(s.to_string()?));
        }

        let Some(array) = value.as_array() else {
            return Err(Exception::throw_type(
                ctx,
                "expected a string or an array of text spans",
            ));
        };

        let mut spans = Vec::with_capacity(array.len());

        for span in array.iter::<Object>() {
            let span = span?;

            let color = match span.get::<_, Option<Value>>("color")? {
                Some(v) if !v.is_undefined() && !v.is_null() => Some(color_from(ctx, v)?),
                _ => None,
            };

            spans.push(OwnedSpan {
                text: span.get("text")?,
                color,
                font: span.get::<_, Option<JsFont>>("font")?.map(|f| f.inner),
                bold: span.get::<_, Option<bool>>("bold")?.unwrap_or(false),
                italic: span.get::<_, Option<bool>>("italic")?.unwrap_or(false),
            });
        }

        Ok(Self::Rich(spans))
    }
}

fn borrow_spans(spans: &[OwnedSpan]) -> Vec<TextSpan<'_>> {
    spans
        .iter()
        .map(|s| {
            let mut span = TextSpan::new(&s.text);

            if let Some(color) = s.color {
                span = span.with_color(color);
            }

            if let Some(font) = s.font {
                span = span.with_font(font);
            }

            if s.bold {
                span = span.bold();
            }

            if s.italic {
                span = span.italic();
            }

            span
        })
        .collect()
}

/// Reads a `TextStyle` out of a plain object, filling in the engine's defaults
/// for anything left out:
///
/// ```js
/// { font, size: 16, lineHeight: 20, wrap: 300, align: "center" }
/// ```
fn style_from<'js>(ctx: &Ctx<'js>, obj: Option<Object<'js>>) -> Result<TextStyle> {
    let mut style = TextStyle::default();

    let Some(obj) = obj else {
        return Ok(style);
    };

    if let Some(font) = obj.get::<_, Option<JsFont>>("font")? {
        style.font = Some(font.inner);
    }

    if let Some(size) = obj.get::<_, Option<f32>>("size")? {
        style.size = size;
        style.line_height = size * 1.25;
    }

    if let Some(line_height) = obj.get::<_, Option<f32>>("lineHeight")? {
        style.line_height = line_height;
    }

    style.wrap = obj.get::<_, Option<f32>>("wrap")?;

    if let Some(align) = obj.get::<_, Option<String>>("align")? {
        style.align = match align.as_str() {
            "left" => TextAlign::Left,
            "right" => TextAlign::Right,
            "center" => TextAlign::Center,
            "justified" => TextAlign::Justified,
            "end" => TextAlign::End,
            other => {
                return Err(Exception::throw_type(
                    ctx,
                    &format!(
                        "unknown text align {other:?}, expected \"left\", \"right\", \
                         \"center\", \"justified\" or \"end\""
                    ),
                ));
            }
        };
    }

    Ok(style)
}
