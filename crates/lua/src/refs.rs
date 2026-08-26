//! Tier-3 bindings: types carrying a lifetime, which cannot be registered.
//!
//! `Draw<'a>` and `SceneView<'a>` are not `'static`, so neither
//! `register_userdata_type` nor the blanket `UserData` route accepts them, and
//! `Scope::create_any_userdata` (which does take non-`'static` data) rebuilds
//! the metatable on every call — far too costly at frame rate.
//!
//! Instead each is erased to a `'static` pointer newtype that registers once.
//! The pointer is only ever handed to Lua from inside a [`mlua::Scope`] nested
//! within the borrow it came from, so the userdata is invalidated before the
//! borrow ends: Lua stashing a `draw` in a global gets a "destructed userdata"
//! error on the next frame rather than a dangling read.

use engine::Draw;
use engine::SceneView;
use mlua::Lua;
use mlua::Result;
use mlua::UserData;
use mlua::UserDataMethods;
use mlua::Value;

use crate::enums::LuaImage;
use crate::enums::LuaLayer;
use crate::value::LuaColor;
use crate::value::LuaSize;
use crate::value::LuaVec2;
use crate::value::color_from;

#[derive(Clone, Copy)]
pub struct DrawRef(*mut Draw<'static>);

impl DrawRef {
    /// # Safety
    ///
    /// The returned value must only be published to Lua through a scope whose
    /// lifetime is strictly contained in `'a`.
    pub unsafe fn new<'a>(draw: &mut Draw<'a>) -> Self {
        Self(unsafe { std::mem::transmute::<*mut Draw<'a>, *mut Draw<'static>>(draw) })
    }

    #[allow(clippy::mut_from_ref)]
    fn get(&self) -> &mut Draw<'static> {
        unsafe { &mut *self.0 }
    }
}

#[derive(Clone, Copy)]
pub struct SceneViewRef(#[allow(dead_code)] *mut SceneView<'static>);

impl SceneViewRef {
    /// # Safety
    ///
    /// See [`DrawRef::new`].
    pub unsafe fn new<'a>(view: &mut SceneView<'a>) -> Self {
        Self(unsafe { std::mem::transmute::<*mut SceneView<'a>, *mut SceneView<'static>>(view) })
    }
}

impl UserData for DrawRef {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method("viewport", |_, d, ()| {
            Ok(LuaSize(d.get().viewport().cast::<f32>()))
        });

        m.add_method("color", |_, d, ()| Ok(LuaColor(d.get().color())));
        m.add_method("layer", |_, d, ()| Ok(LuaLayer(d.get().layer())));

        m.add_method("set_color", |_, d, c: Value| {
            Ok(d.get().set_color(color_from(c)?))
        });

        m.add_method("set_layer", |_, d, l: LuaLayer| Ok(d.get().set_layer(l.0)));

        m.add_method("rect", |_, d, (x, y, w, h): (f32, f32, f32, f32)| {
            Ok(d.get().rect(x, y, w, h))
        });

        m.add_method("rect_v", |_, d, (pos, size): (LuaVec2, LuaSize)| {
            Ok(d.get().rect_v(pos.0, size.0))
        });

        m.add_method("image", |_, d, (img, x, y): (LuaImage, f32, f32)| {
            Ok(d.get().image(img.0, x, y))
        });
    }
}

impl UserData for SceneViewRef {
    fn add_methods<M: UserDataMethods<Self>>(_m: &mut M) {
        // `Camera` exposes no public position/zoom setters yet, so there is
        // nothing to bind here beyond the handle itself. Scripts still receive
        // it so their signatures match the Rust `Scene` trait.
    }
}

/// Registers both metatables up front, so the first frame does not pay for it.
pub fn register(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<DrawRef>(<DrawRef as UserData>::register)?;
    lua.register_userdata_type::<SceneViewRef>(<SceneViewRef as UserData>::register)?;

    Ok(())
}
