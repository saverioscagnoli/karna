//! Opaque tokens: keys, mouse buttons, layers, cursors and asset handles.
//!
//! Each is a class rather than a bare number so that a mistyped constant fails
//! at the call site — `input.keyDown(Key.Escpae)` throws where the typo is,
//! instead of quietly reading key 0 somewhere in the engine. The namespaces
//! ([`key_namespace`] and friends) are frozen and proxied for the same reason:
//! reading an unknown member raises instead of yielding `undefined`.

use engine::Cursor;
use engine::Font;
use engine::Image;
use engine::Key;
use engine::Layer;
use engine::MouseButton;
use engine::SystemCursor;
use math as m;
use rquickjs::Ctx;
use rquickjs::Function;
use rquickjs::JsLifetime;
use rquickjs::Object;
use rquickjs::Result;
use rquickjs::class::Trace;
use rquickjs::function::Opt;
use utils::Handle;

macro_rules! token {
    ($($js:literal => $name:ident($inner:ty), $show:expr;)*) => {$(
        #[derive(Clone, Copy, Trace, JsLifetime)]
        #[rquickjs::class(rename = $js)]
        pub struct $name {
            #[qjs(skip_trace)]
            pub inner: $inner,
        }

        impl From<$inner> for $name {
            fn from(inner: $inner) -> Self {
                Self { inner }
            }
        }

        // `eq` and `toString` are the names JavaScript expects, not attempts
        // at the Rust traits clippy has in mind.
        #[allow(clippy::should_implement_trait, clippy::inherent_to_string)]
        #[rquickjs::methods]
        impl $name {
            /// A human-readable name, for logging and debugging.
            ///
            /// Formatted on each call rather than stored, hence a method.
            pub fn name(&self) -> String {
                let show: fn(&$inner) -> String = $show;
                show(&self.inner)
            }

            pub fn eq(&self, other: Self) -> bool {
                self.inner == other.inner
            }

            #[qjs(rename = "toString")]
            pub fn to_string(&self) -> String {
                format!("{}.{}", $js, self.name())
            }
        }
    )*};
}

token! {
    "Key"     => JsKey(Key),              |k| format!("{k:?}");
    "Button"  => JsButton(MouseButton),   |b| format!("{b:?}");
    "Layer"   => JsLayer(Layer),          layer_name;
    "Image"   => JsImage(Handle<Image>),  |h: &Handle<Image>| format!("#{}", h.index());
    "Font"    => JsFont(Handle<Font>),    |h: &Handle<Font>| format!("#{}", h.index());
    "Cursor"  => JsCursor(Cursor),        cursor_name;
}

/// `Layer`'s `Debug` prints the raw FNV hash, which says nothing in a stack
/// trace.
fn layer_name(l: &Layer) -> String {
    match *l {
        Layer::WORLD => "WORLD".into(),
        Layer::UI => "UI".into(),
        Layer::DEBUG => "DEBUG".into(),
        other => format!("{other:?}"),
    }
}

fn cursor_name(c: &Cursor) -> String {
    match c {
        Cursor::System(s) => SYSTEM_CURSORS
            .iter()
            .find(|(_, v)| v == s)
            .map(|(n, _)| (*n).to_string())
            .unwrap_or_else(|| "SYSTEM".into()),
        Cursor::Custom(h, _) => format!("CUSTOM(#{})", h.index()),
    }
}

const SYSTEM_CURSORS: &[(&str, SystemCursor)] = &[
    ("DEFAULT", SystemCursor::DEFAULT),
    ("TEXT", SystemCursor::TEXT),
    ("WAIT", SystemCursor::WAIT),
    ("CROSSHAIR", SystemCursor::CROSSHAIR),
    ("PROGRESS", SystemCursor::PROGRESS),
    ("POINTER", SystemCursor::POINTER),
    ("MOVE", SystemCursor::MOVE),
    ("NOT_ALLOWED", SystemCursor::NOT_ALLOWED),
    ("N_RESIZE", SystemCursor::N_RESIZE),
    ("S_RESIZE", SystemCursor::S_RESIZE),
    ("E_RESIZE", SystemCursor::E_RESIZE),
    ("W_RESIZE", SystemCursor::W_RESIZE),
    ("NE_RESIZE", SystemCursor::NE_RESIZE),
    ("NW_RESIZE", SystemCursor::NW_RESIZE),
    ("SE_RESIZE", SystemCursor::SE_RESIZE),
    ("SW_RESIZE", SystemCursor::SW_RESIZE),
    ("NS_RESIZE", SystemCursor::NS_RESIZE),
    ("EW_RESIZE", SystemCursor::EW_RESIZE),
    ("NESW_RESIZE", SystemCursor::NESW_RESIZE),
    ("NWSE_RESIZE", SystemCursor::NWSE_RESIZE),
];

/// `Key.W`, `Key.Space`, ... — one member per key the engine models.
///
/// The names come from the `Debug` impl the engine's `keys!` macro derives, so
/// this list stays in sync with `crates/engine/src/input/mod.rs` for free.
pub fn key_namespace<'js>(ctx: &Ctx<'js>) -> Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;

    for &k in Key::ALL {
        obj.set(format!("{k:?}"), JsKey::from(k))?;
    }

    seal(ctx, obj, "Key")
}

pub fn button_namespace<'js>(ctx: &Ctx<'js>) -> Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;

    for b in [
        MouseButton::Left,
        MouseButton::Middle,
        MouseButton::Right,
        MouseButton::X1,
        MouseButton::X2,
    ] {
        obj.set(format!("{b:?}"), JsButton::from(b))?;
    }

    seal(ctx, obj, "Button")
}

/// Only the three layers the renderer sets up; a made-up layer would silently
/// never be drawn.
pub fn layer_namespace<'js>(ctx: &Ctx<'js>) -> Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;

    obj.set("WORLD", JsLayer::from(Layer::WORLD))?;
    obj.set("UI", JsLayer::from(Layer::UI))?;
    obj.set("DEBUG", JsLayer::from(Layer::DEBUG))?;

    seal(ctx, obj, "Layer")
}

/// The system cursors, plus `Cursor.custom(image, hotspotX, hotspotY)`.
pub fn cursor_namespace<'js>(ctx: &Ctx<'js>) -> Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;

    for (name, cursor) in SYSTEM_CURSORS {
        obj.set(*name, JsCursor::from(Cursor::System(*cursor)))?;
    }

    obj.set(
        "custom",
        Function::new(ctx.clone(), |image: JsImage, hx: Opt<u16>, hy: Opt<u16>| {
            let hotspot = m::Vector2::new(hx.0.unwrap_or(0), hy.0.unwrap_or(0));

            JsCursor::from(Cursor::Custom(image.inner, hotspot))
        })?,
    )?;

    seal(ctx, obj, "Cursor")
}

/// Freezes `obj` and wraps it so that unknown reads raise.
///
/// Without this, `Key.Escpae` is `undefined` and only fails later inside
/// `keyDown`, with nothing pointing at the typo.
fn seal<'js>(ctx: &Ctx<'js>, obj: Object<'js>, what: &str) -> Result<Object<'js>> {
    let wrap: Function<'js> = ctx.eval(
        r#"
        (target, what) => new Proxy(Object.freeze(target), {
            get(t, k) {
                if (k in t || typeof k === "symbol") return t[k];
                throw new ReferenceError(`unknown ${what}: ${String(k)}`);
            },
            set() {
                throw new TypeError(`${what} is read-only`);
            },
        })
        "#,
    )?;

    wrap.call((obj, what))
}
