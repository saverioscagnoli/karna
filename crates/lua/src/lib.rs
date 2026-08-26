//! Lua bindings for the karna engine.
//!
//! A scene is a `.lua` file returning a table with `load`, `update`,
//! `fixed_update` and `draw` — the same shape as the Rust [`engine::Scene`]
//! trait:
//!
//! ```lua
//! local karna = require("karna")
//! local Game = {}
//!
//! function Game:load(ctx, scene) self.x = 0 end
//! function Game:update(ctx, scene) self.x = self.x + ctx.time:delta() end
//! function Game:draw(ctx, draw) draw:rect(self.x, 0, 10, 10) end
//!
//! return Game
//! ```
//!
//! ```no_run
//! # use engine::{App, WindowBuilder, SceneId};
//! # use lua::LuaWindowBuilderExt;
//! App::builder()
//!     .with_window(
//!         WindowBuilder::new()
//!             .with_lua_scene(SceneId::new_label("game"), "examples/game.lua")
//!             .with_active_scene(SceneId::new_label("game")),
//!     )
//!     .build()
//!     .run();
//! ```
//!
//! The binding is organised by how each type relates to Rust's lifetimes:
//! [`value`] and [`enums`] are owned `Copy` values, [`context`] holds `'static`
//! engine types Lua sees through a scoped borrow, and [`refs`] covers the two
//! types that carry a lifetime and must be pointer-erased.

mod context;
mod enums;
mod module;
mod refs;
mod scene;
mod value;

use std::path::PathBuf;

use engine::SceneId;
use engine::WindowBuilder;

pub use crate::enums::LuaButton;
pub use crate::enums::LuaImage;
pub use crate::enums::LuaKey;
pub use crate::enums::LuaLayer;
pub use crate::scene::LuaScene;
pub use crate::value::LuaColor;
pub use crate::value::LuaSize;
pub use crate::value::LuaVec2;

/// Registers Lua scripts as scenes on a [`WindowBuilder`].
pub trait LuaWindowBuilderExt {
    /// Registers the script at `path` as the scene `id`.
    ///
    /// The file is read and evaluated lazily, when the scene is first
    /// activated — the same point at which `Scene::load` would run.
    ///
    /// `path` is resolved relative to the process working directory, not the
    /// app's asset root; asset paths *inside* the script still go through the
    /// asset root as usual.
    fn with_lua_scene<P>(self, id: SceneId, path: P) -> Self
    where
        P: Into<PathBuf>;
}

impl LuaWindowBuilderExt for WindowBuilder {
    fn with_lua_scene<P>(mut self, id: SceneId, path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        let path = path.into();

        self.scenes.insert(
            id,
            Box::new(move |ctx, view| Box::new(LuaScene::new(path, ctx, view))),
        );

        self
    }
}
