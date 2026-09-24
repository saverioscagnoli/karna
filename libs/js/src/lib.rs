#![no_std]

mod api;
mod host;

use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use engine::builder::WindowBuilder;
use engine::context::DrawContext;
use engine::context::LoadContext;
use engine::context::UpdateContext;
use engine::render::Draw;
use engine::render::Layer;
use engine::scene::BoxedScene;
use engine::scene::Scene;
use engine::scene::SceneId;
use nostd::alloc::boxed::Box;
use nostd::alloc::format;
use nostd::alloc::rc::Rc;
use nostd::alloc::string::String;
use nostd::alloc::vec;
use nostd::path::Path;
use nostd::path::PathBuf;
use quickjs::Context;
use quickjs::Error;
use quickjs::Persistent;
use quickjs::Runtime;
use sdl3::render::Color;
use traccia::error;
use traccia::info;

use crate::api::Api;
use crate::host::Assets;
use crate::host::Frame;
use crate::host::Host;

pub const TYPES: &str = include_str!("../karna.d.ts");

pub struct ScriptScene {
    // All of these borrow the runtime behind `rt`; `Drop` frees them first.
    api: Option<Api<'static>>,
    scene: Option<Persistent<'static>>,
    js: ManuallyDrop<Context<'static>>,
    rt: NonNull<Runtime>,

    host: Rc<Host>,
    path: PathBuf,
    error: Option<String>,
}

#[derive(Clone, Copy)]
enum Phase {
    Update,
    Draw,
}

impl ScriptScene {
    pub fn from_path(ctx: &mut LoadContext, path: impl AsRef<Path>) -> Self {
        let path = ctx.assets.root().join(path);

        let rt = Box::new(Runtime::new().expect("failed to create a JS runtime"));
        let rt = NonNull::from(Box::leak(rt));
        let js = Context::new(unsafe { rt.as_ref() }).expect("failed to create a JS context");

        unsafe { rt.as_ref() }.set_module_loader(|name| {
            nostd::fs::read(name)
                .map(|blob| String::from_utf8_lossy(&blob).into_owned())
                .map_err(Error::custom)
        });

        let mut this = Self {
            api: None,
            scene: None,
            js: ManuallyDrop::new(js),
            rt,
            host: Rc::new(Host::default()),
            path,
            error: None,
        };

        match api::install(&this.js, &this.host) {
            Ok(api) => this.api = Some(api),
            Err(e) => {
                this.fail("installing the karna API", e);
                return this;
            }
        }

        let source = match nostd::fs::read(&this.path) {
            Ok(blob) => String::from_utf8_lossy(&blob).into_owned(),
            Err(e) => {
                this.fail("loading", Error::custom(e));
                return this;
            }
        };

        info!("Running script {}", this.path);

        let frame = Frame::new(
            &mut ctx.window,
            &mut ctx.time,
            ctx.input,
            Assets::Mut(ctx.assets),
            None,
        );

        let js = &*this.js;
        let rt = unsafe { this.rt.as_ref() };
        let filename = this.path.as_str();

        let scene = this.host.enter(frame, || {
            let module = js.eval_module(&source, filename)?;
            let export = module.get("default")?;

            // A class is instantiated; a plain object is the scene itself.
            let scene = if export.is_function() {
                export.construct(&[])?
            } else if export.is_object() {
                export
            } else {
                return Err(Error::custom(
                    "the script must `export default` a scene class or object",
                ));
            };

            let scene = js.persist(scene);
            rt.run_jobs()?;

            Ok(scene)
        });

        match scene {
            Ok(scene) => this.scene = Some(scene),
            Err(e) => {
                this.fail("the module", e);
                return this;
            }
        }

        this.call(frame, "load", Phase::Update);
        this
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    fn call(&mut self, frame: Frame, name: &str, phase: Phase) {
        if self.error.is_some() {
            return;
        }

        let (Some(api), Some(scene)) = (&self.api, &self.scene) else {
            return;
        };

        let js = &*self.js;
        let rt = unsafe { self.rt.as_ref() };

        let result = self.host.enter(frame, || {
            let scene = scene.get(js);
            let method = scene.get(name)?;

            if method.is_function() {
                let args = match phase {
                    Phase::Update => vec![api.ctx.get(js)],
                    Phase::Draw => vec![api.draw_ctx.get(js), api.graphics.get(js)],
                };

                method.call_with(&scene, &args)?;
            }

            rt.run_jobs()
        });

        if let Err(e) = result {
            self.fail(name, e);
        }
    }

    fn fail(&mut self, what: &str, e: Error) {
        let msg = format!("{}: error in {what}: {e}", self.path);
        error!("{msg}");
        self.error = Some(msg);
    }

    fn draw_error(&self, draw: &mut Draw) {
        let Some(msg) = &self.error else {
            return;
        };

        let (layer, color) = (draw.layer(), draw.color());

        draw.with_layer(Layer::UI).set_color(Color::RED);
        draw.print(msg, 10.0, 10.0);
        draw.with_layer(layer).set_color(color);
    }
}

impl Scene for ScriptScene {
    fn load(ctx: &mut LoadContext) -> Self {
        Self::from_path(ctx, "main.js")
    }

    fn unload(&mut self, ctx: &mut LoadContext) {
        let frame = Frame::new(
            &mut ctx.window,
            &mut ctx.time,
            ctx.input,
            Assets::Mut(ctx.assets),
            None,
        );

        self.call(frame, "unload", Phase::Update);
    }

    fn fixed_update(&mut self, ctx: &mut UpdateContext) {
        let frame = Frame::new(
            &mut ctx.window,
            &mut ctx.time,
            ctx.input,
            Assets::Mut(ctx.assets),
            None,
        );
        self.call(frame, "fixedUpdate", Phase::Update);
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        let frame = Frame::new(
            &mut ctx.window,
            &mut ctx.time,
            ctx.input,
            Assets::Mut(ctx.assets),
            None,
        );

        self.call(frame, "update", Phase::Update);
    }

    fn draw(&mut self, ctx: &mut DrawContext, draw: &mut Draw) {
        let frame = Frame::new(
            &mut ctx.window,
            &mut ctx.time,
            ctx.input,
            Assets::Ref(ctx.assets),
            Some(&mut *draw),
        );

        self.call(frame, "draw", Phase::Draw);
        self.draw_error(draw);
    }
}

impl Drop for ScriptScene {
    fn drop(&mut self) {
        unsafe {
            self.scene = None;
            self.api = None;
            ManuallyDrop::drop(&mut self.js);
            drop(Box::from_raw(self.rt.as_ptr()));
        }
    }
}

pub trait WindowBuilderExt {
    fn with_js_script(self, id: SceneId, path: impl Into<PathBuf>) -> Self;
}

impl WindowBuilderExt for WindowBuilder {
    fn with_js_script(self, id: SceneId, path: impl Into<PathBuf>) -> Self {
        let path = path.into();

        self.with_scene_fn(id, move |ctx| {
            Box::new(ScriptScene::from_path(ctx, &path)) as BoxedScene
        })
    }
}
