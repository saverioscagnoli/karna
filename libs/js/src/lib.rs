#![no_std]

mod api;

use core::mem::ManuallyDrop;
use core::ptr;

use nostd::alloc::format;
use nostd::alloc::string::String;
use nostd::alloc::string::ToString;
use nostd::fs;
use quickjs as qjs;

use engine::builder::WindowBuilder;
use engine::scene::SceneId;
use nostd::alloc::boxed::Box;
use nostd::alloc::rc::Rc;
use nostd::path::Path;
use nostd::path::PathBuf;
use scripting::Hook;
use scripting::Host;
use scripting::Lang;
use scripting::ScriptScene;
use traccia::info;

use crate::api::Api;

pub const TYPES: &str = include_str!("../../../assets/karna.d.ts");

pub struct Js {
    api: Option<Api<'static>>,
    module: Option<qjs::Persistent<'static>>,
    scene: Option<qjs::Persistent<'static>>,
    js: ManuallyDrop<qjs::Context<'static>>,
    rt: ptr::NonNull<qjs::Runtime>,
}

impl Lang for Js {
    type Error = qjs::Error;
    const ENTRY: &'static str = "main.js";

    fn host_error(e: scripting::HostError) -> Self::Error {
        qjs::Error::custom(e.to_string())
    }

    fn open(path: &Path, host: &Rc<scripting::Host<Self>>) -> Result<Self, Self::Error> {
        let rt = Box::new(qjs::Runtime::new()?);
        let rt = ptr::NonNull::from(Box::leak(rt));
        let js = qjs::Context::new(unsafe { rt.as_ref() })
            .inspect_err(|_| unsafe { drop(Box::from_raw(rt.as_ptr())) })?;

        unsafe { rt.as_ref() }.set_module_loader(|name| {
            fs::read(name)
                .map(|blob| String::from_utf8_lossy(&blob).into_owned())
                .map_err(qjs::Error::custom)
        });

        let mut this = Self {
            api: None,
            module: None,
            scene: None,
            js: ManuallyDrop::new(js),
            rt,
        };

        // From here on, Drop cleans up this if ? bails out
        this.api = Some(api::install(&this.js, host)?);

        let source = fs::read(path)
            .map(|blob| String::from_utf8_lossy(&blob).into_owned())
            .map_err(qjs::Error::custom)?;

        info!("Running script {path}");

        let module = this.js.eval_module(&source, path.as_str())?;
        this.module = Some(this.js.persist(module));

        Ok(this)
    }

    fn configure(&mut self, mut b: WindowBuilder) -> Result<WindowBuilder, Self::Error> {
        let Some(module) = &self.module else {
            return Ok(b);
        };

        let js = &*self.js;
        let fields = (|| {
            let window = module.get(js).get("window")?;

            if window.is_undefined() {
                return Ok(None);
            }

            if !window.is_object() {
                return Err(qjs::Error::Type(format!(
                    "the `window` export must be a karna.WindowBuilder, got {}",
                    window.type_name(),
                )));
            }

            fn field<T: qjs::FromJs>(window: &qjs::Value<'_>, name: &str) -> Result<T, qjs::Error> {
                T::from_js(&window.get(name)?).map_err(|e| match e {
                    qjs::Error::Type(msg) => qjs::Error::Type(format!("window.{name}: {msg}")),
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

        let Some((title, width, height, resizable)) = fields? else {
            return Ok(b);
        };

        if let Some(title) = title {
            b = b.with_title(title);
        }

        if width.is_some() || height.is_some() {
            let size = b.size;
            b = b.with_size((width.unwrap_or(size.w()), height.unwrap_or(size.h())));
        }

        if let Some(resizable) = resizable {
            b = b.with_resizable(resizable);
        }

        Ok(b)
    }

    fn start(&mut self) -> Result<(), Self::Error> {
        let Some(module) = &self.module else {
            return Ok(());
        };

        let js = &*self.js;
        let export = module.get(js).get("default")?;

        let scene = if export.is_function() {
            export.construct(&[])?
        } else if export.is_object() {
            export
        } else {
            return Err(qjs::Error::custom(
                "the script must `export default` a scene class or object",
            ));
        };

        self.scene = Some(js.persist(scene));

        unsafe { self.rt.as_ref() }.run_jobs()
    }

    fn call(&mut self, hook: Hook) -> Result<(), Self::Error> {
        let Some(scene) = &self.scene else {
            return Ok(());
        };

        let (name, arg) = match hook {
            Hook::Load => ("load", None),
            Hook::Unload => ("unload", None),
            Hook::FixedUpdate => ("fixedUpdate", None),
            Hook::Update => ("update", None),
            Hook::Draw => ("draw", self.api.as_ref().map(|api| &api.graphics)),
        };

        let js = &*self.js;
        let scene = scene.get(js);
        let method = scene.get(name)?;

        if method.is_function() {
            let arg = arg.map(|a| a.get(js));
            method.call_with(&scene, arg.as_slice())?;
        }

        unsafe { self.rt.as_ref() }.run_jobs()
    }
}

impl Drop for Js {
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

pub type JsHost = Host<Js>;
pub type JsScene = ScriptScene<Js>;

pub trait JsWindowBuilderExt {
    fn with_js_scene(self, id: SceneId, path: impl Into<PathBuf>) -> Self;
}

impl JsWindowBuilderExt for WindowBuilder {
    fn with_js_scene(self, id: SceneId, path: impl Into<PathBuf>) -> Self {
        let path = path.into();

        self.with_scene_fn(id, move |ctx| {
            Box::new(JsScene::from_path(ctx, &path))
        })
    }
}
