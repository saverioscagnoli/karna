//! The bridge from the Rust [`Scene`] trait to a Lua table.
//!
//! A script returns a table with `load` / `update` / `fixed_update` / `draw`.
//! Each trait method opens a [`mlua::Scope`], wraps the borrowed context in
//! userdata that expires when the call returns, and invokes the Lua method with
//! the table as `self` — so `function Arena:update(ctx, scene)` colon syntax
//! works exactly as written.

use std::path::Path;
use std::path::PathBuf;

use engine::Draw;
use engine::DrawContext;
use engine::LoadContext;
use engine::Scene;
use engine::SceneView;
use engine::UpdateContext;
use logging::error;
use mlua::Function;
use mlua::Lua;
use mlua::Result;
use mlua::Scope;
use mlua::Table;

use crate::module;
use crate::refs::DrawRef;
use crate::refs::SceneViewRef;

pub struct LuaScene {
    lua: Lua,
    table: Table,
    path: PathBuf,

    /// Set once a callback raises, to stop re-reporting the same error at
    /// frame rate. The scene stays resident but inert.
    failed: bool,
}

impl LuaScene {
    /// Loads `path`, evaluates it, and runs its `load` method.
    ///
    /// Errors are logged rather than propagated: a broken script should leave a
    /// visible, empty scene, not take the process down.
    pub fn new(path: PathBuf, ctx: LoadContext, view: &mut SceneView) -> Self {
        let lua = Lua::new();

        let table = match Self::open(&lua, &path) {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to load scene '{}': {}", path.display(), e);
                let empty = lua.create_table().expect("failed to create fallback table");
                return Self {
                    lua,
                    table: empty,
                    path,
                    failed: true,
                };
            }
        };

        let mut scene = Self {
            lua,
            table,
            path,
            failed: false,
        };
        scene.call_load(ctx, view);
        scene
    }

    fn open(lua: &Lua, path: &Path) -> Result<Table> {
        module::install(lua)?;

        let src = std::fs::read_to_string(path).map_err(mlua::Error::external)?;
        let name = format!("@{}", path.display());

        lua.load(&src).set_name(name).eval::<Table>()
    }

    /// Builds the per-callback `ctx` table.
    ///
    /// The context fields arrive as `&'a mut` / `&'a`; they are passed into
    /// `create_any_userdata_ref*` **by move**, not reborrowed, so that `&'a mut
    /// T`'s covariance in `'a` can shorten them to the scope's lifetime. A
    /// reborrow inside the closure would be tied to the closure body instead
    /// and would not compile.
    fn ctx_table<'s, 'e>(
        lua: &Lua,
        scope: &'s Scope<'s, 'e>,
        window: &'e mut engine::WindowHandle,
        time: &'e engine::Time,
        input: &'e engine::Input,
        assets: &'e mut engine::AssetServer,
    ) -> Result<Table> {
        let ctx = lua.create_table_with_capacity(0, 4)?;

        ctx.set("window", scope.create_any_userdata_ref_mut(window)?)?;
        ctx.set("time", scope.create_any_userdata_ref(time)?)?;
        ctx.set("input", scope.create_any_userdata_ref(input)?)?;
        ctx.set("assets", scope.create_any_userdata_ref_mut(assets)?)?;

        Ok(ctx)
    }

    fn call_load(&mut self, ctx: LoadContext, view: &mut SceneView) {
        if self.failed {
            return;
        }

        #[rustfmt::skip]
        let LoadContext { window, time, input, assets } = ctx;

        let lua = &self.lua;
        let table = &self.table;

        // SAFETY: `vref` outlives the scope below, and the scope is contained
        // in the borrow of `view`; see `refs`.
        let vref = unsafe { SceneViewRef::new(view) };

        let res = lua.scope(|scope| {
            let Some(f) = optional_method(table, "load")? else {
                return Ok(());
            };

            let ctx = Self::ctx_table(lua, scope, window, time, input, assets)?;
            let view = scope.create_userdata_ref(&vref)?;

            f.call::<()>((table.clone(), ctx, view))
        });

        self.report("load", res);
    }

    fn call_update(&mut self, ctx: UpdateContext, view: &mut SceneView, method: &'static str) {
        if self.failed {
            return;
        }

        #[rustfmt::skip]
        let UpdateContext { window, time, input, assets } = ctx;

        let lua = &self.lua;
        let table = &self.table;

        // SAFETY: see `call_load`.
        let vref = unsafe { SceneViewRef::new(view) };

        let res = lua.scope(|scope| {
            let Some(f) = optional_method(table, method)? else {
                return Ok(());
            };

            let ctx = Self::ctx_table(lua, scope, window, time, input, assets)?;
            let view = scope.create_userdata_ref(&vref)?;

            f.call::<()>((table.clone(), ctx, view))
        });

        self.report(method, res);
    }

    fn call_draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        if self.failed {
            return;
        }

        #[rustfmt::skip]
        let DrawContext { window, time, input, assets } = ctx;

        let lua = &self.lua;
        let table = &self.table;

        // SAFETY: `dref` outlives the scope below, and the scope is contained
        // in the borrow of `draw`; see `refs`.
        let dref = unsafe { DrawRef::new(draw) };

        let res = lua.scope(|scope| {
            let Some(f) = optional_method(table, "draw")? else {
                return Ok(());
            };

            let ctx = lua.create_table_with_capacity(0, 4)?;
            ctx.set("window", scope.create_any_userdata_ref(window)?)?;
            ctx.set("time", scope.create_any_userdata_ref(time)?)?;
            ctx.set("input", scope.create_any_userdata_ref(input)?)?;
            ctx.set("assets", scope.create_any_userdata_ref(assets)?)?;

            f.call::<()>((table.clone(), ctx, scope.create_userdata_ref(&dref)?))
        });

        self.report("draw", res);
    }

    fn report(&mut self, method: &str, res: Result<()>) {
        let Err(e) = res else {
            return;
        };

        error!("{}:{} -- {}", self.path.display(), method, e);
        error!("Disabling scene '{}'.", self.path.display());
        self.failed = true;
    }
}

/// `fixed_update` is optional in the Rust trait, so it is optional here too;
/// a missing `draw` or `update` is likewise tolerated rather than fatal.
fn optional_method(table: &Table, name: &str) -> Result<Option<Function>> {
    match table.get::<mlua::Value>(name)? {
        mlua::Value::Function(f) => Ok(Some(f)),
        mlua::Value::Nil => Ok(None),
        other => Err(mlua::Error::runtime(format!(
            "scene field '{name}' must be a function, got {}",
            other.type_name()
        ))),
    }
}

impl Scene for LuaScene {
    fn load(_ctx: LoadContext, _scene: &mut SceneView) -> Self {
        unreachable!("LuaScene is constructed through LuaScene::new, not Scene::load")
    }

    fn update(&mut self, ctx: UpdateContext, scene: &mut SceneView) {
        self.call_update(ctx, scene, "update");
    }

    fn fixed_update(&mut self, ctx: UpdateContext, scene: &mut SceneView) {
        self.call_update(ctx, scene, "fixed_update");
    }

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        self.call_draw(ctx, draw);
    }
}
