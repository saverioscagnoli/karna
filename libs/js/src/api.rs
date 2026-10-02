use engine::assets::Audio;
use engine::assets::Image;
use engine::audio::PlayOptions;
use engine::audio::Voice;
use engine::render::Layer;
use nostd::alloc::borrow::ToOwned;
use nostd::alloc::format;
use nostd::alloc::rc::Rc;
use nostd::alloc::string::String;
use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use quickjs::Context;
use quickjs::Error;
use quickjs::FromJs;
use quickjs::IntoJs;
use quickjs::Persistent;
use quickjs::Value;
use sdl3::events::Key;
use sdl3::events::MouseButton;
use sdl3::gpu::PresentMode;
use sdl3::render::Color;
use sdl3::window::FullscreenMode;
use traccia::debug;
use traccia::error;
use traccia::info;
use traccia::trace;
use traccia::warn;

use crate::JsHost;

struct ImageRef(Handle<Image>);
struct AudioRef(Handle<Audio>);
struct VoiceRef(Handle<Voice>);

struct JsPlayOptions(PlayOptions);

impl FromJs for JsPlayOptions {
    fn from_js(value: &Value<'_>) -> Result<Self, Error> {
        if !value.is_object() {
            return Err(Error::Type(format!(
                "expected play options, got {}",
                value.type_name()
            )));
        }

        let defaults = PlayOptions::default();

        Ok(Self(PlayOptions {
            looping: Option::<bool>::from_js(&value.get("loop")?)?.unwrap_or(defaults.looping),
            gain: Option::<f32>::from_js(&value.get("gain")?)?.unwrap_or(defaults.gain),
        }))
    }
}

struct JsFullscreenMode(FullscreenMode);

impl FromJs for JsFullscreenMode {
    fn from_js(value: &Value<'_>) -> Result<Self, Error> {
        if !value.is_object() {
            return Err(Error::Type(format!(
                "expected a karna.FullscreenMode, got {}",
                value.type_name()
            )));
        }

        let kind = String::from_js(&value.get("kind")?)?;

        match kind.as_str() {
            "borderless" => Ok(Self(FullscreenMode::Borderless)),
            "exclusive" => Ok(Self(FullscreenMode::Exclusive {
                width: i32::from_js(&value.get("width")?)?,
                height: i32::from_js(&value.get("height")?)?,
                refresh_rate: Option::<f32>::from_js(&value.get("refreshRate")?)?.unwrap_or(0.0),
            })),
            _ => Err(Error::Type(format!(
                "unknown fullscreen mode '{kind}' (expected 'borderless' or 'exclusive')"
            ))),
        }
    }
}

struct JsColor(Color);

struct JsSize(math::Size<u32>);

impl IntoJs for JsSize {
    fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
        let s = ctx.object()?;
        s.set("width", ctx.f64(self.0.w() as f64))?;
        s.set("height", ctx.f64(self.0.h() as f64))?;

        Ok(s)
    }
}

impl FromJs for JsSize {
    fn from_js(value: &Value<'_>) -> Result<Self, Error> {
        let (w, h) = if value.is_array() {
            ("0", "1")
        } else if value.is_object() {
            ("width", "height")
        } else {
            return Err(Error::Type(format!(
                "expected a size, got {}",
                value.type_name()
            )));
        };

        let width = u32::from_js(&value.get(w)?)?;
        let height = u32::from_js(&value.get(h)?)?;

        Ok(Self(math::Size::from((width, height))))
    }
}

impl FromJs for JsColor {
    fn from_js(value: &Value<'_>) -> Result<Self, Error> {
        if value.is_string() {
            let hex = value.to_string()?;

            return Color::try_hex(&hex)
                .map(Self)
                .ok_or_else(|| Error::Type(format!("'{hex}' is not a hex color")));
        }

        let [r, g, b, a] = if value.is_array() {
            ["0", "1", "2", "3"]
        } else if value.is_object() {
            ["r", "g", "b", "a"]
        } else {
            return Err(Error::Type(format!(
                "expected a color, got {}",
                value.type_name()
            )));
        };

        let c = |k: &str| f32::from_js(&value.get(k)?);
        let a = Option::<f32>::from_js(&value.get(a)?)?.unwrap_or(1.0);

        Ok(Self(Color::rgba(c(r)?, c(g)?, c(b)?, a)))
    }
}

impl IntoJs for JsColor {
    fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
        let c = ctx.object()?;
        c.set("r", ctx.f64(self.0.r as f64))?;
        c.set("g", ctx.f64(self.0.g as f64))?;
        c.set("b", ctx.f64(self.0.b as f64))?;
        c.set("a", ctx.f64(self.0.a as f64))?;

        Ok(c)
    }
}

pub(crate) struct Api<'rt> {
    pub graphics: Persistent<'rt>,
}

// Plain data on purpose: `ScriptScene::configure` reads the fields back, so a
// hand-written `{ title, width, height, resizable }` works just as well.
const PRELUDE: &str = r#"
karna.WindowBuilder = class WindowBuilder {
    withTitle(title) { this.title = title; return this; }
    withSize(width, height) { this.width = width; this.height = height; return this; }
    withResizable(resizable = true) { this.resizable = resizable; return this; }
};

karna.Color = class Color {
    constructor(r, g, b, a = 1) { this.r = r; this.g = g; this.b = b; this.a = a; }
};

karna.Size = class Size {
    constructor(width, height) { this.width = width; this.height = height; }
};

karna.FullscreenMode = Object.freeze({
    Borderless: Object.freeze({ kind: "borderless" }),
    Exclusive: (width, height, refreshRate = 0) =>
        Object.freeze({ kind: "exclusive", width, height, refreshRate }),
});
"#;

pub(crate) fn install<'rt>(ctx: &Context<'rt>, host: &Rc<JsHost>) -> Result<Api<'rt>, Error> {
    let karna = ctx.object()?;

    karna.set("Key", keys(ctx)?)?;
    karna.set("Mouse", mouse_buttons(ctx)?)?;
    karna.set("PresentMode", present_modes(ctx)?)?;

    karna.set("window", window(ctx, host)?)?;
    karna.set("time", time(ctx, host)?)?;
    karna.set("input", input(ctx, host)?)?;
    karna.set("assets", assets(ctx, host)?)?;
    karna.set("audio", audio(ctx, host)?)?;

    let console = ctx.object()?;

    console.set("log", log(ctx, "log", |s| info!("{s}"))?)?;
    console.set("trace", log(ctx, "trace", |s| trace!("{s}"))?)?;
    console.set("debug", log(ctx, "debug", |s| debug!("{s}"))?)?;
    console.set("info", log(ctx, "info", |s| info!("{s}"))?)?;
    console.set("warn", log(ctx, "warn", |s| warn!("{s}"))?)?;
    console.set("error", log(ctx, "error", |s| error!("{s}"))?)?;

    let global = ctx.global();
    global.set("karna", karna)?;
    global.set("console", console)?;

    ctx.eval(PRELUDE, "<karna>")?;

    Ok(Api {
        graphics: ctx.persist(graphics(ctx, host)?),
    })
}

fn vec2<'c>(ctx: &'c Context<'_>, x: f32, y: f32) -> Result<Value<'c>, Error> {
    let v = ctx.object()?;
    v.set("x", ctx.f64(x as f64))?;
    v.set("y", ctx.f64(y as f64))?;

    Ok(v)
}

fn log<'c>(ctx: &'c Context<'_>, name: &str, sink: fn(&str)) -> Result<Value<'c>, Error> {
    ctx.function_raw(name, 0, move |ctx, _, args| {
        let parts = args
            .iter()
            .map(|a| a.to_string())
            .collect::<Result<Vec<_>, _>>()?;
        sink(&parts.join(" "));

        Ok(ctx.undefined())
    })
}

fn window<'c>(ctx: &'c Context<'_>, host: &Rc<JsHost>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    let h = Rc::clone(&host);
    obj.set_fn("title", move || h.window(|w| w.title().to_owned()))?;

    let h = Rc::clone(&host);
    obj.set_fn("size", move || h.window(|w| JsSize(w.size())))?;

    let h = Rc::clone(&host);
    obj.set_fn("pixelSize", move || h.window(|w| JsSize(w.pixel_size())))?;

    let h = Rc::clone(&host);
    obj.set_fn("aspectRatio", move || {
        let _ = h.window(|w| w.pixel_size().cast::<f32>().aspect_ratio());
    })?;

    let h = Rc::clone(&host);
    let f = ctx.function_raw("mousePosition", 0, move |ctx, _, _| {
        let m = h.window(|w| w.mouse_position())?;
        vec2(ctx, m.x, m.y)
    })?;

    obj.set("mousePosition", f)?;

    let h = Rc::clone(&host);
    let f = ctx.function_raw("mouseDelta", 0, move |ctx, _, _| {
        let d = h.window(|w| w.mouse_delta())?;
        vec2(ctx, d.x, d.y)
    })?;

    obj.set("mouseDelta", f)?;

    let h = Rc::clone(&host);
    obj.set_fn("presentMode", move || h.window(|w| w.present_mode() as u32))?;

    let h = Rc::clone(&host);
    obj.set_fn("isWindowed", move || h.window(|w| w.is_windowed()))?;

    let h = Rc::clone(&host);
    obj.set_fn("isMaximized", move || h.window(|w| w.is_maximized()))?;

    let h = Rc::clone(&host);
    obj.set_fn("isMinimized", move || h.window(|w| w.is_minimized()))?;

    let h = Rc::clone(&host);
    obj.set_fn("isFullscreen", move || h.window(|w| w.is_fullscreen()))?;

    let h = Rc::clone(&host);
    obj.set_fn("isHidden", move || h.window(|w| w.is_hidden()))?;

    let h = Rc::clone(&host);
    obj.set_fn("isResizable", move || h.window(|w| w.is_resizable()))?;

    let h = Rc::clone(&host);
    obj.set_fn("isDecorated", move || h.window(|w| w.is_decorated()))?;

    let h = Rc::clone(&host);
    obj.set_fn("isAlwaysOnTop", move || h.window(|w| w.is_always_on_top()))?;

    let h = Rc::clone(&host);
    obj.set_fn("isUtility", move || h.window(|w| w.is_utility()))?;

    let h = Rc::clone(&host);
    obj.set_fn("isTransparent", move || h.window(|w| w.is_transparent()))?;

    let h = Rc::clone(&host);
    obj.set_fn("isFocusable", move || h.window(|w| w.is_focusable()))?;

    let h = Rc::clone(&host);
    obj.set_fn("isHighPixelDensity", move || {
        h.window(|w| w.is_high_pixel_density())
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("isMouseGrabbed", move || h.window(|w| w.is_mouse_grabbed()))?;

    let h = Rc::clone(&host);
    obj.set_fn("isKeyboardGrabbed", move || {
        h.window(|w| w.is_mouse_grabbed())
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("isRelativeMouse", move || {
        h.window(|w| w.is_relatve_mouse())
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("opacity", move || h.window(|w| w.opacity()))?;

    let h = Rc::clone(&host);
    obj.set_fn("clearColor", move || h.window(|w| JsColor(w.clear_color())))?;

    // Mutable

    let h = Rc::clone(&host);
    obj.set_fn("setTitle", move |t: String| {
        h.window_mut(|w| w.set_title(t))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setSize", move |size: JsSize| {
        h.window_mut(|w| w.set_size(size.0))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setWindowed", move || h.window_mut(|w| w.set_windowed()))?;

    let h = Rc::clone(&host);
    obj.set_fn("setMaximized", move || h.window_mut(|w| w.set_maximized()))?;

    let h = Rc::clone(&host);
    obj.set_fn("setMinimized", move || h.window_mut(|w| w.set_minimized()))?;

    let h = Rc::clone(&host);
    obj.set_fn("setFullscreen", move |mode: Option<JsFullscreenMode>| {
        let mode = mode.map_or(FullscreenMode::Borderless, |m| m.0);
        h.window_mut(|w| w.set_fullscreen(mode))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setHidden", move |v: bool| {
        h.window_mut(|w| w.set_hidden(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setResizable", move |v: bool| {
        h.window_mut(|w| w.set_resizable(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setDecorated", move |v: bool| {
        h.window_mut(|w| w.set_decorated(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setAlwaysOnTop", move |v: bool| {
        h.window_mut(|w| w.set_always_on_top(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setFocusable", move |v: bool| {
        h.window_mut(|w| w.set_focusable(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setMouseGrabbed", move |v: bool| {
        h.window_mut(|w| w.set_mouse_grabbed(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setKeyboardGrabbed", move |v: bool| {
        h.window_mut(|w| w.set_mouse_grabbed(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setRelativeMouse", move |v: bool| {
        h.window_mut(|w| w.set_relative_mouse(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setOpacity", move |v: f32| {
        h.window_mut(|w| w.set_opacity(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setPresentMode", move |m: u32| {
        let m = present_mode(m)?;
        h.window_mut(|w| w.set_present_mode(m))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setClearColor", move |c: JsColor| {
        h.window_mut(|w| w.set_clear_color(c.0))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("restore", move || h.window_mut(|w| w.restore()))?;

    Ok(obj)
}

fn time<'c>(ctx: &'c Context<'_>, host: &Rc<JsHost>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    let h = Rc::clone(&host);
    obj.set_fn("delta", move || h.time(|t| t.delta()))?;

    let h = Rc::clone(&host);
    obj.set_fn("fixedDelta", move || h.time(|t| t.fixed_delta()))?;

    let h = Rc::clone(&host);
    obj.set_fn("alpha", move || h.time(|t| t.alpha()))?;

    let h = Rc::clone(&host);
    obj.set_fn("fps", move || h.time(|t| t.fps()))?;

    // Mutable

    let h = Rc::clone(&host);
    obj.set_fn("setTargetFps", move |fps: u32| {
        h.time_mut(|t| t.set_target_fps(fps))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setTargetTps", move |tps: u32| {
        h.time_mut(|t| t.set_target_tps(tps))
    })?;

    Ok(obj)
}

fn key(i: u32) -> Result<Key, Error> {
    Key::ALL
        .get(i as usize)
        .copied()
        .ok_or_else(|| Error::Type(format!("{i} is not a karna.Key")))
}

const MOUSE_BUTTONS: [(&str, MouseButton); 5] = [
    ("Left", MouseButton::Left),
    ("Middle", MouseButton::Middle),
    ("Right", MouseButton::Right),
    ("X1", MouseButton::X1),
    ("X2", MouseButton::X2),
];

fn mouse_button(i: u32) -> Result<MouseButton, Error> {
    MOUSE_BUTTONS
        .get(i as usize)
        .map(|(_, b)| *b)
        .ok_or_else(|| Error::Type(format!("{i} is not a karna.Mouse button")))
}

const PRESENT_MODES: [(&str, PresentMode); 3] = [
    ("Vsync", PresentMode::Vsync),
    ("Mailbox", PresentMode::Mailbox),
    ("Immediate", PresentMode::Immediate),
];

fn present_mode(i: u32) -> Result<PresentMode, Error> {
    PRESENT_MODES
        .get(i as usize)
        .map(|(_, m)| *m)
        .ok_or_else(|| Error::Type(format!("{i} is not a karna.PresentMode")))
}

fn input<'c>(ctx: &'c Context<'_>, host: &Rc<JsHost>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    let h = Rc::clone(&host);
    obj.set_fn("keyDown", move |k: u32| {
        let k = key(k)?;
        h.input(|i| i.key_down(k))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("keyPressed", move |k: u32| {
        let k = key(k)?;
        h.input(|i| i.key_pressed(k))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("keyReleased", move |k: u32| {
        let k = key(k)?;
        h.input(|i| i.key_released(k))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("mouseDown", move |b: u32| {
        let b = mouse_button(b)?;
        h.input(|i| i.mouse_down(b))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("mousePressed", move |b: u32| {
        let b = mouse_button(b)?;
        h.input(|i| i.mouse_pressed(b))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("mouseReleased", move |b: u32| {
        let b = mouse_button(b)?;
        h.input(|i| i.mouse_released(b))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("text", move || h.input(|i| String::from(i.text())))?;

    let h = Rc::clone(&host);
    let f = ctx.function_raw("wheel", 0, move |ctx, _, _| {
        let w = h.input(|i| i.mouse_wheel())?;
        vec2(ctx, w.x, w.y)
    })?;

    obj.set("wheel", f)?;

    Ok(obj)
}

fn keys<'c>(ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    for (i, k) in Key::ALL.iter().enumerate() {
        obj.set(&format!("{k:?}"), ctx.i32(i as i32))?;
    }

    Ok(obj)
}

fn mouse_buttons<'c>(ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    for (i, (name, _)) in MOUSE_BUTTONS.iter().enumerate() {
        obj.set(name, ctx.i32(i as i32))?;
    }

    Ok(obj)
}

fn present_modes<'c>(ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    for (i, (name, _)) in PRESENT_MODES.iter().enumerate() {
        obj.set(name, ctx.i32(i as i32))?;
    }

    Ok(obj)
}

fn assets<'c>(ctx: &'c Context<'_>, host: &Rc<JsHost>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    let h = Rc::clone(&host);
    let f = ctx.function_raw("loadImage", 1, move |ctx, _, args| {
        let path = String::from_js(args.first().unwrap_or(&ctx.undefined()))?;
        let handle = h.assets_mut(|a| a.load_image(path.as_str()))?;

        ctx.instance(ImageRef(handle))
    })?;

    obj.set("loadImage", f)?;

    let h = Rc::clone(&host);
    let f = ctx.function_raw("loadAudio", 1, move |ctx, _, args| {
        let path = String::from_js(args.first().unwrap_or(&ctx.undefined()))?;
        let handle = h.assets_mut(|a| a.load_audio(path.as_str()))?;

        ctx.instance(AudioRef(handle))
    })?;

    obj.set("loadAudio", f)?;

    Ok(obj)
}

fn audio<'c>(ctx: &'c Context<'_>, host: &Rc<JsHost>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    let h = Rc::clone(&host);
    let f = ctx.function_raw("play", 1, move |ctx, _, args| {
        let undefined = ctx.undefined();
        let arg = |i: usize| args.get(i).unwrap_or(&undefined);

        let audio = arg(0)
            .opaque::<AudioRef>()
            .ok_or_else(|| Error::Type("argument 1: expected an audio".into()))?
            .0;
        let options =
            Option::<JsPlayOptions>::from_js(arg(1))?.map_or_else(PlayOptions::default, |o| o.0);

        let voice = h.audio_mut(|a| a.play_with(audio, options))?;

        ctx.instance(VoiceRef(voice))
    })?;

    obj.set("play", f)?;

    Ok(obj)
}

fn graphics<'c>(ctx: &'c Context<'_>, host: &Rc<JsHost>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    let h = Rc::clone(&host);
    obj.set_fn("color", move || h.draw(|d| JsColor(d.color())))?;

    let h = Rc::clone(&host);
    obj.set_fn("setColor", move |color: JsColor| {
        h.draw(|d| d.set_color(color.0))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("setThickness", move |t: f32| h.draw(|d| d.set_thickness(t)))?;

    let h = Rc::clone(&host);
    obj.set_fn("setLayer", move |name: String| {
        let layer = match name.as_str() {
            "world" => Layer::WORLD,
            "ui" => Layer::UI,
            "debug" => Layer::DEBUG,
            _ => {
                return Err(Error::Type(format!(
                    "unknown layer '{name}' (expected 'world', 'ui' or 'debug')"
                )));
            }
        };

        h.draw(|d| {
            d.with_layer(layer);
        })
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("line", move |x1: f32, y1: f32, x2: f32, y2: f32| {
        h.draw(|d| d.line(x1, y1, x2, y2))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("rect", move |x: f32, y: f32, w: f32, hh: f32| {
        h.draw(|d| d.rect(x, y, w, hh))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("rectOutline", move |x: f32, y: f32, w: f32, hh: f32| {
        h.draw(|d| d.rect_outline(x, y, w, hh))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn(
        "triangle",
        move |x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32| {
            h.draw(|d| d.triangle(x1, y1, x2, y2, x3, y3))
        },
    )?;

    let h = Rc::clone(&host);
    obj.set_fn("circle", move |x: f32, y: f32, r: f32| {
        h.draw(|d| d.circle(x, y, r))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("circleOutline", move |x: f32, y: f32, r: f32| {
        h.draw(|d| d.circle_outline(x, y, r))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("print", move |text: String, x: f32, y: f32| {
        h.draw(|d| d.print(text, x, y))
    })?;

    let h = Rc::clone(&host);
    let f = ctx.function_raw("image", 3, move |ctx, _, args| {
        let undefined = ctx.undefined();
        let arg = |i: usize| args.get(i).unwrap_or(&undefined);

        let image = arg(0)
            .opaque::<ImageRef>()
            .ok_or_else(|| Error::Type("argument 1: expected an image".into()))?
            .0;
        let x = f32::from_js(arg(1))?;
        let y = f32::from_js(arg(2))?;
        let w = Option::<f32>::from_js(arg(3))?;
        let hh = Option::<f32>::from_js(arg(4))?;

        h.draw(|d| match (w, hh) {
            (Some(w), Some(hh)) => d.image_sized(image, x, y, w, hh),
            _ => d.image(image, x, y),
        })?;

        Ok(ctx.undefined())
    })?;

    obj.set("image", f)?;

    Ok(obj)
}
