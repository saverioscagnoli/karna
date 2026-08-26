//! Enum-like handles Lua receives as opaque userdata: keys, mouse buttons,
//! layers and asset handles.
//!
//! These are values rather than integers so a typo produces a `nil` index
//! error at the call site instead of silently reading the wrong key.

use engine::Image;
use engine::Key;
use engine::Layer;
use engine::MouseButton;
use mlua::Lua;
use mlua::MetaMethod;
use mlua::Result;
use mlua::Table;
use mlua::UserData;
use mlua::UserDataMethods;
use mlua::Value;
use utils::Handle;

macro_rules! opaque {
    ($($name:ident($inner:ty) => $label:literal, $show:expr),* $(,)?) => {$(
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name(pub $inner);

        impl UserData for $name {
            fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
                m.add_meta_method(MetaMethod::Eq, |_, a, b: Self| Ok(a.0 == b.0));
                m.add_meta_method(MetaMethod::ToString, |_, v, ()| {
                    let show: fn(&$inner) -> String = $show;
                    Ok(format!("{}({})", $label, show(&v.0)))
                });
            }
        }

        impl mlua::FromLua for $name {
            fn from_lua(value: Value, _: &Lua) -> Result<Self> {
                match value {
                    Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
                    other => Err(mlua::Error::FromLuaConversionError { from: other.type_name(), to: ($label).into(), message: None }),
                }
            }
        }
    )*};
}

opaque! {
    LuaKey(Key)             => "key",    |k| format!("{k:?}"),
    LuaButton(MouseButton)  => "button", |b| format!("{b:?}"),
    LuaLayer(Layer)         => "layer",  layer_name,
    LuaImage(Handle<Image>) => "image",  |h: &Handle<Image>| format!("#{}", h.index()),
}

/// `Layer`'s `Debug` prints the raw FNV hash, which is useless in a traceback.
fn layer_name(l: &Layer) -> String {
    match *l {
        Layer::WORLD => "WORLD".into(),
        Layer::UI => "UI".into(),
        Layer::DEBUG => "DEBUG".into(),
        other => format!("{:?}", other),
    }
}

/// `karna.key.W`, `karna.key.Space`, ... — one entry per key the engine models.
///
/// Names come from the `Debug` impl the `keys!` macro derives, so this stays in
/// sync with `crates/engine/src/input/keys.rs` for free.
pub fn key_table(lua: &Lua) -> Result<Table> {
    let t = lua.create_table_with_capacity(0, Key::ALL.len())?;

    for &k in Key::ALL {
        t.set(format!("{k:?}"), LuaKey(k))?;
    }

    seal(lua, &t, "key")
}

pub fn button_table(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;

    for &b in MouseButton::ALL {
        t.set(format!("{b:?}"), LuaButton(b))?;
    }

    seal(lua, &t, "button")
}

pub fn layer_table(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;

    t.set("WORLD", LuaLayer(Layer::WORLD))?;
    t.set("UI", LuaLayer(Layer::UI))?;
    t.set("DEBUG", LuaLayer(Layer::DEBUG))?;

    seal(lua, &t, "layer")
}

/// Make unknown lookups raise instead of returning `nil`.
///
/// Without this, `key.Escpae` reads as `nil` and only fails much later inside
/// `key_down`, with no hint about where the typo was.
fn seal(lua: &Lua, t: &Table, what: &'static str) -> Result<Table> {
    let meta = lua.create_table()?;

    meta.set(
        "__index",
        lua.create_function(move |_, (_, k): (Table, String)| -> Result<Value> {
            Err(mlua::Error::runtime(format!("unknown {what}: {k}")))
        })?,
    )?;

    meta.set(
        "__newindex",
        lua.create_function(move |_, ()| -> Result<Value> {
            Err(mlua::Error::runtime(format!("{what} table is read-only")))
        })?,
    )?;

    t.set_metatable(Some(meta))?;

    Ok(t.clone())
}
