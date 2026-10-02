use engine::context::DrawContext;
use engine::context::LoadContext;
use engine::context::UpdateContext;
use engine::render::Draw;
use engine::render::Layer;
use engine::scene::Scene;
use nostd::alloc::format;
use nostd::alloc::rc::Rc;
use nostd::alloc::string::String;
use nostd::path::Path;
use nostd::path::PathBuf;
use sdl3::render::Color;
use traccia::error;

use crate::Host;
use crate::lang::Hook;
use crate::lang::Lang;

pub struct ScriptScene<L: Lang> {
    lang: Option<L>,
    host: Rc<Host<L>>,
    path: PathBuf,
    error: Option<String>,
}

impl<L: Lang> ScriptScene<L> {
    pub fn open(root: &Path, path: &Path) -> Self {
        let path = root.join(path);
        let host = Rc::new(Host::default());

        let mut this = Self {
            lang: None,
            host,
            path,
            error: None,
        };

        match L::open(&this.path, &this.host) {
            Ok(lang) => this.lang = Some(lang),
            Err(e) => this.fail("loading", e),
        }

        this
    }

    pub fn from_path(ctx: &mut LoadContext, path: impl AsRef<Path>) -> Self {
        Self::open(ctx.assets.root(), path.as_ref()).start(ctx)
    }

    pub fn start(mut self, ctx: &mut LoadContext) -> Self {
        let Some(lang) = &mut self.lang else {
            return self;
        };

        let res = self.host.with_load(ctx, || {
            lang.start()?;
            lang.call(Hook::Load)
        });

        if let Err(e) = res {
            self.fail("load", e);
        }

        self
    }

    fn fail(&mut self, what: &str, e: L::Error) {
        let msg = format!("{}: error in {what}: {e}", self.path);
        error!("{msg}");
        self.error = Some(msg);
    }

    fn run<E>(&mut self, hook: Hook, enter: E)
    where
        E: FnOnce(&mut dyn FnMut() -> Result<(), L::Error>) -> Result<(), L::Error>,
    {
        if self.error.is_some() {
            return;
        }

        let Some(lang) = &mut self.lang else { return };
        let mut call = || lang.call(hook);

        if let Err(e) = enter(&mut call) {
            self.fail(hook.name(), e);
        }
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

impl<L: Lang> Scene for ScriptScene<L> {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        Self::from_path(ctx, L::ENTRY)
    }

    fn unload(&mut self, ctx: &mut LoadContext) {
        let host = Rc::clone(&self.host);
        self.run(Hook::Unload, |call| host.with_load(ctx, call));
    }

    fn fixed_update(&mut self, ctx: &mut UpdateContext) {
        let host = Rc::clone(&self.host);
        self.run(Hook::FixedUpdate, |call| host.with_update(ctx, call));
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        let host = Rc::clone(&self.host);
        self.run(Hook::Update, |call| host.with_update(ctx, call));
    }

    fn draw(&mut self, ctx: &mut DrawContext, draw: &mut Draw) {
        let host = Rc::clone(&self.host);
        self.run(Hook::Draw, |call| host.with_draw(ctx, draw, call));
        self.draw_error(draw);
    }
}
