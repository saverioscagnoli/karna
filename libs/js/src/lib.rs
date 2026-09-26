#![no_std]

mod api;
mod host;

use core::cell::RefCell;
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
use quickjs::FromJs;
use quickjs::Persistent;
use quickjs::Runtime;
use sdl3::render::Color;
use traccia::error;
use traccia::info;

use crate::api::Api;
use crate::host::Assets;
use crate::host::Frame;
use crate::host::Host;

pub const TYPES: &str = include_str!("../../../assets/karna.d.ts");

pub struct JsScene {
    // All of these borrow the runtime behind `rt`; `Drop` frees them first.
    api: Option<Api<'static>>,
    module: Option<Persistent<'static>>,
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

impl JsScene {
    pub fn from_path(ctx: &mut LoadContext, path: impl AsRef<Path>) -> Self {
        Self::open(ctx.assets.root(), path).start(ctx)
    }

    pub fn open(root: &Path, path: impl AsRef<Path>) -> Self {
        let path = root.join(path);

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
            module: None,
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

        let module = this
            .js
            .eval_module(&source, this.path.as_str())
            .map(|module| this.js.persist(module));

        match module {
            Ok(module) => this.module = Some(module),
            Err(e) => this.fail("the module", e),
        }

        this
    }

    pub fn configure(&mut self, builder: WindowBuilder) -> WindowBuilder {
        let Some(module) = &self.module else {
            return builder;
        };

        let js = &*self.js;

        let fields = (|| {
            let window = module.get(js).get("window")?;

            if window.is_undefined() {
                return Ok(None);
            }

            if !window.is_object() {
                return Err(Error::Type(format!(
                    "the `window` export must be a karna.WindowBuilder, got {}",
                    window.type_name()
                )));
            }

            fn field<T: FromJs>(window: &quickjs::Value<'_>, name: &str) -> Result<T, Error> {
                T::from_js(&window.get(name)?).map_err(|e| match e {
                    Error::Type(msg) => Error::Type(format!("window.{name}: {msg}")),
                    e => e,
                })
            }

            Ok(Some((
                field::<Option<String>>(&window, "title")?,
                field::<Option<u32>>(&window, "width")?,
                field::<Option<u32>>(&window, "height")?,
                field::<Option<bool>>(&window, "resizable")?,
            )))
        })();

        let (title, width, height, resizable) = match fields {
            Ok(Some(fields)) => fields,
            Ok(None) => return builder,
            Err(e) => {
                self.fail("the window export", e);
                return builder;
            }
        };

        let mut builder = builder;

        if let Some(title) = title {
            builder = builder.with_title(title);
        }

        if width.is_some() || height.is_some() {
            let size = builder.size;
            builder = builder.with_size((width.unwrap_or(size.w()), height.unwrap_or(size.h())));
        }

        if let Some(resizable) = resizable {
            builder = builder.with_resizable(resizable);
        }

        builder
    }

    pub fn start(mut self, ctx: &mut LoadContext) -> Self {
        let Some(module) = &self.module else {
            return self;
        };

        let frame = Frame::new(
            &mut ctx.window,
            &mut ctx.time,
            ctx.input,
            Assets::Mut(ctx.assets),
            None,
        );

        let js = &*self.js;
        let rt = unsafe { self.rt.as_ref() };

        let scene = self.host.enter(frame, || {
            let export = module.get(js).get("default")?;

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
            Ok(scene) => self.scene = Some(scene),
            Err(e) => {
                self.fail("the module", e);
                return self;
            }
        }

        self.call(frame, "load", Phase::Update);
        self
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

impl Scene for JsScene {
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

impl Drop for JsScene {
    fn drop(&mut self) {
        unsafe {
            self.scene = None;
            self.module = None;
            self.api = None;
            ManuallyDrop::drop(&mut self.js);
            drop(Box::from_raw(self.rt.as_ptr()));
        }
    }
}

pub trait WindowBuilderExt {
    fn with_js_scene(self, id: SceneId, path: impl Into<PathBuf>) -> Self;
    fn with_js_entry(self, root: impl AsRef<Path>, id: SceneId, path: impl Into<PathBuf>) -> Self;
}

impl WindowBuilderExt for WindowBuilder {
    fn with_js_scene(self, id: SceneId, path: impl Into<PathBuf>) -> Self {
        let path = path.into();

        self.with_scene_fn(id, move |ctx| {
            Box::new(JsScene::from_path(ctx, &path)) as BoxedScene
        })
    }

    fn with_js_entry(self, root: impl AsRef<Path>, id: SceneId, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let mut scene = JsScene::open(root.as_ref(), &path);
        let builder = scene.configure(self);

        let opened = RefCell::new(Some(scene));

        builder.with_scene_fn(id, move |ctx| {
            let scene = opened
                .borrow_mut()
                .take()
                .unwrap_or_else(|| JsScene::open(ctx.assets.root(), &path));

            Box::new(scene.start(ctx)) as BoxedScene
        })
    }
}
