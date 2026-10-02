#![no_std]

use core::ptr::{self};

use lua::FromLua;
use nostd::alloc::boxed::Box;
use nostd::alloc::format;
use nostd::alloc::rc::Rc;
use nostd::alloc::string::String;
use nostd::alloc::string::ToString;
use nostd::fs;
use nostd::path::Path;
use nostd::path::PathBuf;
use engine::builder::WindowBuilder;
use engine::scene::SceneId;
use scripting::Hook;
use scripting::ScriptScene;
use traccia::info;

use crate::api::Api;

mod api;

pub const TYPES: &str = include_str!("../../../assets/karna.lua");

pub struct LuaScript {
    api: Option<Api<'static>>,
    module: Option<lua::Persistent<'static>>,
    scene: Option<lua::Persistent<'static>>,
    lua: ptr::NonNull<lua::Lua>,
}

impl scripting::Lang for LuaScript {
    type Error = lua::Error;
    const ENTRY: &'static str = "main.lua";

    fn host_error(e: scripting::HostError) -> Self::Error {
        lua::Error::custom(e.to_string())
    }

    fn open(path: &Path, host: &Rc<scripting::Host<Self>>) -> Result<Self, Self::Error> {
        let lua = Box::new(lua::Lua::new()?);
        let lua = ptr::NonNull::from(Box::leak(lua));

        let mut this = Self {
            api: None,
            module: None,
            scene: None,
            lua,
        };

        let lua: &'static lua::Lua = unsafe { this.lua.as_ref() };

        this.api = Some(api::install(lua, host)?);

        let dir = path.parent().map_or_else(PathBuf::new, Path::to_path_buf);

        lua.set_module_loader(move |name| {
            let file = dir
                .join(Path::new(&name.replace('.', "/")))
                .with_extension("lua");

            fs::read(&file)
                .map(|blob| String::from_utf8_lossy(&blob).into_owned())
                .map_err(lua::Error::custom)
        })?;

        let source = fs::read(path)
            .map(|blob| String::from_utf8_lossy(&blob).into_owned())
            .map_err(lua::Error::custom)?;

        info!("Running lua script {path}");

        let module = lua.load(&source, path.as_str())?.call(&[])?;
        this.module = Some(lua.persist(module));

        Ok(this)
    }

    fn configure(
        &mut self,
        mut b: engine::builder::WindowBuilder,
    ) -> Result<engine::builder::WindowBuilder, Self::Error> {
        let Some(module) = &self.module else {
            return Ok(b);
        };

        let lua = self.lua();
        let module = module.get(lua);

        if !module.is_table() {
            return Ok(b);
        }

        let window = module.get("window")?;

        if window.is_nil() {
            return Ok(b);
        }

        if !window.is_table() {
            return Err(lua::Error::Type(format!(
                "the `window` field must be a karna.WindowBuilder, got {}",
                window.type_name()
            )));
        }

        fn field<T: FromLua>(window: &lua::Value<'_>, name: &str) -> Result<T, lua::Error> {
            T::from_lua(&window.get(name)?).map_err(|e| match e {
                lua::Error::Type(msg) => lua::Error::Type(format!("window.{name}: {msg}")),
                e => e,
            })
        }

        let title = field::<Option<String>>(&window, "title")?;
        let width = field::<Option<u32>>(&window, "width")?;
        let height = field::<Option<u32>>(&window, "height")?;
        let resizable = field::<Option<bool>>(&window, "resizable")?;

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

        let lua = self.lua();
        let module = module.get(lua);

        if !module.is_table() {
            return Err(lua::Error::custom(format!(
                "the script must return a scene table, got {}",
                module.type_name()
            )));
        }

        let new = module.get("new")?;

        let scene = if new.is_function() {
            new.call_with(&module, &[])?
        } else {
            module
        };

        if !scene.is_table() && !scene.is_userdata() {
            return Err(lua::Error::custom(format!(
                "`new` must return a scene table, got {}",
                scene.type_name()
            )));
        }

        self.scene = Some(lua.persist(scene));

        Ok(())
    }

    fn call(&mut self, hook: scripting::Hook) -> Result<(), Self::Error> {
        let Some(scene) = &self.scene else {
            return Ok(());
        };

        let (name, arg) = match hook {
            Hook::Load => ("load", None),
            Hook::Unload => ("unload", None),
            Hook::FixedUpdate => ("fixed_update", None),
            Hook::Update => ("update", None),
            Hook::Draw => ("draw", self.api.as_ref().map(|api| &api.graphics)),
        };

        let lua = self.lua();
        let scene = scene.get(lua);
        let method = scene.get(name)?;

        if method.is_function() {
            let arg = arg.map(|a| a.get(lua));
            method.call_with(&scene, arg.as_slice())?;
        }

        Ok(())
    }
}

impl LuaScript {
    fn lua(&self) -> &'static lua::Lua {
        unsafe { self.lua.as_ref() }
    }
}

impl Drop for LuaScript {
    fn drop(&mut self) {
        self.scene = None;
        self.module = None;
        self.api = None;

        drop(unsafe { Box::from_raw(self.lua.as_ptr()) });
    }
}

pub type LuaHost = scripting::Host<LuaScript>;
pub type LuaScene = ScriptScene<LuaScript>;

pub trait LuaWindowBuilderExt {
    fn with_lua_scene(self, id: SceneId, path: impl Into<PathBuf>) -> Self;
}

impl LuaWindowBuilderExt for WindowBuilder {
    fn with_lua_scene(self, id: SceneId, path: impl Into<PathBuf>) -> Self {
        let path = path.into();

        self.with_scene_fn(id, move |ctx| {
            Box::new(LuaScene::from_path(ctx, &path))
        })
    }
}
