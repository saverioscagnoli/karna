//! Builds the `karna` table and installs it in `package.preload`, so scripts
//! reach it with `local karna = require("karna")`.

use engine::Color;
use math::Size;
use math::Vector2;
use mlua::Lua;
use mlua::Result;
use mlua::Table;
use mlua::Value;
use mlua::Variadic;

use crate::context;
use crate::enums;
use crate::refs;
use crate::value::LuaColor;
use crate::value::LuaSize;
use crate::value::LuaVec2;

/// Registers every metatable and preloads the module.
///
/// Called once per `Lua` state, before the scene script is evaluated.
pub fn install(lua: &Lua) -> Result<()> {
    context::register(lua)?;
    refs::register(lua)?;

    let preload: Table = lua.globals().get::<Table>("package")?.get("preload")?;
    preload.set("karna", lua.create_function(|lua, ()| build(lua))?)?;

    Ok(())
}

fn build(lua: &Lua) -> Result<Table> {
    let karna = lua.create_table()?;

    karna.set("vec2", vec2_ctor(lua)?)?;
    karna.set("size", size_ctor(lua)?)?;
    karna.set("color", color_ctor(lua)?)?;

    karna.set("key", enums::key_table(lua)?)?;
    karna.set("button", enums::button_table(lua)?)?;
    karna.set("layer", enums::layer_table(lua)?)?;

    Ok(karna)
}

/// Callable table: `vec2(x, y)` via `__call`, plus `vec2.zero()` and friends.
fn vec2_ctor(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;

    t.set(
        "zero",
        lua.create_function(|_, ()| Ok(LuaVec2(Vector2::zero())))?,
    )?;
    t.set(
        "one",
        lua.create_function(|_, ()| Ok(LuaVec2(Vector2::one())))?,
    )?;
    t.set(
        "splat",
        lua.create_function(|_, v: f32| Ok(LuaVec2(Vector2::splat(v))))?,
    )?;
    t.set(
        "from_angle",
        lua.create_function(|_, a: f32| Ok(LuaVec2(Vector2::from_angle(a))))?,
    )?;

    callable(lua, t, |_, (x, y): (f32, f32)| {
        Ok(LuaVec2(Vector2::new(x, y)))
    })
}

fn size_ctor(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;

    t.set(
        "zero",
        lua.create_function(|_, ()| Ok(LuaSize(Size::zero())))?,
    )?;
    t.set(
        "square",
        lua.create_function(|_, s: f32| Ok(LuaSize(Size::square(s))))?,
    )?;

    callable(lua, t, |_, (w, h): (f32, f32)| Ok(LuaSize(Size::new(w, h))))
}

/// `color.rgb`, `color.rgba`, `color.hex("#rrggbb")`, the named constants, and
/// `color(r, g, b [, a])` through `__call`.
fn color_ctor(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;

    t.set(
        "rgb",
        lua.create_function(|_, (r, g, b): (f32, f32, f32)| Ok(LuaColor(Color::rgb(r, g, b))))?,
    )?;

    t.set(
        "rgba",
        lua.create_function(|_, (r, g, b, a): (f32, f32, f32, f32)| {
            Ok(LuaColor(Color::rgba(r, g, b, a)))
        })?,
    )?;

    // Accepts both `color.hex("#89b4fa")` and `color.hex(0x89b4fa)`.
    t.set(
        "hex",
        lua.create_function(|_, v: Value| match v {
            Value::String(s) => Color::try_hex(&s.to_str()?)
                .map(LuaColor)
                .ok_or_else(|| mlua::Error::runtime(format!("not a hex color: {}", s.display()))),
            Value::Integer(n) => Ok(LuaColor(Color::hex(n as u32))),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: ("hex color").into(),
                message: None,
            }),
        })?,
    )?;

    for (name, c) in [
        ("RED", Color::RED),
        ("GREEN", Color::GREEN),
        ("BLUE", Color::BLUE),
        ("WHITE", Color::WHITE),
        ("BLACK", Color::BLACK),
        ("YELLOW", Color::YELLOW),
        ("CYAN", Color::CYAN),
        ("MAGENTA", Color::MAGENTA),
        ("GRAY", Color::GRAY),
        ("ORANGE", Color::ORANGE),
        ("PURPLE", Color::PURPLE),
        ("BROWN", Color::BROWN),
        ("PINK", Color::PINK),
    ] {
        t.set(name, LuaColor(c))?;
    }

    callable(lua, t, |_, args: Variadic<f32>| match args.len() {
        3 => Ok(LuaColor(Color::rgb(args[0], args[1], args[2]))),
        4 => Ok(LuaColor(Color::rgba(args[0], args[1], args[2], args[3]))),
        n => Err(mlua::Error::runtime(format!(
            "color expects 3 or 4 components, got {n}"
        ))),
    })
}

/// Gives `t` a `__call` metamethod so it doubles as a constructor.
fn callable<A, R, F>(lua: &Lua, t: Table, f: F) -> Result<Table>
where
    F: Fn(&Lua, A) -> Result<R> + 'static,
    A: mlua::FromLuaMulti,
    R: mlua::IntoLuaMulti,
{
    let meta = lua.create_table()?;

    // Lua passes the table itself as the first `__call` argument; drop it.
    meta.set(
        "__call",
        lua.create_function(move |lua, (_, args): (Table, A)| f(lua, args))?,
    )?;

    t.set_metatable(Some(meta))?;

    Ok(t)
}
