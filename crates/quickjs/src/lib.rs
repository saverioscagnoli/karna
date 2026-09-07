//! QuickJS bindings for the karna engine.
//!
//! A scene is a `.js` module whose default export has the same shape as the
//! Rust [`engine::Scene`] trait:
//!
//! ```js
//! import { Key } from "karna";
//!
//! export default {
//!     load(ctx) {
//!         this.x = 0;
//!     },
//!
//!     update(ctx) {
//!         if (ctx.input.keyDown(Key.D)) this.x += 200 * ctx.time.delta();
//!     },
//!
//!     draw(ctx, draw) {
//!         draw.setColor("#89b4fa");
//!         draw.rect(this.x, 100, 50, 50);
//!     },
//! };
//! ```
//!
//! ```no_run
//! use engine::App;
//! use engine::SceneId;
//! use engine::WindowBuilder;
//! use quickjs::JsWindowBuilderExt;
//!
//! const GAME: SceneId = SceneId::new_str("game");
//!
//! App::builder()
//!     .with_window(
//!         WindowBuilder::new()
//!             .with_js_scene(GAME, "examples/js/main.js")
//!             .with_active_scene(GAME),
//!     )
//!     .build()
//!     .run();
//! ```
//!
//! The binding is laid out by how each type relates to Rust's lifetimes.
//! [`value`] and [`enums`] are owned values JavaScript may keep for as long as
//! it likes; [`context`] and [`draw`] are engine borrows that live only for the
//! callback they were passed to, lent through the machinery in [`slot`].
//!
//! Only plain data on an owned value is a property — `v.x`, `color.r`. Anything
//! that reaches into the engine or computes on access is a method, so that its
//! cost is visible at the call site.

mod console;
mod context;
mod draw;
mod enums;
mod module;
mod scene;
mod slot;
mod value;

use std::path::PathBuf;

use engine::SceneId;
use engine::WindowBuilder;

pub use crate::context::JsAssets;
pub use crate::context::JsInput;
pub use crate::context::JsTime;
pub use crate::context::JsWindow;
pub use crate::draw::JsDraw;
pub use crate::draw::JsText;
pub use crate::enums::JsButton;
pub use crate::enums::JsCursor;
pub use crate::enums::JsFont;
pub use crate::enums::JsImage;
pub use crate::enums::JsKey;
pub use crate::enums::JsLayer;
pub use crate::module::KarnaModule;
pub use crate::module::NAME as MODULE_NAME;
pub use crate::scene::JsScene;
pub use crate::value::JsColor;
pub use crate::value::JsSize;
pub use crate::value::JsVec2;

/// Registers JavaScript modules as scenes on a [`WindowBuilder`].
pub trait JsWindowBuilderExt {
    /// Registers the module at `path` as the scene `id`.
    ///
    /// The file is read and evaluated lazily, when the scene is first loaded —
    /// the same point at which `Scene::load` would run.
    ///
    /// `path` is resolved relative to the process working directory, as are
    /// the script's own `import`s. Asset paths *inside* the script still go
    /// through the app's asset root as usual.
    fn with_js_scene<P>(self, id: SceneId, path: P) -> Self
    where
        P: Into<PathBuf>;
}

impl JsWindowBuilderExt for WindowBuilder {
    fn with_js_scene<P>(mut self, id: SceneId, path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        let path = path.into();

        self.scene_builders.insert(
            id,
            Box::new(move |ctx| Box::new(JsScene::new(path.clone(), ctx))),
        );

        self
    }
}
