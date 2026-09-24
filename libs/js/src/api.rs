use engine::assets::Image;
use engine::render::Layer;
use nostd::alloc::format;
use nostd::alloc::rc::Rc;
use nostd::alloc::string::String;
use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use quickjs::Context;
use quickjs::Error;
use quickjs::FromJs;
use quickjs::IntoFunction;
use quickjs::Persistent;
use quickjs::Value;
use sdl3::events::Key;
use sdl3::events::MouseButton;
use sdl3::render::Color;
use traccia::debug;
use traccia::error;
use traccia::info;
use traccia::warn;

use crate::host::Host;

struct ImageRef(Handle<Image>);

pub(crate) struct Api<'rt> {
    pub ctx: Persistent<'rt>,
    pub draw_ctx: Persistent<'rt>,
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
"#;

pub(crate) fn install<'rt>(ctx: &Context<'rt>, host: &Rc<Host>) -> Result<Api<'rt>, Error> {
    let window = window(ctx, host)?;
    let time = time(ctx, host)?;
    let input = input(ctx, host)?;

    let draw_ctx = ctx.object()?;
    draw_ctx.set("window", window.clone())?;
    draw_ctx.set("time", time.clone())?;
    draw_ctx.set("input", input.clone())?;

    let update_ctx = ctx.object()?;
    update_ctx.set("window", window)?;
    update_ctx.set("time", time)?;
    update_ctx.set("input", input)?;
    update_ctx.set("assets", assets(ctx, host)?)?;

    let karna = ctx.object()?;
    karna.set("log", log(ctx, "log", |s| info!("{s}"))?)?;
    karna.set("Key", keys(ctx)?)?;
    karna.set("Mouse", mouse_buttons(ctx)?)?;

    let console = ctx.object()?;
    console.set("log", log(ctx, "log", |s| info!("{s}"))?)?;
    console.set("info", log(ctx, "info", |s| info!("{s}"))?)?;
    console.set("debug", log(ctx, "debug", |s| debug!("{s}"))?)?;
    console.set("warn", log(ctx, "warn", |s| warn!("{s}"))?)?;
    console.set("error", log(ctx, "error", |s| error!("{s}"))?)?;

    let global = ctx.global();
    global.set("karna", karna)?;
    global.set("console", console)?;

    ctx.eval(PRELUDE, "<karna>")?;

    Ok(Api {
        ctx: ctx.persist(update_ctx),
        draw_ctx: ctx.persist(draw_ctx),
        graphics: ctx.persist(graphics(ctx, host)?),
    })
}

fn set<Args, F>(ctx: &Context<'_>, obj: &Value<'_>, name: &str, f: F) -> Result<(), Error>
where
    F: IntoFunction<Args>,
{
    obj.set(name, ctx.function(name, f)?)
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

fn window<'c>(ctx: &'c Context<'_>, host: &Rc<Host>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    let h = host.clone();
    set(ctx, &obj, "title", move || {
        h.window(|w| String::from(w.title()))
    })?;

    let h = host.clone();
    set(ctx, &obj, "setTitle", move |t: String| {
        h.window(|w| w.set_title(t))
    })?;

    let h = host.clone();
    set(ctx, &obj, "width", move || h.window(|w| w.size().w()))?;

    let h = host.clone();
    set(ctx, &obj, "height", move || h.window(|w| w.size().h()))?;

    let h = host.clone();
    set(ctx, &obj, "setSize", move |width: u32, height: u32| {
        h.window(|w| w.set_size((width, height)))
    })?;

    let h = host.clone();
    let f = ctx.function_raw("mouse", 0, move |ctx, _, _| {
        let m = h.window(|w| w.mouse_position())?;
        vec2(ctx, m.x, m.y)
    })?;

    obj.set("mouse", f)?;

    let h = host.clone();
    let f = ctx.function_raw("mouseDelta", 0, move |ctx, _, _| {
        let d = h.window(|w| w.mouse_delta())?;
        vec2(ctx, d.x, d.y)
    })?;

    obj.set("mouseDelta", f)?;

    Ok(obj)
}

fn time<'c>(ctx: &'c Context<'_>, host: &Rc<Host>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    let h = host.clone();
    set(ctx, &obj, "delta", move || h.time(|t| t.delta()))?;

    let h = host.clone();
    set(ctx, &obj, "fixedDelta", move || h.time(|t| t.fixed_delta()))?;

    let h = host.clone();
    set(ctx, &obj, "alpha", move || h.time(|t| t.alpha()))?;

    let h = host.clone();
    set(ctx, &obj, "fps", move || h.time(|t| t.fps()))?;

    let h = host.clone();
    set(ctx, &obj, "setTargetFps", move |fps: u32| {
        h.time(|t| t.set_target_fps(fps))
    })?;

    let h = host.clone();
    set(ctx, &obj, "setTargetTps", move |tps: u32| {
        h.time(|t| t.set_target_tps(tps))
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

fn input<'c>(ctx: &'c Context<'_>, host: &Rc<Host>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    let h = host.clone();
    set(ctx, &obj, "keyDown", move |k: u32| {
        let k = key(k)?;
        h.input(|i| i.key_down(k))
    })?;

    let h = host.clone();
    set(ctx, &obj, "keyPressed", move |k: u32| {
        let k = key(k)?;
        h.input(|i| i.key_pressed(k))
    })?;

    let h = host.clone();
    set(ctx, &obj, "keyReleased", move |k: u32| {
        let k = key(k)?;
        h.input(|i| i.key_released(k))
    })?;

    let h = host.clone();
    set(ctx, &obj, "mouseDown", move |b: u32| {
        let b = mouse_button(b)?;
        h.input(|i| i.mouse_down(b))
    })?;

    let h = host.clone();
    set(ctx, &obj, "mousePressed", move |b: u32| {
        let b = mouse_button(b)?;
        h.input(|i| i.mouse_pressed(b))
    })?;

    let h = host.clone();
    set(ctx, &obj, "mouseReleased", move |b: u32| {
        let b = mouse_button(b)?;
        h.input(|i| i.mouse_released(b))
    })?;

    let h = host.clone();
    set(ctx, &obj, "text", move || {
        h.input(|i| String::from(i.text()))
    })?;

    let h = host.clone();
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

fn assets<'c>(ctx: &'c Context<'_>, host: &Rc<Host>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    let h = host.clone();
    let f = ctx.function_raw("loadImage", 1, move |ctx, _, args| {
        let path = String::from_js(args.first().unwrap_or(&ctx.undefined()))?;
        let handle = h.assets_mut(|a| a.load_image(path.as_str()))?;

        ctx.instance(ImageRef(handle))
    })?;

    obj.set("loadImage", f)?;

    Ok(obj)
}

fn graphics<'c>(ctx: &'c Context<'_>, host: &Rc<Host>) -> Result<Value<'c>, Error> {
    let obj = ctx.object()?;

    let h = host.clone();
    set(
        ctx,
        &obj,
        "setColor",
        move |r: f32, g: f32, b: f32, a: Option<f32>| {
            h.draw(|d| d.set_color(Color::rgba(r, g, b, a.unwrap_or(1.0))))
        },
    )?;

    let h = host.clone();
    set(ctx, &obj, "setThickness", move |t: f32| {
        h.draw(|d| d.set_thickness(t))
    })?;

    let h = host.clone();
    set(ctx, &obj, "setLayer", move |name: String| {
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

    let h = host.clone();
    set(
        ctx,
        &obj,
        "line",
        move |x1: f32, y1: f32, x2: f32, y2: f32| h.draw(|d| d.line(x1, y1, x2, y2)),
    )?;

    let h = host.clone();
    set(ctx, &obj, "rect", move |x: f32, y: f32, w: f32, hh: f32| {
        h.draw(|d| d.rect(x, y, w, hh))
    })?;

    let h = host.clone();
    set(
        ctx,
        &obj,
        "rectOutline",
        move |x: f32, y: f32, w: f32, hh: f32| h.draw(|d| d.rect_outline(x, y, w, hh)),
    )?;

    let h = host.clone();
    set(
        ctx,
        &obj,
        "triangle",
        move |x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32| {
            h.draw(|d| d.triangle(x1, y1, x2, y2, x3, y3))
        },
    )?;

    let h = host.clone();
    set(ctx, &obj, "circle", move |x: f32, y: f32, r: f32| {
        h.draw(|d| d.circle(x, y, r))
    })?;

    let h = host.clone();
    set(ctx, &obj, "circleOutline", move |x: f32, y: f32, r: f32| {
        h.draw(|d| d.circle_outline(x, y, r))
    })?;

    let h = host.clone();
    set(ctx, &obj, "print", move |text: String, x: f32, y: f32| {
        h.draw(|d| d.print(text, x, y))
    })?;

    let h = host.clone();
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
