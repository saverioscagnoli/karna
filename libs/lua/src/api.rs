use engine::assets::Audio;
use engine::assets::Image;
use engine::audio::PlayOptions;
use engine::audio::Voice;
use engine::render::Layer;
use lua::FromLua;
use lua::IntoLua;
use lua::Lua;
use nostd::alloc::borrow::ToOwned;
use nostd::alloc::format;
use nostd::alloc::rc::Rc;
use nostd::alloc::string::String;
use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
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

use crate::LuaHost;

struct ImageRef(Handle<Image>);
struct AudioRef(Handle<Audio>);
struct VoiceRef(Handle<Voice>);

struct LuaPlayOptions(PlayOptions);

impl FromLua for LuaPlayOptions {
    fn from_lua(value: &lua::Value<'_>) -> Result<Self, lua::Error> {
        if !value.is_table() {
            return Err(lua::Error::Type(format!(
                "expected play options, got {}",
                value.type_name()
            )));
        }

        let defaults = PlayOptions::default();

        Ok(Self(PlayOptions {
            looping: Option::from_lua(&value.get("loop")?)?.unwrap_or(defaults.looping),
            gain: Option::from_lua(&value.get("gain")?)?.unwrap_or(defaults.gain),
        }))
    }
}

struct LuaFullscreenMode(FullscreenMode);

impl FromLua for LuaFullscreenMode {
    fn from_lua(value: &lua::Value<'_>) -> Result<Self, lua::Error> {
        if !value.is_table() {
            return Err(lua::Error::Type(format!(
                "expected a karna.FullscreenMode, got {}",
                value.type_name()
            )));
        }

        let kind = String::from_lua(&value.get("kind")?)?;

        match kind.as_str() {
            "borderless" => Ok(Self(FullscreenMode::Borderless)),
            "exclusive" => Ok(Self(FullscreenMode::Exclusive {
                width: i32::from_lua(&value.get("width")?)?,
                height: i32::from_lua(&value.get("height")?)?,
                refresh_rate: Option::from_lua(&value.get("refresh_rate")?)?.unwrap_or(0.0),
            })),
            _ => Err(lua::Error::Type(format!(
                "unknown fullscreen mode '{kind}' (expected 'borderless' or 'exclusive')"
            ))),
        }
    }
}

struct LuaColor(Color);

impl FromLua for LuaColor {
    fn from_lua(value: &lua::Value<'_>) -> Result<Self, lua::Error> {
        if value.is_string() {
            let hex = value.to_string()?;

            return Color::try_hex(&hex)
                .map(Self)
                .ok_or_else(|| lua::Error::Type(format!("'{hex}' is not a valid hex color")));
        }

        if !value.is_table() {
            return Err(lua::Error::Type(format!(
                "expected a color, got {}",
                value.type_name()
            )));
        }

        let named = !value.get("r")?.is_nil();
        let field = |k: &str, i: i64| match named {
            true => value.get(k),
            false => value.geti(i),
        };

        let c = |k: &str, i: i64| f32::from_lua(&field(k, i)?);
        let a = Option::from_lua(&field("a", 4)?)?.unwrap_or(1.0);

        Ok(Self(Color::rgba(c("r", 1)?, c("g", 2)?, c("b", 3)?, a)))
    }
}

impl IntoLua for LuaColor {
    fn into_lua<'l>(self, lua: &'l Lua) -> Result<lua::Value<'l>, lua::Error> {
        let c = lua.table()?;

        c.set("r", lua.number(self.0.r as f64))?;
        c.set("g", lua.number(self.0.g as f64))?;
        c.set("b", lua.number(self.0.b as f64))?;
        c.set("a", lua.number(self.0.a as f64))?;

        Ok(c)
    }
}

struct LuaVec2(math::Vector2<f32>);

impl IntoLua for LuaVec2 {
    fn into_lua<'l>(self, lua: &'l Lua) -> Result<lua::Value<'l>, lua::Error> {
        let v = lua.table()?;
        v.set("x", lua.number(self.0.x as f64))?;
        v.set("y", lua.number(self.0.y as f64))?;

        Ok(v)
    }
}

struct LuaSize(math::Size<u32>);

impl FromLua for LuaSize {
    fn from_lua(value: &lua::Value<'_>) -> Result<Self, lua::Error> {
        if !value.is_table() {
            return Err(lua::Error::Type(format!(
                "expected size, got {}",
                value.type_name()
            )));
        }

        let (width, height) = match value.get("width")?.is_nil() {
            true => (value.geti(1)?, value.geti(2)?),
            false => (value.get("width")?, value.get("height")?),
        };

        let width = u32::from_lua(&width)?;
        let height = u32::from_lua(&height)?;

        Ok(Self(math::size!(width, height)))
    }
}

impl IntoLua for LuaSize {
    fn into_lua<'l>(self, lua: &'l Lua) -> Result<lua::Value<'l>, lua::Error> {
        let s = lua.table()?;
        s.set("width", lua.integer(self.0.w() as i64))?;
        s.set("height", lua.integer(self.0.h() as i64))?;

        Ok(s)
    }
}

pub struct Api<'rt> {
    pub graphics: lua::Persistent<'rt>,
}

const PRESENT_MODES: [(&str, PresentMode); 3] = [
    ("Vsync", PresentMode::Vsync),
    ("Mailbox", PresentMode::Mailbox),
    ("Immediate", PresentMode::Immediate),
];

fn present_mode(i: u32) -> Result<PresentMode, lua::Error> {
    PRESENT_MODES
        .get(i as usize)
        .map(|(_, m)| *m)
        .ok_or_else(|| lua::Error::Type(format!("{i} is not a karna.PresentMode")))
}

fn window<'l>(lua: &'l lua::Lua, host: &Rc<LuaHost>) -> Result<lua::Value<'l>, lua::Error> {
    let obj = lua.table()?;

    let h = Rc::clone(&host);
    obj.set_fn("title", move || h.window(|w| w.title().to_owned()))?;

    let h = Rc::clone(&host);
    obj.set_fn("size", move || h.window(|w| LuaSize(w.size())))?;

    let h = Rc::clone(&host);
    obj.set_fn("pixel_size", move || h.window(|w| LuaSize(w.pixel_size())))?;

    let h = Rc::clone(&host);
    obj.set_fn("aspect_ratio", move || {
        h.window(|w| w.pixel_size().cast::<f32>().aspect_ratio())
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("mouse_position", move || {
        h.window(|w| LuaVec2(w.mouse_position()))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("mouse_delta", move || {
        h.window(|w| LuaVec2(w.mouse_delta()))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("present_mode", move || {
        h.window(|w| w.present_mode() as u32)
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("is_windowed", move || h.window(|w| w.is_windowed()))?;

    let h = Rc::clone(&host);
    obj.set_fn("is_maximized", move || h.window(|w| w.is_maximized()))?;

    let h = Rc::clone(&host);
    obj.set_fn("is_minimized", move || h.window(|w| w.is_minimized()))?;

    let h = Rc::clone(&host);
    obj.set_fn("is_fullscreen", move || h.window(|w| w.is_fullscreen()))?;

    let h = Rc::clone(&host);
    obj.set_fn("is_hidden", move || h.window(|w| w.is_hidden()))?;

    let h = Rc::clone(&host);
    obj.set_fn("is_resizable", move || h.window(|w| w.is_resizable()))?;

    let h = Rc::clone(&host);
    obj.set_fn("is_decorated", move || h.window(|w| w.is_decorated()))?;

    let h = Rc::clone(&host);
    obj.set_fn("is_always_on_top", move || {
        h.window(|w| w.is_always_on_top())
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("is_utility", move || h.window(|w| w.is_utility()))?;

    let h = Rc::clone(&host);
    obj.set_fn("is_transparent", move || h.window(|w| w.is_transparent()))?;

    let h = Rc::clone(&host);
    obj.set_fn("is_focusable", move || h.window(|w| w.is_focusable()))?;

    let h = Rc::clone(&host);
    obj.set_fn("is_high_pixel_density", move || {
        h.window(|w| w.is_high_pixel_density())
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("is_mouse_grabbed", move || {
        h.window(|w| w.is_mouse_grabbed())
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("is_keyboard_grabbed", move || {
        h.window(|w| w.is_keyboard_grabbed())
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("is_relative_mouse", move || {
        h.window(|w| w.is_relatve_mouse())
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("opacity", move || h.window(|w| w.opacity()))?;

    let h = Rc::clone(&host);
    obj.set_fn("clear_color", move || {
        h.window(|w| LuaColor(w.clear_color()))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_title", move |t: String| {
        h.window_mut(|w| w.set_title(t))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_size", move |s: LuaSize| {
        h.window_mut(|w| w.set_size(s.0))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_windowed", move || h.window_mut(|w| w.set_windowed()))?;

    let h = Rc::clone(&host);
    obj.set_fn("set_maximized", move || h.window_mut(|w| w.set_maximized()))?;

    let h = Rc::clone(&host);
    obj.set_fn("set_minimized", move || h.window_mut(|w| w.set_minimized()))?;

    let h = Rc::clone(&host);
    obj.set_fn("set_fullscreen", move |m: Option<LuaFullscreenMode>| {
        let m = m.map_or(FullscreenMode::Borderless, |m| m.0);
        h.window_mut(|w| w.set_fullscreen(m))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_hidden", move |v| h.window_mut(|w| w.set_hidden(v)))?;

    let h = Rc::clone(&host);
    obj.set_fn("set_resizable", move |v| {
        h.window_mut(|w| w.set_resizable(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_decorated", move |v| {
        h.window_mut(|w| w.set_decorated(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_always_on_top", move |v| {
        h.window_mut(|w| w.set_always_on_top(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_focusable", move |v| {
        h.window_mut(|w| w.set_focusable(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_mouse_grabbed", move |v| {
        h.window_mut(|w| w.set_mouse_grabbed(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_keyboard_grabbed", move |v| {
        h.window_mut(|w| w.set_keyboard_grabbed(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_relative_mouse", move |v| {
        h.window_mut(|w| w.set_relative_mouse(v))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_opacity", move |v| h.window_mut(|w| w.set_opacity(v)))?;

    let h = Rc::clone(&host);
    obj.set_fn("set_present_mode", move |m| {
        let m = present_mode(m)?;
        h.window_mut(|w| w.set_present_mode(m))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_clear_color", move |c: LuaColor| {
        h.window_mut(|w| w.set_clear_color(c.0))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("restore", move || h.window_mut(|w| w.restore()))?;

    Ok(obj)
}

fn time<'l>(lua: &'l lua::Lua, host: &Rc<LuaHost>) -> Result<lua::Value<'l>, lua::Error> {
    let obj = lua.table()?;

    let h = Rc::clone(&host);
    obj.set_fn("delta", move || h.time(|t| t.delta()))?;

    let h = Rc::clone(&host);
    obj.set_fn("fixed_delta", move || h.time(|t| t.fixed_delta()))?;

    let h = Rc::clone(&host);
    obj.set_fn("alpha", move || h.time(|t| t.alpha()))?;

    let h = Rc::clone(&host);
    obj.set_fn("fps", move || h.time(|t| t.fps()))?;

    let h = Rc::clone(&host);
    obj.set_fn("set_target_fps", move |fps: u32| {
        h.time_mut(|t| t.set_target_fps(fps))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_target_tps", move |tps: u32| {
        h.time_mut(|t| t.set_target_tps(tps))
    })?;

    Ok(obj)
}

fn key(i: u32) -> Result<Key, lua::Error> {
    Key::ALL
        .get(i as usize)
        .copied()
        .ok_or_else(|| lua::Error::Type(format!("{i} is not a karna.Key")))
}

const MOUSE_BUTTONS: [(&str, MouseButton); 5] = [
    ("Left", MouseButton::Left),
    ("Middle", MouseButton::Middle),
    ("Right", MouseButton::Right),
    ("X1", MouseButton::X1),
    ("X2", MouseButton::X2),
];

fn mouse_button(i: u32) -> Result<MouseButton, lua::Error> {
    MOUSE_BUTTONS
        .get(i as usize)
        .map(|(_, b)| *b)
        .ok_or_else(|| lua::Error::Type(format!("{i} is not a karna.Mouse button")))
}

fn input<'l>(lua: &'l lua::Lua, host: &Rc<LuaHost>) -> Result<lua::Value<'l>, lua::Error> {
    let obj = lua.table()?;

    let h = Rc::clone(&host);
    obj.set_fn("key_down", move |k: u32| {
        let k = key(k)?;
        h.input(|i| i.key_down(k))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("key_pressed", move |k: u32| {
        let k = key(k)?;
        h.input(|i| i.key_pressed(k))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("key_released", move |k: u32| {
        let k = key(k)?;
        h.input(|i| i.key_released(k))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("mouse_down", move |b: u32| {
        let b = mouse_button(b)?;
        h.input(|i| i.mouse_down(b))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("mouse_pressed", move |b: u32| {
        let b = mouse_button(b)?;
        h.input(|i| i.mouse_pressed(b))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("mouse_released", move |b: u32| {
        let b = mouse_button(b)?;
        h.input(|i| i.mouse_released(b))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("text", move || h.input(|i| String::from(i.text())))?;

    let h = Rc::clone(&host);
    obj.set_fn("wheel", move || h.input(|i| LuaVec2(i.mouse_wheel())))?;

    Ok(obj)
}

fn keys<'l>(lua: &'l lua::Lua) -> Result<lua::Value<'l>, lua::Error> {
    let obj = lua.table()?;

    for (i, k) in Key::ALL.iter().enumerate() {
        obj.set(&format!("{k:?}"), lua.integer(i as i64))?;
    }

    Ok(obj)
}

fn mouse_buttons<'l>(lua: &'l lua::Lua) -> Result<lua::Value<'l>, lua::Error> {
    let obj = lua.table()?;

    for (i, (name, _)) in MOUSE_BUTTONS.iter().enumerate() {
        obj.set(name, lua.integer(i as i64))?;
    }

    Ok(obj)
}

fn present_modes<'l>(lua: &'l lua::Lua) -> Result<lua::Value<'l>, lua::Error> {
    let obj = lua.table()?;

    for (i, (name, _)) in PRESENT_MODES.iter().enumerate() {
        obj.set(name, lua.integer(i as i64))?;
    }

    Ok(obj)
}

fn assets<'l>(lua: &'l lua::Lua, host: &Rc<LuaHost>) -> Result<lua::Value<'l>, lua::Error> {
    let obj = lua.table()?;

    let h = Rc::clone(&host);
    let f = lua.function_raw(move |lua, args| {
        let path = String::from_lua(args.first().unwrap_or(&lua.nil()))?;
        let handle = h.assets_mut(|a| a.load_image(path.as_str()))?;

        lua.instance(ImageRef(handle))
    })?;

    obj.set("load_image", f)?;

    let h = Rc::clone(&host);
    let f = lua.function_raw(move |lua, args| {
        let path = String::from_lua(args.first().unwrap_or(&lua.nil()))?;
        let handle = h.assets_mut(|a| a.load_audio(path.as_str()))?;

        lua.instance(AudioRef(handle))
    })?;

    obj.set("load_audio", f)?;

    Ok(obj)
}

fn audio<'l>(lua: &'l lua::Lua, host: &Rc<LuaHost>) -> Result<lua::Value<'l>, lua::Error> {
    let obj = lua.table()?;

    let h = Rc::clone(&host);
    let f = lua.function_raw(move |lua, args| {
        let nil = lua.nil();
        let arg = |i: usize| args.get(i).unwrap_or(&nil);

        let audio = arg(0)
            .opaque::<AudioRef>()
            .ok_or_else(|| lua::Error::Type("argument 1: expected an audio".into()))?
            .0;
        let options =
            Option::<LuaPlayOptions>::from_lua(arg(1))?.map_or_else(PlayOptions::default, |o| o.0);

        let voice = h.audio_mut(|a| a.play_with(audio, options))?;

        lua.instance(VoiceRef(voice))
    })?;

    obj.set("play", f)?;

    let h = Rc::clone(&host);
    let f = lua.function_raw(move |lua, args| {
        let nil = lua.nil();
        let voice = args
            .first()
            .unwrap_or(&nil)
            .opaque::<VoiceRef>()
            .ok_or_else(|| lua::Error::Type("argument 1: expected a voice".into()))?
            .0;

        h.audio_mut(|a| a.stop(voice))?;

        Ok(lua.nil())
    })?;

    obj.set("stop", f)?;

    let h = Rc::clone(&host);
    let f = lua.function_raw(move |lua, args| {
        let nil = lua.nil();
        let arg = |i: usize| args.get(i).unwrap_or(&nil);

        let voice = arg(0)
            .opaque::<VoiceRef>()
            .ok_or_else(|| lua::Error::Type("argument 1: expected a voice".into()))?
            .0;
        let gain = f32::from_lua(arg(1))?;

        h.audio_mut(|a| a.set_gain(voice, gain))?;

        Ok(lua.nil())
    })?;

    obj.set("set_gain", f)?;

    Ok(obj)
}

fn graphics<'l>(lua: &'l lua::Lua, host: &Rc<LuaHost>) -> Result<lua::Value<'l>, lua::Error> {
    let obj = lua.table()?;

    let h = Rc::clone(&host);
    obj.set_fn("color", move || h.draw(|d| LuaColor(d.color())))?;

    let h = Rc::clone(&host);
    obj.set_fn("set_color", move |color: LuaColor| {
        h.draw(|d| d.set_color(color.0))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_thickness", move |t: f32| {
        h.draw(|d| d.set_thickness(t))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("set_layer", move |name: String| {
        let layer = match name.as_str() {
            "world" => Layer::WORLD,
            "ui" => Layer::UI,
            "debug" => Layer::DEBUG,
            _ => {
                return Err(lua::Error::Type(format!(
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
    obj.set_fn("rect_outline", move |x: f32, y: f32, w: f32, hh: f32| {
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
    obj.set_fn("circle_outline", move |x: f32, y: f32, r: f32| {
        h.draw(|d| d.circle_outline(x, y, r))
    })?;

    let h = Rc::clone(&host);
    obj.set_fn("print", move |text: String, x: f32, y: f32| {
        h.draw(|d| d.print(text, x, y))
    })?;

    let h = Rc::clone(&host);
    let f = lua.function_raw(move |lua, args| {
        let nil = lua.nil();
        let arg = |i: usize| args.get(i).unwrap_or(&nil);

        let image = arg(0)
            .opaque::<ImageRef>()
            .ok_or_else(|| lua::Error::Type("argument 1: expected an image".into()))?
            .0;
        let x = f32::from_lua(arg(1))?;
        let y = f32::from_lua(arg(2))?;
        let w = Option::<f32>::from_lua(arg(3))?;
        let hh = Option::<f32>::from_lua(arg(4))?;

        h.draw(|d| match (w, hh) {
            (Some(w), Some(hh)) => d.image_sized(image, x, y, w, hh),
            _ => d.image(image, x, y),
        })?;

        Ok(lua.nil())
    })?;

    obj.set("image", f)?;

    Ok(obj)
}

fn log<'l>(lua: &'l lua::Lua, sink: fn(&str)) -> Result<lua::Value<'l>, lua::Error> {
    lua.function_raw(move |lua, args| {
        let parts = args
            .iter()
            .map(|a| a.to_string())
            .collect::<Result<Vec<_>, _>>()?;
        sink(&parts.join(" "));

        Ok(lua.nil())
    })
}

const PRELUDE: &str = r#"
local karna = ...

local WindowBuilder = {}
WindowBuilder.__index = WindowBuilder

function karna.WindowBuilder()
    return setmetatable({}, WindowBuilder)
end

function WindowBuilder:with_title(title)
    self.title = title
    return self
end

function WindowBuilder:with_size(width, height)
    self.width = width
    self.height = height
    return self
end

function WindowBuilder:with_resizable(resizable)
    self.resizable = resizable ~= false
    return self
end

function karna.Color(r, g, b, a)
    return { r = r, g = g, b = b, a = a or 1 }
end

function karna.Size(width, height)
    return { width = width, height = height }
end

karna.FullscreenMode = {
    Borderless = { kind = "borderless" },
    Exclusive = function(width, height, refresh_rate)
        return { kind = "exclusive", width = width, height = height, refresh_rate = refresh_rate or 0 }
    end,
}
"#;

pub fn install<'l>(lua: &'l Lua, host: &Rc<LuaHost>) -> Result<Api<'l>, lua::Error> {
    let karna = lua.table()?;

    karna.set("Key", keys(lua)?)?;
    karna.set("Mouse", mouse_buttons(lua)?)?;
    karna.set("PresentMode", present_modes(lua)?)?;

    karna.set("window", window(lua, host)?)?;
    karna.set("time", time(lua, host)?)?;
    karna.set("input", input(lua, host)?)?;
    karna.set("assets", assets(lua, host)?)?;
    karna.set("audio", audio(lua, host)?)?;

    let log_table = lua.table()?;

    log_table.set("trace", log(lua, |s| trace!("{s}"))?)?;
    log_table.set("debug", log(lua, |s| debug!("{s}"))?)?;
    log_table.set("info", log(lua, |s| info!("{s}"))?)?;
    log_table.set("warn", log(lua, |s| warn!("{s}"))?)?;
    log_table.set("error", log(lua, |s| error!("{s}"))?)?;

    karna.set("log", log_table)?;

    let globals = lua.globals();
    globals.set("print", log(lua, |s| info!("{s}"))?)?;

    lua.load(PRELUDE, "<karna>")?.call(&[karna.clone()])?;
    globals.set("karna", karna)?;

    Ok(Api {
        graphics: lua.persist(graphics(lua, host)?),
    })
}
