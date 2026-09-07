//! The bridge from the Rust [`Scene`] trait to a JavaScript module.
//!
//! A scene script is an ES module whose default export is an object with
//! `load` / `update` / `fixedUpdate` / `draw` / `unload` methods — the same
//! shape as the trait, minus the ones it does not need. Each is called with
//! that object as `this`, so a script can keep its state on `this` exactly as
//! the Rust side keeps it in `Self`.
//!
//! Errors never propagate: a script that throws is logged, with its JS stack,
//! and then disabled. A broken scene should leave a visible empty screen, not
//! take the process down mid-frame.

use std::fs;
use std::mem;
use std::path::Path;
use std::path::PathBuf;

use engine::AssetServer;
use engine::Draw;
use engine::DrawContext;
use engine::Input;
use engine::LoadContext;
use engine::Scene;
use engine::Time;
use engine::UpdateContext;
use engine::WindowHandle;
use logging::error;
use rquickjs::CatchResultExt;
use rquickjs::Class;
use rquickjs::Context;
use rquickjs::Ctx;
use rquickjs::Function;
use rquickjs::Module;
use rquickjs::Object;
use rquickjs::Persistent;
use rquickjs::Result;
use rquickjs::Runtime;
use rquickjs::Value;
use rquickjs::function::This;
use rquickjs::loader::BuiltinResolver;
use rquickjs::loader::FileResolver;
use rquickjs::loader::ModuleLoader;
use rquickjs::loader::ScriptLoader;

use crate::console;
use crate::context::JsAssets;
use crate::context::JsInput;
use crate::context::JsTime;
use crate::context::JsWindow;
use crate::draw::JsDraw;
use crate::module;
use crate::module::KarnaModule;
use crate::slot::Slot;

/// The borrows a callback lends to JavaScript, one slot per `ctx` field.
struct Slots {
    window: Slot<WindowHandle>,
    time: Slot<Time>,
    input: Slot<Input>,
    assets: Slot<AssetServer>,
    draw: Slot<Draw<'static>>,
}

impl Slots {
    fn new() -> Self {
        Self {
            window: Slot::new("ctx.window"),
            time: Slot::new("ctx.time"),
            input: Slot::new("ctx.input"),
            assets: Slot::new("ctx.assets"),
            draw: Slot::new("draw"),
        }
    }
}

pub struct JsScene {
    // Persistent values must be dropped before the context that keeps their
    // runtime alive, and struct fields drop in declaration order.
    scene: Persistent<Object<'static>>,
    bridge: Persistent<Object<'static>>,
    draw: Persistent<Object<'static>>,
    context: Context,

    slots: Slots,
    path: PathBuf,

    /// Set once a callback throws, so the same error is not reported at frame
    /// rate. The scene stays resident but inert.
    failed: bool,
}

impl JsScene {
    /// Loads and evaluates the module at `path`, then runs its `load` method.
    pub fn new(path: PathBuf, ctx: &mut LoadContext) -> Self {
        let slots = Slots::new();

        let context = match open(&path) {
            Ok(context) => context,
            Err(e) => return Self::stub(path, e),
        };

        let built = context.with(|jsctx| -> Result<_> {
            let bridge = build_bridge(&jsctx, &slots)?;
            let draw = Class::instance(
                jsctx.clone(),
                JsDraw {
                    slot: slots.draw.clone(),
                },
            )?;

            let scene = evaluate(&jsctx, &path)?;

            Ok((
                Persistent::save(&jsctx, scene),
                Persistent::save(&jsctx, bridge),
                Persistent::save(&jsctx, draw.into_inner()),
            ))
        });

        let (scene, bridge, draw) = match built.catch_string(&context) {
            Ok(saved) => saved,
            Err(e) => return Self::stub(path, e),
        };

        let mut scene = Self {
            scene,
            bridge,
            draw,
            context,
            slots,
            path,
            failed: false,
        };

        let LoadContext {
            window,
            time,
            input,
            assets,
        } = ctx;

        scene.invoke("load", window, time, input, assets);
        scene
    }

    /// A scene that failed to load: resident, inert, and quiet from here on.
    fn stub(path: PathBuf, error: String) -> Self {
        error!("Failed to load scene '{}': {}", path.display(), error);

        let context = Context::full(&Runtime::new().expect("failed to create a QuickJS runtime"))
            .expect("failed to create a QuickJS context");

        let (scene, bridge, draw) = context.with(|ctx| {
            let empty = Object::new(ctx.clone()).expect("failed to create a fallback object");

            (
                Persistent::save(&ctx, empty.clone()),
                Persistent::save(&ctx, empty.clone()),
                Persistent::save(&ctx, empty),
            )
        });

        Self {
            scene,
            bridge,
            draw,
            context,
            slots: Slots::new(),
            path,
            failed: true,
        }
    }

    /// Calls `method` with the four context borrows lent for the call.
    fn invoke(
        &mut self,
        method: &'static str,
        window: &mut WindowHandle,
        time: &Time,
        input: &Input,
        assets: &mut AssetServer,
    ) {
        if self.failed {
            return;
        }

        let Self {
            scene,
            bridge,
            context,
            slots,
            ..
        } = self;

        let _window = slots.window.lend_mut(window);
        let _time = slots.time.lend(time);
        let _input = slots.input.lend(input);
        let _assets = slots.assets.lend_mut(assets);

        let res = context.with(|ctx| {
            let scene = scene.clone().restore(&ctx)?;

            let Some(f) = optional_method(&scene, method)? else {
                return Ok(());
            };

            f.call::<_, ()>((This(scene), bridge.clone().restore(&ctx)?))
        });

        let res = res.catch_string(context);

        self.report(method, res);
    }

    fn invoke_draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        if self.failed {
            return;
        }

        let DrawContext {
            window,
            time,
            input,
        } = ctx;

        let Self {
            scene,
            bridge,
            draw: draw_object,
            context,
            slots,
            ..
        } = self;

        // SAFETY: `Draw<'w>` is not `'static`, so the slot stores it erased.
        // The lease below is dropped before this function returns, and the
        // pointer is only reachable from JavaScript while the lease is live,
        // so it can never outlive the borrow it came from.
        let erased: &mut Draw<'static> = unsafe { mem::transmute(draw) };

        // `draw` receives the window immutably, matching `DrawContext`; the
        // window setters therefore raise if a script calls them from here.
        let _window = slots.window.lend(window);
        let _time = slots.time.lend(time);
        let _input = slots.input.lend(input);
        let _draw = slots.draw.lend_mut(erased);

        let res = context.with(|ctx| {
            let scene = scene.clone().restore(&ctx)?;

            let Some(f) = optional_method(&scene, "draw")? else {
                return Ok(());
            };

            let bridge = bridge.clone().restore(&ctx)?;
            let draw = draw_object.clone().restore(&ctx)?;

            f.call::<_, ()>((This(scene), bridge, draw))
        });

        let res = res.catch_string(context);

        self.report("draw", res);
    }

    fn report(&mut self, method: &str, res: std::result::Result<(), String>) {
        let Err(e) = res else {
            return;
        };

        error!("{}:{} -- {}", self.path.display(), method, e);
        error!("Disabling scene '{}'.", self.path.display());

        self.failed = true;
    }
}

impl Scene for JsScene {
    fn load(_ctx: &mut LoadContext) -> Self {
        unreachable!("JsScene is built through JsScene::new, not Scene::load")
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        let UpdateContext {
            window,
            time,
            input,
            assets,
        } = ctx;

        self.invoke("update", window, time, input, assets);
    }

    fn fixed_update(&mut self, ctx: &mut UpdateContext) {
        let UpdateContext {
            window,
            time,
            input,
            assets,
        } = ctx;

        self.invoke("fixedUpdate", window, time, input, assets);
    }

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        self.invoke_draw(ctx, draw);
    }

    fn unload(&mut self, ctx: &mut LoadContext) {
        let LoadContext {
            window,
            time,
            input,
            assets,
        } = ctx;

        self.invoke("unload", window, time, input, assets);
    }
}

/// A fresh runtime wired up to resolve `"karna"` natively and everything else
/// as a file next to `path`.
fn open(path: &Path) -> std::result::Result<Context, String> {
    if !path.is_file() {
        return Err(format!("no such file: {}", path.display()));
    }

    let runtime = Runtime::new().map_err(|e| e.to_string())?;

    let mut files = FileResolver::default()
        .with_pattern("{}.js")
        .with_pattern("{}.mjs")
        .with_path(".");

    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        files.add_path(dir.to_string_lossy().to_string());
    }

    runtime.set_loader(
        (BuiltinResolver::default().with_module(module::NAME), files),
        (
            ModuleLoader::default().with_module(module::NAME, KarnaModule),
            ScriptLoader::default()
                .with_extension("js")
                .with_extension("mjs"),
        ),
    );

    let context = Context::full(&runtime).map_err(|e| e.to_string())?;

    context
        .with(|ctx| console::install(&ctx, &path.display().to_string()))
        .map_err(|e| e.to_string())?;

    Ok(context)
}

/// Evaluates the module and returns its default export.
fn evaluate<'js>(ctx: &Ctx<'js>, path: &Path) -> Result<Object<'js>> {
    let source = fs::read_to_string(path)
        .map_err(|e| rquickjs::Error::new_loading_message(path.to_string_lossy(), e.to_string()))?;

    let name = path.to_string_lossy().to_string();
    let (module, promise) = Module::declare(ctx.clone(), name, source)?.eval()?;

    // Drives the module's top-level code, including any pending jobs it
    // queued, so a failure surfaces here rather than on the first frame.
    promise.finish::<()>()?;

    let default: Value = module.get("default")?;

    default.into_object().ok_or_else(|| {
        rquickjs::Exception::throw_type(
            ctx,
            "a scene module must `export default` an object with load/update/draw methods",
        )
    })
}

/// The `ctx` object handed to every callback, built once and reused.
fn build_bridge<'js>(ctx: &Ctx<'js>, slots: &Slots) -> Result<Object<'js>> {
    let bridge = Object::new(ctx.clone())?;

    bridge.set(
        "window",
        JsWindow {
            slot: slots.window.clone(),
        },
    )?;

    bridge.set(
        "time",
        JsTime {
            slot: slots.time.clone(),
        },
    )?;

    bridge.set(
        "input",
        JsInput {
            slot: slots.input.clone(),
        },
    )?;

    bridge.set(
        "assets",
        JsAssets {
            slot: slots.assets.clone(),
        },
    )?;

    Ok(bridge)
}

/// Every callback is optional, as in the Rust trait; a field that is present
/// but not callable is a mistake worth reporting.
fn optional_method<'js>(scene: &Object<'js>, name: &str) -> Result<Option<Function<'js>>> {
    match scene.get::<_, Value<'js>>(name)? {
        v if v.is_undefined() || v.is_null() => Ok(None),
        v => match v.into_function() {
            Some(f) => Ok(Some(f)),
            None => Err(rquickjs::Exception::throw_type(
                scene.ctx(),
                &format!("scene field '{name}' must be a function"),
            )),
        },
    }
}

/// Runs a fallible block against `context` and flattens any JS exception into
/// a formatted string, so the error can escape the `with` closure it was
/// raised in.
trait CatchString<T> {
    fn catch_string(self, context: &Context) -> std::result::Result<T, String>;
}

impl<T> CatchString<T> for Result<T> {
    fn catch_string(self, context: &Context) -> std::result::Result<T, String> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(context.with(|ctx| {
                Result::<T>::Err(e)
                    .catch(&ctx)
                    .err()
                    .map(|c| c.to_string())
                    .unwrap_or_default()
            })),
        }
    }
}
