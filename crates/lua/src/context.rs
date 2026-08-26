//! Tier-1 bindings: `'static` engine types that Lua only ever sees through a
//! scoped borrow.
//!
//! These are foreign types, so `impl UserData for WindowHandle` would trip the
//! orphan rule. `Lua::register_userdata_type` attaches the same metatable
//! without the trait, and `Scope::create_any_userdata_ref{,_mut}` hands out a
//! borrow that Lua cannot outlive.

use engine::AssetServer;
use engine::Input;
use engine::Time;
use engine::WindowHandle;
use mlua::Lua;
use mlua::Result;
use mlua::UserDataMethods;
use mlua::Value;

use crate::enums::LuaButton;
use crate::enums::LuaImage;
use crate::enums::LuaKey;
use crate::value::LuaColor;
use crate::value::LuaSize;
use crate::value::LuaVec2;
use crate::value::color_from;

pub fn register(lua: &Lua) -> Result<()> {
    window(lua)?;
    time(lua)?;
    input(lua)?;
    assets(lua)?;

    Ok(())
}

fn window(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<WindowHandle>(|reg| {
        reg.add_method("title", |_, w, ()| Ok(w.title().to_string()));

        // `Size<u32>` on the Rust side, but scripts do float math on it and Lua
        // has no integer/float distinction worth preserving here.
        reg.add_method("size", |_, w, ()| Ok(LuaSize(w.size().cast::<f32>())));
        reg.add_method("width", |_, w, ()| Ok(w.size().width as f32));
        reg.add_method("height", |_, w, ()| Ok(w.size().height as f32));

        reg.add_method("mouse_position", |_, w, ()| Ok(LuaVec2(w.mouse_position())));
        reg.add_method("mouse_delta", |_, w, ()| Ok(LuaVec2(w.mouse_delta())));
        reg.add_method("clear_color", |_, w, ()| Ok(LuaColor(w.clear_color())));

        reg.add_method_mut("set_title", |_, w, t: String| Ok(w.set_title(t)));

        reg.add_method_mut("set_size", |_, w, (width, height): (u32, u32)| {
            Ok(w.set_size(math::Size::new(width, height)))
        });

        reg.add_method_mut("set_clear_color", |_, w, c: Value| {
            Ok(w.set_clear_color(color_from(c)?))
        });

        reg.add_method_mut(
            "set_custom_cursor",
            |_, w, (img, x, y): (LuaImage, Option<u16>, Option<u16>)| {
                let hotspot = math::Vector2::new(x.unwrap_or(0), y.unwrap_or(0));
                Ok(w.set_custom_cursor(img.0, hotspot))
            },
        );
    })
}

fn time(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<Time>(|reg| {
        reg.add_method("delta", |_, t, ()| Ok(t.delta()));
        reg.add_method("fixed_delta", |_, t, ()| Ok(t.fixed_delta()));
        reg.add_method("fps", |_, t, ()| Ok(t.fps()));
        reg.add_method("alpha", |_, t, ()| Ok(t.alpha()));
        reg.add_method("frame", |_, t, ()| Ok(t.frame().as_secs_f32()));

        // Both are dispatched as events, so `&Time` is enough.
        reg.add_method("set_target_fps", |_, t, n: u32| Ok(t.set_target_fps(n)));
        reg.add_method("set_target_tps", |_, t, n: u32| Ok(t.set_target_tps(n)));
    })
}

fn input(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<Input>(|reg| {
        reg.add_method("key_down", |_, i, k: LuaKey| Ok(i.key_down(k.0)));
        reg.add_method("key_pressed", |_, i, k: LuaKey| Ok(i.key_pressed(k.0)));
        reg.add_method("key_released", |_, i, k: LuaKey| Ok(i.key_released(k.0)));

        reg.add_method("mouse_down", |_, i, b: LuaButton| Ok(i.mouse_down(b.0)));
        reg.add_method("mouse_pressed", |_, i, b: LuaButton| {
            Ok(i.mouse_pressed(b.0))
        });
        reg.add_method("mouse_released", |_, i, b: LuaButton| {
            Ok(i.mouse_released(b.0))
        });

        reg.add_method("mouse_wheel", |_, i, ()| Ok(LuaVec2(i.mouse_wheel())));
    })
}

fn assets(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<AssetServer>(|reg| {
        reg.add_method_mut("load_image", |_, a, path: String| {
            Ok(LuaImage(a.load_image(path)))
        });

        reg.add_method("is_image_pending", |_, a, h: LuaImage| {
            Ok(a.is_image_pending(h.0))
        });

        reg.add_method("image_size", |_, a, h: LuaImage| {
            Ok(LuaSize(a.get_image(h.0).size.cast::<f32>()))
        });

        reg.add_method("placeholder_image", |_, a, ()| {
            Ok(LuaImage(a.placeholder_image()))
        });
    })
}
